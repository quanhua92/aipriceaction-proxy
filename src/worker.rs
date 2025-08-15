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