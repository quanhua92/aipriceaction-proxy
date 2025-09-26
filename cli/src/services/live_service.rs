use crate::{
    models::{LiveTickerData, StockDataPoint},
    utils::{is_live_data_stale, log_fetch_live, Logger, Timer},
};
use std::collections::HashMap;

const LIVE_API_URL: &str = "https://api.aipriceaction.com/tickers";

/// Live data fetching service
/// Handles fetching current market data from the live API
pub struct LiveDataService {
    client: reqwest::Client,
    logger: Logger,
}

impl LiveDataService {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10)) // Short timeout for live data
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            logger: Logger::new("LIVE_SERVICE"),
        }
    }

    /// Fetch latest live data for specified tickers
    pub async fn fetch_latest(
        &self,
        tickers: &[String],
        timeout_ms: Option<u64>,
    ) -> anyhow::Result<HashMap<String, Vec<StockDataPoint>>> {
        let mut result = HashMap::new();

        log_fetch_live(&format!("Fetching live data for {} tickers", tickers.len()));

        let timeout_duration = std::time::Duration::from_millis(timeout_ms.unwrap_or(15000));

        // Fetch live data from API with timeout
        let live_ticker_data = tokio::time::timeout(
            timeout_duration,
            self.fetch_live_data_from_api(),
        ).await??;

        // Process each requested ticker
        for ticker in tickers {
            if let Some(live_points) = live_ticker_data.tickers.get(ticker) {
                if let Some(latest_point) = live_points.first() {
                    // Convert to StockDataPoint
                    match latest_point.to_stock_data_point(ticker) {
                        Ok(stock_point) => {
                            // Filter to only recent data (last 3 days)
                            let three_days_ago = chrono::Utc::now() - chrono::Duration::days(3);

                            if stock_point.date >= three_days_ago {
                                // Only log for major tickers to reduce spam
                                let major_tickers = ["VNINDEX", "VCB", "BID", "CTG", "ACB"];
                                if major_tickers.contains(&ticker.as_str()) {
                                    self.logger.debug(&format!(
                                        "Live data for {}: {} ({})",
                                        ticker,
                                        stock_point.time,
                                        if is_live_data_stale(stock_point.date) { "stale" } else { "fresh" }
                                    ));
                                }

                                result.insert(ticker.clone(), vec![stock_point]);
                            }
                        }
                        Err(e) => {
                            self.logger.warn(&format!("Failed to convert live data for {}: {}", ticker, e));
                        }
                    }
                }
            }
        }

        log_fetch_live(&format!(
            "Live fetch completed: {}/{} tickers had recent data",
            result.len(),
            tickers.len()
        ));

        Ok(result)
    }

    /// Fetch today's data specifically for a ticker
    pub async fn fetch_today(&self, ticker: &str) -> anyhow::Result<Vec<StockDataPoint>> {
        self.logger.debug(&format!("Fetching today's data for {}", ticker));

        let live_data = self.fetch_live_data_from_api().await?;

        if let Some(ticker_data) = live_data.tickers.get(ticker) {
            let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

            for live_point in ticker_data {
                if live_point.time == today {
                    match live_point.to_stock_data_point(ticker) {
                        Ok(stock_point) => {
                            self.logger.debug(&format!("Found today's data for {}", ticker));
                            return Ok(vec![stock_point]);
                        }
                        Err(e) => {
                            self.logger.error(&format!(
                                "Failed to convert today's data for {}: {}",
                                ticker, e
                            ));
                        }
                    }
                }
            }
        }

        self.logger.debug(&format!("No today's data for {}", ticker));
        Ok(Vec::new())
    }

    /// Fetch live VNINDEX data specifically
    pub async fn fetch_vnindex_live(&self) -> anyhow::Result<Vec<StockDataPoint>> {
        self.logger.debug("Fetching live VNINDEX data");

        let live_data = self.fetch_live_data_from_api().await?;

        if let Some(vnindex_data) = live_data.tickers.get("VNINDEX") {
            let mut recent_points = Vec::new();
            let three_days_ago = chrono::Utc::now() - chrono::Duration::days(3);

            for live_point in vnindex_data {
                match live_point.to_stock_data_point("VNINDEX") {
                    Ok(stock_point) => {
                        if stock_point.date >= three_days_ago {
                            recent_points.push(stock_point);
                        }
                    }
                    Err(e) => {
                        self.logger.warn(&format!("Failed to convert VNINDEX live data: {}", e));
                    }
                }
            }

            // Sort by date
            recent_points.sort_by(|a, b| a.date.cmp(&b.date));

            self.logger.debug(&format!("VNINDEX live data: {} recent points", recent_points.len()));
            Ok(recent_points)
        } else {
            self.logger.warn("No VNINDEX data in live response");
            Ok(Vec::new())
        }
    }

    /// Check if live data has changed compared to cached data
    pub fn has_data_changed(
        &self,
        cached_data: &[StockDataPoint],
        live_data: &[StockDataPoint],
    ) -> bool {
        if live_data.is_empty() {
            return false;
        }

        // Check if any point in live data is different from cached data
        for live_point in live_data {
            if let Some(cached_point) = cached_data
                .iter()
                .find(|point| point.time == live_point.time)
            {
                // Check if any value changed (with small tolerance for floating point)
                if live_point.data_changed(cached_point) {
                    return true;
                }
            } else {
                // New data point
                return true;
            }
        }

        false
    }

    /// Extract changed dates from live data comparison
    pub fn get_changed_dates(
        &self,
        cached_data: &[StockDataPoint],
        live_data: &[StockDataPoint],
    ) -> Vec<String> {
        let mut changed_dates = Vec::new();

        for live_point in live_data {
            let date = &live_point.time;

            if let Some(cached_point) = cached_data
                .iter()
                .find(|point| point.time == *date)
            {
                if live_point.data_changed(cached_point) {
                    if !changed_dates.contains(date) {
                        changed_dates.push(date.clone());
                    }
                }
            } else {
                // New date
                if !changed_dates.contains(date) {
                    changed_dates.push(date.clone());
                }
            }
        }

        changed_dates
    }

    /// Batch fetch live data for multiple tickers
    pub async fn fetch_batch(
        &self,
        tickers: &[String],
        batch_size: usize,
    ) -> anyhow::Result<HashMap<String, Vec<StockDataPoint>>> {
        let mut result = HashMap::new();

        log_fetch_live(&format!(
            "Batch fetching live data for {} tickers (batch size: {})",
            tickers.len(),
            batch_size
        ));

        // For live data, we only need to make one API call regardless of batch size
        // since the API returns data for all tickers at once
        let live_data = self.fetch_live_data_from_api().await?;

        // Process requested tickers
        for ticker in tickers {
            if let Some(ticker_live_data) = live_data.tickers.get(ticker) {
                if let Some(latest_point) = ticker_live_data.first() {
                    match latest_point.to_stock_data_point(ticker) {
                        Ok(stock_point) => {
                            // Only include recent data (last 3 days)
                            let three_days_ago = chrono::Utc::now() - chrono::Duration::days(3);
                            if stock_point.date >= three_days_ago {
                                result.insert(ticker.clone(), vec![stock_point]);
                            }
                        }
                        Err(e) => {
                            self.logger.warn(&format!(
                                "Failed to convert live data for {}: {}",
                                ticker, e
                            ));
                        }
                    }
                }
            }
        }

        self.logger.info(&format!(
            "Batch live fetch completed: {}/{} tickers had recent data",
            result.len(),
            tickers.len()
        ));

        Ok(result)
    }

    /// Fetch raw live data from the API
    async fn fetch_live_data_from_api(&self) -> anyhow::Result<LiveTickerData> {
        let timer = Timer::start("live API fetch");

        self.logger.debug(&format!(
            "🌐 Fetching from {} (10s timeout)",
            LIVE_API_URL
        ));

        let response = self.client.get(LIVE_API_URL).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Live API fetch failed: {} {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown error")
            ));
        }

        let live_data: LiveTickerData = response.json().await?;
        let ticker_count = live_data.tickers.len();

        // Debug: Log what dates are returned for key tickers
        let key_tickers = ["VNINDEX", "VCB", "CTG", "BID"];
        for ticker in &key_tickers {
            if let Some(ticker_data) = live_data.tickers.get(*ticker) {
                let dates: Vec<String> = ticker_data.iter().map(|p| p.time.clone()).collect();
                self.logger.debug(&format!(
                    "Live API returned dates for {}: {:?}",
                    ticker, dates
                ));
            }
        }

        // Check if the live data is stale
        let is_stale_data = self.is_live_data_stale(&live_data);

        timer.log_elapsed("LIVE_SERVICE");

        self.logger.info(&format!(
            "✅ API success - {} tickers fetched in {:.1}ms ({})",
            ticker_count,
            timer.elapsed_ms(),
            if is_stale_data { "stale data" } else { "fresh data" }
        ));

        if is_stale_data {
            self.logger.info(
                "📊 Data is from yesterday (today's data not available yet)"
            );
        }

        Ok(live_data)
    }

    /// Check if live data is stale (older than expected)
    fn is_live_data_stale(&self, live_data: &LiveTickerData) -> bool {
        // Check a few major tickers to determine staleness
        let check_tickers = ["VNINDEX", "VCB", "CTG", "BID"];

        for ticker in &check_tickers {
            if let Some(ticker_data) = live_data.tickers.get(*ticker) {
                if let Some(latest_point) = ticker_data.first() {
                    match chrono::NaiveDate::parse_from_str(&latest_point.time, "%Y-%m-%d") {
                        Ok(data_date) => {
                            let data_date_utc = data_date.and_hms_opt(0, 0, 0)
                                .unwrap()
                                .and_utc();

                            if !is_live_data_stale(data_date_utc) {
                                return false; // Found fresh data
                            }
                        }
                        Err(_) => continue,
                    }
                }
            }
        }

        true // All checked tickers have stale data
    }

    /// Get service statistics
    pub fn get_stats(&self) -> LiveServiceStats {
        LiveServiceStats {
            api_url: LIVE_API_URL.to_string(),
            timeout_seconds: 10,
        }
    }
}

impl Default for LiveDataService {
    fn default() -> Self {
        Self::new()
    }
}

/// Live service statistics
#[derive(Debug, Clone)]
pub struct LiveServiceStats {
    pub api_url: String,
    pub timeout_seconds: u64,
}