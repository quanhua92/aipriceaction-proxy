use crate::config::AppConfig;
use crate::data_structures::{InMemoryData, SharedData};
use std::time::Duration;
use reqwest::Client as ReqwestClient;
use tracing::{info, debug, warn, error, instrument};

#[instrument(skip(data, config))]
pub async fn run(data: SharedData, config: AppConfig) {
    if let Some(core_url) = &config.core_network_url {
        info!(%core_url, "Starting as public node worker");
        run_public_node_worker(data, core_url.clone(), config.public_refresh_interval).await;
    } else {
        info!(environment = %config.environment, "Starting as core node worker");
        run_core_node_worker(data, config).await;
    }
}

#[instrument(skip(data, config))]
async fn run_core_node_worker(data: SharedData, config: AppConfig) {
    info!("Initializing core node worker");
    
    let mut vci_client = match crate::vci::VciClient::new(true, 30) {
        Ok(client) => {
            info!("VCI client initialized successfully");
            client
        }
        Err(e) => {
            error!(?e, "Failed to initialize VCI client");
            return;
        }
    };
    
    let gossip_client = ReqwestClient::new();
    let all_tickers = vec!["VCB".to_string(), "TCB".to_string(), "FPT".to_string(), "ACB".to_string()];
    info!(?all_tickers, "Available tickers for random selection");

    let mut iteration_count = 0;
    loop {
        iteration_count += 1;
        debug!(iteration = iteration_count, "Starting data fetch cycle");
        
        // Randomly select 2 out of 4 tickers using simple approach
        let first_idx = iteration_count % all_tickers.len();
        let second_idx = (iteration_count + 2) % all_tickers.len();
        let selected_tickers = vec![
            all_tickers[first_idx].clone(),
            all_tickers[second_idx].clone()
        ];
        info!(iteration = iteration_count, ?selected_tickers, "Selected tickers for this cycle");
        
        match vci_client.get_batch_history(&selected_tickers, "2025-08-14", Some("2025-08-15"), "1D").await {
            Ok(batch_data) => {
                info!(iteration = iteration_count, symbols_count = batch_data.len(), "Successfully fetched batch data from VCI");
                
                let mut data_guard = data.lock().await;
                let mut updated_symbols = Vec::new();
                
                for (symbol, ohlcv_data_vec) in batch_data {
                    if let Some(data_vec) = ohlcv_data_vec {
                        let data_points = data_vec.len();
                        let latest_data = data_vec.last().cloned();
                        data_guard.insert(symbol.clone(), data_vec);
                        updated_symbols.push(symbol.clone());
                        debug!(symbol, data_points, "Updated symbol data");

                        if let Some(gossip_payload) = latest_data {
                            // --- 1. Broadcast to INTERNAL peers (trusted, with token) ---
                            let auth_token = format!("Bearer {}", config.tokens.primary);
                            let internal_peer_count = config.internal_peers.len();
                            
                            debug!(symbol, internal_peers = internal_peer_count, "Broadcasting to internal peers");
                            for peer_url in config.internal_peers.iter() {
                                let client = gossip_client.clone();
                                let token = auth_token.clone();
                                let payload = gossip_payload.clone();
                                let url = format!("{}/gossip", peer_url);
                                let peer_url_clone = peer_url.clone();
                                
                                tokio::spawn(async move {
                                    match client.post(&url).header("Authorization", token).json(&payload).send().await {
                                        Ok(response) => {
                                            if response.status().is_success() {
                                                debug!(peer = %peer_url_clone, "Successfully sent to internal peer");
                                            } else {
                                                warn!(peer = %peer_url_clone, status = %response.status(), "Internal peer responded with error");
                                            }
                                        }
                                        Err(e) => {
                                            warn!(peer = %peer_url_clone, error = ?e, "Failed to send to internal peer");
                                        }
                                    }
                                });
                            }
                            
                            // --- 2. Broadcast to PUBLIC peers (untrusted, no token) - only in production ---
                            if config.environment == "production" {
                                let public_peer_count = config.public_peers.len();
                                info!(symbol, public_peers = public_peer_count, "Broadcasting to public peers");
                                
                                for peer_url in config.public_peers.iter() {
                                    let client = gossip_client.clone();
                                    let payload = gossip_payload.clone();
                                    let url = format!("{}/public/gossip", peer_url);
                                    let peer_url_clone = peer_url.clone();
                                    
                                    tokio::spawn(async move {
                                        match client.post(&url).json(&payload).send().await {
                                            Ok(response) => {
                                                if response.status().is_success() {
                                                    debug!(peer = %peer_url_clone, "Successfully sent to public peer");
                                                } else {
                                                    warn!(peer = %peer_url_clone, status = %response.status(), "Public peer responded with error");
                                                }
                                            }
                                            Err(e) => {
                                                warn!(peer = %peer_url_clone, error = ?e, "Failed to send to public peer");
                                            }
                                        }
                                    });
                                }
                            } else {
                                debug!(environment = %config.environment, "Skipping public peer broadcast (not in production)");
                            }
                        }
                    } else {
                        warn!(symbol, "No data available for symbol");
                    }
                }
                
                drop(data_guard);
                info!(iteration = iteration_count, updated_symbols = ?updated_symbols, "Completed data update cycle");
            }
            Err(e) => {
                error!(iteration = iteration_count, error = ?e, "Failed to fetch batch data from VCI");
            }
        }
        
        debug!(interval = ?config.core_worker_interval, "Sleeping before next cycle");
        tokio::time::sleep(config.core_worker_interval).await;
    }
}

#[instrument(skip(data), fields(core_url = %core_network_url, refresh_interval = ?refresh_interval))]
async fn run_public_node_worker(data: SharedData, core_network_url: String, refresh_interval: Duration) {
    info!("Initializing public node worker");
    let http_client = ReqwestClient::new();
    let mut iteration_count = 0;
    
    loop {
        iteration_count += 1;
        debug!(iteration = iteration_count, "Starting core data sync cycle");
        
        let core_tickers_url = format!("{}/tickers", core_network_url);
        
        match http_client.get(&core_tickers_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<InMemoryData>().await {
                        Ok(core_data) => {
                            info!(iteration = iteration_count, symbols_count = core_data.len(), "Successfully fetched data from core network");
                            
                            let mut local_data_guard = data.lock().await;
                            let mut updated_symbols = Vec::new();
                            let mut new_symbols = Vec::new();
                            
                            for (symbol, core_ohlcv_vec) in core_data {
                                let local_entry = local_data_guard.entry(symbol.clone()).or_default();
                                
                                if let (Some(core_last), Some(local_last)) = (core_ohlcv_vec.last(), local_entry.last()) {
                                    if core_last.time > local_last.time {
                                        *local_entry = core_ohlcv_vec;
                                        updated_symbols.push(symbol.clone());
                                        debug!(symbol = %symbol, "Updated existing symbol with newer data");
                                    }
                                } else if local_entry.is_empty() {
                                    *local_entry = core_ohlcv_vec;
                                    new_symbols.push(symbol.clone());
                                    debug!(symbol = %symbol, "Added new symbol data");
                                }
                            }
                            
                            drop(local_data_guard);
                            info!(iteration = iteration_count, updated = ?updated_symbols, new = ?new_symbols, "Completed core data sync");
                        }
                        Err(e) => {
                            error!(iteration = iteration_count, error = ?e, "Failed to parse core network response as JSON");
                        }
                    }
                } else {
                    warn!(iteration = iteration_count, status = %response.status(), "Core network responded with error status");
                }
            }
            Err(e) => {
                error!(iteration = iteration_count, error = ?e, core_url = %core_tickers_url, "Failed to fetch data from core network");
            }
        }
        
        debug!(refresh_interval = ?refresh_interval, "Sleeping before next sync cycle");
        tokio::time::sleep(refresh_interval).await;
    }
}