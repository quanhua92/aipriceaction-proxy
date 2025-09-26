use super::{State, StateContext};
use crate::states::*;

/// State transition logic and utilities
pub struct StateTransitions;

impl StateTransitions {
    /// Determine the next state based on current state and context
    pub async fn determine_next_state(
        current_state_name: &str,
        context: &StateContext,
    ) -> Option<Box<dyn State + Send + Sync>> {
        match current_state_name {
            "FETCH_CSV" => Self::from_fetch_csv(context).await,
            "FETCH_LIVE" => Self::from_fetch_live(context).await,
            "MONEY_FLOW" => Self::from_money_flow(context).await,
            "MA_SCORE" => Self::from_ma_score(context).await,
            "READY" => Self::from_ready(context).await,
            _ => None,
        }
    }

    /// Transitions from FETCH_CSV state
    async fn from_fetch_csv(context: &StateContext) -> Option<Box<dyn State + Send + Sync>> {
        // Prerequisites must be loaded first
        if !context.has_prerequisites() {
            return None; // Stay in FETCH_CSV to load prerequisites
        }

        // If there are CSV requests pending, stay in FETCH_CSV
        if context.request_queue.has_csv_requests() {
            return None;
        }

        // No CSV requests, move to fetch live data
        Some(Box::new(FetchLiveState::new()))
    }

    /// Transitions from FETCH_LIVE state
    async fn from_fetch_live(context: &StateContext) -> Option<Box<dyn State + Send + Sync>> {
        // If there are new CSV requests, go back to CSV processing
        if context.request_queue.has_csv_requests() {
            return Some(Box::new(FetchCSVState::new()));
        }

        // Always proceed to MONEY_FLOW after FETCH_LIVE (sequential flow)
        Some(Box::new(MoneyFlowState::new()))
    }

    /// Transitions from MONEY_FLOW state
    async fn from_money_flow(context: &StateContext) -> Option<Box<dyn State + Send + Sync>> {
        // If there are CSV requests, prioritize those
        if context.request_queue.has_csv_requests() {
            return Some(Box::new(FetchCSVState::new()));
        }

        // Always proceed to MA_SCORE after MONEY_FLOW (sequential flow)
        Some(Box::new(MAScoreState::new()))
    }

    /// Transitions from MA_SCORE state
    async fn from_ma_score(context: &StateContext) -> Option<Box<dyn State + Send + Sync>> {
        // If there are CSV requests, prioritize those
        if context.request_queue.has_csv_requests() {
            return Some(Box::new(FetchCSVState::new()));
        }

        // Move to ready state
        Some(Box::new(ReadyState::new()))
    }

    /// Transitions from READY state
    async fn from_ready(context: &StateContext) -> Option<Box<dyn State + Send + Sync>> {
        // Check for new requests in priority order

        // 1. CSV requests (highest priority)
        if context.request_queue.has_csv_requests() {
            return Some(Box::new(FetchCSVState::new()));
        }

        // 2. Money flow calculation needed
        if Self::should_calculate_money_flow(context) {
            return Some(Box::new(MoneyFlowState::new()));
        }

        // 3. MA score calculation needed
        if Self::should_calculate_ma_score(context) {
            return Some(Box::new(MAScoreState::new()));
        }

        // 4. Live data refresh (periodic)
        if Self::should_refresh_live_data(context) {
            return Some(Box::new(FetchLiveState::new()));
        }

        // Stay in ready state
        None
    }

    /// Check if money flow calculation is needed
    fn should_calculate_money_flow(context: &StateContext) -> bool {
        // Check if there are uncalculated dates or if data has changed
        let money_flow_stats = context.cache.get_money_flow_stats();

        // If we have uncalculated dates, we should calculate
        if !money_flow_stats.uncalculated_dates.is_empty() {
            return true;
        }

        // If money flow data is older than ticker data, recalculate
        if let (Some(money_flow_updated), Some(ticker_updated)) = (
            money_flow_stats.last_update,
            context.cache.get_last_ticker_update(),
        ) {
            return money_flow_updated < ticker_updated;
        }

        // If we don't have any money flow data but have ticker data, calculate
        money_flow_stats.total_calculations == 0 && context.cache.get_ticker_count() > 0
    }

    /// Check if MA score calculation is needed
    fn should_calculate_ma_score(context: &StateContext) -> bool {
        let ma_score_stats = context.cache.get_ma_score_stats();

        // If we have uncalculated dates, we should calculate
        if !ma_score_stats.uncalculated_dates.is_empty() {
            return true;
        }

        // If MA score data is older than ticker data, recalculate
        if let (Some(ma_score_updated), Some(ticker_updated)) = (
            ma_score_stats.last_update,
            context.cache.get_last_ticker_update(),
        ) {
            return ma_score_updated < ticker_updated;
        }

        // If we don't have any MA score data but have ticker data, calculate
        ma_score_stats.total_calculations == 0 && context.cache.get_ticker_count() > 0
    }

    /// Check if live data should be refreshed
    fn should_refresh_live_data(context: &StateContext) -> bool {
        // Refresh live data every 5 minutes during market hours
        // or every hour outside market hours

        let now = chrono::Utc::now();

        // Check if it's market hours
        let is_market_hours = crate::utils::is_market_hours(now) && !crate::utils::is_weekend(now);

        let refresh_interval = if is_market_hours {
            chrono::Duration::minutes(5) // 5 minutes during market hours
        } else {
            chrono::Duration::hours(1) // 1 hour outside market hours
        };

        // Get last live data update time
        if let Some(last_update) = context.cache.get_last_live_update() {
            now - last_update > refresh_interval
        } else {
            true // No live data yet, refresh immediately
        }
    }

    /// Create state instance by name
    pub fn create_state_by_name(name: &str) -> anyhow::Result<Box<dyn State + Send + Sync>> {
        let state: Box<dyn State + Send + Sync> = match name {
            "FETCH_CSV" => Box::new(FetchCSVState::new()),
            "FETCH_LIVE" => Box::new(FetchLiveState::new()),
            "MONEY_FLOW" => Box::new(MoneyFlowState::new()),
            "MA_SCORE" => Box::new(MAScoreState::new()),
            "READY" => Box::new(ReadyState::new()),
            _ => return Err(anyhow::anyhow!("Unknown state name: {}", name)),
        };

        Ok(state)
    }

    /// Validate state transition
    pub fn is_valid_transition(from: &str, to: &str) -> bool {
        match (from, to) {
            // From FETCH_CSV
            ("FETCH_CSV", "FETCH_LIVE") => true,
            ("FETCH_CSV", "FETCH_CSV") => true, // Can stay in same state

            // From FETCH_LIVE
            ("FETCH_LIVE", "FETCH_CSV") => true, // Can go back for more CSV requests
            ("FETCH_LIVE", "MONEY_FLOW") => true,
            ("FETCH_LIVE", "READY") => true,

            // From MONEY_FLOW
            ("MONEY_FLOW", "FETCH_CSV") => true, // Priority to CSV requests
            ("MONEY_FLOW", "MA_SCORE") => true,
            ("MONEY_FLOW", "READY") => true,

            // From MA_SCORE
            ("MA_SCORE", "FETCH_CSV") => true, // Priority to CSV requests
            ("MA_SCORE", "READY") => true,

            // From READY
            ("READY", "FETCH_CSV") => true,
            ("READY", "FETCH_LIVE") => true,
            ("READY", "MONEY_FLOW") => true,
            ("READY", "MA_SCORE") => true,
            ("READY", "READY") => true, // Can stay ready

            // Invalid transitions
            _ => false,
        }
    }

    /// Get recommended next states for current state (sequential flow)
    pub fn get_recommended_next_states(current_state: &str) -> Vec<&'static str> {
        match current_state {
            "FETCH_CSV" => vec!["FETCH_LIVE"],
            "FETCH_LIVE" => vec!["MONEY_FLOW"],
            "MONEY_FLOW" => vec!["MA_SCORE"],
            "MA_SCORE" => vec!["READY"],
            "READY" => vec!["FETCH_CSV", "FETCH_LIVE", "MONEY_FLOW", "MA_SCORE"],
            _ => vec![],
        }
    }
}