use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StockDataPoint {
    pub ticker: String,
    pub time: String,    // Format: YYYY-MM-DD
    pub date: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
}

impl StockDataPoint {
    pub fn new(
        ticker: String,
        time: String,
        date: DateTime<Utc>,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: i64,
    ) -> Self {
        Self {
            ticker,
            time,
            date,
            open,
            high,
            low,
            close,
            volume,
        }
    }

    /// Scale prices for VND stocks (multiply by 1000), but not for VNINDEX
    pub fn scale_prices(&mut self) {
        if self.ticker != "VNINDEX" {
            self.open *= 1000.0;
            self.high *= 1000.0;
            self.low *= 1000.0;
            self.close *= 1000.0;
        }
    }

    /// Check if this point has the same date as another
    pub fn same_date(&self, other: &StockDataPoint) -> bool {
        self.time == other.time
    }

    /// Check if data values are different from another point (excluding MA values)
    pub fn data_changed(&self, other: &StockDataPoint) -> bool {
        (self.open - other.open).abs() > 0.01
            || (self.high - other.high).abs() > 0.01
            || (self.low - other.low).abs() > 0.01
            || (self.close - other.close).abs() > 0.01
            || (self.volume - other.volume).abs() > 1
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TimeRange {
    #[serde(rename = "1W")]
    OneWeek,
    #[serde(rename = "2W")]
    TwoWeeks,
    #[serde(rename = "1M")]
    OneMonth,
    #[serde(rename = "2M")]
    TwoMonths,
    #[serde(rename = "3M")]
    ThreeMonths,
    #[serde(rename = "4M")]
    FourMonths,
    #[serde(rename = "6M")]
    SixMonths,
    #[serde(rename = "1Y")]
    OneYear,
    #[serde(rename = "2Y")]
    TwoYears,
    #[serde(rename = "ALL")]
    All,
    #[serde(rename = "CUSTOM")]
    Custom,
}

impl TimeRange {
    pub fn as_str(&self) -> &'static str {
        match self {
            TimeRange::OneWeek => "1W",
            TimeRange::TwoWeeks => "2W",
            TimeRange::OneMonth => "1M",
            TimeRange::TwoMonths => "2M",
            TimeRange::ThreeMonths => "3M",
            TimeRange::FourMonths => "4M",
            TimeRange::SixMonths => "6M",
            TimeRange::OneYear => "1Y",
            TimeRange::TwoYears => "2Y",
            TimeRange::All => "ALL",
            TimeRange::Custom => "CUSTOM",
        }
    }

    pub fn expected_min_points(&self) -> usize {
        match self {
            TimeRange::OneWeek => 5,
            TimeRange::TwoWeeks => 10,
            TimeRange::OneMonth => 20,
            TimeRange::TwoMonths => 40,
            TimeRange::ThreeMonths => 60,
            TimeRange::FourMonths => 80,
            TimeRange::SixMonths => 120,
            TimeRange::OneYear => 250,
            TimeRange::TwoYears => 500,
            TimeRange::All => 1000,
            TimeRange::Custom => 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRangeConfig {
    pub range: TimeRange,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

impl DateRangeConfig {
    pub fn new(range: TimeRange) -> Self {
        Self {
            range,
            start_date: None,
            end_date: None,
        }
    }

    pub fn custom(start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Self {
        Self {
            range: TimeRange::Custom,
            start_date: Some(start_date),
            end_date: Some(end_date),
        }
    }

    pub fn default_3m() -> Self {
        Self::new(TimeRange::ThreeMonths)
    }

    pub fn default_1m() -> Self {
        Self::new(TimeRange::OneMonth)
    }

    pub fn default_1y() -> Self {
        Self::new(TimeRange::OneYear)
    }
}

/// Raw CSV data format from GitHub
#[derive(Debug, Deserialize)]
pub struct RawStockData {
    pub ticker: String,
    pub time: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
}

impl RawStockData {
    /// Convert to StockDataPoint with proper date parsing and scaling
    pub fn to_stock_data_point(&self) -> anyhow::Result<StockDataPoint> {
        // Parse date from YYYY-MM-DD format to UTC DateTime
        let naive_date = NaiveDate::parse_from_str(&self.time, "%Y-%m-%d")?;
        let date = naive_date.and_hms_opt(0, 0, 0)
            .ok_or_else(|| anyhow::anyhow!("Invalid date format"))?
            .and_utc();

        let mut point = StockDataPoint::new(
            self.ticker.clone(),
            self.time.clone(),
            date,
            self.open,
            self.high,
            self.low,
            self.close,
            self.volume,
        );

        // Apply price scaling for VND stocks
        point.scale_prices();

        Ok(point)
    }
}

/// Live data format from API
#[derive(Debug, Deserialize)]
pub struct LiveTickerData {
    #[serde(flatten)]
    pub tickers: HashMap<String, Vec<LiveDataPoint>>,
}

#[derive(Debug, Deserialize)]
pub struct LiveDataPoint {
    pub time: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
    pub symbol: String,
}

impl LiveDataPoint {
    /// Convert to StockDataPoint (no scaling needed - API values already in correct format)
    pub fn to_stock_data_point(&self, ticker: &str) -> anyhow::Result<StockDataPoint> {
        let naive_date = NaiveDate::parse_from_str(&self.time, "%Y-%m-%d")?;
        let date = naive_date.and_hms_opt(0, 0, 0)
            .ok_or_else(|| anyhow::anyhow!("Invalid date format"))?
            .and_utc();

        Ok(StockDataPoint::new(
            ticker.to_string(),
            self.time.clone(),
            date,
            self.open,    // No scaling - API values already in correct format
            self.high,
            self.low,
            self.close,
            self.volume,
        ))
    }
}

/// Data filtering utilities
impl StockDataPoint {
    /// Filter data points by date range
    pub fn filter_by_date_range(
        data: Vec<StockDataPoint>,
        config: &DateRangeConfig,
    ) -> Vec<StockDataPoint> {
        match config.range {
            TimeRange::Custom => {
                data.into_iter()
                    .filter(|point| {
                        if let Some(start) = config.start_date {
                            if point.date < start {
                                return false;
                            }
                        }
                        if let Some(end) = config.end_date {
                            if point.date > end {
                                return false;
                            }
                        }
                        true
                    })
                    .collect()
            }
            TimeRange::All => data,
            _ => {
                let cutoff_date = Self::calculate_cutoff_date(&config.range);
                data.into_iter()
                    .filter(|point| point.date >= cutoff_date)
                    .collect()
            }
        }
    }

    /// Calculate cutoff date for time ranges
    fn calculate_cutoff_date(range: &TimeRange) -> DateTime<Utc> {
        let now = Utc::now();
        match range {
            TimeRange::OneWeek => now - chrono::Duration::weeks(1),
            TimeRange::TwoWeeks => now - chrono::Duration::weeks(2),
            TimeRange::OneMonth => now - chrono::Duration::weeks(4),
            TimeRange::TwoMonths => now - chrono::Duration::weeks(8),
            TimeRange::ThreeMonths => now - chrono::Duration::weeks(12),
            TimeRange::FourMonths => now - chrono::Duration::weeks(16),
            TimeRange::SixMonths => now - chrono::Duration::weeks(24),
            TimeRange::OneYear => now - chrono::Duration::weeks(52),
            TimeRange::TwoYears => now - chrono::Duration::weeks(104),
            _ => DateTime::from_timestamp(0, 0).unwrap(), // Very old date for ALL and CUSTOM
        }
    }
}