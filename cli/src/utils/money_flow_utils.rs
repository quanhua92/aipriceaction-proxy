use crate::models::StockDataPoint;
use std::collections::HashMap;

/// Money flow ticker data structure matching TypeScript version
#[derive(Debug, Clone)]
pub struct MoneyFlowTickerData {
    pub ticker: String,
    pub name: String,
    pub market_cap: f64,
    pub daily_data: HashMap<String, f64>,
    pub signed_flow_data: HashMap<String, f64>,
    pub signed_percentage_data: HashMap<String, f64>,
    pub activity_flow_data: HashMap<String, f64>,
    pub dollar_flow_data: HashMap<String, f64>,
    pub volume_data: HashMap<String, VolumeData>,
    pub trend_score: f64,
    pub debug_data: Option<HashMap<String, DebugData>>,
}

/// Volume data structure
#[derive(Debug, Clone)]
pub struct VolumeData {
    pub volume: f64,
    pub change: f64,
}

/// Debug data structure
#[derive(Debug, Clone)]
pub struct DebugData {
    pub effective_low: f64,
    pub effective_high: f64,
    pub effective_range: f64,
    pub multiplier: f64,
    pub is_limit_move: bool,
    pub prev_close: f64,
    pub price_change_percent: f64,
}

/// Money flow result structure
#[derive(Debug, Clone)]
pub struct MoneyFlowResult {
    pub date_range: Vec<String>,
    pub sector_data: HashMap<String, Vec<MoneyFlowTickerData>>,
    pub vnindex_volume_scaling: HashMap<String, f64>,
}

/// Performance metrics structure
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub vectorized_time: f64,
    pub traditional_time: Option<f64>,
    pub speedup_factor: Option<f64>,
    pub ticker_count: usize,
    pub date_count: usize,
    pub calculation_count: usize,
}

/// Build date range from ticker data
pub fn build_date_range_from_data(
    ticker_data: &HashMap<String, Vec<StockDataPoint>>,
    days_back: usize,
    current_date: Option<String>,
    most_recent_date: Option<chrono::DateTime<chrono::Utc>>,
) -> Vec<String> {
    let mut all_dates = std::collections::HashSet::new();

    // Collect all dates from ticker data
    for data_points in ticker_data.values() {
        for point in data_points {
            all_dates.insert(point.time.clone());
        }
    }

    // Convert to sorted vector (chronological order)
    let mut date_vector: Vec<String> = all_dates.into_iter().collect();
    date_vector.sort(); // Sort ascending (oldest first) for consistent matrix calculations

    // Filter based on current_date or most_recent_date (working backwards from target date)
    if let Some(target_date) = current_date {
        if let Some(pos) = date_vector.iter().position(|d| d == &target_date) {
            // Take dates up to and including the target date, then limit to days_back from the end
            let end_pos = pos + 1;
            let start_pos = if end_pos > days_back { end_pos - days_back } else { 0 };
            date_vector = date_vector[start_pos..end_pos].to_vec();
        }
    } else if let Some(recent_date) = most_recent_date {
        let recent_date_str = recent_date.format("%Y-%m-%d").to_string();
        if let Some(pos) = date_vector.iter().rposition(|d| d <= &recent_date_str) {
            // Take dates up to and including the recent date, then limit to days_back from the end
            let end_pos = pos + 1;
            let start_pos = if end_pos > days_back { end_pos - days_back } else { 0 };
            date_vector = date_vector[start_pos..end_pos].to_vec();
        }
    } else {
        // No specific date filter - just take the most recent days_back dates
        if date_vector.len() > days_back {
            let start_pos = date_vector.len() - days_back;
            date_vector = date_vector[start_pos..].to_vec();
        }
    }

    date_vector
}

/// Calculate VNINDEX volume scaling factors
pub fn calculate_vnindex_volume_scaling(
    vnindex_data: Option<&[StockDataPoint]>,
    date_range: &[String],
    enable_weighting: bool,
) -> HashMap<String, f64> {
    let mut scaling_factors = HashMap::new();

    if !enable_weighting || vnindex_data.is_none() {
        // No scaling - all factors are 1.0
        for date in date_range {
            scaling_factors.insert(date.clone(), 1.0);
        }
        return scaling_factors;
    }

    let vnindex_points = vnindex_data.unwrap();

    // Create date-to-volume mapping for VNINDEX
    let mut vnindex_volume_map = HashMap::new();
    for point in vnindex_points {
        vnindex_volume_map.insert(point.time.clone(), point.volume);
    }

    // Calculate average volume for normalization
    let total_volume: f64 = vnindex_points.iter().map(|p| p.volume as f64).sum();
    let avg_volume = if vnindex_points.is_empty() {
        1.0
    } else {
        total_volume / vnindex_points.len() as f64
    };

    // Calculate scaling factors
    for date in date_range {
        let volume = vnindex_volume_map.get(date).map(|v| *v as f64).unwrap_or(avg_volume);
        let scaling_factor = if avg_volume > 0.0 {
            volume / avg_volume
        } else {
            1.0
        };
        scaling_factors.insert(date.clone(), scaling_factor);
    }

    scaling_factors
}

/// Vectorized money flow configuration
#[derive(Debug, Clone)]
pub struct VectorizedMoneyFlowConfig {
    pub days_back: usize,
    pub current_date: Option<String>,
    pub vnindex_volume_weighting: bool,
    pub directional_colors: bool,
    pub enable_vectorization: bool,
}

impl Default for VectorizedMoneyFlowConfig {
    fn default() -> Self {
        Self {
            days_back: 30,
            current_date: None,
            vnindex_volume_weighting: true,
            directional_colors: true,
            enable_vectorization: true,
        }
    }
}

/// Check if vectorization is enabled (always true in Rust implementation)
pub fn is_vectorization_enabled() -> bool {
    true
}

/// Multiple dates vectorized calculation result
#[derive(Debug, Clone)]
pub struct MultipleDatesResult {
    pub results: HashMap<String, Vec<MoneyFlowTickerData>>,
    pub metrics: PerformanceMetrics,
}

/// Single date vectorized calculation result
#[derive(Debug, Clone)]
pub struct SingleDateResult {
    pub result: Vec<MoneyFlowTickerData>,
    pub metrics: PerformanceMetrics,
}