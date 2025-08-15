use crate::data_structures::{SharedTickerGroups, TickerGroups};
use std::env;
use std::fs;
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
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

// Office hours configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OfficeHours {
    pub start_hour: u32,     // e.g., 9 for 9am
    pub end_hour: u32,       // e.g., 16 for 4pm
    pub timezone: String,    // e.g., "Asia/Ho_Chi_Minh"
    pub weekdays_only: bool, // true for Monday-Friday only
}

impl Default for OfficeHours {
    fn default() -> Self {
        Self {
            start_hour: 9,
            end_hour: 16,
            timezone: "Asia/Ho_Chi_Minh".to_string(),
            weekdays_only: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OfficeHoursConfig {
    pub default_office_hours: OfficeHours,
    pub ticker_specific: HashMap<String, OfficeHours>, // Future: per-ticker hours
}

impl Default for OfficeHoursConfig {
    fn default() -> Self {
        Self {
            default_office_hours: OfficeHours::default(),
            ticker_specific: HashMap::new(),
        }
    }
}

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
    pub non_office_hours_interval_secs: Option<u64>,
    pub enable_office_hours: Option<bool>,
    pub office_hours_config: Option<OfficeHoursConfig>,
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
    pub non_office_hours_interval: Duration,
    pub enable_office_hours: bool,
    pub office_hours_config: OfficeHoursConfig,
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
            non_office_hours_interval: Duration::from_secs(yaml_config.non_office_hours_interval_secs.unwrap_or(300)),
            enable_office_hours: yaml_config.enable_office_hours.unwrap_or(true),
            office_hours_config: yaml_config.office_hours_config.unwrap_or_default(),
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

        let non_office_hours_interval_secs = env::var("NON_OFFICE_HOURS_INTERVAL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(300); // Default to 5 minutes

        let enable_office_hours = env::var("ENABLE_OFFICE_HOURS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(true); // Default to true

        Self {
            node_name,
            tokens,
            internal_peers,
            public_peers,
            core_network_url,
            public_refresh_interval: Duration::from_secs(public_refresh_interval_secs),
            core_worker_interval: Duration::from_secs(core_worker_interval_secs),
            non_office_hours_interval: Duration::from_secs(non_office_hours_interval_secs),
            enable_office_hours,
            office_hours_config: OfficeHoursConfig::default(), // Use default Vietnam office hours
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