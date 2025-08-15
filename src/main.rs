pub mod api;
pub mod config;
pub mod data_structures;
pub mod vci;
pub mod worker;

use crate::config::SharedTokenConfig;
use crate::data_structures::{InMemoryData, PublicActorReputation, LastInternalUpdate, SharedData, SharedReputation};
use axum::{extract::FromRef, routing::{get, post}, Router};
use std::{net::SocketAddr, sync::Arc, time::Instant};
use tokio::sync::Mutex;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

#[derive(Clone)]
struct AppState {
    data: SharedData,
    reputation: SharedReputation,
    last_update: LastInternalUpdate,
    tokens: SharedTokenConfig,
}

impl FromRef<AppState> for SharedData {
    fn from_ref(app_state: &AppState) -> SharedData {
        app_state.data.clone()
    }
}

impl FromRef<AppState> for SharedReputation {
    fn from_ref(app_state: &AppState) -> SharedReputation {
        app_state.reputation.clone()
    }
}

impl FromRef<AppState> for LastInternalUpdate {
    fn from_ref(app_state: &AppState) -> LastInternalUpdate {
        app_state.last_update.clone()
    }
}

impl FromRef<AppState> for SharedTokenConfig {
    fn from_ref(app_state: &AppState) -> SharedTokenConfig {
        app_state.tokens.clone()
    }
}

#[tokio::main]
async fn main() {
    let app_config = config::AppConfig::load();
    
    // Initialize tracing with node_name in all logs
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .init();

    // Set a global span with node_name for all subsequent logs
    let _span = tracing::info_span!("node", name = %app_config.node_name).entered();
    
    tracing::info!("Starting aipriceaction-proxy");
    tracing::info!(?app_config.environment, port = app_config.port, "Loaded configuration");
    
    let shared_data: SharedData = Arc::new(Mutex::new(InMemoryData::new()));
    let shared_reputation: SharedReputation = Arc::new(Mutex::new(PublicActorReputation::new()));
    let last_internal_update: LastInternalUpdate = Arc::new(Mutex::new(Instant::now()));
    let shared_tokens: SharedTokenConfig = app_config.tokens.clone();

    let app_state = AppState {
        data: shared_data.clone(),
        reputation: shared_reputation,
        last_update: last_internal_update,
        tokens: shared_tokens,
    };

    tracing::info!("Spawning background worker");
    tokio::spawn(worker::run(
        shared_data.clone(),
        app_config.clone(),
    ));

    let governor_conf = Arc::new(
        GovernorConfigBuilder::default().per_second(10).burst_size(20).finish().unwrap(),
    );

    let app = Router::new()
        .route("/tickers", get(api::get_all_tickers_handler))
        .route("/gossip", post(api::internal_gossip_handler))
        .route(
            "/public/gossip",
            post(api::public_gossip_handler).layer(GovernorLayer::new(governor_conf)),
        )
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], app_config.port));
    tracing::info!(%addr, "Server listening");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
