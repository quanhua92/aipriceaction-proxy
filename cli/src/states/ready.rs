use crate::{
    state_machine::{State, StateContext},
    states::{FetchCSVState, FetchLiveState},
    utils::Logger,
};
use chrono::{DateTime, Utc, Timelike, Datelike};

/// ReadyState - The system is ready and waiting for requests
pub struct ReadyState {
    logger: Logger,
    tick_count: usize,
    last_live_check: DateTime<Utc>,
}

impl ReadyState {
    pub fn new() -> Self {
        Self {
            logger: Logger::new("READY"),
            tick_count: 0,
            last_live_check: Utc::now(),
        }
    }
}

#[async_trait::async_trait]
impl State for ReadyState {
    fn name(&self) -> &'static str {
        "READY"
    }

    async fn enter(&mut self, _context: &mut StateContext) -> anyhow::Result<()> {
        let now = chrono::Utc::now();
        self.logger.info(&format!(
            "🟢 [READY] [{}] System ready and waiting for requests",
            now.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        Ok(())
    }

    async fn exit(&mut self, _context: &mut StateContext) -> anyhow::Result<()> {
        let now = chrono::Utc::now();
        self.logger.info(&format!(
            "⏹️ [READY] [{}] Exiting ready state after {} ticks",
            now.format("%Y-%m-%d %H:%M:%S UTC"),
            self.tick_count
        ));
        Ok(())
    }

    async fn tick(
        &mut self,
        context: &mut StateContext,
    ) -> anyhow::Result<Option<Box<dyn State + Send + Sync>>> {
        self.tick_count += 1;

        let now = chrono::Utc::now();
        let _is_market_hours = self.is_market_hours();
        self.logger.info(&format!(
            "🔄 [READY] [{}] Tick #{} (Market: {})",
            now.format("%Y-%m-%d %H:%M:%S UTC"),
            self.tick_count,
            if _is_market_hours { "OPEN" } else { "CLOSED" }
        ));

        // 1. Check for new CSV requests first (highest priority)
        if context.request_queue.has_csv_requests() {
            let transition_time = chrono::Utc::now();
            self.logger.info(&format!(
                "📦 [READY] [{}] New CSV requests detected - transitioning to CSV processing",
                transition_time.format("%Y-%m-%d %H:%M:%S UTC")
            ));
            return Ok(Some(Box::new(FetchCSVState::new())));
        }

        // 2. Periodic live data checks (every 3 ticks = 15 seconds during market hours)
        if self.should_perform_live_check() {
            self.last_live_check = Utc::now();
            let transition_time = chrono::Utc::now();
            self.logger.info(&format!(
                "📊 [READY] [{}] Periodic live check: updating market data ({})",
                transition_time.format("%Y-%m-%d %H:%M:%S UTC"),
                if _is_market_hours { "market hours" } else { "after hours" }
            ));
            return Ok(Some(Box::new(FetchLiveState::new())));
        }

        // 3. Money flow is triggered by data changes in FETCH_LIVE, no periodic checking needed

        // 4. Log system status every 5 ticks
        if self.tick_count == 1 || self.tick_count % 5 == 0 {
            let status_time = chrono::Utc::now();
            self.logger.debug(&format!(
                "📊 [READY] [{}] Logging system status (tick #{})",
                status_time.format("%Y-%m-%d %H:%M:%S UTC"),
                self.tick_count
            ));
            self.log_system_status(context);
        }

        // 5. Periodic cleanup (every 60 ticks = 5 minutes)
        if self.tick_count % 60 == 0 {
            self.perform_periodic_cleanup();
        }

        // Stay in ready state
        Ok(None)
    }
}

impl ReadyState {
    /// Check if we should perform a live data check
    fn should_perform_live_check(&self) -> bool {
        let now = Utc::now();
        let time_since_last_check = (now - self.last_live_check).num_seconds();

        // Check if it's market hours (simplified - always assume market hours for now)
        let _is_market_hours = self.is_market_hours();

        // Relaxed for debugging - same interval regardless of market hours
        time_since_last_check > 15
    }


    /// Check if Vietnamese market is currently open
    /// Vietnam market: 9:00 - 15:00, Monday-Friday
    fn is_market_hours(&self) -> bool {
        let now = Utc::now();
        // Convert to Vietnam timezone (UTC+7)
        let vietnam_time = now + chrono::Duration::hours(7);

        let hour = vietnam_time.hour();
        let weekday = vietnam_time.weekday();

        // Monday = 1, Sunday = 7 in chrono weekday
        let is_weekday = matches!(weekday, chrono::Weekday::Mon
                                        | chrono::Weekday::Tue
                                        | chrono::Weekday::Wed
                                        | chrono::Weekday::Thu
                                        | chrono::Weekday::Fri);

        is_weekday && hour >= 9 && hour < 15
    }

    /// Perform periodic cleanup
    fn perform_periodic_cleanup(&self) {
        self.logger.info(&format!("Performing periodic cleanup - tick #{}", self.tick_count));
    }

    /// Log system status and key ticker information
    fn log_system_status(&self, context: &StateContext) {
        self.logger.info(&format!("📊 READY State - Tick #{}", self.tick_count));

        // Get cache stats
        let ticker_count = context.cache.get_ticker_count();
        let money_flow_stats = context.cache.get_money_flow_stats();
        let ma_score_stats = context.cache.get_ma_score_stats();

        self.logger.info(&format!(
            "💾 Cache: {} tickers, Money Flow: {} calculations, MA Score: {} calculations",
            ticker_count,
            money_flow_stats.total_calculations,
            ma_score_stats.total_calculations
        ));

        // Show key ticker information with latest 3 dates (VCB, BID, TCB, CTG)
        let key_tickers = vec!["VCB", "BID", "TCB", "CTG"];
        for ticker in key_tickers {
            self.logger.info(&format!("🏦 === {} ===", ticker));

            if let Some(ticker_data) = context.cache.get_ticker_data(ticker, None) {
                // Show latest 3 data points
                let latest_points: Vec<_> = ticker_data.iter().rev().take(3).collect();

                for (i, point) in latest_points.iter().enumerate() {
                    // Get money flow data for this ticker and date
                    let money_flow_info = self.get_money_flow_for_ticker_date(context, ticker, &point.time);

                    // Get MA score data for this ticker and date
                    let ma_score_info = self.get_ma_score_for_ticker_date(context, ticker, &point.time);

                    let date_label = match i {
                        0 => "Latest",
                        1 => "  -1d ",
                        2 => "  -2d ",
                        _ => "     ",
                    };

                    // Get MA values from MA score cache (single source of truth)
                    let ma_values_info = self.get_ma_values_for_ticker_date(context, ticker, &point.time);

                    self.logger.info(&format!(
                        "    TICKER={} {} {}: Close={:.1}k Vol={:.1}M{}{}{}",
                        ticker,
                        date_label,
                        point.time,
                        point.close / 1000.0,
                        point.volume as f64 / 1_000_000.0,
                        ma_values_info,
                        money_flow_info,
                        ma_score_info
                    ));
                }
            } else {
                self.logger.warn(&format!("    TICKER={} ❌ No data available", ticker));
            }
        }
    }

    /// Get money flow information for a specific ticker and date
    fn get_money_flow_for_ticker_date(&self, context: &StateContext, ticker: &str, date: &str) -> String {
        // Try to get money flow data from cache
        if let Some(money_flow_data) = context.cache.get_cache().money_flow_data.get(date) {
            if let Some(ticker_data) = money_flow_data.iter().find(|d| d.ticker == ticker) {
                // Get multiple money flow metrics from the rich data structure
                let money_flow_percentage = ticker_data.signed_percentage_data.get(date).copied().unwrap_or(0.0);
                let activity_flow = ticker_data.activity_flow_data.get(date).copied().unwrap_or(0.0);
                let dollar_flow = ticker_data.dollar_flow_data.get(date).copied().unwrap_or(0.0);
                let trend_score = ticker_data.trend_score;

                return format!(
                    " | MF={:.2}% AF={:.2} DF={:.2} TS={:.2}",
                    money_flow_percentage,
                    activity_flow,
                    dollar_flow,
                    trend_score
                );
            }
        }
        " | MF=N/A".to_string()
    }

    /// Get MA values information for a specific ticker and date from MA score cache
    fn get_ma_values_for_ticker_date(&self, context: &StateContext, ticker: &str, date: &str) -> String {
        // Get MA values from MA score calculation results (single source of truth)
        if let Some(ma_score_data) = context.cache.get_cache().ma_score_data.get(date) {
            if let Some(ticker_data) = ma_score_data.iter().find(|d| d.ticker == ticker) {
                // Get MA values from debug_data if available
                if let Some(debug_data) = &ticker_data.debug_data {
                    if let Some(date_debug) = debug_data.get(date) {
                        return format!(
                            " | MAs: 10={} 20={} 50={}",
                            date_debug.ma10_value.map(|v| format!("{:.1}k", v/1000.0)).unwrap_or("N/A".to_string()),
                            date_debug.ma20_value.map(|v| format!("{:.1}k", v/1000.0)).unwrap_or("N/A".to_string()),
                            date_debug.ma50_value.map(|v| format!("{:.1}k", v/1000.0)).unwrap_or("N/A".to_string())
                        );
                    }
                }
            }
        }
        " | MAs: N/A N/A N/A".to_string()
    }

    /// Get MA score information for a specific ticker and date
    fn get_ma_score_for_ticker_date(&self, context: &StateContext, ticker: &str, date: &str) -> String {
        // Try to get MA score data from cache
        if let Some(ma_score_data) = context.cache.get_cache().ma_score_data.get(date) {
            if let Some(ticker_data) = ma_score_data.iter().find(|d| d.ticker == ticker) {
                // Get the MA scores for this date
                let ma10_score = ticker_data.ma10_scores.get(date).copied().unwrap_or(0.0);
                let ma20_score = ticker_data.ma20_scores.get(date).copied().unwrap_or(0.0);
                let ma50_score = ticker_data.ma50_scores.get(date).copied().unwrap_or(0.0);

                return format!(
                    " | Scores: 10={:.1}% 20={:.1}% 50={:.1}%",
                    ma10_score,
                    ma20_score,
                    ma50_score
                );
            }
        }
        " | Scores=N/A".to_string()
    }
}

impl Default for ReadyState {
    fn default() -> Self {
        Self::new()
    }
}