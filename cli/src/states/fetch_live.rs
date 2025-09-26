use crate::{
    services::LiveDataService,
    state_machine::{State, StateContext},
    states::{FetchCSVState, MoneyFlowState, ReadyState},
    utils::{Logger, Timer},
};

/// FetchLiveState - Handles live data fetching from API
pub struct FetchLiveState {
    logger: Logger,
    live_service: LiveDataService,
}

impl FetchLiveState {
    pub fn new() -> Self {
        Self {
            logger: Logger::new("FETCH_LIVE"),
            live_service: LiveDataService::new(),
        }
    }
}

#[async_trait::async_trait]
impl State for FetchLiveState {
    fn name(&self) -> &'static str {
        "FETCH_LIVE"
    }

    async fn enter(&mut self, _context: &mut StateContext) -> anyhow::Result<()> {
        self.logger.info("Entering live data fetch state");
        Ok(())
    }

    async fn exit(&mut self, _context: &mut StateContext) -> anyhow::Result<()> {
        self.logger.info("Exiting live data fetch state");
        Ok(())
    }

    async fn tick(
        &mut self,
        context: &mut StateContext,
    ) -> anyhow::Result<Option<Box<dyn State + Send + Sync>>> {
        // 1. Check if there are new CSV requests (higher priority)
        if context.request_queue.has_csv_requests() {
            self.logger.info("New CSV requests detected - transitioning back to CSV processing");
            return Ok(Some(Box::new(FetchCSVState::new())));
        }

        // 2. Fetch live data for all tickers
        let all_tickers = context.get_all_tickers();

        if !all_tickers.is_empty() {
            self.logger.info(&format!("Fetching live data for {} tickers", all_tickers.len()));

            let timer = Timer::start("live data fetch");

            match self.live_service.fetch_latest(&all_tickers, Some(15000)).await {
                Ok(live_data) => {
                    if !live_data.is_empty() {
                        // Merge live data into cache
                        let changed_dates = context.cache.merge_live_data(live_data);

                        timer.log_elapsed("FETCH_LIVE");

                        self.logger.info(&format!(
                            "Live data fetch completed: {} tickers updated, {} dates changed in {:.1}ms",
                            all_tickers.len(),
                            changed_dates.len(),
                            timer.elapsed_ms()
                        ));

                        // If data changed, we may need to recalculate money flow
                        if !changed_dates.is_empty() {
                            let now = chrono::Utc::now();
                            self.logger.info(&format!(
                                "[{}] ♻️ [FETCH_LIVE] Live data values changed for {} dates - triggering recalculation",
                                now.format("%Y-%m-%d %H:%M:%S UTC"),
                                changed_dates.len()
                            ));
                            return Ok(Some(Box::new(MoneyFlowState::new())));
                        }
                    } else {
                        self.logger.info("No recent live data available for any tickers");
                    }
                }
                Err(e) => {
                    self.logger.warn(&format!("Failed to fetch live data: {}", e));
                    // Continue to next state even if live data fetch fails
                    // This ensures the system doesn't get stuck on API failures
                }
            }
        } else {
            self.logger.warn("No tickers available for live data fetch");
        }

        // 3. Check if we need to calculate money flow
        if self.should_calculate_money_flow(context) {
            self.logger.info("Money flow calculation needed - transitioning");
            return Ok(Some(Box::new(MoneyFlowState::new())));
        }

        // 4. Move to ready state if nothing else to process
        self.logger.info("Live data processing complete - transitioning to ready state");
        Ok(Some(Box::new(ReadyState::new())))
    }
}

impl FetchLiveState {
    /// Check if money flow calculation is needed
    fn should_calculate_money_flow(&self, context: &StateContext) -> bool {
        let money_flow_stats = context.cache.get_money_flow_stats();

        // If we have uncalculated dates, we should calculate
        if !money_flow_stats.uncalculated_dates.is_empty() {
            self.logger.debug(&format!(
                "Money flow calculation needed: {} uncalculated dates",
                money_flow_stats.uncalculated_dates.len()
            ));
            return true;
        }

        // If money flow data is older than ticker data, recalculate
        if let (Some(money_flow_updated), Some(ticker_updated)) = (
            money_flow_stats.last_update,
            context.cache.get_last_ticker_update(),
        ) {
            if money_flow_updated < ticker_updated {
                self.logger.debug("Money flow data is older than ticker data - recalculation needed");
                return true;
            }
        }

        // If we don't have any money flow data but have ticker data, calculate
        if money_flow_stats.total_calculations == 0 && context.cache.get_ticker_count() > 0 {
            self.logger.debug("No money flow calculations yet but have ticker data - calculation needed");
            return true;
        }

        false
    }

    /// Fetch live data for specific tickers (used by other states)
    #[allow(dead_code)]
    pub async fn fetch_specific_tickers(
        &mut self,
        tickers: &[String],
        context: &mut StateContext,
    ) -> anyhow::Result<usize> {
        self.logger.info(&format!("Fetching live data for {} specific tickers", tickers.len()));

        let timer = Timer::start("specific live data fetch");

        match self.live_service.fetch_latest(tickers, Some(10000)).await {
            Ok(live_data) => {
                if !live_data.is_empty() {
                    let changed_dates = context.cache.merge_live_data(live_data.clone());

                    timer.log_elapsed("FETCH_LIVE");

                    self.logger.info(&format!(
                        "Specific live data fetch completed: {}/{} tickers updated, {} dates changed in {:.1}ms",
                        live_data.len(),
                        tickers.len(),
                        changed_dates.len(),
                        timer.elapsed_ms()
                    ));

                    Ok(live_data.len())
                } else {
                    self.logger.info("No recent live data available for specific tickers");
                    Ok(0)
                }
            }
            Err(e) => {
                self.logger.error(&format!("Failed to fetch specific live data: {}", e));
                Err(e)
            }
        }
    }

    /// Check if live data refresh is needed (used for periodic refreshes)
    #[allow(dead_code)]
    pub fn should_refresh_live_data(&self) -> bool {
        let now = chrono::Utc::now();

        // Check if it's market hours
        let is_market_hours = crate::utils::is_market_hours(now) && !crate::utils::is_weekend(now);

        // During market hours, refresh every 5 minutes
        // Outside market hours, refresh every hour
        let _refresh_interval = if is_market_hours {
            chrono::Duration::minutes(5)
        } else {
            chrono::Duration::hours(1)
        };

        // For now, we'll always return true since we don't track last refresh time
        // In a full implementation, this would check against the last refresh timestamp
        true
    }

    /// Get statistics about live data fetching
    #[allow(dead_code)]
    pub fn get_live_fetch_stats(&self) -> LiveFetchStats {
        LiveFetchStats {
            service_stats: self.live_service.get_stats(),
        }
    }
}

impl Default for FetchLiveState {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for live data fetching
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LiveFetchStats {
    pub service_stats: crate::services::LiveServiceStats,
}