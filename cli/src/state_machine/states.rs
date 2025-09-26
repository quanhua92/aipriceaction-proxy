use super::StateContext;

/// State trait that all states must implement
/// Based on the TypeScript BaseState abstract class
#[async_trait::async_trait]
pub trait State {
    /// Get the name of this state
    fn name(&self) -> &'static str;

    /// Called when entering this state
    async fn enter(&mut self, context: &mut StateContext) -> anyhow::Result<()>;

    /// Called when exiting this state
    async fn exit(&mut self, context: &mut StateContext) -> anyhow::Result<()>;

    /// Called every 5 seconds (game loop tick)
    /// Returns Some(next_state) to transition, or None to stay in current state
    async fn tick(
        &mut self,
        context: &mut StateContext,
    ) -> anyhow::Result<Option<Box<dyn State + Send + Sync>>>;

    /// Handle specific events (optional - not implemented yet)
    async fn handle_event(
        &mut self,
        _event: StateEvent,
        _context: &mut StateContext,
    ) -> anyhow::Result<Option<Box<dyn State + Send + Sync>>> {
        Ok(None) // Default: don't handle events
    }
}

/// Base state implementation with common functionality
/// Provides helper methods that concrete states can use
pub struct BaseStateImpl {
    pub name: &'static str,
    pub logger: crate::utils::Logger,
}

impl BaseStateImpl {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            logger: crate::utils::Logger::new(name),
        }
    }

    /// Helper method to log messages with state context
    pub fn log(&self, level: &str, message: &str) {
        match level {
            "info" => self.logger.info(message),
            "warn" => self.logger.warn(message),
            "error" => self.logger.error(message),
            "debug" => self.logger.debug(message),
            _ => self.logger.info(message),
        }
    }

    /// Helper method to log with additional data
    pub fn log_with_data<T: std::fmt::Debug>(&self, level: &str, message: &str, data: T) {
        match level {
            "info" => self.logger.info_with_data(message, data),
            "debug" => self.logger.debug_with_data(message, data),
            _ => self.logger.info_with_data(message, data),
        }
    }

    /// Helper method to create transition reason
    pub fn create_transition_reason(&self, reason: &str) -> String {
        format!("{}: {}", self.name, reason)
    }

    /// Helper method to determine if we should stay in current state
    pub fn stay_in_state(&self, reason: &str) -> Option<Box<dyn State + Send + Sync>> {
        self.log("debug", &format!("Staying in state: {}", reason));
        None
    }

    /// Helper method to determine if we should transition to another state
    pub fn should_transition_to(
        &self,
        next_state: Box<dyn State + Send + Sync>,
        reason: &str,
    ) -> Option<Box<dyn State + Send + Sync>> {
        self.log("info", &format!("Transitioning: {}", reason));
        Some(next_state)
    }
}

/// State events for event-driven transitions (not implemented yet)
#[derive(Debug, Clone)]
pub struct StateEvent {
    pub event_type: StateEventType,
    pub payload: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub enum StateEventType {
    ComponentRequest,
    DataFetched,
    LiveUpdate,
    CalculationComplete,
    MarketHours,
    Tick,
}

/// State names enum for type safety
#[derive(Debug, Clone, PartialEq)]
pub enum StateName {
    FetchCsv,
    FetchLive,
    MoneyFlow,
    MaScore,
    Ready,
}

impl StateName {
    pub fn as_str(&self) -> &'static str {
        match self {
            StateName::FetchCsv => "FETCH_CSV",
            StateName::FetchLive => "FETCH_LIVE",
            StateName::MoneyFlow => "MONEY_FLOW",
            StateName::MaScore => "MA_SCORE",
            StateName::Ready => "READY",
        }
    }
}

impl std::fmt::Display for StateName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}