use super::{DateRangeConfig, StockDataPoint, TickerGroups};
use crate::utils::money_flow_utils::MoneyFlowTickerData;
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};

// Type aliases for MA Score types defined in ma_score module
type MAScoreTickerData = crate::models::ma_score::MAScoreTickerData;
type MAScoreProcessConfig = crate::models::ma_score::MAScoreProcessConfig;
type MAScorePerformanceMetrics = crate::models::ma_score::MAScorePerformanceMetrics;

/// Main cache structure holding all data
#[derive(Debug, Clone)]
pub struct ClientDataCache {
    // Core ticker data
    pub ticker_data: HashMap<String, TickerCacheEntry>,

    // VNINDEX special handling
    pub vnindex_data: Option<Vec<StockDataPoint>>,
    pub vnindex_last_updated: Option<DateTime<Utc>>,

    // Money Flow data and stats
    pub money_flow_data: HashMap<String, Vec<MoneyFlowTickerData>>, // Date -> ticker data
    pub money_flow_last_updated: Option<DateTime<Utc>>,
    pub money_flow_config: Option<VectorizedMoneyFlowConfig>,
    pub money_flow_metrics: Option<PerformanceMetrics>,

    // MA Score data and stats
    pub ma_score_data: HashMap<String, Vec<MAScoreTickerData>>, // Date -> ticker data
    pub ma_score_last_updated: Option<DateTime<Utc>>,
    pub ma_score_config: Option<MAScoreProcessConfig>,
    pub ma_score_metrics: Option<MAScorePerformanceMetrics>,

    // System data
    pub ticker_groups: Option<TickerGroups>,
    pub all_tickers: Vec<String>,
    pub last_requested_range: Option<DateRangeConfig>,

    // Changed/calculated tracking
    pub changed_dates: HashSet<String>,
    pub calculated_dates: HashSet<String>,
    pub ma_score_changed_dates: HashSet<String>,
    pub ma_score_calculated_dates: HashSet<String>,

    // System state
    pub is_initialized: bool,
    pub last_background_update: Option<DateTime<Utc>>,
}

impl ClientDataCache {
    pub fn new() -> Self {
        Self {
            ticker_data: HashMap::new(),
            vnindex_data: None,
            vnindex_last_updated: None,
            money_flow_data: HashMap::new(),
            money_flow_last_updated: None,
            money_flow_config: None,
            money_flow_metrics: None,
            ma_score_data: HashMap::new(),
            ma_score_last_updated: None,
            ma_score_config: None,
            ma_score_metrics: None,
            ticker_groups: None,
            all_tickers: Vec::new(),
            last_requested_range: None,
            changed_dates: HashSet::new(),
            calculated_dates: HashSet::new(),
            ma_score_changed_dates: HashSet::new(),
            ma_score_calculated_dates: HashSet::new(),
            is_initialized: false,
            last_background_update: None,
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn clear_money_flow_cache(&mut self) {
        self.money_flow_data.clear();
        self.money_flow_last_updated = None;
        self.money_flow_config = None;
        self.money_flow_metrics = None;
        self.changed_dates.clear();
        self.calculated_dates.clear();
    }

    pub fn clear_ma_score_cache(&mut self) {
        self.ma_score_data.clear();
        self.ma_score_last_updated = None;
        self.ma_score_config = None;
        self.ma_score_metrics = None;
        self.ma_score_changed_dates.clear();
        self.ma_score_calculated_dates.clear();
    }
}

impl Default for ClientDataCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Individual ticker cache entry
#[derive(Debug, Clone)]
pub struct TickerCacheEntry {
    pub ticker: String,
    pub data: Vec<StockDataPoint>,
    pub last_updated: DateTime<Utc>,
    pub date_range: String, // "60d", "180d", "365d", "all"
}

impl TickerCacheEntry {
    pub fn new(ticker: String, data: Vec<StockDataPoint>, date_range: String) -> Self {
        Self {
            ticker,
            data,
            last_updated: Utc::now(),
            date_range,
        }
    }

    /// Check if cache entry has sufficient data for requested range
    pub fn is_sufficient_for_range(&self, config: &DateRangeConfig) -> bool {
        if self.data.is_empty() {
            return false;
        }

        match config.range {
            super::TimeRange::All => self.date_range == "all",
            super::TimeRange::Custom => {
                // For custom ranges, check date coverage
                if let (Some(start), Some(end)) = (config.start_date, config.end_date) {
                    let cached_start = self.data.first().unwrap().date;
                    let cached_end = self.data.last().unwrap().date;
                    cached_start <= start && cached_end >= end
                } else {
                    true // Can't determine, assume sufficient
                }
            }
            _ => {
                let expected_min_points = config.range.expected_min_points();
                let tolerance = (expected_min_points as f64 * 0.8) as usize;
                self.data.len() >= tolerance
            }
        }
    }

    /// Get the latest data point
    pub fn latest_data_point(&self) -> Option<&StockDataPoint> {
        self.data.last()
    }

    /// Get data points within a date range
    pub fn get_data_in_range(&self, config: &DateRangeConfig) -> Vec<StockDataPoint> {
        StockDataPoint::filter_by_date_range(self.data.clone(), config)
    }
}


/// Configuration structures (simplified versions)
#[derive(Debug, Clone)]
pub struct VectorizedMoneyFlowConfig {
    pub date_range: DateRangeConfig,
    pub calculation_method: String,
}

/// Performance metrics (simplified versions)
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub calculation_time_ms: f64,
    pub processed_tickers: usize,
    pub processed_dates: usize,
}

/// Statistics structures for Money Flow
#[derive(Debug, Clone)]
pub struct MoneyFlowStats {
    pub total_dates: usize,
    pub total_calculations: usize,
    pub last_update: Option<DateTime<Utc>>,
    pub is_calculating: bool,
    pub last_metrics: Option<PerformanceMetrics>,
    pub uncalculated_dates: Vec<String>,
}

impl Default for MoneyFlowStats {
    fn default() -> Self {
        Self {
            total_dates: 0,
            total_calculations: 0,
            last_update: None,
            is_calculating: false,
            last_metrics: None,
            uncalculated_dates: Vec::new(),
        }
    }
}


/// State transition log entry
#[derive(Debug, Clone)]
pub struct StateTransitionLog {
    pub from: String,
    pub to: String,
    pub timestamp: DateTime<Utc>,
    pub reason: String,
}

impl StateTransitionLog {
    pub fn new(from: String, to: String, reason: String) -> Self {
        Self {
            from,
            to,
            timestamp: Utc::now(),
            reason,
        }
    }
}