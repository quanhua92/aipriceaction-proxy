use crate::{
    state_machine::{State, StateContext},
    states::{FetchCSVState, MAScoreState, ReadyState},
    utils::{Logger, Timer, calculate_multiple_dates_vectorized, MultipleDatesResult},
};
use std::collections::HashMap;

/// MoneyFlowState - Handles money flow calculations
/// Full implementation using vectorized money flow algorithms from TypeScript
pub struct MoneyFlowState {
    logger: Logger,
    #[allow(dead_code)]
    calculation_progress: usize,
    total_dates_to_process: usize,
    excluded_tickers: Vec<String>,
}

impl MoneyFlowState {
    pub fn new() -> Self {
        Self {
            logger: Logger::new("MONEY_FLOW"),
            calculation_progress: 0,
            total_dates_to_process: 0,
            excluded_tickers: vec!["VNINDEX".to_string()],
        }
    }
}

#[async_trait::async_trait]
impl State for MoneyFlowState {
    fn name(&self) -> &'static str {
        "MONEY_FLOW"
    }

    async fn enter(&mut self, context: &mut StateContext) -> anyhow::Result<()> {
        self.logger.info("Entering money flow calculation state");

        // Get uncalculated dates count for logging
        let money_flow_stats = context.cache.get_money_flow_stats();
        self.total_dates_to_process = money_flow_stats.uncalculated_dates.len();

        self.logger.info(&format!(
            "Money flow vectorized calculation needed for {} dates",
            self.total_dates_to_process
        ));

        Ok(())
    }

    async fn exit(&mut self, _context: &mut StateContext) -> anyhow::Result<()> {
        self.logger.info("Exiting money flow calculation state - vectorized processing complete");
        Ok(())
    }

    async fn tick(
        &mut self,
        context: &mut StateContext,
    ) -> anyhow::Result<Option<Box<dyn State + Send + Sync>>> {
        // 1. Check if there are CSV requests (higher priority)
        if context.request_queue.has_csv_requests() {
            self.logger.info("New CSV requests detected - transitioning to CSV processing");
            return Ok(Some(Box::new(FetchCSVState::new())));
        }

        // 2. Perform money flow calculations for ALL dates in single vectorized operation
        let money_flow_stats = context.cache.get_money_flow_stats();

        if money_flow_stats.uncalculated_dates.is_empty() {
            self.logger.info("No uncalculated dates for money flow - transitioning to next state");
            return self.determine_next_state(context);
        }

        // Process ALL dates in single vectorized calculation (under 100ms target)
        let dates_to_process = money_flow_stats.uncalculated_dates.clone();

        self.logger.info(&format!(
            "Starting vectorized money flow calculation for ALL {} dates",
            dates_to_process.len()
        ));

        let timer = Timer::start("vectorized money flow calculation");

        // Single vectorized money flow calculation for ALL dates
        let result = self.calculate_money_flow_for_dates(&dates_to_process, &mut *context).await?;

        let now = chrono::Utc::now();
        self.logger.info(&format!(
            "[{}] 📦 [MONEY_FLOW] Money flow calculation returned - storing in cache",
            now.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        // Store money flow results in cache
        context.cache.set_money_flow_data(result.results);

        let elapsed = timer.elapsed_ms();

        self.logger.info(&format!(
            "Vectorized money flow calculation completed: {} dates processed in {:.1}ms",
            dates_to_process.len(),
            elapsed
        ));

        // All money flow calculations completed
        let now = chrono::Utc::now();
        self.logger.info(&format!(
            "[{}] ✅ [MONEY_FLOW] Money flow calculations completed - determining next state",
            now.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        self.determine_next_state(context)
    }
}

impl MoneyFlowState {
    /// Determine the next state after money flow calculations
    fn determine_next_state(
        &self,
        context: &StateContext,
    ) -> anyhow::Result<Option<Box<dyn State + Send + Sync>>> {
        let now = chrono::Utc::now();

        // Check if MA score calculation is needed
        let needs_ma_score = self.should_calculate_ma_score(context);
        self.logger.info(&format!(
            "[{}] 🔍 [MONEY_FLOW] Determining next state: needs_ma_score={}",
            now.format("%Y-%m-%d %H:%M:%S UTC"),
            needs_ma_score
        ));

        if needs_ma_score {
            self.logger.info(&format!(
                "[{}] 🔄 [MONEY_FLOW] MA score calculation needed - transitioning to MA_SCORE",
                now.format("%Y-%m-%d %H:%M:%S UTC")
            ));
            return Ok(Some(Box::new(MAScoreState::new())));
        }

        // Move to ready state
        self.logger.info(&format!(
            "[{}] ✅ [MONEY_FLOW] All calculations complete - transitioning to READY",
            now.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        Ok(Some(Box::new(ReadyState::new())))
    }

    /// Check if MA score calculation is needed
    fn should_calculate_ma_score(&self, context: &StateContext) -> bool {
        let ma_score_stats = context.cache.get_ma_score_stats();

        // If we have uncalculated dates, we should calculate
        if !ma_score_stats.uncalculated_dates.is_empty() {
            self.logger.debug(&format!(
                "MA score calculation needed: {} uncalculated dates",
                ma_score_stats.uncalculated_dates.len()
            ));
            return true;
        }

        // If MA score data is older than ticker data, recalculate
        if let (Some(ma_score_updated), Some(ticker_updated)) = (
            ma_score_stats.last_update,
            context.cache.get_last_ticker_update(),
        ) {
            if ma_score_updated < ticker_updated {
                self.logger.debug("MA score data is older than ticker data - recalculation needed");
                return true;
            }
        }

        // If we don't have any MA score data but have ticker data, calculate
        if ma_score_stats.total_calculations == 0 && context.cache.get_ticker_count() > 0 {
            self.logger.debug("No MA score calculations yet but have ticker data - calculation needed");
            return true;
        }

        false
    }

    /// Perform actual money flow calculation for a batch of dates
    /// Uses vectorized money flow algorithms matching TypeScript implementation
    async fn calculate_money_flow_for_dates(
        &self,
        dates: &[String],
        context: &mut StateContext,
    ) -> anyhow::Result<MultipleDatesResult> {
        self.logger.info(&format!(
            "Starting vectorized money flow calculation for {} dates",
            dates.len()
        ));

        // Get all tickers and their data
        let all_tickers = context.get_all_tickers();
        if all_tickers.is_empty() {
            return Err(anyhow::anyhow!("No tickers available for money flow calculation"));
        }

        // Build ticker data map from cache
        let mut ticker_data = HashMap::new();
        let mut missing_tickers = 0;

        for ticker in &all_tickers {
            if let Some(data) = context.cache.get_ticker_data(ticker, None) {
                if !data.is_empty() {
                    ticker_data.insert(ticker.clone(), data);
                } else {
                    missing_tickers += 1;
                }
            } else {
                missing_tickers += 1;
            }
        }

        self.logger.info(&format!(
            "Available data: {} tickers, {} missing",
            ticker_data.len(),
            missing_tickers
        ));

        if ticker_data.len() < 5 {
            return Err(anyhow::anyhow!("Too few tickers for reliable money flow calculation"));
        }

        // Get VNINDEX data for volume weighting
        let vnindex_data = context.cache.get_vnindex_data(None);

        // Perform vectorized calculation
        let result = calculate_multiple_dates_vectorized(
            &ticker_data,
            &all_tickers,
            dates,
            vnindex_data.as_deref(),
            true,  // Enable VNINDEX volume weighting
            true,  // Enable directional colors
            &self.excluded_tickers,
        );

        self.logger.info(&format!(
            "Money flow calculation completed: {} dates, {} tickers, {:.1}ms",
            result.metrics.date_count,
            result.metrics.ticker_count,
            result.metrics.vectorized_time
        ));

        Ok(result)
    }



    /// Get calculation progress information
    #[allow(dead_code)]
    pub fn get_progress(&self) -> MoneyFlowProgress {
        MoneyFlowProgress {
            processed_dates: self.calculation_progress,
            total_dates: self.total_dates_to_process,
            completion_percentage: if self.total_dates_to_process > 0 {
                (self.calculation_progress as f64 / self.total_dates_to_process as f64) * 100.0
            } else {
                100.0
            },
        }
    }
}

impl Default for MoneyFlowState {
    fn default() -> Self {
        Self::new()
    }
}

/// Money flow calculation progress information
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MoneyFlowProgress {
    pub processed_dates: usize,
    pub total_dates: usize,
    pub completion_percentage: f64,
}

// Vectorized money flow calculations using actual algorithms from TypeScript implementation