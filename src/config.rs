use crate::data_structures::{SharedTickerGroups, TickerGroups};
use std::env;
use std::fs;
use std::sync::Arc;
use std::time::Duration;
use serde::{Deserialize, Serialize};

// Holds tokens for zero-downtime rotation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenConfig {
    pub primary: String,
    pub secondary: String,
}
pub type SharedTokenConfig = Arc<TokenConfig>;

// Holds URLs of peer servers (internal and public)
pub type PeerList = Arc<Vec<String>>;

// YAML-serializable configuration structure
#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigYaml {
    pub node_name: String,
    pub tokens: TokenConfig,
    pub internal_peers: Vec<String>,
    pub public_peers: Vec<String>,
    pub core_network_url: Option<String>,
    pub public_refresh_interval_secs: u64,
    pub core_worker_interval_secs: u64,
    pub environment: String,
    pub port: u16,
}

// Holds application-wide settings
#[derive(Clone)]
pub struct AppConfig {
    pub node_name: String,
    pub tokens: SharedTokenConfig,
    pub internal_peers: PeerList,
    pub public_peers: PeerList,
    pub core_network_url: Option<String>,
    pub public_refresh_interval: Duration,
    pub core_worker_interval: Duration,
    pub environment: String,
    pub port: u16,
}

impl AppConfig {
    // Load configuration from YAML file or environment variables
    pub fn load() -> Self {
        // Check for CONFIG_FILE environment variable first
        if let Ok(config_file) = env::var("CONFIG_FILE") {
            Self::from_yaml(&config_file)
        } else {
            Self::from_env()
        }
    }

    // Load configuration from YAML file
    pub fn from_yaml(file_path: &str) -> Self {
        let yaml_content = fs::read_to_string(file_path)
            .unwrap_or_else(|e| panic!("Failed to read config file {}: {}", file_path, e));
        
        let yaml_config: ConfigYaml = serde_yaml::from_str(&yaml_content)
            .unwrap_or_else(|e| panic!("Failed to parse YAML config: {}", e));

        Self {
            node_name: yaml_config.node_name,
            tokens: Arc::new(yaml_config.tokens),
            internal_peers: Arc::new(yaml_config.internal_peers),
            public_peers: Arc::new(yaml_config.public_peers),
            core_network_url: yaml_config.core_network_url,
            public_refresh_interval: Duration::from_secs(yaml_config.public_refresh_interval_secs),
            core_worker_interval: Duration::from_secs(yaml_config.core_worker_interval_secs),
            environment: yaml_config.environment,
            port: yaml_config.port,
        }
    }

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
        
        let public_refresh_interval_secs = env::var("PUBLIC_REFRESH_INTERVAL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(300); // Default to 5 minutes

        let core_worker_interval_secs = env::var("CORE_WORKER_INTERVAL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30); // Default to 30 seconds

        let environment = env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string());

        let port = env::var("PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8888); // Default to 8888

        let node_name = env::var("NODE_NAME")
            .unwrap_or_else(|_| "aipriceaction-proxy".to_string());

        Self {
            node_name,
            tokens,
            internal_peers,
            public_peers,
            core_network_url,
            public_refresh_interval: Duration::from_secs(public_refresh_interval_secs),
            core_worker_interval: Duration::from_secs(core_worker_interval_secs),
            environment,
            port,
        }
    }
}

/// Load ticker groups from ticker_group.json file
pub fn load_ticker_groups() -> SharedTickerGroups {
    let ticker_group_path = "ticker_group.json";
    
    tracing::info!("Loading ticker groups from: {}", ticker_group_path);
    
    let json_content = fs::read_to_string(ticker_group_path)
        .unwrap_or_else(|e| panic!("Failed to read ticker_group.json: {}", e));
    
    let ticker_groups: TickerGroups = serde_json::from_str(&json_content)
        .unwrap_or_else(|e| panic!("Failed to parse ticker_group.json: {}", e));
    
    tracing::info!("Successfully loaded {} ticker groups", ticker_groups.0.len());
    
    Arc::new(ticker_groups)
}