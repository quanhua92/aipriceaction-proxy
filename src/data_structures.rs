use crate::vci::OhlcvData;
use crate::config::OfficeHoursConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use chrono::{Datelike, Timelike, Utc, Weekday};
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

    // Get current time in the specified timezone
    let now_utc = Utc::now();
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