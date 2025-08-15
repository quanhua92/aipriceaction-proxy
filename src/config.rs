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