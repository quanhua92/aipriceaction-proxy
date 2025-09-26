pub mod context;
pub mod states;
pub mod transitions;

pub use context::*;
pub use states::*;
pub use transitions::*;

use crate::{
    utils::log_state_transition,
    models::StateTransitionLog,
    services::{CacheManager, RequestQueue},
    utils::Logger,
};
use std::{
    sync::{Arc, Mutex},
};
use tokio::sync::RwLock;

/// Core state machine for managing stock data fetching and processing
pub struct ClientDataStateMachine {
    current_state: Box<dyn State + Send + Sync>,
    context: Arc<RwLock<StateContext>>,
    transition_history: Arc<Mutex<Vec<StateTransitionLog>>>,
    is_running: Arc<Mutex<bool>>,
    tick_count: Arc<Mutex<u64>>,
    logger: Logger,
}

impl ClientDataStateMachine {
    /// Create new state machine instance
    pub fn new() -> Self {
        let cache_manager = CacheManager::new();
        let request_queue = RequestQueue::new();

        let context = StateContext {
            cache: cache_manager,
            request_queue,
        };

        let initial_state = Box::new(crate::states::FetchCSVState::new());

        Self {
            current_state: initial_state,
            context: Arc::new(RwLock::new(context)),
            transition_history: Arc::new(Mutex::new(Vec::new())),
            is_running: Arc::new(Mutex::new(false)),
            tick_count: Arc::new(Mutex::new(0)),
            logger: Logger::new("STATE_MACHINE"),
        }
    }

    /// Start the state machine with 5-second tick interval
    pub async fn start(&mut self) -> anyhow::Result<()> {
        {
            let mut running = self.is_running.lock().unwrap();
            if *running {
                self.logger.warn("State machine already running");
                return Ok(());
            }
            *running = true;
        }

        let start_time = std::time::Instant::now();
        self.logger.info("Starting state machine with 5-second game loop");

        // Enter initial state
        let enter_start = std::time::Instant::now();
        {
            let mut context = self.context.write().await;
            self.current_state.enter(&mut context).await?;
        }
        let enter_duration = enter_start.elapsed();
        self.logger.info(&format!("Initial state entry completed in {:.1}ms", enter_duration.as_secs_f64() * 1000.0));

        // Process states until reaching a stable state (READY) - no timer needed
        loop {
            // Check if we should continue running
            {
                let running = self.is_running.lock().unwrap();
                if !*running {
                    break;
                }
            }

            // Execute one tick - each tick should complete all work for that state
            let tick_start = std::time::Instant::now();
            match self.tick().await {
                Ok(()) => {
                    let current_state_name = self.current_state.name();
                    if current_state_name == "READY" {
                        // First time reaching READY state - log initial completion
                        let tick_count = *self.tick_count.lock().unwrap();
                        if tick_count <= 5 {  // Only show this message once
                            let total_duration = start_time.elapsed();
                            let ready_time = chrono::Utc::now();
                            self.logger.info(&format!(
                                "[{}] Reached READY state - initial data processing complete",
                                ready_time.format("%Y-%m-%d %H:%M:%S UTC")
                            ));
                            self.logger.info(&format!(
                                "[{}] 🚀 INITIAL LOAD PERFORMANCE: State machine ready in {:.2}s ({:.0}ms)",
                                ready_time.format("%Y-%m-%d %H:%M:%S UTC"),
                                total_duration.as_secs_f64(),
                                total_duration.as_secs_f64() * 1000.0
                            ));

                            // Show performance breakdown
                            let stats = self.get_stats();
                            self.logger.info(&format!(
                                "[{}] 📊 READY STATE: {} ticks processed, {} state transitions",
                                ready_time.format("%Y-%m-%d %H:%M:%S UTC"),
                                stats.tick_count,
                                stats.transition_count
                            ));
                        }

                        // In READY state, wait 5 seconds before next tick (periodic monitoring)
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    } else {
                        // Log tick performance for non-READY states
                        let tick_duration = tick_start.elapsed();
                        if tick_duration.as_millis() > 10 {  // Only log if tick took more than 10ms
                            let perf_time = chrono::Utc::now();
                            self.logger.info(&format!(
                                "[{}] ⏱️ {} tick completed in {:.1}ms",
                                perf_time.format("%Y-%m-%d %H:%M:%S UTC"),
                                current_state_name,
                                tick_duration.as_secs_f64() * 1000.0
                            ));
                        }
                    }
                    // Continue immediately to next state (no delay for processing states)
                }
                Err(e) => {
                    self.logger.error_with_error("Tick error", &*e);
                    break;
                }
            }
        }

        self.logger.info("State machine stopped");
        Ok(())
    }

    /// Stop the state machine
    pub async fn stop(&mut self) -> anyhow::Result<()> {
        {
            let mut running = self.is_running.lock().unwrap();
            if !*running {
                return Ok(());
            }
            *running = false;
        }

        self.logger.info("Stopping state machine");

        // Exit current state
        {
            let mut context = self.context.write().await;
            self.current_state.exit(&mut context).await?;
        }

        Ok(())
    }

    /// Execute a single tick
    async fn tick(&mut self) -> anyhow::Result<()> {
        let tick_number = {
            let mut count = self.tick_count.lock().unwrap();
            *count += 1;
            *count
        };

        self.logger
            .debug(&format!("Tick #{} in {}", tick_number, self.current_state.name()));

        // Execute current state tick
        let now = chrono::Utc::now();
        self.logger.debug(&format!(
            "[{}] 🔄 [TICK] Starting tick for state: {}",
            now.format("%Y-%m-%d %H:%M:%S UTC"),
            self.current_state.name()
        ));

        let next_state = {
            let mut context = self.context.write().await;
            self.current_state.tick(&mut context).await?
        };

        let tick_complete_time = chrono::Utc::now();
        self.logger.debug(&format!(
            "[{}] ✅ [TICK] Tick completed for state: {} - next_state: {}",
            tick_complete_time.format("%Y-%m-%d %H:%M:%S UTC"),
            self.current_state.name(),
            if next_state.is_some() { "Some(transition)" } else { "None" }
        ));

        // Check if we need to transition
        if let Some(new_state) = next_state {
            let from_name = self.current_state.name().to_string();
            let _to_name = new_state.name().to_string();
            let reason = format!("State tick requested transition from {}", from_name);

            self.transition_to(new_state, reason).await?;
        }

        Ok(())
    }

    /// Transition to a new state
    async fn transition_to(
        &mut self,
        new_state: Box<dyn State + Send + Sync>,
        reason: String,
    ) -> anyhow::Result<()> {
        let from_name = self.current_state.name().to_string();
        let to_name = new_state.name().to_string();

        log_state_transition(&from_name, &to_name, &reason);

        // Record transition in history
        {
            let mut history = self.transition_history.lock().unwrap();
            history.push(StateTransitionLog::new(from_name.clone(), to_name.clone(), reason));

            // Keep history manageable (last 100 transitions)
            if history.len() > 100 {
                history.remove(0);
            }
        }

        // Exit current state
        {
            let mut context = self.context.write().await;
            self.current_state.exit(&mut context).await?;
        }

        // Switch to new state
        self.current_state = new_state;

        // Enter new state
        {
            let mut context = self.context.write().await;
            self.current_state.enter(&mut context).await?;
        }

        self.logger.info(&format!(
            "State transition completed: {} → {}",
            from_name, to_name
        ));

        // Immediately tick the new state
        self.logger.debug(&format!(
            "Immediately ticking new state {} after transition",
            to_name
        ));

        let immediate_next_state = {
            let mut context = self.context.write().await;
            self.current_state.tick(&mut context).await?
        };

        // Handle immediate transitions
        if let Some(next_state) = immediate_next_state {
            let next_name = next_state.name().to_string();
            self.logger.info(&format!(
                "Immediate transition requested: {} → {}",
                to_name, next_name
            ));

            let immediate_reason = format!("Immediate transition from {}", to_name);
            Box::pin(self.transition_to(next_state, immediate_reason)).await?;
        }

        Ok(())
    }

    /// Get current state name
    pub fn current_state_name(&self) -> String {
        self.current_state.name().to_string()
    }

    /// Get transition history
    pub fn get_transition_history(&self) -> Vec<StateTransitionLog> {
        self.transition_history.lock().unwrap().clone()
    }

    /// Check if machine is running
    pub fn is_running(&self) -> bool {
        *self.is_running.lock().unwrap()
    }

    /// Check if system is ready to handle requests
    pub fn is_ready(&self) -> bool {
        self.is_running() && self.current_state.name() == "READY"
    }

    /// Check if system is currently fetching data
    pub fn is_fetching(&self) -> bool {
        matches!(self.current_state.name(), "FETCH_CSV" | "FETCH_LIVE")
    }

    /// Check if system is calculating
    pub fn is_calculating(&self) -> bool {
        matches!(self.current_state.name(), "MONEY_FLOW" | "MA_SCORE")
    }

    /// Get statistics
    pub fn get_stats(&self) -> StateMachineStats {
        let tick_count = *self.tick_count.lock().unwrap();
        let transition_count = self.transition_history.lock().unwrap().len();

        StateMachineStats {
            current_state: self.current_state_name(),
            is_running: self.is_running(),
            tick_count,
            transition_count,
            uptime_seconds: self.calculate_uptime(),
        }
    }

    /// Calculate uptime in seconds
    fn calculate_uptime(&self) -> u64 {
        let history = self.transition_history.lock().unwrap();
        if history.is_empty() {
            return 0;
        }

        let first_transition = &history[0];
        let elapsed = chrono::Utc::now() - first_transition.timestamp;
        elapsed.num_seconds() as u64
    }

    /// Force transition to specific state (for debugging)
    pub async fn force_transition(&mut self, state_name: &str, reason: String) -> anyhow::Result<()> {
        let new_state: Box<dyn State + Send + Sync> = match state_name {
            "FETCH_CSV" => Box::new(crate::states::FetchCSVState::new()),
            "FETCH_LIVE" => Box::new(crate::states::FetchLiveState::new()),
            "MONEY_FLOW" => Box::new(crate::states::MoneyFlowState::new()),
            "MA_SCORE" => Box::new(crate::states::MAScoreState::new()),
            "READY" => Box::new(crate::states::ReadyState::new()),
            _ => return Err(anyhow::anyhow!("Invalid state name: {}", state_name)),
        };

        self.logger.warn(&format!(
            "Forcing transition to {}: {}",
            state_name, reason
        ));

        self.transition_to(new_state, reason).await?;
        Ok(())
    }

    /// Get context for external access (read-only)
    pub fn get_context(&self) -> Arc<RwLock<StateContext>> {
        Arc::clone(&self.context)
    }
}

#[derive(Debug, Clone)]
pub struct StateMachineStats {
    pub current_state: String,
    pub is_running: bool,
    pub tick_count: u64,
    pub transition_count: usize,
    pub uptime_seconds: u64,
}