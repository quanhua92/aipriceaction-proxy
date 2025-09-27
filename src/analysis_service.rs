use crate::data_structures::{EnhancedTickerData, EnhancedInMemoryData};
use aipriceaction::{
    prelude::*,
    services::csv_service::CSVDataService,
    utils::vectorized_money_flow::calculate_multiple_dates_vectorized,
    utils::vectorized_ma_score::calculate_multiple_dates_vectorized_ma_score,
    utils::money_flow_utils::{MoneyFlowTickerData, build_date_range_from_data, MultipleDatesResult, PerformanceMetrics},
    models::{ma_score::{MAScoreProcessConfig, MAScoreTickerData}, stock_data::TimeRange},
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
        tracing::info!("Starting fetch and calculate for {} tickers with date_range: {:?}", tickers.len(), date_range);

        // 1. Fetch CSV data from GitHub using provided date range for enhanced data calculations
        let fetch_range = date_range.clone();
        tracing::debug!("About to call csv_service.fetch_tickers with range: {:?}", fetch_range);
        let ticker_data = self.csv_service.fetch_tickers(&tickers, &fetch_range).await?;
        if ticker_data.is_empty() {
            return Ok(HashMap::new());
        }

        // 2. Data is already filtered by the requested range from fetch, no additional filtering needed
        let filtered_data = ticker_data.clone();

        // 3. Prepare date range for calculations (from filtered data)
        let dates = self.extract_date_range(&filtered_data);
        if dates.is_empty() {
            return Ok(HashMap::new());
        }

        tracing::info!("Processing {} dates for {} tickers (downloaded {} points for largest ticker)",
                      dates.len(), tickers.len(),
                      ticker_data.values().map(|v| v.len()).max().unwrap_or(0));

        // 4. Choose calculation strategy based on date count
        const LARGE_DATASET_THRESHOLD: usize = 500;
        const BATCH_SIZE: usize = 200;

        if dates.len() > LARGE_DATASET_THRESHOLD {
            tracing::info!("Large dataset detected ({} dates), using batched calculation", dates.len());
            let (money_flow_result, ma_scores) = self.calculate_batched(
                filtered_data.clone(),
                tickers.clone(),
                dates.clone(),
                date_range.clone(),
                BATCH_SIZE
            ).await?;
            let enhanced_data = self.merge_calculations(filtered_data, money_flow_result, ma_scores)?;

            // Update last update timestamp
            {
                let mut last_update = self.last_update.lock().await;
                *last_update = Some(Utc::now());
            }

            tracing::info!("Successfully calculated enhanced data for {} tickers using batched approach", enhanced_data.len());
            return Ok(enhanced_data);
        }

        tracing::info!("Using standard calculation for {} dates", dates.len());

        // 4. Calculate money flow on filtered data using spawn_blocking to avoid blocking async runtime
        let filtered_data_clone = filtered_data.clone();
        let tickers_clone = tickers.clone();
        let dates_clone = dates.clone();

        let money_flow_result = tokio::task::spawn_blocking(move || {
            tracing::info!("Starting money flow calculation in blocking thread");
            calculate_multiple_dates_vectorized(
                &filtered_data_clone,
                &tickers_clone,
                &dates_clone,
                None, // VNINDEX data if needed
                false, // vnindex_volume_weighting
                true,  // directional_colors
                &["VNINDEX".to_string()], // excluded_tickers
            )
        }).await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        // 5. Calculate MA scores on filtered data using spawn_blocking
        let filtered_data_clone = filtered_data.clone();
        let tickers_clone = tickers.clone();
        let dates_clone = dates.clone();
        let ma_config = MAScoreProcessConfig {
            date_range_config: date_range.clone(),
            days_back: dates.len(),
            current_date: None,
            default_ma_period: 20,
        };

        let (ma_scores, _metrics) = tokio::task::spawn_blocking(move || {
            tracing::info!("Starting MA score calculation in blocking thread");
            calculate_multiple_dates_vectorized_ma_score(
                &filtered_data_clone,
                &tickers_clone,
                &dates_clone,
                &ma_config,
                &["VNINDEX".to_string()], // excluded_tickers
            )
        }).await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        // 6. Merge all data into enhanced structure
        let enhanced_data = self.merge_calculations(filtered_data, money_flow_result, ma_scores)?;

        // Update last update timestamp
        {
            let mut last_update = self.last_update.lock().await;
            *last_update = Some(Utc::now());
        }

        tracing::info!("Successfully calculated enhanced data for {} tickers", enhanced_data.len());
        Ok(enhanced_data)
    }

    fn filter_data_by_range(
        &self,
        ticker_data: &HashMap<String, Vec<StockDataPoint>>,
        date_range: &DateRangeConfig,
    ) -> HashMap<String, Vec<StockDataPoint>> {
        ticker_data
            .iter()
            .map(|(ticker, data_points)| {
                let filtered_points = StockDataPoint::filter_by_date_range(data_points.clone(), date_range);
                (ticker.clone(), filtered_points)
            })
            .collect()
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


                // For MA scores, use the latest available score if the exact date isn't available
                let get_latest_ma_score = |ma: &MAScoreTickerData, scores: &std::collections::HashMap<String, f64>| -> Option<f64> {
                    // First try to get the exact date
                    if let Some(score) = scores.get(&ohlcv.time) {
                        return Some(*score);
                    }

                    // If not found, get the latest available score (most recent date)
                    if let Some((_, &score)) = scores.iter().max_by_key(|(date, _)| *date) {
                        return Some(score);
                    }

                    None
                };

                let enhanced_point = EnhancedTickerData {
                    date: date_str,
                    open: ohlcv.open,
                    high: ohlcv.high,
                    low: ohlcv.low,
                    close: ohlcv.close,
                    volume: ohlcv.volume,
                    ma10: ma_data.as_ref().and_then(|ma| get_latest_ma_score(ma, &ma.ma10_scores)),
                    ma20: ma_data.as_ref().and_then(|ma| get_latest_ma_score(ma, &ma.ma20_scores)),
                    ma50: ma_data.as_ref().and_then(|ma| get_latest_ma_score(ma, &ma.ma50_scores)),
                    money_flow: money_flow_data.as_ref().and_then(|mf| mf.signed_percentage_data.get(&ohlcv.time).copied()),
                    af: money_flow_data.as_ref().and_then(|mf| mf.activity_flow_data.get(&ohlcv.time).copied()),
                    df: money_flow_data.as_ref().and_then(|mf| mf.dollar_flow_data.get(&ohlcv.time).copied()),
                    ts: money_flow_data.as_ref().map(|mf| mf.trend_score),
                    score10: ma_data.as_ref().and_then(|ma| get_latest_ma_score(ma, &ma.ma10_scores)),
                    score20: ma_data.as_ref().and_then(|ma| get_latest_ma_score(ma, &ma.ma20_scores)),
                    score50: ma_data.as_ref().and_then(|ma| get_latest_ma_score(ma, &ma.ma50_scores)),
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
        date: &str,
        ma_scores: &HashMap<String, Vec<MAScoreTickerData>>,
    ) -> Option<MAScoreTickerData> {
        // First try to find MA score data for the specific date
        if let Some(ticker_data_list) = ma_scores.get(date) {
            for ticker_data in ticker_data_list {
                if ticker_data.ticker == ticker {
                    return Some(ticker_data.clone());
                }
            }
        }

        // If not found for specific date, try to find the ticker in any date (fallback)
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

    async fn calculate_batched(
        &self,
        filtered_data: HashMap<String, Vec<StockDataPoint>>,
        tickers: Vec<String>,
        dates: Vec<String>,
        date_range: DateRangeConfig,
        batch_size: usize,
    ) -> Result<(MultipleDatesResult, HashMap<String, Vec<MAScoreTickerData>>), Box<dyn std::error::Error>> {
        tracing::info!("Starting batched calculation for {} dates with batch size {}", dates.len(), batch_size);

        // Combine money flow results and MA scores from all batches
        let mut combined_money_flow_results = HashMap::new();
        let mut combined_ma_scores = HashMap::new();

        // Process dates in batches
        for (batch_num, date_batch) in dates.chunks(batch_size).enumerate() {
            let batch_num = batch_num + 1;
            let total_batches = (dates.len() + batch_size - 1) / batch_size;

            tracing::info!("Processing batch {}/{}: {} dates", batch_num, total_batches, date_batch.len());

            // Clone data for this batch
            let filtered_data_clone = filtered_data.clone();
            let tickers_clone = tickers.clone();
            let date_batch_clone = date_batch.to_vec();

            // Calculate money flow for this batch
            let money_flow_result = tokio::task::spawn_blocking(move || {
                tracing::info!("Calculating money flow for batch {}/{}", batch_num, total_batches);
                calculate_multiple_dates_vectorized(
                    &filtered_data_clone,
                    &tickers_clone,
                    &date_batch_clone,
                    None, // VNINDEX data if needed
                    false, // vnindex_volume_weighting
                    true,  // directional_colors
                    &["VNINDEX".to_string()], // excluded_tickers
                )
            }).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

            // Calculate MA scores for this batch
            let filtered_data_clone = filtered_data.clone();
            let tickers_clone = tickers.clone();
            let date_batch_clone = date_batch.to_vec();
            let ma_config = MAScoreProcessConfig {
                date_range_config: date_range.clone(),
                days_back: date_batch.len(),
                current_date: None,
                default_ma_period: 20,
            };

            let (ma_scores, _metrics) = tokio::task::spawn_blocking(move || {
                tracing::info!("Calculating MA scores for batch {}/{}", batch_num, total_batches);
                calculate_multiple_dates_vectorized_ma_score(
                    &filtered_data_clone,
                    &tickers_clone,
                    &date_batch_clone,
                    &ma_config,
                    &["VNINDEX".to_string()], // excluded_tickers
                )
            }).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

            // Merge results from this batch
            for (date, ticker_data_list) in money_flow_result.results {
                combined_money_flow_results.insert(date, ticker_data_list);
            }

            for (date, ticker_data_list) in ma_scores {
                combined_ma_scores.insert(date, ticker_data_list);
            }

            tracing::info!("Completed batch {}/{}", batch_num, total_batches);

            // Yield to async runtime between batches to keep server responsive
            if batch_num < total_batches {
                tokio::task::yield_now().await;
                // Small delay to let other tasks run
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        }

        // Create combined results
        let combined_money_flow = MultipleDatesResult {
            results: combined_money_flow_results,
            metrics: PerformanceMetrics {
                vectorized_time: 0.0, // We don't track this for batched results
                traditional_time: None,
                speedup_factor: None,
                ticker_count: tickers.len(),
                date_count: dates.len(),
                calculation_count: tickers.len() * dates.len(),
            },
        };

        tracing::info!("Batched calculation completed: {} money flow dates, {} MA score dates",
                      combined_money_flow.results.len(), combined_ma_scores.len());

        Ok((combined_money_flow, combined_ma_scores))
    }
}

