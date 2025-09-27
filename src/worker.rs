use crate::config::{AppConfig, load_ticker_groups};
use crate::data_structures::{InMemoryData, SharedData, SharedOfficeHoursState, OfficeHoursState, is_within_office_hours, get_current_interval, SharedHealthStats, get_time_info, SharedEnhancedData, SharedCLICacheData, SharedCLICache};
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
    println!("🚀 WORKER FUNCTION CALLED - DEBUG");
    println!("🚀 WORKER FUNCTION ENTRY POINT - INSIDE FUNCTION");
    info!("🚀 Worker function started");
    println!("🚀 AFTER INFO STATEMENT");
    info!("🔍 DEBUG: About to check core_url");
    println!("🚀 AFTER CORE_URL DEBUG");
    info!("🔍 DEBUG: core_network_url = {:?}", config.core_network_url);
    println!("🚀 AFTER CORE_NETWORK_URL PRINT");
    info!("🔍 DEBUG: Reached if-let check");
    println!("🚀 AFTER IF-LET CHECK");
    info!("🔍 DEBUG: Function entry point reached successfully");
    println!("🚀 AFTER FUNCTION ENTRY SUCCESS");
    println!("🚀 ABOUT TO ENTER IF-LET BLOCK");
    println!("🔍 DEBUG: config.core_network_url value: {:?}", config.core_network_url);
    println!("🔍 DEBUG: About to enter if-let block - no panic yet");
    println!("🔍 DEBUG: Right before if-let statement");
    println!("🔍 DEBUG: Checking if core_network_url is Some or None");
    
    if config.core_network_url.is_some() {
        println!("🔍 DEBUG: core_network_url is Some");
        if let Some(core_url) = &config.core_network_url {
            println!("🚀 INSIDE IF-LET BLOCK (core_url: {:?})", core_url);
            info!(%core_url, "Starting as public node worker");
            info!("🔍 DEBUG: This is a public node, calling run_public_node_worker");
            run_public_node_worker(data, core_url.clone(), config.public_refresh_interval, health_stats).await;
        }
    } else {
        println!("🔍 DEBUG: core_network_url is None, executing else branch");
        info!(environment = %config.environment, "Starting as core node worker");
        info!("🔍 DEBUG: This is a core node, calling run_core_node_worker");
        run_core_node_worker(data, enhanced_data, config, health_stats).await;
    }
    info!("🔍 DEBUG: Worker function completed if-let block");
}

async fn run_core_node_worker(data: SharedData, enhanced_data: SharedEnhancedData, config: AppConfig, health_stats: SharedHealthStats) {
    println!("🔍 DEBUG: run_core_node_worker function called");
    println!("🔍 DEBUG: About to initialize office hours state");
    info!("🚀 Initializing core node worker - START");
    info!("🔍 DEBUG: Entered run_core_node_worker function");
    
    // Initialize office hours state
    println!("🔍 DEBUG: Creating office_hours_state");
    let office_hours_state: SharedOfficeHoursState = Arc::new(Mutex::new(OfficeHoursState::default()));
    println!("🔍 DEBUG: office_hours_state created successfully");
    println!("🔍 DEBUG: About to log office hours configuration");
    
    info!(
        enable_office_hours = config.enable_office_hours,
        office_hours_start = config.office_hours_config.default_office_hours.start_hour,
        office_hours_end = config.office_hours_config.default_office_hours.end_hour,
        timezone = config.office_hours_config.default_office_hours.timezone,
        core_interval_secs = config.core_worker_interval.as_secs(),
        non_office_interval_secs = config.non_office_hours_interval.as_secs(),
        "Office hours configuration loaded"
    );
    println!("🔍 DEBUG: Office hours configuration logged successfully");
    
    // Initialize VCI client for live data processing
    println!("🔍 DEBUG: About to initialize VCI client");
    println!("🔍 DEBUG: Calling VciClient::new(true, 30)");
    
    let mut vci_client = match std::panic::catch_unwind(|| {
        crate::vci::VciClient::new(true, 30)
    }) {
        Ok(Ok(client)) => {
            println!("🔍 DEBUG: VCI client initialized successfully");
            info!("VCI client initialized successfully");
            client
        }
        Ok(Err(e)) => {
            println!("🔍 DEBUG: VCI client initialization failed: {:?}", e);
            error!(?e, "Failed to initialize VCI client");
            return;
        }
        Err(panic_info) => {
            println!("🔍 DEBUG: VCI client initialization panicked: {:?}", panic_info);
            error!("VCI client initialization panicked: {:?}", panic_info);
            // Continue without VCI client - we'll use CLI data only
            println!("🔍 DEBUG: Continuing without VCI client");
            return;
        }
    };
    
    println!("🔍 DEBUG: About to load ticker groups");
    // Load ticker groups and combine all tickers into a single array
    let ticker_groups = load_ticker_groups();
    println!("🔍 DEBUG: Ticker groups loaded successfully");
    println!("🔍 DEBUG: About to process tickers");
    let mut all_tickers: Vec<String> = ticker_groups.0.values()
        .flat_map(|group_tickers| group_tickers.iter().cloned())
        .collect();
    println!("🔍 DEBUG: Tickers collected into vector");
    
    // Add VNINDEX (Vietnam stock market index) to the ticker list
    println!("🔍 DEBUG: Adding VNINDEX to ticker list");
    all_tickers.push("VNINDEX".to_string());
    println!("🔍 DEBUG: VNINDEX added successfully");
    
    // Remove duplicates and shuffle
    println!("🔍 DEBUG: About to sort tickers");
    all_tickers.sort();
    println!("🔍 DEBUG: Tickers sorted successfully");
    println!("🔍 DEBUG: About to dedup tickers");
    all_tickers.dedup();
    println!("🔍 DEBUG: Tickers deduped successfully");
    println!("🔍 DEBUG: About to shuffle tickers");
    all_tickers.shuffle(&mut rand::rng());
    println!("🔍 DEBUG: Tickers shuffled successfully");
    
    println!("🔍 DEBUG: About to log ticker info");
    info!(total_tickers = all_tickers.len(), "Loaded and shuffled all tickers from ticker groups");
    debug!(first_10_tickers = ?all_tickers.iter().take(10).collect::<Vec<_>>(), "First 10 tickers after shuffle");
    println!("🔍 DEBUG: Ticker info logged successfully");
    
    println!("🔍 DEBUG: About to create gossip client");
    let gossip_client = ReqwestClient::new();
    println!("🔍 DEBUG: Gossip client created successfully");
    println!("🔍 DEBUG: About to define BATCH_SIZE constant");
    const BATCH_SIZE: usize = 10;
    println!("🔍 DEBUG: BATCH_SIZE defined successfully");
    let mut iteration_count = 0;
    println!("🔍 DEBUG: iteration_count initialized successfully");
    let start_time = std::time::Instant::now();
    println!("🔍 DEBUG: start_time initialized successfully");

    // Initialize shared CLI cache for lock-free access
    println!("🔍 DEBUG: About to initialize shared CLI cache");
    let shared_cli_cache: SharedCLICacheData = Arc::new(Mutex::new(SharedCLICache::default()));
    println!("🔍 DEBUG: Shared CLI cache created successfully");
    info!("Shared CLI cache initialized successfully");
    println!("🔍 DEBUG: Shared CLI cache info logged successfully");
    
    // Debug: Initialize shared cache with minimal test data
    println!("🔍 DEBUG: About to initialize shared cache with test data");
    {
        println!("🔍 DEBUG: About to acquire cache lock");
        let mut cache = shared_cli_cache.lock().await;
        println!("🔍 DEBUG: Cache lock acquired successfully");
        cache.version = 1;
        println!("🔍 DEBUG: Cache version set to 1");
        cache.last_updated = Some(Utc::now());
        println!("🔍 DEBUG: Cache last_updated set successfully");
        info!("🔧 DEBUG: Initialized shared cache with version 1");
        println!("🔍 DEBUG: Cache initialization info logged");
    }
    println!("🔍 DEBUG: Cache initialization block completed");
    
    // Create and start CLI state machine for real data processing
    println!("🔍 DEBUG: About to create CLI state machine");
    info!("🔧 DEBUG: Creating CLI state machine for real data processing");
    let state_machine_instance = ClientDataStateMachine::new();
    println!("🔍 DEBUG: CLI state machine created successfully");
    info!("CLI state machine initialized");
    println!("🔍 DEBUG: CLI state machine info logged successfully");
    
    // Wrap in Arc<Mutex<T>> for thread-safe access
    println!("🔍 DEBUG: About to wrap state machine in Arc<Mutex>");
    let state_machine = Arc::new(Mutex::new(state_machine_instance));
    println!("🔍 DEBUG: State machine wrapped successfully");
    let state_machine_for_monitoring = Arc::clone(&state_machine);
    println!("🔍 DEBUG: State machine monitoring clone created successfully");
    
    // Start the state machine using the shared method
    println!("🔍 DEBUG: About to create state machine start clone");
    let state_machine_for_start = Arc::clone(&state_machine);
    println!("🔍 DEBUG: State machine start clone created successfully");
    println!("🔍 DEBUG: About to spawn state machine task");
    tokio::spawn(async move {
        println!("🔍 DEBUG: State machine task spawned successfully");
            info!("🚀 Starting CLI state machine processing");
            
            // Start the state machine using the shared method
            match state_machine_for_start.lock().await.start_shared().await {
                Ok(_) => info!("✅ State machine started successfully"),
                Err(e) => error!("❌ Failed to start state machine: {}", e),
            }
            
            // Run periodic ticks
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                
                if let Err(e) = state_machine_for_start.lock().await.tick_shared().await {
                    error!("❌ State machine tick failed: {}", e);
                }
            }
        });
    
    // Monitor state machine progress
    tokio::spawn(async move {
        let mut tick_count = 0;
        loop {
            tick_count += 1;
            if tick_count % 10 == 1 {
                info!("🔄 State machine monitor tick #{}", tick_count);
            }
            
            // Check state machine status
            let guard = state_machine_for_monitoring.lock().await;
            let current_state = guard.current_state_name().await;
            let is_ready = guard.is_ready().await;
            
            if tick_count % 10 == 1 {
                info!("📍 Current state: {} (ready: {})", current_state, is_ready);
            }
            
            // Log when we reach READY state
            if is_ready && tick_count % 30 == 0 {
                info!("🎉 State machine reached READY state!");
            }
            
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });
    
    let enhanced_data_clone = enhanced_data.clone();
    let shared_cli_cache_clone = Arc::clone(&shared_cli_cache);
    
    // Start independent enhanced data update loop
    tokio::spawn(async move {
        info!("Starting independent enhanced data update loop");
        let mut last_enhanced_update = std::time::Instant::now() - std::time::Duration::from_secs(60); // Start with 60 seconds ago to trigger first update immediately
        
        loop {
            // Update enhanced data from state machine every 10 seconds
            const ENHANCED_UPDATE_INTERVAL: Duration = Duration::from_secs(10); // 10 seconds
            if last_enhanced_update.elapsed() > ENHANCED_UPDATE_INTERVAL {
                info!("Starting enhanced data update from shared CLI cache");

                match update_enhanced_data_from_state_machine(
                    enhanced_data_clone.clone(),
                    shared_cli_cache_clone.clone()
                ).await {
                    Ok(count) => {
                        info!(enhanced_tickers = count, "Successfully updated enhanced data from state machine");
                        last_enhanced_update = std::time::Instant::now();
                        
                        // Test: Check if enhanced data is actually stored
                        {
                            let test_data = enhanced_data_clone.lock().await;
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
            
            // Sleep for a short interval before checking again
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });
    
    // Start shared cache sync loop
    let shared_cli_cache_sync = Arc::clone(&shared_cli_cache);
    let state_machine_sync = Arc::clone(&state_machine);
    tokio::spawn(async move {
        info!("Starting shared CLI cache sync loop");
        let mut last_sync = std::time::Instant::now();
        let mut iteration_count = 0;
        
        info!("About to enter shared cache sync loop");
        loop {
            iteration_count += 1;
            info!(iteration = iteration_count, elapsed = ?last_sync.elapsed(), "Shared cache sync loop tick");
            
            // Sync shared cache from state machine every 30 seconds
            const SYNC_INTERVAL: Duration = Duration::from_secs(30);
            if last_sync.elapsed() > SYNC_INTERVAL {
                info!("Starting shared CLI cache sync from state machine");
                
                if let Err(e) = update_shared_cli_cache_from_state_machine(
                    shared_cli_cache_sync.clone(),
                    Arc::clone(&state_machine_sync)
                ).await {
                    error!(?e, "Failed to sync shared CLI cache from state machine");
                }
                
                last_sync = std::time::Instant::now();
            }
            
            // Sleep for a short interval before checking again
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

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
        
        // VCI LIVE DATA PROCESSING - RE-ENABLED
        // Calculate date range for VCI API call (current date and 7 days ago)
        let current_date = chrono::Utc::now();
        let end_date = current_date.format("%Y-%m-%d").to_string();
        let start_date = (current_date - chrono::Duration::days(7)).format("%Y-%m-%d").to_string();

        debug!(
            iteration = iteration_count,
            start_date = %start_date,
            end_date = %end_date,
            "Using dynamic date range for VCI API calls"
        );

        // Process VCI live data for tickers in batches
        let batch_size = 20;
        for (batch_index, ticker_batch) in all_tickers.chunks(batch_size).enumerate() {
            debug!(
                iteration = iteration_count,
                batch = batch_index,
                batch_size = ticker_batch.len(),
                "Processing VCI batch"
            );

            match vci_client.get_batch_history(ticker_batch, &start_date, Some(&end_date), "1D").await {
                Ok(vci_results) => {
                    let mut successful_tickers = 0;
                    let mut failed_tickers = 0;

                    for (symbol, ohlcv_data) in vci_results {
                        match ohlcv_data {
                            Some(vci_data) => {
                                let data_points = vci_data.len();
                                if !vci_data.is_empty() {
                                    // Store VCI data in shared data structure
                                    let mut data_guard = data.lock().await;
                                    data_guard.insert(symbol.clone(), vci_data);
                                    successful_tickers += 1;
                                    
                                    debug!(
                                        symbol = %symbol,
                                        data_points = data_points,
                                        "Successfully fetched VCI data"
                                    );
                                } else {
                                    debug!(symbol = %symbol, "No VCI data available");
                                    failed_tickers += 1;
                                }
                            }
                            None => {
                                debug!(symbol = %symbol, "Failed to fetch VCI data");
                                failed_tickers += 1;
                            }
                        }
                    }

                    info!(
                        iteration = iteration_count,
                        batch = batch_index,
                        successful = successful_tickers,
                        failed = failed_tickers,
                        "VCI batch processing completed"
                    );
                }
                Err(e) => {
                    error!(
                        iteration = iteration_count,
                        batch = batch_index,
                        error = ?e,
                        "Failed to fetch VCI batch data"
                    );
                }
            }

            // Small delay between batches to respect rate limits
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        info!(iteration = iteration_count, "VCI live data processing completed");
        
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

        debug!(interval = ?current_interval, "Sleeping before next full cycle");
        tokio::time::sleep(current_interval).await;
        
        // Re-shuffle for next iteration
        all_tickers.shuffle(&mut rand::rng());
        debug!("Reshuffled tickers for next iteration");
    }
}

async fn update_enhanced_data_from_state_machine(
    enhanced_data: SharedEnhancedData,
    shared_cli_cache: SharedCLICacheData,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Starting enhanced data update from CLI state machine");
    
    // Get data from shared CLI cache (no state machine locks needed)
    let (money_flow_data, ma_score_data, ticker_data, cache_version) = {
        tracing::info!("Accessing shared CLI cache for data extraction");
        
        let cache_guard = shared_cli_cache.lock().await;
        tracing::info!("Shared CLI cache accessed successfully");
        
        // Extract money flow data
        let money_flow_data = cache_guard.money_flow_data.values()
            .flat_map(|v| v.clone())
            .collect::<Vec<_>>();
        
        // Extract MA score data
        let ma_score_data = cache_guard.ma_score_data.values()
            .flat_map(|v| v.clone())
            .collect::<Vec<_>>();
        
        // Extract ticker data
        let ticker_data = cache_guard.ticker_data.clone();
        
        // Get cache version for tracking
        let cache_version = cache_guard.version;
        
        (money_flow_data, ma_score_data, ticker_data, cache_version)
    };
    
    // Check if we have meaningful data to process
    if money_flow_data.is_empty() && ma_score_data.is_empty() && ticker_data.is_empty() {
        tracing::warn!("No data available in shared CLI cache - CLI may still be loading");
        return Ok(0);
    }
    
    tracing::info!(
        money_flow_tickers = money_flow_data.len(),
        ma_score_tickers = ma_score_data.len(),
        ticker_count = ticker_data.len(),
        cache_version = cache_version,
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
    
    // Group money flow data by symbol
    let mut money_flow_by_symbol = std::collections::HashMap::new();
    for mf_ticker in money_flow_data {
        money_flow_by_symbol
            .entry(mf_ticker.ticker.clone())
            .or_insert_with(Vec::new)
            .push(mf_ticker);
    }
    
    // Group MA score data by symbol
    let mut ma_score_by_symbol = std::collections::HashMap::new();
    for ma_ticker in ma_score_data {
        ma_score_by_symbol
            .entry(ma_ticker.ticker.clone())
            .or_insert_with(Vec::new)
            .push(ma_ticker);
    }
    
    // Process each symbol that has either money flow or MA score data
    let all_symbols: std::collections::HashSet<String> = money_flow_by_symbol.keys()
        .chain(ma_score_by_symbol.keys())
        .cloned()
        .collect();
    
    for symbol in all_symbols {
        // Skip VNINDEX in enhanced calculations
        if symbol == "VNINDEX" {
            continue;
        }
        
        let mut enhanced_tickers = Vec::new();
        
        // Get money flow data for this symbol
        if let Some(mf_tickers) = money_flow_by_symbol.get(&symbol) {
            for mf_ticker in mf_tickers {
                // Get corresponding ticker data for OHLCV values
                if let Some(ticker_entry) = ticker_data.get(&symbol) {
                    for (date, money_flow_value) in &mf_ticker.daily_data {
                        if let Some(ohlcv_point) = ticker_entry.data.iter().find(|p| p.time == *date) {
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
                                money_flow: Some(*money_flow_value),
                                af: mf_ticker.activity_flow_data.get(date).copied(),
                                df: mf_ticker.dollar_flow_data.get(date).copied(),
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
            }
        }
        
        // Get MA score data for this symbol and merge with existing data
        if let Some(ma_tickers) = ma_score_by_symbol.get(&symbol) {
            for ma_ticker in ma_tickers {
                // Get corresponding ticker data for OHLCV values
                if let Some(ticker_entry) = ticker_data.get(&symbol) {
                    // Collect all dates from MA score data
                    let all_dates: std::collections::HashSet<String> = ma_ticker.ma10_scores.keys()
                        .chain(ma_ticker.ma20_scores.keys())
                        .chain(ma_ticker.ma50_scores.keys())
                        .cloned()
                        .collect();
                    
                    for date in all_dates {
                        if let Some(ohlcv_point) = ticker_entry.data.iter().find(|p| p.time == date) {
                            // Check if we already have an enhanced ticker for this date
                            let existing_ticker = enhanced_tickers.iter_mut()
                                .find(|t| t.date == date);
                            
                            if let Some(ticker) = existing_ticker {
                                // Update existing ticker with MA scores and moving averages
                                ticker.score10 = ma_ticker.ma10_scores.get(&date).copied();
                                ticker.score20 = ma_ticker.ma20_scores.get(&date).copied();
                                ticker.score50 = ma_ticker.ma50_scores.get(&date).copied();
                                
                                // Extract moving averages from debug data if available
                                if let Some(debug_data) = &ma_ticker.debug_data {
                                    if let Some(debug) = debug_data.get(&date) {
                                        ticker.ma10 = debug.ma10_value;
                                        ticker.ma20 = debug.ma20_value;
                                        ticker.ma50 = debug.ma50_value;
                                    }
                                }
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
                                
                                enhanced_tickers.push(enhanced_ticker);
                            }
                        }
                    }
                }
            }
        }
        
        // Sort enhanced tickers by date
        enhanced_tickers.sort_by(|a, b| a.date.cmp(&b.date));
        
        if !enhanced_tickers.is_empty() {
            enhanced_data_map.insert(symbol, enhanced_tickers);
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

/// Update shared CLI cache from state machine (called periodically)
async fn update_shared_cli_cache_from_state_machine(
    shared_cli_cache: SharedCLICacheData,
    state_machine: Arc<Mutex<ClientDataStateMachine>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("🚀 Updating shared CLI cache from state machine");
    tracing::info!("🔍 Function entry point reached");
    tracing::info!("🔍 DEBUG: Starting cache sync process");
    
    // Check if state machine is ready and extract real data
    let (is_ready, current_state) = {
        let guard = state_machine.lock().await;
        let ready = guard.is_ready().await;
        let state = guard.current_state_name().await;
        tracing::info!("🔍 State machine ready check: {}", ready);
        tracing::info!("🔍 State machine current state: {}", state);
        (ready, state)
    };
    
    if !is_ready {
        tracing::warn!("⚠️ State machine not ready yet, skipping cache sync");
        return Ok(());
    }
    
    // Extract real data from state machine
    tracing::info!("🔧 DEBUG: Extracting real data from state machine");
    
    // Extract real data from state machine using shared methods
    tracing::info!("🔧 DEBUG: Extracting real data from state machine using shared methods");
    
    // Extract real data using shared methods that work with Arc<Mutex<T>>
    let (money_flow_data, ma_score_data, ticker_data) = {
        let guard = state_machine.lock().await;
        
        // Call the shared methods that work with Arc<Mutex<T>>
        let money_flow_data = guard.get_money_flow_data_shared().await;
        let ma_score_data = guard.get_ma_score_data_shared().await;
        let ticker_data = guard.get_ticker_data_shared().await;
        
        tracing::info!("🔧 REAL DATA: money_flow: {} entries, ma_scores: {} entries, ticker_data: {} symbols", 
                      money_flow_data.as_ref().map_or(0, |d| d.len()),
                      ma_score_data.as_ref().map_or(0, |d| d.len()),
                      ticker_data.len());
        
        (money_flow_data, ma_score_data, ticker_data)
    };
    
    let has_money_flow = money_flow_data.is_some();
    let has_ma_scores = ma_score_data.is_some();
    let has_ticker_data = !ticker_data.is_empty();
    
    tracing::info!("🔧 DEBUG: Real data availability - money_flow: {}, ma_scores: {}, ticker_data: {}", 
                  has_money_flow, has_ma_scores, has_ticker_data);
    
    tracing::info!("🔧 DEBUG: Real data availability - money_flow: {}, ma_scores: {}, ticker_data: {}", 
                  has_money_flow, has_ma_scores, has_ticker_data);
    
    // Extract real data from state machine and update shared cache
    let old_version = {
        let shared_cache = shared_cli_cache.lock().await;
        shared_cache.version
    };
    
    {
        let mut shared_cache = shared_cli_cache.lock().await;
        
        // Extract real money flow data
        if has_money_flow {
            if let Some(money_flow_data) = money_flow_data {
                tracing::info!("🔧 REAL DATA: Extracted {} money flow entries", money_flow_data.len());
                
                // Convert to shared cache format
                shared_cache.money_flow_data.clear();
                for mf_ticker in money_flow_data {
                    shared_cache.money_flow_data.insert(mf_ticker.ticker.clone(), vec![mf_ticker]);
                }
            } else {
                tracing::warn!("⚠️ No money flow data available from state machine");
            }
        }
        
        // Extract real MA score data
        if has_ma_scores {
            if let Some(ma_score_data) = &ma_score_data {
                tracing::info!("🔧 REAL DATA: Extracted {} MA score entries", ma_score_data.len());
                
                // Convert to shared cache format
                shared_cache.ma_score_data.clear();
                for ma_ticker in ma_score_data {
                    shared_cache.ma_score_data.insert(ma_ticker.ticker.clone(), vec![ma_ticker.clone()]);
                }
            } else {
                tracing::warn!("⚠️ No MA score data available from state machine");
            }
        }
        
        // Extract real ticker data
        if has_ticker_data {
            tracing::info!("🔧 REAL DATA: Extracted {} ticker entries", ticker_data.len());
            shared_cache.ticker_data = ticker_data.clone();
        }
        
        // Update version and timestamp
        shared_cache.version += 1;
        shared_cache.last_updated = Some(Utc::now());
        
        tracing::info!("🔧 REAL DATA: Cache updated to version {}", shared_cache.version);
    }
    
    let new_version = {
        let shared_cache = shared_cli_cache.lock().await;
        shared_cache.version
    };
    
    tracing::info!(
        old_version = old_version,
        new_version = new_version,
        "✅ Shared CLI cache updated successfully (REAL DATA)"
    );
    
    Ok(())
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