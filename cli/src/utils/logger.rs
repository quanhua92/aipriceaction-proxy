use tracing::{debug, error, info, warn};
use tracing_subscriber::{
    fmt::{self, time::ChronoUtc},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// Initialize logging with different levels
pub fn init_logger() -> anyhow::Result<()> {
    // Create a custom time format for Vietnam timezone
    let timer = ChronoUtc::rfc_3339();

    // Create the format layer
    let format_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_timer(timer)
        .compact();

    // Set up the environment filter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("aipriceaction=info"));

    // Initialize the subscriber
    tracing_subscriber::registry()
        .with(env_filter)
        .with(format_layer)
        .init();

    Ok(())
}

/// Logger struct for contextual logging (similar to TypeScript version)
#[derive(Debug)]
pub struct Logger {
    context: String,
}

impl Logger {
    pub fn new(context: &str) -> Self {
        Self {
            context: context.to_string(),
        }
    }

    pub fn info(&self, message: &str) {
        info!("{}: {}", self.context, message);
    }

    pub fn info_with_data<T>(&self, message: &str, data: T)
    where
        T: std::fmt::Debug,
    {
        info!("{}: {} - {:?}", self.context, message, data);
    }

    pub fn warn(&self, message: &str) {
        warn!("{}: {}", self.context, message);
    }

    pub fn warn_with_error(&self, message: &str, error: &dyn std::error::Error) {
        warn!("{}: {}: {}", self.context, message, error);
    }

    pub fn error(&self, message: &str) {
        error!("{}: {}", self.context, message);
    }

    pub fn error_with_error(&self, message: &str, error: &dyn std::error::Error) {
        error!("{}: {}: {}", self.context, message, error);
    }

    pub fn debug(&self, message: &str) {
        debug!("{}: {}", self.context, message);
    }

    pub fn debug_with_data<T>(&self, message: &str, data: T)
    where
        T: std::fmt::Debug,
    {
        debug!("{}: {} - {:?}", self.context, message, data);
    }
}

/// State transition logging (matches TypeScript logger behavior)
pub fn log_state_transition(from: &str, to: &str, reason: &str) {
    let now = chrono::Utc::now();
    info!(
        "➡️ [TRANSITION] [{}] {} → {} ({})",
        now.format("%Y-%m-%d %H:%M:%S UTC"),
        from,
        to,
        reason
    );
}

/// Hierarchical logging functions for different contexts

/// [CONSOLIDATION] - Request consolidation and batching
pub fn log_consolidation(message: &str) {
    info!("CONSOLIDATION: {}", message);
}

/// [CONSOLIDATION:DEDUP] - Deduplication of consolidated requests
pub fn log_consolidation_dedup(message: &str) {
    info!("CONSOLIDATION:DEDUP: {}", message);
}

/// [CLIENT DATA] - High-level data operations
pub fn log_client_data(message: &str) {
    info!("CLIENT_DATA: {}", message);
}

/// [BACKGROUND] - Background processing details
pub fn log_background(message: &str) {
    info!("BACKGROUND: {}", message);
}

/// [FETCH] - Data fetch operations
pub fn log_fetch(message: &str) {
    info!("FETCH: {}", message);
}

/// [FETCH_CSV] - CSV fetch operations
pub fn log_fetch_csv(message: &str) {
    info!("FETCH_CSV: {}", message);
}

/// [FETCH_LIVE] - Live data fetch operations
pub fn log_fetch_live(message: &str) {
    info!("FETCH_LIVE: {}", message);
}

/// [CACHE] - Cache operations
pub fn log_cache(message: &str) {
    info!("CACHE: {}", message);
}

/// [ERROR] - Error logging with context
pub fn log_error(message: &str) {
    error!("ERROR: {}", message);
}

/// Format date range info for logging (matches TypeScript helper)
pub fn format_date_range_info(config: &crate::models::DateRangeConfig) -> String {
    match &config.range {
        crate::models::TimeRange::Custom => {
            if let (Some(start), Some(end)) = (config.start_date, config.end_date) {
                format!(
                    "CUSTOM({} to {})",
                    start.format("%Y-%m-%d"),
                    end.format("%Y-%m-%d")
                )
            } else {
                "CUSTOM".to_string()
            }
        }
        _ => config.range.as_str().to_string(),
    }
}

/// Performance timing helper
pub struct Timer {
    start: std::time::Instant,
    name: String,
}

impl Timer {
    pub fn start(name: &str) -> Self {
        Self {
            start: std::time::Instant::now(),
            name: name.to_string(),
        }
    }

    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }

    pub fn log_elapsed(&self, _context: &str) {
        let elapsed = self.elapsed_ms();
        info!(
            "{} completed in {:.1}ms",
            self.name,
            elapsed
        );
    }
}

/// Logging macros for consistent formatting

#[macro_export]
macro_rules! log_with_context {
    ($level:ident, $context:expr, $($arg:tt)*) => {
        tracing::$level!("{}: {}", $context, format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_performance {
    ($context:expr, $name:expr, $duration_ms:expr) => {
        tracing::info!(
            "{}: {} completed in {:.1}ms",
            $context,
            $name,
            $duration_ms
        );
    };
}

#[macro_export]
macro_rules! log_batch_progress {
    ($context:expr, $current:expr, $total:expr, $item_type:expr) => {
        tracing::info!(
            "{}: Processing batch {}/{}: {} {}",
            $context,
            $current,
            $total,
            $item_type,
            if $item_type == "ticker" && $total > 1 { "tickers" } else { "items" }
        );
    };
}