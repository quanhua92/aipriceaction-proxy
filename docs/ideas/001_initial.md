**Project Implementation Plan: `aipriceaction-proxy`**

This document provides a complete, step-by-step guide to building the `aipriceaction-proxy` service. The system is a resilient, distributed stock data proxy designed for high availability and real-time performance. It can be configured to run in two modes: as a **Core Node** that fetches data directly from the source and broadcasts to trusted and public peers, or as a **Public Node** that synchronizes its data from a core network.

-----

### **Step 1: Project Setup & Dependencies**

This phase initializes the Rust project and adds all necessary dependencies using the specified versions for Rust `1.89.0`.

1.  **Create the new Rust binary project:**

    ```bash
    cargo new --bin aipriceaction-proxy
    cd aipriceaction-proxy
    ```

2.  **Create the required module directories:**

    ```bash
    mkdir -p src/api src/config src/worker
    ```

3.  **Add all project dependencies via cargo:**

    ```bash
    cargo add tokio@1.38.0 --features full
    cargo add axum@0.8.4
    cargo add reqwest@0.12.23 --features json
    cargo add serde@1.0 --features derive
    cargo add serde_json@1.0
    cargo add chrono@0.4
    cargo add rand@0.8
    cargo add dotenvy@0.15
    cargo add tower-governor@0.1.1 # For public API rate limiting
    cargo add nextest-runner
    ```

-----

### **Step 2: Core Data Structures (`src/data_structures.rs`)**

Define all shared data structures and types in a central module for clarity and reusability.

**File: `src/data_structures.rs`**

```rust
use crate::vci::OhlcvData;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

// --- Core Data Structures ---

#[derive(Clone, Debug)]
pub struct ActorMetadata {
    pub successful_updates: u32,
    pub failed_updates: u32,
    pub status: ActorStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ActorStatus {
    Probation,
    Trusted,
    Banned,
}

impl Default for ActorMetadata {
    fn default() -> Self {
        Self {
            successful_updates: 0,
            failed_updates: 0,
            status: ActorStatus::Probation,
        }
    }
}

// --- Type Aliases for Shared State ---

// Main in-memory cache for all stock data
pub type InMemoryData = HashMap<String, Vec<OhlcvData>>;
pub type SharedData = Arc<Mutex<InMemoryData>>;

// Reputation tracker for public contributors
pub type PublicActorReputation = HashMap<IpAddr, ActorMetadata>;
pub type SharedReputation = Arc<Mutex<PublicActorReputation>>;

// Timestamp of the last trusted internal update
pub type LastInternalUpdate = Arc<Mutex<Instant>>;
```

-----

### **Step 3: Configuration Management (`src/config.rs`)**

This module handles loading secrets and configuration from environment variables, including the new public peer list.

**File: `src/config.rs`**

```rust
use std::env;
use std::sync::Arc;
use std::time::Duration;

// Holds tokens for zero-downtime rotation
#[derive(Clone)]
pub struct TokenConfig {
    pub primary: String,
    pub secondary: String,
}
pub type SharedTokenConfig = Arc<TokenConfig>;

// Holds URLs of peer servers (internal and public)
pub type PeerList = Arc<Vec<String>>;

// Holds application-wide settings
#[derive(Clone)]
pub struct AppConfig {
    pub tokens: SharedTokenConfig,
    pub internal_peers: PeerList,
    pub public_peers: PeerList,
    pub core_network_url: Option<String>,
    pub public_refresh_interval: Duration,
}

impl AppConfig {
    // Load all configuration from environment variables
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok(); // Load .env file if present

        let tokens = Arc::new(TokenConfig {
            primary: env::var("PRIMARY_TOKEN").expect("PRIMARY_TOKEN must be set"),
            secondary: env::var("SECONDARY_TOKEN").expect("SECONDARY_TOKEN must be set"),
        });

        let internal_peers = Arc::new(
            env::var("INTERNAL_PEER_URLS")
                .expect("INTERNAL_PEER_URLS must be set")
                .split(',')
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect::<Vec<String>>(),
        );

        let public_peers = Arc::new(
            env::var("PUBLIC_PEER_URLS")
                .unwrap_or_else(|_| "https://api.aipriceaction.com".to_string())
                .split(',')
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect::<Vec<String>>(),
        );

        let core_network_url = env::var("CORE_NETWORK_URL").ok();
        let refresh_interval_secs = env::var("PUBLIC_REFRESH_INTERVAL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(300); // Default to 5 minutes

        Self {
            tokens,
            internal_peers,
            public_peers,
            core_network_url,
            public_refresh_interval: Duration::from_secs(refresh_interval_secs),
        }
    }
}
```

-----

### **Step 4: VCI Client Library (`src/vci.rs`)**

Place the complete code for the `VciClient` into this file. This module is responsible for all communication with the external stock API.

-----

### **Step 5: The Background Worker (`src/worker.rs`)**

The Core Node worker is updated to broadcast to both internal (trusted) and public (untrusted) peers.

**File: `src/worker.rs`**

```rust
use crate::config::AppConfig;
use crate::data_structures::{InMemoryData, SharedData};
use std::time::Duration;
use reqwest::Client as ReqwestClient;

pub async fn run(data: SharedData, config: AppConfig) {
    if let Some(core_url) = config.core_network_url {
        run_public_node_worker(data, core_url, config.public_refresh_interval).await;
    } else {
        run_core_node_worker(data, config).await;
    }
}

async fn run_core_node_worker(data: SharedData, config: AppConfig) {
    let mut vci_client = crate::vci::VciClient::new(true, 30).unwrap();
    let gossip_client = ReqwestClient::new();
    let tickers = vec!["VCB".to_string(), "TCB".to_string(), "FPT".to_string(), "ACB".to_string()];

    loop {
        if let Ok(batch_data) = vci_client.get_batch_history(&tickers, "2024-01-01", None, "1D").await {
            let mut data_guard = data.lock().await;
            for (symbol, ohlcv_data_vec) in batch_data {
                if let Some(data_vec) = ohlcv_data_vec {
                    let latest_data = data_vec.last().cloned();
                    data_guard.insert(symbol.clone(), data_vec);

                    if let Some(gossip_payload) = latest_data {
                        // --- 1. Broadcast to INTERNAL peers (trusted, with token) ---
                        let auth_token = format!("Bearer {}", config.tokens.primary);
                        for peer_url in config.internal_peers.iter() {
                            let client = gossip_client.clone();
                            let token = auth_token.clone();
                            let payload = gossip_payload.clone();
                            let url = format!("{}/gossip", peer_url);
                            tokio::spawn(async move {
                                let _ = client.post(&url).header("Authorization", token).json(&payload).send().await;
                            });
                        }
                        
                        // --- 2. Broadcast to PUBLIC peers (untrusted, no token) ---
                        for peer_url in config.public_peers.iter() {
                            let client = gossip_client.clone();
                            let payload = gossip_payload.clone();
                            let url = format!("{}/public/gossip", peer_url);
                             tokio::spawn(async move {
                                let _ = client.post(&url).json(&payload).send().await;
                            });
                        }
                    }
                }
            }
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn run_public_node_worker(data: SharedData, core_network_url: String, refresh_interval: Duration) {
    let http_client = ReqwestClient::new();
    loop {
        let core_tickers_url = format!("{}/tickers", core_network_url);
        if let Ok(response) = http_client.get(&core_tickers_url).send().await {
            if let Ok(core_data) = response.json::<InMemoryData>().await {
                let mut local_data_guard = data.lock().await;
                for (symbol, core_ohlcv_vec) in core_data {
                    let local_entry = local_data_guard.entry(symbol).or_default();
                    if let (Some(core_last), Some(local_last)) = (core_ohlcv_vec.last(), local_entry.last()) {
                        if core_last.time > local_last.time {
                            *local_entry = core_ohlcv_vec;
                        }
                    } else if local_entry.is_empty() {
                         *local_entry = core_ohlcv_vec;
                    }
                }
            }
        }
        tokio::time::sleep(refresh_interval).await;
    }
}
```

-----

### **Step 6: API Handlers (`src/api.rs`)**

This module contains all three Axum endpoint handlers. The logic is unchanged.

**File: `src/api.rs`**

```rust
use crate::config::SharedTokenConfig;
use crate::data_structures::{LastInternalUpdate, OhlcvData, SharedData, SharedReputation};
use axum::{
    extract::{ConnectInfo, State, Json},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use std::net::SocketAddr;
use std::time::Duration;

pub async fn get_all_tickers_handler(State(state): State<SharedData>) -> impl IntoResponse {
    let data = state.lock().await;
    (StatusCode::OK, Json(data.clone()))
}

pub async fn internal_gossip_handler(
    State(data_state): State<SharedData>,
    State(token_state): State<SharedTokenConfig>,
    State(last_update_state): State<LastInternalUpdate>,
    headers: HeaderMap,
    Json(payload): Json<OhlcvData>,
) -> impl IntoResponse {
    let auth_header = headers.get("Authorization").and_then(|h| h.to_str().ok());
    let token_is_valid = match auth_header {
        Some(header) => {
            let token = header.trim_start_matches("Bearer ");
            token == token_state.primary || token == token_state.secondary
        }
        None => false,
    };

    if !token_is_valid {
        return (StatusCode::UNAUTHORIZED, "Invalid or missing token").into_response();
    }

    *last_update_state.lock().await = std::time::Instant::now();

    let mut data_guard = data_state.lock().await;
    if let Some(symbol) = &payload.symbol {
        let entry = data_guard.entry(symbol.clone()).or_default();
        if entry.last().map_or(true, |last| payload.time > last.time) {
            entry.push(payload);
            entry.sort_by_key(|d| d.time);
        }
    }

    (StatusCode::OK, "OK").into_response()
}

pub async fn public_gossip_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(data_state): State<SharedData>,
    State(reputation_state): State<SharedReputation>,
    State(last_update_state): State<LastInternalUpdate>,
    Json(payload): Json<OhlcvData>,
) -> Response {
    let source_ip = addr.ip();
    let mut reputation_guard = reputation_state.lock().await;
    let actor = reputation_guard.entry(source_ip).or_default();

    if actor.status == crate::data_structures::ActorStatus::Banned {
        return (StatusCode::FORBIDDEN, "Source IP is banned").into_response();
    }

    let last_internal_update = last_update_state.lock().await;
    if last_internal_update.elapsed() > Duration::from_secs(300) {
        return (StatusCode::SERVICE_UNAVAILABLE, "System is running on untrusted data").into_response();
    }

    let mut data_guard = data_state.lock().await;
    if let Some(symbol) = &payload.symbol {
        if let Some(entry) = data_guard.get(symbol) {
            if let Some(last_data) = entry.last() {
                let price_change_percent = (payload.close - last_data.close).abs() / last_data.close;
                if price_change_percent > 0.10 {
                    actor.failed_updates += 1;
                    if actor.failed_updates > 5 {
                        actor.status = crate::data_structures::ActorStatus::Banned;
                    }
                    return (StatusCode::BAD_REQUEST, "Implausible price change").into_response();
                }
            }
        }
        
        actor.successful_updates += 1;
        let entry = data_guard.entry(symbol.clone()).or_default();
        entry.push(payload);
        entry.sort_by_key(|d| d.time);
    }

    (StatusCode::OK, "OK").into_response()
}
```

-----

### **Step 7: Main Application (`src/main.rs`)**

This file wires everything together: initializes state, spawns the worker, and launches the Axum web server.

**File: `src/main.rs`**

```rust
mod api;
mod config;
mod data_structures;
mod vci;
mod worker;

use crate::data_structures::{InMemoryData, PublicActorReputation};
use axum::{routing::{get, post}, Router};
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Instant};
use tokio::sync::Mutex;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

#[tokio::main]
async fn main() {
    let app_config = config::AppConfig::from_env();
    
    let shared_data = Arc::new(Mutex::new(InMemoryData::new()));
    let shared_reputation = Arc::new(Mutex::new(PublicActorReputation::new()));
    let last_internal_update = Arc::new(Mutex::new(Instant::now()));

    tokio::spawn(worker::run(
        shared_data.clone(),
        app_config.clone(),
    ));

    let governor_conf = Box::new(
        GovernorConfigBuilder::default().per_second(10).burst_size(20).finish().unwrap(),
    );

    let app = Router::new()
        .route("/tickers", get(api::get_all_tickers_handler))
        .route("/gossip", post(api::internal_gossip_handler))
        .route(
            "/public/gossip",
            post(api::public_gossip_handler).layer(GovernorLayer {
                config: Box::leak(governor_conf),
            }),
        )
        .with_state(shared_data)
        .with_state(shared_reputation)
        .with_state(last_internal_update)
        .with_state(app_config.tokens);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
```

-----

### **Step 8: Local Environment Setup (`.env`)**

The `.env` file now includes the `PUBLIC_PEER_URLS` variable.

**File: `.env` (for a Core Node)**

```env
PRIMARY_TOKEN="secret-token-A-12345"
SECONDARY_TOKEN="secret-token-B-67890"

# Comma-separated list of trusted servers in the cluster
INTERNAL_PEER_URLS="http://localhost:3001,http://localhost:3002"

# Comma-separated list of public-facing APIs to notify
PUBLIC_PEER_URLS="https://api.aipriceaction.com"
```

**File: `.env` (for a Public Node)**

```env
PRIMARY_TOKEN="some-secret-for-my-own-peers"
SECONDARY_TOKEN="another-secret-for-my-own-peers"
INTERNAL_PEER_URLS="http://localhost:9001" 
PUBLIC_PEER_URLS="https://api.aipriceaction.com"

# --- Public Node Settings ---
CORE_NETWORK_URL="http://api.main-network.com"
PUBLIC_REFRESH_INTERVAL="60"
```

-----

### **Step 9: Testing and Execution**

1.  **Run Tests:**

    ```bash
    cargo nextest run
    ```

2.  **Build Release Binary:**

    ```bash
    cargo build --release
    ```

3.  **Run the Application:**

    ```bash
    ./target/release/aipriceaction-proxy
    ```
