use crate::config::{AppConfig, load_ticker_groups};
use crate::data_structures::{InMemoryData, SharedData, SharedOfficeHoursState, OfficeHoursState, is_within_office_hours, get_current_interval, SharedHealthStats, get_time_info, get_current_time, SharedEnhancedData};
use crate::analysis_service::AnalysisService;
use aipriceaction::{prelude::*, data::TimeRange};
use std::time::Duration;
use std::sync::Arc;
use reqwest::Client as ReqwestClient;
use rand::prelude::SliceRandom;
use tokio::sync::Mutex;
use chrono::Utc;
use tracing::{info, debug, warn, error};

pub async fn run(data: SharedData, enhanced_data: SharedEnhancedData, config: AppConfig, health_stats: SharedHealthStats) {
    if let Some(core_url) = &config.core_network_url {
        info!(%core_url, "Starting as public node worker");
        run_public_node_worker(data, core_url.clone(), config.public_refresh_interval, health_stats).await;
    } else {
        info!(environment = %config.environment, "Starting as core node worker");
        run_core_node_worker(data, enhanced_data, config, health_stats).await;
    }
}

async fn run_core_node_worker(data: SharedData, enhanced_data: SharedEnhancedData, config: AppConfig, health_stats: SharedHealthStats) {
    info!("Initializing core node worker");
    
    // Initialize office hours state
    let office_hours_state: SharedOfficeHoursState = Arc::new(Mutex::new(OfficeHoursState::default()));
    
    info!(
        enable_office_hours = config.enable_office_hours,
        office_hours_start = config.office_hours_config.default_office_hours.start_hour,
        office_hours_end = config.office_hours_config.default_office_hours.end_hour,
        timezone = config.office_hours_config.default_office_hours.timezone,
        core_interval_secs = config.core_worker_interval.as_secs(),
        non_office_interval_secs = config.non_office_hours_interval.as_secs(),
        "Office hours configuration loaded"
    );
    
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
    
    // Load ticker groups and combine all tickers into a single array
    let ticker_groups = load_ticker_groups();
    let mut all_tickers: Vec<String> = ticker_groups.0.values()
        .flat_map(|group_tickers| group_tickers.iter().cloned())
        .collect();
    
    // Add VNINDEX (Vietnam stock market index) to the ticker list
    all_tickers.push("VNINDEX".to_string());
    
    // Remove duplicates and shuffle
    all_tickers.sort();
    all_tickers.dedup();
    all_tickers.shuffle(&mut rand::rng());
    
    info!(total_tickers = all_tickers.len(), "Loaded and shuffled all tickers from ticker groups");
    debug!(first_10_tickers = ?all_tickers.iter().take(10).collect::<Vec<_>>(), "First 10 tickers after shuffle");
    
    let gossip_client = ReqwestClient::new();
    const BATCH_SIZE: usize = 10;
    let mut iteration_count = 0;
    let start_time = std::time::Instant::now();

    // Initialize analysis service for calculations
    let analysis_service: Option<Arc<AnalysisService>> = match AnalysisService::new() {
        Ok(service) => {
            info!("Analysis service initialized successfully");
            Some(Arc::new(service))
        }
        Err(e) => {
            error!(?e, "Failed to initialize analysis service - calculations will be disabled");
            None
        }
    };

    let mut last_calculation_time = std::time::Instant::now();

    loop {
        iteration_count += 1;
        
        // Check office hours and update state
        let is_office_hours = is_within_office_hours(&config.office_hours_config);
        let current_interval = get_current_interval(
            &config.office_hours_config,
            config.core_worker_interval,
            config.non_office_hours_interval,
            config.enable_office_hours
        );
        
        // Update office hours state
        {
            let mut state = office_hours_state.lock().await;
            let state_changed = state.is_office_hours != is_office_hours;
            state.is_office_hours = is_office_hours;
            state.current_interval = current_interval;
            state.last_check = std::time::Instant::now();
            
            if state_changed {
                info!(
                    iteration = iteration_count,
                    is_office_hours,
                    current_interval_secs = current_interval.as_secs(),
                    "Office hours status changed"
                );
            }
        }
        
        // Update health stats and check memory usage
        {
            let mut health = health_stats.lock().await;
            let data_guard = data.lock().await;
            let (current_time, debug_override) = get_time_info();
            
            // Calculate memory usage
            let memory_bytes = crate::data_structures::estimate_memory_usage(&*data_guard);
            let memory_mb = memory_bytes as f64 / (1024.0 * 1024.0);
            let memory_percent = (memory_bytes as f64 / crate::data_structures::MAX_MEMORY_BYTES as f64) * 100.0;
            
            health.is_office_hours = is_office_hours;
            health.current_interval_secs = current_interval.as_secs();
            health.iteration_count = iteration_count;
            health.uptime_secs = start_time.elapsed().as_secs();
            health.total_tickers_count = all_tickers.len();
            health.active_tickers_count = data_guard.len();
            health.memory_usage_bytes = memory_bytes;
            health.memory_usage_mb = memory_mb;
            health.memory_usage_percent = memory_percent;
            health.last_update_timestamp = Some(Utc::now().to_rfc3339());
            health.current_system_time = current_time;
            health.debug_time_override = debug_override;
            
            drop(data_guard);
            
            info!(
                iteration = iteration_count,
                memory_mb = format!("{:.2}", memory_mb),
                memory_percent = format!("{:.1}%", memory_percent),
                active_tickers = health.active_tickers_count,
                "Memory usage stats"
            );
        }
        
        debug!(
            iteration = iteration_count,
            is_office_hours,
            current_interval_secs = current_interval.as_secs(),
            "Starting data fetch cycle"
        );
        
        /*
        ████████████████████████████████████████████████████████████████████████████████
        ██                                                                            ██
        ██    🚧 VCI LIVE DATA PROCESSING TEMPORARILY DISABLED 🚧                    ██
        ██                                                                            ██
        ██    REASON: We are focusing on HISTORICAL data integration with CLI        ██
        ██             module. VCI is a 3rd party service for live data.             ██
        ██                                                                            ██
        ██    TODO: Re-enable this section once CLI historical integration           ██
        ██          is complete and we want to add live data updates.                ██
        ██                                                                            ██
        ██    CURRENT FOCUS:                                                          ██
        ██    - CLI module fetches CSV from GitHub (historical data)                 ██
        ██    - Enhanced calculations (money flow, MA scores)                        ██
        ██    - Background worker for periodic calculations                          ██
        ██                                                                            ██
        ████████████████████████████████████████████████████████████████████████████████
        */

        // COMMENTED OUT: VCI live data processing
        //
        // // Calculate date range for VCI API call (current date and 7 days ago)
        // let current_date = get_current_time();
        // let end_date = current_date.format("%Y-%m-%d").to_string();
        // let start_date = (current_date - chrono::Duration::days(7)).format("%Y-%m-%d").to_string();
        //
        // debug!(
        //     iteration = iteration_count,
        //     start_date = %start_date,
        //     end_date = %end_date,
        //     "Using dynamic date range for VCI API calls"
        // );

        // DISABLED: VCI processing - focusing on CLI historical data integration
        debug!(iteration = iteration_count, "VCI processing disabled - using CLI for enhanced data calculations");
        
        // Check memory usage and cleanup if needed
        {
            let mut data_guard = data.lock().await;
            let memory_bytes = crate::data_structures::estimate_memory_usage(&*data_guard);
            let memory_mb = memory_bytes as f64 / (1024.0 * 1024.0);
            
            if memory_bytes > crate::data_structures::MAX_MEMORY_BYTES {
                warn!(
                    memory_mb = format!("{:.2}", memory_mb),
                    limit_mb = crate::data_structures::MAX_MEMORY_MB,
                    "Memory limit exceeded, cleaning up old data"
                );
                
                let (cleaned_symbols, cleaned_data_points) = crate::data_structures::cleanup_old_data(&mut *data_guard);
                let new_memory_bytes = crate::data_structures::estimate_memory_usage(&*data_guard);
                let new_memory_mb = new_memory_bytes as f64 / (1024.0 * 1024.0);
                
                info!(
                    cleaned_symbols,
                    cleaned_data_points,
                    old_memory_mb = format!("{:.2}", memory_mb),
                    new_memory_mb = format!("{:.2}", new_memory_mb),
                    "Memory cleanup completed"
                );
            } else {
                debug!(
                    memory_mb = format!("{:.2}", memory_mb),
                    limit_mb = crate::data_structures::MAX_MEMORY_MB,
                    "Memory usage within limits"
                );
            }
        }

        // Update enhanced data with calculations every 30 seconds (for testing)
        const CALCULATION_INTERVAL: Duration = Duration::from_secs(30); // 30 seconds for testing
        if analysis_service.is_some() && last_calculation_time.elapsed() > CALCULATION_INTERVAL {
            info!(iteration = iteration_count, "Starting calculation update for enhanced data");

            if let Some(ref service) = analysis_service {
                match update_enhanced_data(
                    enhanced_data.clone(),
                    Arc::clone(service),
                    all_tickers.clone()
                ).await {
                    Ok(count) => {
                        info!(enhanced_tickers = count, "Successfully updated enhanced data calculations");
                        last_calculation_time = std::time::Instant::now();
                    }
                    Err(e) => {
                        error!(?e, "Failed to update enhanced data calculations");
                    }
                }
            }
        }

        debug!(interval = ?current_interval, "Sleeping before next full cycle");
        tokio::time::sleep(current_interval).await;
        
        // Re-shuffle for next iteration
        all_tickers.shuffle(&mut rand::rng());
        debug!("Reshuffled tickers for next iteration");
    }
}

async fn update_enhanced_data(
    enhanced_data: SharedEnhancedData,
    analysis_service: Arc<AnalysisService>,
    tickers: Vec<String>,
) -> Result<usize, Box<dyn std::error::Error>> {
    // Define date range for calculations - use 1 year for enhanced data to avoid timeout issues
    // Using ALL range would take too long and cause timeouts in the background worker
    let date_range = DateRangeConfig::new(TimeRange::OneYear);

    // Fetch and calculate enhanced data
    tracing::info!("About to call fetch_and_calculate for {} tickers", tickers.len());
    let calculated_data = analysis_service
        .fetch_and_calculate(tickers, date_range)
        .await?;

    let ticker_count = calculated_data.len();
    tracing::info!("fetch_and_calculate completed successfully, got data for {} tickers", ticker_count);

    // Update shared enhanced data
    tracing::info!("About to acquire lock for storing enhanced data");
    {
        let mut data_guard = enhanced_data.lock().await;
        tracing::info!("Lock acquired, storing enhanced data for {} tickers, {} total data points",
                      calculated_data.len(),
                      calculated_data.values().map(|v| v.len()).sum::<usize>());
        *data_guard = calculated_data;
        let stored_count = data_guard.len();
        tracing::info!("Enhanced data stored successfully, {} tickers now available in shared state", stored_count);
    }
    tracing::info!("Lock released, enhanced data storage complete");

    Ok(ticker_count)
}

async fn run_public_node_worker(data: SharedData, core_network_url: String, refresh_interval: Duration, _health_stats: SharedHealthStats) {
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