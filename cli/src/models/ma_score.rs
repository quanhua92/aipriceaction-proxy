use crate::models::DateRangeConfig;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// MA Score data for a single ticker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MAScoreTickerData {
    pub ticker: String,
    pub name: String,
    pub market_cap: f64,
    // Daily MA scores - all periods for comprehensive analysis
    pub ma10_scores: HashMap<String, f64>,
    pub ma20_scores: HashMap<String, f64>,
    pub ma50_scores: HashMap<String, f64>,
    // Trend analysis
    pub trend_score: f64,
    pub consecutive_days_above_ma: i32,
    pub consecutive_days_below_ma: i32,
    // Debug fields (optional)
    pub debug_data: Option<HashMap<String, MAScoreDebugData>>,
}

/// Debug data for MA score calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MAScoreDebugData {
    pub current_price: f64,
    pub ma10_value: Option<f64>,
    pub ma20_value: Option<f64>,
    pub ma50_value: Option<f64>,
}

/// MA Score data for a sector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MAScoreSectorData {
    pub sector: String,
    // Daily sector scores - all periods for comprehensive analysis
    pub ma10_sector_scores: HashMap<String, f64>,
    pub ma20_sector_scores: HashMap<String, f64>,
    pub ma50_sector_scores: HashMap<String, f64>,
    // Ticker breakdown
    pub tickers: Vec<MAScoreTickerData>,
}

/// Complete MA Score result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MAScoreResult {
    pub date_range: Vec<String>,
    pub sector_data: HashMap<String, MAScoreSectorData>,
}

/// Configuration for MA score processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MAScoreProcessConfig {
    pub date_range_config: DateRangeConfig,
    pub days_back: usize,
    pub current_date: Option<String>,
    pub default_ma_period: i32, // Default display period (10, 20, or 50)
}

/// Performance metrics for MA score calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MAScorePerformanceMetrics {
    pub calculation_time: f64,
    pub ticker_count: usize,
    pub date_count: usize,
    pub calculation_count: usize,
    pub ma_period: i32,
}

/// MA Score calculation results matrix
#[derive(Debug, Clone)]
pub struct MAScoreMatrix {
    /// MA10 scores: ((price - ma10) / ma10) * 100
    pub ma10_scores: Vec<f64>, // [tickers, dates] flattened
    /// MA10 moving averages
    pub ma10_values: Vec<f64>, // [tickers, dates] flattened
    /// MA20 scores: ((price - ma20) / ma20) * 100
    pub ma20_scores: Vec<f64>, // [tickers, dates] flattened
    /// MA20 moving averages
    pub ma20_values: Vec<f64>, // [tickers, dates] flattened
    /// MA50 scores: ((price - ma50) / ma50) * 100
    pub ma50_scores: Vec<f64>, // [tickers, dates] flattened
    /// MA50 moving averages
    pub ma50_values: Vec<f64>, // [tickers, dates] flattened
    /// Close prices for reference
    pub closes: Vec<f64>, // [tickers, dates] flattened
    pub shape: (usize, usize), // [tickers, dates]
    pub ticker_index: HashMap<String, usize>,
    pub date_index: HashMap<String, usize>,
    pub tickers: Vec<String>,
    pub dates: Vec<String>,
}

/// Single date MA score result structure
#[derive(Debug, Clone)]
pub struct SingleDateMAScoreResult {
    pub ticker: String,
    pub ma10_score: f64,
    pub ma10_value: f64,
    pub ma20_score: f64,
    pub ma20_value: f64,
    pub ma50_score: f64,
    pub ma50_value: f64,
    pub close_price: f64,
}

/// Cache statistics for MA Score calculations
#[derive(Debug, Clone, Default)]
pub struct MAScoreStats {
    pub calculated_dates: usize,
    pub uncalculated_dates: Vec<String>,
    pub changed_dates: Vec<String>,
    pub last_calculation: Option<DateTime<Utc>>,
    pub total_calculations: usize,
    pub calculation_time_ms: f64,
    pub last_update: Option<DateTime<Utc>>,
    pub is_calculating: bool,
    pub last_metrics: Option<MAScorePerformanceMetrics>,
    pub default_ma_period: u32,
}

impl MAScoreStats {
    pub fn default_with_period(period: u32) -> Self {
        Self {
            calculated_dates: 0,
            uncalculated_dates: Vec::new(),
            changed_dates: Vec::new(),
            last_calculation: None,
            total_calculations: 0,
            calculation_time_ms: 0.0,
            last_update: None,
            is_calculating: false,
            last_metrics: None,
            default_ma_period: period,
        }
    }
}

impl Default for MAScoreProcessConfig {
    fn default() -> Self {
        Self {
            date_range_config: DateRangeConfig::default_3m(),
            days_back: 60,
            current_date: None,
            default_ma_period: 20,
        }
    }
}

impl Default for MAScorePerformanceMetrics {
    fn default() -> Self {
        Self {
            calculation_time: 0.0,
            ticker_count: 0,
            date_count: 0,
            calculation_count: 0,
            ma_period: 20,
        }
    }
}

/// Calculate MA Score for a ticker on a specific date
/// MA Score = ((current_price - moving_average) / moving_average) * 100
pub fn calculate_ma_score(current_price: f64, moving_average: f64) -> Option<f64> {
    if moving_average == 0.0 || !moving_average.is_finite() {
        return None;
    }
    Some(((current_price - moving_average) / moving_average) * 100.0)
}

/// Calculate sector MA score as simple average of all tickers
pub fn calculate_sector_ma_score(ticker_scores: &[(String, Option<f64>)]) -> (f64, usize) {
    let valid_scores: Vec<f64> = ticker_scores
        .iter()
        .filter_map(|(_, score)| *score)
        .collect();

    if valid_scores.is_empty() {
        return (0.0, 0);
    }

    let sector_score = valid_scores.iter().sum::<f64>() / valid_scores.len() as f64;
    (sector_score, valid_scores.len())
}

/// Calculate consecutive days above/below MA for trend analysis
pub fn calculate_consecutive_days(
    ma_scores: &HashMap<String, f64>,
    sorted_dates: &[String],
) -> (i32, i32) {
    let mut consecutive_days_above = 0;
    let mut consecutive_days_below = 0;

    // Start from the most recent date and work backwards
    for date in sorted_dates.iter().rev() {
        if let Some(score) = ma_scores.get(date) {
            if *score > 0.0 {
                consecutive_days_above += 1;
                consecutive_days_below = 0; // Reset below counter
            } else if *score < 0.0 {
                consecutive_days_below += 1;
                consecutive_days_above = 0; // Reset above counter
            } else {
                // Score is exactly 0, break the streak
                break;
            }
        } else {
            break; // Stop if we hit missing data
        }
    }

    (consecutive_days_above, consecutive_days_below)
}

/// Calculate trend score based on recent MA performance using linear regression
pub fn calculate_ma_score_trend_score(
    ma_scores: &HashMap<String, f64>,
    sorted_dates: &[String],
    lookback_days: usize,
) -> f64 {
    let recent_dates = if sorted_dates.len() > lookback_days {
        &sorted_dates[sorted_dates.len() - lookback_days..]
    } else {
        sorted_dates
    };

    let recent_scores: Vec<f64> = recent_dates
        .iter()
        .filter_map(|date| ma_scores.get(date))
        .copied()
        .collect();

    if recent_scores.len() < 3 {
        return 0.0; // Not enough data for trend
    }

    // Calculate trend using linear regression slope
    let n = recent_scores.len() as f64;
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut sum_xy = 0.0;
    let mut sum_xx = 0.0;

    for (i, score) in recent_scores.iter().enumerate() {
        let x = i as f64; // Time index
        let y = *score; // MA score

        sum_x += x;
        sum_y += y;
        sum_xy += x * y;
        sum_xx += x * x;
    }

    // Calculate slope (trend)
    let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);

    // Normalize slope to -100 to +100 range
    slope.max(-100.0).min(100.0) * 10.0
}