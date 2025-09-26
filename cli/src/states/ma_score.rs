use crate::{
    models::ma_score::{MAScoreProcessConfig},
    state_machine::{State, StateContext},
    states::{FetchCSVState, FetchLiveState, ReadyState},
    utils::{
        Logger, Timer,
        vectorized_ma_score::{
            calculate_for_current_range, calculate_for_dates,
        }
    },
};

/// MAScoreState - Handles MA score calculations
/// Exactly matches the TypeScript MAScoreState implementation
pub struct MAScoreState {
    logger: Logger,
    excluded_tickers: Vec<String>,
}

impl MAScoreState {
    pub fn new() -> Self {
        Self {
            logger: Logger::new("MA_SCORE"),
            excluded_tickers: vec!["VNINDEX".to_string()],
        }
    }
}

#[async_trait::async_trait]
impl State for MAScoreState {
    fn name(&self) -> &'static str {
        "MA_SCORE"
    }

    async fn enter(&mut self, _context: &mut StateContext) -> anyhow::Result<()> {
        self.logger.info("Entering MA score calculation state");
        Ok(())
    }

    async fn exit(&mut self, _context: &mut StateContext) -> anyhow::Result<()> {
        self.logger.info("Exiting MA score calculation state");
        Ok(())
    }

    async fn tick(
        &mut self,
        context: &mut StateContext,
    ) -> anyhow::Result<Option<Box<dyn State + Send + Sync>>> {
        // Check for CSV requests (higher priority)
        if context.request_queue.has_csv_requests() {
            self.logger.info("New CSV requests detected - transitioning back to CSV processing");
            return Ok(Some(Box::new(FetchCSVState::new())));
        }

        // 1. Simple check like Money Flow - just calculate missing dates
        let uncalculated_dates = context.cache.get_dates_needing_ma_score();

        if uncalculated_dates.is_empty() {
            // No calculation needed, check if we should try live data
            let cache = context.cache.get_cache();
            let has_attempted_live_data = cache.last_background_update.is_some();

            if !has_attempted_live_data {
                self.logger.info("No MA score calculation needed - fetching live data");
                return Ok(Some(Box::new(FetchLiveState::new())));
            } else {
                self.logger.info("No MA score calculation needed");
                return Ok(Some(Box::new(ReadyState::new())));
            }
        }

        self.logger.info(&format!(
            "MA score calculation needed: {} uncalculated dates",
            uncalculated_dates.len()
        ));

        // 2. Always do incremental calculation (like Money Flow)
        match self.perform_calculation(context, &uncalculated_dates).await {
            Ok(_) => {
                // 4. After successful calculation, check if we should try live data first
                let cache = context.cache.get_cache();
                let has_attempted_live_data = cache.last_background_update.is_some();

                let now = chrono::Utc::now();
                self.logger.info(&format!(
                    "[{}] 🚀 [MA_SCORE] MA score calculation completed - checking next state (has_attempted_live_data={})",
                    now.format("%Y-%m-%d %H:%M:%S UTC"),
                    has_attempted_live_data
                ));

                if !has_attempted_live_data {
                    self.logger.info(&format!(
                        "[{}] 🔄 [MA_SCORE] Initial MA score calculation completed - transitioning to FETCH_LIVE",
                        now.format("%Y-%m-%d %H:%M:%S UTC")
                    ));
                    Ok(Some(Box::new(FetchLiveState::new())))
                } else {
                    self.logger.info(&format!(
                        "[{}] ✅ [MA_SCORE] Final MA score calculation completed - transitioning to READY",
                        now.format("%Y-%m-%d %H:%M:%S UTC")
                    ));
                    Ok(Some(Box::new(ReadyState::new())))
                }
            }
            Err(e) => {
                self.logger.error(&format!("MA score calculation failed: {}", e));
                // On error, still move to READY state to avoid getting stuck
                Ok(Some(Box::new(ReadyState::new())))
            }
        }
    }
}

impl MAScoreState {
    async fn perform_calculation(
        &mut self,
        context: &mut StateContext,
        uncalculated_dates: &[String],
    ) -> anyhow::Result<()> {
        // Simple approach like Money Flow - just calculate the missing dates
        self.perform_incremental_calculation(context, uncalculated_dates).await
    }

    #[allow(dead_code)]
    async fn perform_full_calculation(&mut self, context: &mut StateContext) -> anyhow::Result<()> {
        self.logger.info("Starting full MA score calculation for current range");

        let timer = Timer::start("full MA score calculation");

        // Get current range configuration
        let current_range = context.cache.get_cache().last_requested_range.clone()
            .ok_or_else(|| anyhow::anyhow!("No requested range set"))?;

        // Get all available tickers
        let all_tickers = context.cache.get_all_tickers();
        if all_tickers.is_empty() {
            return Err(anyhow::anyhow!("No tickers available for MA score calculation"));
        }

        // Get ticker data
        let mut ticker_data = std::collections::HashMap::new();
        for ticker in &all_tickers {
            if let Some(data) = context.cache.get_ticker_data(ticker, None) {
                ticker_data.insert(ticker.clone(), data);
            }
        }

        // Use trading dates within the configured range
        let date_range = context.cache.get_all_trading_dates_in_range();

        // Configure calculation
        let config = MAScoreProcessConfig {
            date_range_config: current_range,
            days_back: date_range.len(),
            current_date: date_range.last().cloned(),
            default_ma_period: 20, // Default to MA20
        };

        // Calculate MA scores
        let (results, metrics) = calculate_for_current_range(
            &ticker_data,
            &all_tickers,
            &date_range,
            &config,
            &self.excluded_tickers,
        );

        // MA values are now handled exclusively by MA score calculation system
        // No longer updating StockDataPoint to eliminate dual sources

        let elapsed = timer.elapsed_ms();

        self.logger.info(&format!(
            "Full MA score calculation completed in {:.1}ms: {} dates calculated",
            elapsed,
            results.len()
        ));

        // Update cache with results
        context.cache.set_ma_score_data(results, config, metrics);

        Ok(())
    }

    async fn perform_incremental_calculation(
        &mut self,
        context: &mut StateContext,
        uncalculated_dates: &[String],
    ) -> anyhow::Result<()> {
        // Simple like Money Flow - just calculate the missing dates passed in
        let dates_to_calculate = uncalculated_dates.to_vec();

        if dates_to_calculate.is_empty() {
            self.logger.debug("No dates need incremental MA score calculation");
            return Ok(());
        }

        self.logger.info(&format!(
            "Starting incremental MA score calculation: {} dates",
            dates_to_calculate.len()
        ));

        let timer = Timer::start("incremental MA score calculation");

        // Get current range configuration
        let current_range = context.cache.get_cache().last_requested_range.clone()
            .ok_or_else(|| anyhow::anyhow!("No requested range set"))?;

        // Get all available tickers
        let all_tickers = context.cache.get_all_tickers();
        if all_tickers.is_empty() {
            return Err(anyhow::anyhow!("No tickers available for MA score calculation"));
        }

        // Get ticker data
        let mut ticker_data = std::collections::HashMap::new();
        for ticker in &all_tickers {
            if let Some(data) = context.cache.get_ticker_data(ticker, None) {
                ticker_data.insert(ticker.clone(), data);
            }
        }

        // CRITICAL FIX: For incremental MA calculation, we need sufficient historical context
        // MA10 needs 10+ days, MA20 needs 20+ days, MA50 needs 50+ days
        let required_history_days = 60; // Buffer for MA50 + safety margin

        // Configure calculation for incremental dates
        let config = MAScoreProcessConfig {
            date_range_config: current_range,
            days_back: required_history_days, // ✅ FIXED: Provide enough history for MA calculations
            current_date: dates_to_calculate.last().cloned(),
            default_ma_period: 20,
        };

        // CRITICAL FIX: For MA calculations, we need to include sufficient historical dates
        // Get all available dates to find the historical context
        let all_available_dates = context.cache.get_all_trading_dates_in_range();

        // Find the earliest date we need to calculate and include 60 days before it for context
        let earliest_calculation_date = dates_to_calculate.iter().min().cloned().unwrap_or_default();

        // Find position of earliest calculation date in all available dates
        if let Some(earliest_pos) = all_available_dates.iter().position(|d| d == &earliest_calculation_date) {
            // Include 60 days before the earliest calculation date for MA context
            let history_start = if earliest_pos >= required_history_days {
                earliest_pos - required_history_days
            } else {
                0 // Use all available historical data
            };

            // Include all dates from history_start to the end for proper MA calculation
            let dates_with_context: Vec<String> = all_available_dates[history_start..].to_vec();

            self.logger.info(&format!(
                "🔧 CRITICAL FIX: Including {} total dates for MA context ({} historical + {} calculation dates)",
                dates_with_context.len(),
                dates_with_context.len() - dates_to_calculate.len(),
                dates_to_calculate.len()
            ));

            // Calculate MA scores for dates with sufficient historical context
            let (results, metrics) = calculate_for_dates(
                &ticker_data,
                &all_tickers,
                &dates_with_context, // ✅ FIXED: Use dates with historical context, not just calculation dates
                &config,
                &self.excluded_tickers,
            );

            // Filter results to only include the dates we actually wanted to calculate
            let mut filtered_results = std::collections::HashMap::new();
            for (date, ticker_data) in results {
                if dates_to_calculate.contains(&date) {
                    filtered_results.insert(date, ticker_data);
                }
            }

            self.logger.info(&format!(
                "🎯 Filtered MA results: {} dates calculated, {} dates stored",
                dates_with_context.len(),
                filtered_results.len()
            ));

            // Store filtered results and update cache (temporarily - will be overwritten below)
            // context.cache.update_ma_score_data(filtered_results, metrics); // Will be done later
            // CRITICAL FIX: Extract MA values from the calculation results, not from ticker_data
            let mut ma_values_map = std::collections::HashMap::new();

            for (date, ticker_results) in &filtered_results {
                for ticker_result in ticker_results {
                    let ticker = &ticker_result.ticker;

                    // Get the MA values from the debug_data if available
                    if let Some(debug_data) = &ticker_result.debug_data {
                        if let Some(date_debug) = debug_data.get(date) {
                            let ma_values_entry = ma_values_map.entry(ticker.clone()).or_insert_with(Vec::new);

                            ma_values_entry.push((
                                date.clone(),
                                date_debug.ma10_value,
                                date_debug.ma20_value,
                                date_debug.ma50_value,
                            ));

                            // Only log for key tickers and recent dates to avoid spam
                            let key_tickers = ["VCB", "BID", "TCB", "CTG"];
                            let should_log = key_tickers.contains(&ticker.as_str()) &&
                                           (date == "2025-09-25" || date == "2025-09-26");

                            if should_log {
                                self.logger.info(&format!(
                                    "🔧 EXTRACTING CORRECT MA VALUES: {} {}: MA10={:?} MA20={:?} MA50={:?}",
                                    ticker, date,
                                    date_debug.ma10_value.map(|v| format!("{:.1}k", v/1000.0)),
                                    date_debug.ma20_value.map(|v| format!("{:.1}k", v/1000.0)),
                                    date_debug.ma50_value.map(|v| format!("{:.1}k", v/1000.0))
                                ));
                            }
                        }
                    }
                }
            }

            // MA values are now handled exclusively by MA score calculation system
            // No longer updating StockDataPoint to eliminate dual sources

            let elapsed = timer.elapsed_ms();

            self.logger.info(&format!(
                "Incremental MA score calculation completed in {:.1}ms: {} dates calculated",
                elapsed,
                filtered_results.len()
            ));

            // Update cache with incremental results
            context.cache.update_ma_score_data(filtered_results, metrics);
        } else {
            self.logger.error(&format!("Could not find calculation date {} in available dates", earliest_calculation_date));
            return Err(anyhow::anyhow!("Invalid calculation date"));
        }

        Ok(())
    }
}

impl Default for MAScoreState {
    fn default() -> Self {
        Self::new()
    }
}