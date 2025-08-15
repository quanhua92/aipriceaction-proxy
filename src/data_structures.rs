use crate::vci::OhlcvData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

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

// --- Ticker Groups ---

// Ticker groups loaded from JSON file
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TickerGroups(pub HashMap<String, Vec<String>>);

pub type SharedTickerGroups = Arc<TickerGroups>;