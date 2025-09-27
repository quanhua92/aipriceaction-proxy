pub mod analysis_service;
pub mod api;
pub mod config;
pub mod data_structures;
pub mod vci;
pub mod worker;

use crate::config::SharedTokenConfig;
use crate::data_structures::{InMemoryData, PublicActorReputation, LastInternalUpdate, SharedData, SharedReputation, SharedTickerGroups, SharedHealthStats, HealthStats, SharedEnhancedData, EnhancedInMemoryData};
use axum::{extract::FromRef, routing::{get, post}, Router};
use std::{net::SocketAddr, sync::Arc, time::Instant};
use tokio::sync::Mutex;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::cors::{CorsLayer, Any};

#[derive(Clone)]
struct AppState {
    data: SharedData,
    enhanced_data: SharedEnhancedData,
    reputation: SharedReputation,
    last_update: LastInternalUpdate,
    tokens: SharedTokenConfig,
    ticker_groups: SharedTickerGroups,
    health_stats: SharedHealthStats,
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

impl FromRef<AppState> for SharedTickerGroups {
    fn from_ref(app_state: &AppState) -> SharedTickerGroups {
        app_state.ticker_groups.clone()
    }
}

impl FromRef<AppState> for SharedHealthStats {
    fn from_ref(app_state: &AppState) -> SharedHealthStats {
        app_state.health_stats.clone()
    }
}

impl FromRef<AppState> for SharedEnhancedData {
    fn from_ref(app_state: &AppState) -> SharedEnhancedData {
        app_state.enhanced_data.clone()
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

    // Initialize custom rayon thread pool to avoid blocking async runtime
    let num_threads = num_cpus::get().saturating_sub(2).max(1);
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .thread_name(|index| format!("rayon-{}", index))
        .build_global()
        .expect("Failed to initialize rayon thread pool");

    tracing::info!("Starting aipriceaction-proxy");
    tracing::info!(?app_config.environment, port = app_config.port, rayon_threads = num_threads, "Loaded configuration");
    
    // Load ticker groups after tracing is initialized
    let shared_ticker_groups: SharedTickerGroups = config::load_ticker_groups();
    
    let shared_data: SharedData = Arc::new(Mutex::new(InMemoryData::new()));
    let shared_enhanced_data: SharedEnhancedData = Arc::new(Mutex::new(EnhancedInMemoryData::default()));
    let shared_reputation: SharedReputation = Arc::new(Mutex::new(PublicActorReputation::new()));
    let last_internal_update: LastInternalUpdate = Arc::new(Mutex::new(Instant::now()));
    let shared_tokens: SharedTokenConfig = app_config.tokens.clone();
    // Note: shared_ticker_groups will be loaded after tracing is initialized
    
    // Initialize health stats with app config
    let health_stats = HealthStats {
        office_hours_enabled: app_config.enable_office_hours,
        timezone: app_config.office_hours_config.default_office_hours.timezone.clone(),
        office_start_hour: app_config.office_hours_config.default_office_hours.start_hour,
        office_end_hour: app_config.office_hours_config.default_office_hours.end_hour,
        environment: app_config.environment.clone(),
        node_name: app_config.node_name.clone(),
        internal_peers_count: app_config.internal_peers.len(),
        public_peers_count: app_config.public_peers.len(),
        build_date: app_config.build_date.clone(),
        git_commit: app_config.git_commit.clone(),
        ..HealthStats::default()
    };
    let shared_health_stats: SharedHealthStats = Arc::new(Mutex::new(health_stats));

    let app_state = AppState {
        data: shared_data.clone(),
        enhanced_data: shared_enhanced_data.clone(),
        reputation: shared_reputation,
        last_update: last_internal_update,
        tokens: shared_tokens,
        ticker_groups: shared_ticker_groups,
        health_stats: shared_health_stats.clone(),
    };

    tracing::info!("About to spawn background worker");
    let worker_handle = tokio::spawn(worker::run(
        shared_data.clone(),
        shared_enhanced_data.clone(),
        app_config.clone(),
        shared_health_stats.clone(),
    ));
    
    tracing::info!("Worker spawned successfully, continuing to HTTP server setup");

    let governor_conf = Arc::new(
        GovernorConfigBuilder::default().per_second(10).burst_size(20).finish().unwrap(),
    );

    // Configure CORS to allow aipriceaction.com to call api.aipriceaction.com
    let cors = CorsLayer::new()
        .allow_origin([
            "https://aipriceaction.com".parse().unwrap(),
            "https://www.aipriceaction.com".parse().unwrap(),
            "http://localhost:3000".parse().unwrap(), // For local development
            "http://100.121.116.69:9876".parse().unwrap(),
            "http://100.121.116.69:5173".parse().unwrap(),
            "http://192.168.1.13:5173".parse().unwrap(),
            "http://192.168.1.13:9876".parse().unwrap(),
        ])
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST, axum::http::Method::OPTIONS])
        .allow_headers(Any);

    let app = Router::new()
        .route("/tickers", get(api::get_all_tickers_handler))
        .route("/tickers/group", get(api::get_ticker_groups_handler))
        .route("/gossip", post(api::internal_gossip_handler))
        .route(
            "/public/gossip",
            post(api::public_gossip_handler).layer(GovernorLayer::new(governor_conf)),
        )
        .route("/health", get(api::health_handler))
        .layer(cors)
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], app_config.port));
    tracing::info!(%addr, "About to bind server");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!(%addr, "Server bound, starting to serve");
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
