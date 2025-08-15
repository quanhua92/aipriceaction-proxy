use crate::vci::OhlcvData;
use crate::config::OfficeHoursConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use chrono::{DateTime, Datelike, Timelike, Utc, Weekday};
use chrono_tz::Tz;

// --- Core Data Structures ---

#[derive(Clone, Debug)]
pub struct ActorMetadata {
    pub successful_updates: u32,
    pub failed_updates: u32,
    pub status: ActorStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ActorStatus {
    Probation,
    Trusted,
    Banned,
}

impl Default for ActorMetadata {
    fn default() -> Self {
        Self {
            successful_updates: 0,
            failed_updates: 0,
            status: ActorStatus::Probation,
        }
    }
}

// --- Type Aliases for Shared State ---

// Main in-memory cache for all stock data
pub type InMemoryData = HashMap<String, Vec<OhlcvData>>;

// Memory management constants
pub const MAX_MEMORY_MB: usize = 100;
pub const MAX_MEMORY_BYTES: usize = MAX_MEMORY_MB * 1024 * 1024;
pub const MAX_DATA_POINTS_PER_SYMBOL: usize = 100; // Limit historical data per symbol

// Memory estimation functions
pub fn estimate_ohlcv_data_size(data: &OhlcvData) -> usize {
    std::mem::size_of::<DateTime<Utc>>() +  // time
    std::mem::size_of::<f64>() * 4 +        // open, high, low, close
    std::mem::size_of::<u64>() +            // volume
    data.symbol.as_ref().map_or(0, |s| s.len()) // symbol string
}

pub fn estimate_memory_usage(data: &InMemoryData) -> usize {
    let mut total_size = std::mem::size_of::<HashMap<String, Vec<OhlcvData>>>();
    
    for (symbol, ohlcv_vec) in data {
        total_size += symbol.len(); // Key string
        total_size += std::mem::size_of::<Vec<OhlcvData>>(); // Vec overhead
        total_size += ohlcv_vec.capacity() * std::mem::size_of::<OhlcvData>(); // Vec capacity
        
        for ohlcv in ohlcv_vec {
            total_size += estimate_ohlcv_data_size(ohlcv);
        }
    }
    
    total_size
}

pub fn cleanup_old_data(data: &mut InMemoryData) -> (usize, usize) {
    let mut cleaned_symbols = 0;
    let mut cleaned_data_points = 0;
    
    for (_symbol, ohlcv_vec) in data.iter_mut() {
        if ohlcv_vec.len() > MAX_DATA_POINTS_PER_SYMBOL {
            // Sort by time and keep only the most recent data points
            ohlcv_vec.sort_by(|a, b| b.time.cmp(&a.time)); // Newest first
            let original_len = ohlcv_vec.len();
            ohlcv_vec.truncate(MAX_DATA_POINTS_PER_SYMBOL);
            cleaned_data_points += original_len - ohlcv_vec.len();
            cleaned_symbols += 1;
        }
    }
    
    (cleaned_symbols, cleaned_data_points)
}
pub type SharedData = Arc<Mutex<InMemoryData>>;

// Reputation tracker for public contributors
pub type PublicActorReputation = HashMap<IpAddr, ActorMetadata>;
pub type SharedReputation = Arc<Mutex<PublicActorReputation>>;

// Timestamp of the last trusted internal update
pub type LastInternalUpdate = Arc<Mutex<Instant>>;

// --- Office Hours State ---

#[derive(Clone, Debug)]
pub struct OfficeHoursState {
    pub is_office_hours: bool,
    pub current_interval: Duration,
    pub last_check: Instant,
}

impl Default for OfficeHoursState {
    fn default() -> Self {
        Self {
            is_office_hours: false,
            current_interval: Duration::from_secs(300), // Default to 5 minutes
            last_check: Instant::now(),
        }
    }
}

pub type SharedOfficeHoursState = Arc<Mutex<OfficeHoursState>>;

// --- Health Statistics ---

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthStats {
    // Office hours info
    pub is_office_hours: bool,
    pub current_interval_secs: u64,
    pub office_hours_enabled: bool,
    pub timezone: String,
    pub office_start_hour: u32,
    pub office_end_hour: u32,
    
    // System info
    pub environment: String,
    pub node_name: String,
    pub uptime_secs: u64,
    
    // Ticker statistics
    pub total_tickers_count: usize,
    pub active_tickers_count: usize,
    
    // Memory statistics
    pub memory_usage_bytes: usize,
    pub memory_usage_mb: f64,
    pub memory_limit_mb: usize,
    pub memory_usage_percent: f64,
    
    // Peer counts (safe - no addresses)
    pub internal_peers_count: usize,
    pub public_peers_count: usize,
    
    // Worker statistics
    pub iteration_count: u64,
    pub last_update_timestamp: Option<String>, // ISO format
    
    // Debug info
    pub current_system_time: String, // Current system time (ISO format)
    pub debug_time_override: Option<String>, // Debug time override if set
    
    // Build info
    pub build_date: Option<String>, // Build timestamp from Docker
    pub git_commit: Option<String>, // Git commit hash
}

impl Default for HealthStats {
    fn default() -> Self {
        Self {
            is_office_hours: false,
            current_interval_secs: 300,
            office_hours_enabled: true,
            timezone: "Asia/Ho_Chi_Minh".to_string(),
            office_start_hour: 9,
            office_end_hour: 16,
            environment: "development".to_string(),
            node_name: "aipriceaction-proxy".to_string(),
            uptime_secs: 0,
            total_tickers_count: 0,
            active_tickers_count: 0,
            memory_usage_bytes: 0,
            memory_usage_mb: 0.0,
            memory_limit_mb: MAX_MEMORY_MB,
            memory_usage_percent: 0.0,
            internal_peers_count: 0,
            public_peers_count: 0,
            iteration_count: 0,
            last_update_timestamp: None,
            current_system_time: Utc::now().to_rfc3339(),
            debug_time_override: None,
            build_date: None,
            git_commit: None,
        }
    }
}

pub type SharedHealthStats = Arc<Mutex<HealthStats>>;

// --- Time Functions ---

/// Get the current time, potentially overridden for debugging
/// This is the single method used throughout the system for time operations
pub fn get_current_time() -> DateTime<Utc> {
    if let Ok(debug_time_str) = std::env::var("DEBUG_SYSTEM_TIME") {
        // Only allow debug time override in non-production environments
        let environment = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        
        if environment != "production" {
            if let Ok(debug_time) = chrono::DateTime::parse_from_rfc3339(&debug_time_str) {
                tracing::warn!(
                    debug_time = %debug_time,
                    environment = %environment,
                    "⚠️  DEBUG TIME OVERRIDE ACTIVE - Using custom time instead of system time! ⚠️"
                );
                return debug_time.with_timezone(&Utc);
            } else {
                tracing::error!(
                    debug_time_str = %debug_time_str,
                    "Invalid DEBUG_SYSTEM_TIME format, falling back to system time. Expected RFC3339 format."
                );
            }
        } else {
            tracing::warn!(
                environment = %environment,
                "DEBUG_SYSTEM_TIME ignored in production environment"
            );
        }
    }
    
    Utc::now()
}

/// Get time info for health endpoint and other uses
pub fn get_time_info() -> (String, Option<String>) {
    let current_time = get_current_time();
    
    // Check if debug time override was actually applied
    let debug_override = if let Ok(debug_time_str) = std::env::var("DEBUG_SYSTEM_TIME") {
        let environment = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        
        if environment != "production" {
            // Try to parse the debug time to see if it's valid
            if chrono::DateTime::parse_from_rfc3339(&debug_time_str).is_ok() {
                Some(debug_time_str)
            } else {
                None // Invalid format, so not actually used
            }
        } else {
            None // Production environment, so not used
        }
    } else {
        None // No debug time set
    };
    
    (current_time.to_rfc3339(), debug_override)
}

// --- Ticker Groups ---

// Ticker groups loaded from JSON file
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TickerGroups(pub HashMap<String, Vec<String>>);

pub type SharedTickerGroups = Arc<TickerGroups>;

// --- Office Hours Utility Functions ---

pub fn is_within_office_hours(config: &OfficeHoursConfig) -> bool {
    let office_hours = &config.default_office_hours;
    
    // Parse timezone
    let tz: Tz = match office_hours.timezone.parse() {
        Ok(tz) => tz,
        Err(e) => {
            tracing::warn!("Failed to parse timezone '{}': {}", office_hours.timezone, e);
            return false; // Default to non-office hours if timezone parsing fails
        }
    };

    // Get current time (potentially debug-overridden) in the specified timezone
    let now_utc = get_current_time();
    let now_local = now_utc.with_timezone(&tz);
    
    // Check weekday if weekdays_only is true
    if office_hours.weekdays_only {
        let weekday = now_local.weekday();
        match weekday {
            Weekday::Mon | Weekday::Tue | Weekday::Wed | Weekday::Thu | Weekday::Fri => {
                // Continue to hour check
            }
            Weekday::Sat | Weekday::Sun => {
                return false; // Weekend - not office hours
            }
        }
    }
    
    // Check hour range
    let current_hour = now_local.hour();
    current_hour >= office_hours.start_hour && current_hour < office_hours.end_hour
}

pub fn get_current_interval(
    config: &OfficeHoursConfig, 
    core_interval: Duration, 
    non_office_interval: Duration,
    enable_office_hours: bool
) -> Duration {
    if !enable_office_hours {
        return core_interval; // Always use core interval if office hours are disabled
    }
    
    if is_within_office_hours(config) {
        core_interval
    } else {
        non_office_interval
    }
}