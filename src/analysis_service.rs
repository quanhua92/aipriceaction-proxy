use crate::data_structures::{EnhancedTickerData, EnhancedInMemoryData};
use aipriceaction::{
    prelude::*,
    services::csv_service::CSVDataService,
    utils::vectorized_money_flow::calculate_multiple_dates_vectorized,
    utils::vectorized_ma_score::calculate_multiple_dates_vectorized_ma_score,
    utils::money_flow_utils::{MoneyFlowTickerData, build_date_range_from_data, MultipleDatesResult},
    models::ma_score::{MAScoreProcessConfig, MAScoreTickerData},
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::{DateTime, Utc};

pub struct AnalysisService {
    csv_service: CSVDataService,
    last_update: Arc<Mutex<Option<DateTime<Utc>>>>,
}

impl AnalysisService {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            csv_service: CSVDataService::new()?,
            last_update: Arc::new(Mutex::new(None)),
        })
    }

    pub async fn fetch_and_calculate(
        &self,
        tickers: Vec<String>,
        date_range: DateRangeConfig,
    ) -> Result<HashMap<String, Vec<EnhancedTickerData>>, Box<dyn std::error::Error>> {
        tracing::info!("Starting fetch and calculate for {} tickers", tickers.len());

        // 1. Fetch CSV data from GitHub
        let ticker_data = self.csv_service.fetch_tickers(&tickers, &date_range).await?;
        if ticker_data.is_empty() {
            return Ok(HashMap::new());
        }

        // 2. Prepare date range for calculations
        let dates = self.extract_date_range(&ticker_data);
        if dates.is_empty() {
            return Ok(HashMap::new());
        }

        tracing::info!("Processing {} dates for {} tickers", dates.len(), tickers.len());

        // 3. Calculate money flow
        let money_flow_result = calculate_multiple_dates_vectorized(
            &ticker_data,
            &tickers,
            &dates,
            None, // VNINDEX data if needed
            false, // vnindex_volume_weighting
            true,  // directional_colors
        );

        // 4. Calculate MA scores
        let ma_config = MAScoreProcessConfig {
            date_range_config: date_range.clone(),
            days_back: dates.len(),
            current_date: None,
            default_ma_period: 20,
        };

        let (ma_scores, _metrics) = calculate_multiple_dates_vectorized_ma_score(
            &ticker_data,
            &tickers,
            &dates,
            &ma_config,
        );

        // 5. Merge all data into enhanced structure
        let enhanced_data = self.merge_calculations(ticker_data, money_flow_result, ma_scores)?;

        // Update last update timestamp
        {
            let mut last_update = self.last_update.lock().await;
            *last_update = Some(Utc::now());
        }

        tracing::info!("Successfully calculated enhanced data for {} tickers", enhanced_data.len());
        Ok(enhanced_data)
    }

    fn extract_date_range(&self, ticker_data: &HashMap<String, Vec<StockDataPoint>>) -> Vec<String> {
        let mut all_dates = std::collections::HashSet::new();

        // Collect all dates from ticker data
        for data_points in ticker_data.values() {
            for point in data_points {
                all_dates.insert(point.time.clone());
            }
        }

        // Convert to sorted vector (chronological order)
        let mut date_vector: Vec<String> = all_dates.into_iter().collect();
        date_vector.sort();

        date_vector
    }

    fn merge_calculations(
        &self,
        ohlcv_data: HashMap<String, Vec<StockDataPoint>>,
        money_flow: MultipleDatesResult,
        ma_scores: HashMap<String, Vec<MAScoreTickerData>>,
    ) -> Result<HashMap<String, Vec<EnhancedTickerData>>, Box<dyn std::error::Error>> {
        let mut enhanced_data: HashMap<String, Vec<EnhancedTickerData>> = HashMap::new();

        for (ticker, ohlcv_points) in ohlcv_data {
            let mut enhanced_points = Vec::new();

            for ohlcv in ohlcv_points {
                let date_str = ohlcv.time.clone();

                // Extract money flow data for this ticker and date
                let money_flow_data = self.get_money_flow_for_date(&ticker, &date_str, &money_flow);

                // Extract MA score data for this ticker and date
                let ma_data = self.get_ma_score_for_date(&ticker, &date_str, &ma_scores);

                let enhanced_point = EnhancedTickerData {
                    date: date_str,
                    open: ohlcv.open,
                    high: ohlcv.high,
                    low: ohlcv.low,
                    close: ohlcv.close,
                    volume: ohlcv.volume,
                    ma10: ma_data.as_ref().and_then(|ma| ma.ma10_scores.get(&ohlcv.time).copied()),
                    ma20: ma_data.as_ref().and_then(|ma| ma.ma20_scores.get(&ohlcv.time).copied()),
                    ma50: ma_data.as_ref().and_then(|ma| ma.ma50_scores.get(&ohlcv.time).copied()),
                    money_flow: money_flow_data.as_ref().and_then(|mf| mf.signed_percentage_data.get(&ohlcv.time).copied()),
                    af: money_flow_data.as_ref().and_then(|mf| mf.activity_flow_data.get(&ohlcv.time).copied()),
                    df: money_flow_data.as_ref().and_then(|mf| mf.dollar_flow_data.get(&ohlcv.time).copied()),
                    ts: money_flow_data.as_ref().map(|mf| mf.trend_score),
                    score10: ma_data.as_ref().and_then(|ma| ma.ma10_scores.get(&ohlcv.time).copied()),
                    score20: ma_data.as_ref().and_then(|ma| ma.ma20_scores.get(&ohlcv.time).copied()),
                    score50: ma_data.as_ref().and_then(|ma| ma.ma50_scores.get(&ohlcv.time).copied()),
                };


                enhanced_points.push(enhanced_point);
            }

            if !enhanced_points.is_empty() {
                enhanced_data.insert(ticker, enhanced_points);
            }
        }

        Ok(enhanced_data)
    }

    fn get_money_flow_for_date(
        &self,
        ticker: &str,
        date: &str,
        money_flow_result: &aipriceaction::utils::money_flow_utils::MultipleDatesResult,
    ) -> Option<MoneyFlowTickerData> {
        // Find money flow data for this ticker on this date
        for ticker_data_list in money_flow_result.results.values() {
            for ticker_data in ticker_data_list {
                if ticker_data.ticker == ticker && ticker_data.daily_data.contains_key(date) {
                    return Some(ticker_data.clone());
                }
            }
        }
        None
    }

    fn get_ma_score_for_date(
        &self,
        ticker: &str,
        _date: &str,
        ma_scores: &HashMap<String, Vec<MAScoreTickerData>>,
    ) -> Option<MAScoreTickerData> {
        // Find MA score data for this ticker
        for ticker_data_list in ma_scores.values() {
            for ticker_data in ticker_data_list {
                if ticker_data.ticker == ticker {
                    return Some(ticker_data.clone());
                }
            }
        }
        None
    }

    pub async fn get_last_update(&self) -> Option<DateTime<Utc>> {
        let last_update = self.last_update.lock().await;
        *last_update
    }
}

