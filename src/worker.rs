use crate::config::{AppConfig, load_ticker_groups};
use crate::data_structures::{InMemoryData, SharedData, SharedOfficeHoursState, OfficeHoursState, is_within_office_hours, get_current_interval, SharedHealthStats, get_time_info, SharedEnhancedData};
use crate::analysis_service::AnalysisService;
use aipriceaction::{prelude::*, data::TimeRange, state_machine::ClientDataStateMachine};
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

    // Initialize CLI state machine for calculations
    let state_machine = Arc::new(Mutex::new(ClientDataStateMachine::new()));
    info!("CLI state machine initialized successfully");
    
    // Start state machine in background
    let state_machine_clone = Arc::clone(&state_machine);
    tokio::spawn(async move {
        info!("Starting CLI state machine in background");
        if let Err(e) = state_machine_clone.lock().await.start().await {
            error!(?e, "CLI state machine failed");
        }
    });
    
    let mut last_enhanced_update = std::time::Instant::now() - std::time::Duration::from_secs(60); // Start with 60 seconds ago to trigger first update immediately

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

        // Update enhanced data from state machine every 10 seconds
        const ENHANCED_UPDATE_INTERVAL: Duration = Duration::from_secs(10); // 10 seconds
        if last_enhanced_update.elapsed() > ENHANCED_UPDATE_INTERVAL {
            info!(iteration = iteration_count, "Starting enhanced data update from state machine");

            match update_enhanced_data_from_state_machine(
                enhanced_data.clone(),
                Arc::clone(&state_machine)
            ).await {
                Ok(count) => {
                    info!(enhanced_tickers = count, "Successfully updated enhanced data from state machine");
                    last_enhanced_update = std::time::Instant::now();
                    
                    // Test: Check if enhanced data is actually stored
                    {
                        let test_data = enhanced_data.lock().await;
                        info!(stored_dates = test_data.len(), "Enhanced data storage verification");
                        if !test_data.is_empty() {
                            let sample_dates: Vec<String> = test_data.keys().take(3).cloned().collect();
                            info!(sample_dates = ?sample_dates, "Sample stored dates");
                        }
                    }
                }
                Err(e) => {
                    error!(?e, "Failed to update enhanced data from state machine");
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

async fn update_enhanced_data_from_state_machine(
    enhanced_data: SharedEnhancedData,
    state_machine: Arc<Mutex<ClientDataStateMachine>>,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Starting enhanced data update from CLI state machine");
    
    // Get data from state machine cache
    let (money_flow_data, ma_score_data, ticker_data) = {
        let state_machine_guard = state_machine.lock().await;
        let context = state_machine_guard.get_context();
        let context_guard = context.read().await;
        let cache = &context_guard.cache;
        
        // Extract money flow data
        let money_flow_data = cache.get_money_flow_data(None).unwrap_or_default();
        
        // Extract MA score data
        let ma_score_data = cache.get_ma_score_data(None, None).unwrap_or_default();
        
        // Extract ticker data for OHLCV values
        let client_cache = cache.get_cache();
        let ticker_data = client_cache.ticker_data.clone();
        
        (money_flow_data, ma_score_data, ticker_data)
    };
    
    tracing::info!(
        money_flow_tickers = money_flow_data.len(),
        ma_score_tickers = ma_score_data.len(),
        ticker_count = ticker_data.len(),
        "Extracted data from state machine cache"
    );
    
    // Debug: Log first few tickers to verify data structure
    if !money_flow_data.is_empty() {
        tracing::info!(
            first_money_flow_tickers = ?money_flow_data.iter().take(3).map(|mf| &mf.ticker).collect::<Vec<_>>(),
            "Sample money flow tickers"
        );
    }
    if !ma_score_data.is_empty() {
        tracing::info!(
            first_ma_score_tickers = ?ma_score_data.iter().take(3).map(|ma| &ma.ticker).collect::<Vec<_>>(),
            "Sample MA score tickers"
        );
    }
    if !ticker_data.is_empty() {
        tracing::info!(
            first_ticker_data_keys = ?ticker_data.keys().take(3).collect::<Vec<_>>(),
            "Sample ticker data keys"
        );
    }
    
    // Convert CLI data structures to enhanced data structures
    let mut enhanced_data_map = std::collections::HashMap::new();
    
    // Group money flow data by date
    let mut money_flow_by_date = std::collections::HashMap::new();
    for mf_ticker in money_flow_data {
        for (date, money_flow_value) in &mf_ticker.daily_data {
            money_flow_by_date
                .entry(date.clone())
                .or_insert_with(Vec::new)
                .push(mf_ticker.clone());
        }
    }
    
    // Group MA score data by date
    let mut ma_score_by_date = std::collections::HashMap::new();
    for ma_ticker in ma_score_data {
        // Collect all dates from MA score data
        let all_dates: Vec<String> = ma_ticker.ma10_scores.keys()
            .chain(ma_ticker.ma20_scores.keys())
            .chain(ma_ticker.ma50_scores.keys())
            .cloned()
            .collect();
        
        for date in all_dates {
            ma_score_by_date
                .entry(date.clone())
                .or_insert_with(Vec::new)
                .push(ma_ticker.clone());
        }
    }
    
    // Process money flow data
    for (date, money_flow_tickers) in money_flow_by_date {
        let mut enhanced_tickers = Vec::new();
        
        for mf_ticker in money_flow_tickers {
            // Skip VNINDEX in enhanced calculations
            if mf_ticker.ticker == "VNINDEX" {
                continue;
            }
            
            // Get corresponding ticker data for OHLCV values
            if let Some(ticker_entry) = ticker_data.get(&mf_ticker.ticker) {
                // Find the data point for this specific date
                if let Some(ohlcv_point) = ticker_entry.data.iter().find(|p| p.time == date) {
                    let enhanced_ticker = crate::data_structures::EnhancedTickerData {
                        date: date.clone(),
                        open: ohlcv_point.open,
                        high: ohlcv_point.high,
                        low: ohlcv_point.low,
                        close: ohlcv_point.close,
                        volume: ohlcv_point.volume,
                        
                        // Moving averages (will be calculated from MA score data)
                        ma10: None,
                        ma20: None,
                        ma50: None,
                        
                        // Money flow metrics
                        money_flow: mf_ticker.daily_data.get(&date).copied(),
                        af: mf_ticker.activity_flow_data.get(&date).copied(),
                        df: mf_ticker.dollar_flow_data.get(&date).copied(),
                        ts: Some(mf_ticker.trend_score),
                        
                        // MA scores (will be populated from MA score data)
                        score10: None,
                        score20: None,
                        score50: None,
                    };
                    
                    enhanced_tickers.push(enhanced_ticker);
                }
            }
        }
        
        if !enhanced_tickers.is_empty() {
            enhanced_data_map.insert(date, enhanced_tickers);
        }
    }
    
    // Process MA score data and merge with existing enhanced data
    for (date, ma_score_tickers) in ma_score_by_date {
        // Get existing enhanced tickers for this date, or create new ones
        let existing_tickers = enhanced_data_map.remove(&date).unwrap_or_default();
        
        let mut updated_tickers = Vec::new();
        
        // Create a map of existing tickers by OHLCV for quick lookup
        let mut existing_map = std::collections::HashMap::new();
        for ticker in existing_tickers {
            let key = format!("{}:{}:{}:{}", ticker.open, ticker.high, ticker.low, ticker.close);
            existing_map.insert(key, ticker);
        }
        
        for ma_ticker in ma_score_tickers {
            // Skip VNINDEX in enhanced calculations
            if ma_ticker.ticker == "VNINDEX" {
                continue;
            }
            
            // Try to find corresponding ticker data for OHLCV values
            if let Some(ticker_entry) = ticker_data.get(&ma_ticker.ticker) {
                if let Some(ohlcv_point) = ticker_entry.data.iter().find(|p| p.time == date) {
                    let key = format!("{}:{}:{}:{}", ohlcv_point.open, ohlcv_point.high, ohlcv_point.low, ohlcv_point.close);
                    
                    if let Some(mut existing_ticker) = existing_map.remove(&key) {
                        // Update existing ticker with MA scores and moving averages
                        existing_ticker.score10 = ma_ticker.ma10_scores.get(&date).copied();
                        existing_ticker.score20 = ma_ticker.ma20_scores.get(&date).copied();
                        existing_ticker.score50 = ma_ticker.ma50_scores.get(&date).copied();
                        
                        // Extract moving averages from debug data if available
                        if let Some(debug_data) = &ma_ticker.debug_data {
                            if let Some(debug) = debug_data.get(&date) {
                                existing_ticker.ma10 = debug.ma10_value;
                                existing_ticker.ma20 = debug.ma20_value;
                                existing_ticker.ma50 = debug.ma50_value;
                            }
                        }
                        
                        updated_tickers.push(existing_ticker);
                    } else {
                        // Create new enhanced ticker
                        let enhanced_ticker = crate::data_structures::EnhancedTickerData {
                            date: date.clone(),
                            open: ohlcv_point.open,
                            high: ohlcv_point.high,
                            low: ohlcv_point.low,
                            close: ohlcv_point.close,
                            volume: ohlcv_point.volume,
                            
                            // Moving averages from debug data
                            ma10: ma_ticker.debug_data
                                .as_ref()
                                .and_then(|debug| debug.get(&date))
                                .and_then(|debug| debug.ma10_value),
                            ma20: ma_ticker.debug_data
                                .as_ref()
                                .and_then(|debug| debug.get(&date))
                                .and_then(|debug| debug.ma20_value),
                            ma50: ma_ticker.debug_data
                                .as_ref()
                                .and_then(|debug| debug.get(&date))
                                .and_then(|debug| debug.ma50_value),
                            
                            // Money flow metrics (not available from MA score data alone)
                            money_flow: None,
                            af: None,
                            df: None,
                            ts: Some(ma_ticker.trend_score),
                            
                            // MA scores
                            score10: ma_ticker.ma10_scores.get(&date).copied(),
                            score20: ma_ticker.ma20_scores.get(&date).copied(),
                            score50: ma_ticker.ma50_scores.get(&date).copied(),
                        };
                        
                        updated_tickers.push(enhanced_ticker);
                    }
                }
            }
        }
        
        // Add back any remaining existing tickers that weren't updated
        updated_tickers.extend(existing_map.into_values());
        
        if !updated_tickers.is_empty() {
            enhanced_data_map.insert(date, updated_tickers);
        }
    }
    
    // Store the count before moving the map
    let enhanced_data_count = enhanced_data_map.len();
    let total_data_points = enhanced_data_map.values().map(|v| v.len()).sum::<usize>();
    
    // Update shared enhanced data
    tracing::info!("About to acquire lock for storing enhanced data");
    {
        let mut data_guard = enhanced_data.lock().await;
        tracing::info!(
            "Lock acquired, storing enhanced data for {} dates, {} total data points",
            enhanced_data_count,
            total_data_points
        );
        *data_guard = enhanced_data_map;
        let stored_count = data_guard.len();
        tracing::info!("Enhanced data stored successfully, {} dates now available in shared state", stored_count);
    }
    tracing::info!("Lock released, enhanced data storage complete");
    
    Ok(enhanced_data_count)
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