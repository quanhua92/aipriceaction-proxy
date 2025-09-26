use crate::services::{CacheManager, RequestQueue};

/// State context shared between all states
/// Contains services that states can use to perform their work
#[derive(Debug)]
pub struct StateContext {
    pub cache: CacheManager,
    pub request_queue: RequestQueue,
}

impl StateContext {
    /// Create a new state context
    pub fn new() -> Self {
        Self {
            cache: CacheManager::new(),
            request_queue: RequestQueue::new(),
        }
    }

    /// Check if prerequisites are loaded (ticker groups and VNINDEX)
    pub fn has_prerequisites(&self) -> bool {
        self.cache.has_ticker_groups() && self.cache.has_vnindex()
    }

    /// Get all available tickers
    pub fn get_all_tickers(&self) -> Vec<String> {
        self.cache.get_all_tickers()
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            version: self.cache.get_cache_version(),
            money_flow_version: self.cache.get_money_flow_version(),
            ma_score_version: self.cache.get_ma_score_version(),
            ticker_count: self.cache.get_ticker_count(),
            money_flow_dates: self.cache.get_money_flow_date_count(),
            ma_score_dates: self.cache.get_ma_score_date_count(),
            is_initialized: self.cache.is_initialized(),
        }
    }

    /// Get request queue statistics
    pub fn get_queue_stats(&self) -> QueueStats {
        QueueStats {
            csv_requests: self.request_queue.csv_request_count(),
            pending_requests: self.request_queue.total_pending(),
            processed_requests: self.request_queue.total_processed(),
        }
    }
}

impl Default for StateContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics for debugging
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub version: u64,
    pub money_flow_version: u64,
    pub ma_score_version: u64,
    pub ticker_count: usize,
    pub money_flow_dates: usize,
    pub ma_score_dates: usize,
    pub is_initialized: bool,
}

/// Queue statistics for debugging
#[derive(Debug, Clone)]
pub struct QueueStats {
    pub csv_requests: usize,
    pub pending_requests: usize,
    pub processed_requests: usize,
}