use crate::{
    models::{DateRangeConfig, RawStockData, StockDataPoint, TickerGroups, TimeRange},
    utils::{format_date_range_info, Logger, Timer},
};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::PathBuf,
    time::SystemTime,
};

const GITHUB_RAW_BASE_URL: &str = "https://raw.githubusercontent.com/quanhua92/aipriceaction-data/refs/heads/main";
const MARKET_DATA_URL: &str = "https://raw.githubusercontent.com/quanhua92/aipriceaction-data/refs/heads/main/market_data";
#[allow(dead_code)]
const TICKER_INFO_URL: &str = "https://raw.githubusercontent.com/quanhua92/aipriceaction-data/refs/heads/main/ticker_info.json";
const TICKER_GROUPS_URL: &str = "https://raw.githubusercontent.com/quanhua92/aipriceaction-data/refs/heads/main/ticker_group.json";

/// CSV data fetching service with /tmp file caching
/// Handles downloading and caching CSV files from GitHub
pub struct CSVDataService {
    client: reqwest::Client,
    cache_dir: PathBuf,
    logger: Logger,
}

impl CSVDataService {
    pub fn new() -> anyhow::Result<Self> {
        let cache_dir = std::env::temp_dir().join("aipriceaction_cli_cache");
        fs::create_dir_all(&cache_dir)?;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        let logger = Logger::new("CSV_SERVICE");

        // Log cache directory initialization
        let now = chrono::Utc::now();
        logger.info(&format!(
            "📁 [FETCH_CSV] [{}] CSV Cache initialized: folder={}",
            now.format("%Y-%m-%d %H:%M:%S UTC"),
            cache_dir.display()
        ));

        Ok(Self {
            client,
            cache_dir,
            logger,
        })
    }

    /// Fetch ticker groups from GitHub (cached for 1 hour)
    pub async fn fetch_ticker_groups(&self) -> anyhow::Result<TickerGroups> {
        let cache_file = self.cache_dir.join("ticker_groups.json");

        self.logger.info("Loading ticker groups...");

        // Check cache first
        if let Some(cached_data) = self.load_from_cache(&cache_file, "ticker groups").await? {
            let now = chrono::Utc::now();
            self.logger.info(&format!(
                "💾 [FETCH_CSV] [{}] Cache HIT: ticker_groups.json (read from cache)",
                now.format("%Y-%m-%d %H:%M:%S UTC")
            ));
            let groups: TickerGroups = serde_json::from_str(&cached_data)?;
            return Ok(groups);
        }

        // Fetch from GitHub
        let now = chrono::Utc::now();
        self.logger.info(&format!(
            "🌐 [FETCH_CSV] [{}] Downloading: ticker_groups.json from GitHub",
            now.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        let timer = Timer::start("ticker groups fetch");
        let response = self.client.get(TICKER_GROUPS_URL).send().await?;
        let content = response.text().await?;

        let download_complete = chrono::Utc::now();
        self.logger.info(&format!(
            "✅ [FETCH_CSV] [{}] Download complete: ticker_groups.json ({:.1}ms)",
            download_complete.format("%Y-%m-%d %H:%M:%S UTC"),
            timer.elapsed().as_secs_f64() * 1000.0
        ));

        // Validate JSON
        let groups: TickerGroups = serde_json::from_str(&content)?;

        // Cache the result
        self.save_to_cache(&cache_file, &content).await?;

        timer.log_elapsed("CSV_SERVICE");
        self.logger.info("Ticker groups loaded successfully");
        Ok(groups)
    }

    /// Fetch VNINDEX data with ALL range
    pub async fn fetch_vnindex(&self) -> anyhow::Result<Vec<StockDataPoint>> {
        self.logger.info("Loading VNINDEX data (ALL range)...");

        let data = self.fetch_single_ticker("VNINDEX", &DateRangeConfig::new(TimeRange::All)).await?;

        self.logger.info(&format!("VNINDEX data loaded: {} points", data.len()));
        Ok(data)
    }

    /// Fetch VNINDEX data with 1Y range for faster startup
    pub async fn fetch_vnindex_1y(&self) -> anyhow::Result<Vec<StockDataPoint>> {
        self.logger.info("Loading VNINDEX data (1Y range)...");

        let data = self.fetch_single_ticker("VNINDEX", &DateRangeConfig::new(TimeRange::OneYear)).await?;

        self.logger.info(&format!("VNINDEX 1Y data loaded: {} points", data.len()));
        Ok(data)
    }

    /// Fetch ticker data with intelligent cache selection
    pub async fn fetch_tickers(
        &self,
        tickers: &[String],
        date_range: &DateRangeConfig,
    ) -> anyhow::Result<HashMap<String, Vec<StockDataPoint>>> {
        let mut result = HashMap::new();

        self.logger.info(&format!(
            "Fetching CSV data for {} tickers ({})",
            tickers.len(),
            format_date_range_info(date_range)
        ));

        // For ALL range, fetch individual files
        if matches!(date_range.range, TimeRange::All) {
            return self.fetch_individual_files(tickers, date_range).await;
        }

        // Try cache files first for non-ALL ranges
        if let Some(cache_data) = self.try_cache_files(tickers, date_range).await? {
            for (ticker, data) in cache_data {
                result.insert(ticker, data);
            }
        }

        // Fetch remaining tickers individually
        let remaining_tickers: Vec<String> = tickers
            .iter()
            .filter(|ticker| !result.contains_key(*ticker))
            .cloned()
            .collect();

        if !remaining_tickers.is_empty() {
            let individual_data = self.fetch_individual_files(&remaining_tickers, date_range).await?;
            result.extend(individual_data);
        }

        self.logger.info(&format!(
            "CSV fetch completed: {}/{} tickers successfully loaded",
            result.len(),
            tickers.len()
        ));

        Ok(result)
    }

    /// Fetch single ticker data
    pub async fn fetch_single_ticker(
        &self,
        ticker: &str,
        date_range: &DateRangeConfig,
    ) -> anyhow::Result<Vec<StockDataPoint>> {
        let cache_file = self.cache_dir.join(format!("{}.csv", ticker));

        // Check cache first
        if let Some(cached_content) = self.load_from_cache(&cache_file, &format!("{} data", ticker)).await? {
            if let Ok(data) = self.parse_csv_content(&cached_content, ticker) {
                let filtered_data = StockDataPoint::filter_by_date_range(data, date_range);
                if !filtered_data.is_empty() {
                    let now = chrono::Utc::now();
                    self.logger.info(&format!(
                        "💾 [FETCH_CSV] [{}] Cache HIT: {}.csv ({} points)",
                        now.format("%Y-%m-%d %H:%M:%S UTC"),
                        ticker,
                        filtered_data.len()
                    ));
                    return Ok(filtered_data);
                }
            }
        }

        // Fetch from GitHub
        let url = format!("{}/{}.csv", MARKET_DATA_URL, ticker);

        let now = chrono::Utc::now();
        self.logger.info(&format!(
            "🌐 [FETCH_CSV] [{}] Downloading: {}.csv from GitHub",
            now.format("%Y-%m-%d %H:%M:%S UTC"),
            ticker
        ));

        let timer = Timer::start(&format!("{} fetch", ticker));
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP error {} for ticker {}", response.status(), ticker));
        }

        let content = response.text().await?;

        // Cache the raw content
        self.save_to_cache(&cache_file, &content).await?;

        // Parse and filter data
        let data = self.parse_csv_content(&content, ticker)?;
        let filtered_data = StockDataPoint::filter_by_date_range(data, date_range);

        let download_complete = chrono::Utc::now();
        self.logger.info(&format!(
            "✅ [FETCH_CSV] [{}] Download complete: {}.csv ({} points, {:.1}ms)",
            download_complete.format("%Y-%m-%d %H:%M:%S UTC"),
            ticker,
            filtered_data.len(),
            timer.elapsed().as_secs_f64() * 1000.0
        ));
        timer.log_elapsed("CSV_SERVICE");

        Ok(filtered_data)
    }

    /// Try to load data from aggregated cache files (60d, 180d, 365d)
    async fn try_cache_files(
        &self,
        tickers: &[String],
        date_range: &DateRangeConfig,
    ) -> anyhow::Result<Option<HashMap<String, Vec<StockDataPoint>>>> {
        let cache_file_names = match date_range.range {
            TimeRange::OneWeek | TimeRange::TwoWeeks | TimeRange::OneMonth
            | TimeRange::TwoMonths | TimeRange::ThreeMonths => {
                vec!["ticker_60_days.csv"]
            }
            TimeRange::FourMonths | TimeRange::SixMonths => {
                vec!["ticker_180_days.csv"]
            }
            TimeRange::OneYear => {
                vec!["ticker_365_days.csv"]
            }
            TimeRange::All => {
                // Skip aggregated cache for ALL - download individual CSVs for full historical data
                return Ok(None);
            }
            TimeRange::Custom => {
                // For custom ranges, estimate which cache to use
                if let (Some(start), Some(end)) = (date_range.start_date, date_range.end_date) {
                    let days = (end - start).num_days();
                    if days <= 60 {
                        vec!["ticker_60_days.csv"]
                    } else if days <= 180 {
                        vec!["ticker_180_days.csv"]
                    } else if days <= 365 {
                        vec!["ticker_365_days.csv"]
                    } else {
                        return Ok(None); // Too long for cache files
                    }
                } else {
                    return Ok(None);
                }
            }
            _ => return Ok(None),
        };

        // Try each cache file
        for cache_file_name in cache_file_names {
            if let Some(data) = self.load_aggregated_cache_file(cache_file_name, tickers, date_range).await? {
                return Ok(Some(data));
            }
        }

        Ok(None)
    }

    /// Load and parse aggregated cache file
    async fn load_aggregated_cache_file(
        &self,
        cache_file_name: &str,
        tickers: &[String],
        date_range: &DateRangeConfig,
    ) -> anyhow::Result<Option<HashMap<String, Vec<StockDataPoint>>>> {
        let cache_file = self.cache_dir.join(cache_file_name);
        let url = format!("{}/{}", GITHUB_RAW_BASE_URL, cache_file_name);

        let content = if let Some(cached_content) = self.load_from_cache(&cache_file, cache_file_name).await? {
            let now = chrono::Utc::now();
            self.logger.info(&format!(
                "💾 [FETCH_CSV] [{}] Cache HIT: {} (aggregated cache)",
                now.format("%Y-%m-%d %H:%M:%S UTC"),
                cache_file_name
            ));
            cached_content
        } else {
            // Fetch from GitHub
            let now = chrono::Utc::now();
            self.logger.info(&format!(
                "🌐 [FETCH_CSV] [{}] Downloading: {} from GitHub",
                now.format("%Y-%m-%d %H:%M:%S UTC"),
                cache_file_name
            ));

            let timer = Timer::start(&format!("{} fetch", cache_file_name));
            let response = self.client.get(&url).send().await?;

            if !response.status().is_success() {
                self.logger.warn(&format!("Failed to fetch {}: HTTP {}", cache_file_name, response.status()));
                return Ok(None);
            }

            let content = response.text().await?;
            self.save_to_cache(&cache_file, &content).await?;

            let download_complete = chrono::Utc::now();
            self.logger.info(&format!(
                "✅ [FETCH_CSV] [{}] Download complete: {} ({:.1}ms)",
                download_complete.format("%Y-%m-%d %H:%M:%S UTC"),
                cache_file_name,
                timer.elapsed().as_secs_f64() * 1000.0
            ));
            timer.log_elapsed("CSV_SERVICE");
            content
        };

        // Parse aggregated CSV data
        let all_data = self.parse_aggregated_csv_content(&content)?;

        // Extract data for requested tickers
        let mut result = HashMap::new();
        for ticker in tickers {
            if let Some(ticker_data) = all_data.get(ticker) {
                let filtered_data = StockDataPoint::filter_by_date_range(ticker_data.clone(), date_range);
                if !filtered_data.is_empty() {
                    result.insert(ticker.clone(), filtered_data);
                }
            }
        }

        if !result.is_empty() {
            self.logger.info(&format!(
                "Cache load completed: {}/{} tickers loaded from {}",
                result.len(),
                tickers.len(),
                cache_file_name
            ));
        }

        Ok(if result.is_empty() { None } else { Some(result) })
    }

    /// Fetch tickers individually from GitHub
    async fn fetch_individual_files(
        &self,
        tickers: &[String],
        date_range: &DateRangeConfig,
    ) -> anyhow::Result<HashMap<String, Vec<StockDataPoint>>> {
        let mut result = HashMap::new();
        let batch_size = 50;

        self.logger.info(&format!("Fetching {} tickers individually", tickers.len()));

        for (batch_idx, batch) in tickers.chunks(batch_size).enumerate() {
            let mut batch_tasks = Vec::new();

            for ticker in batch {
                let ticker = ticker.clone();
                let date_range = date_range.clone();
                let service = &*self; // Create reference for async closure

                let task = async move {
                    match service.fetch_single_ticker(&ticker, &date_range).await {
                        Ok(data) => Ok((ticker, data)),
                        Err(e) => {
                            service.logger.warn(&format!("Failed to fetch data for {}: {}", ticker, e));
                            Ok((ticker, Vec::new()))
                        }
                    }
                };

                batch_tasks.push(task);
            }

            // Execute batch concurrently
            let batch_results: Vec<Result<(String, Vec<StockDataPoint>), anyhow::Error>> = futures::future::join_all(batch_tasks).await;

            // Collect results
            for result_item in batch_results {
                if let Ok((ticker, data)) = result_item {
                    if !data.is_empty() {
                        result.insert(ticker, data);
                    }
                }
            }

            // Small delay between batches to avoid overwhelming the server
            if batch_idx * batch_size + batch.len() < tickers.len() {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }

        Ok(result)
    }

    /// Parse single ticker CSV content
    fn parse_csv_content(&self, content: &str, ticker: &str) -> anyhow::Result<Vec<StockDataPoint>> {
        let mut reader = csv::Reader::from_reader(content.as_bytes());
        let mut data_points = Vec::new();

        for result in reader.deserialize() {
            let raw_data: RawStockData = result?;

            // Validate that the ticker matches
            if raw_data.ticker != ticker {
                continue;
            }

            let data_point = raw_data.to_stock_data_point()?;
            data_points.push(data_point);
        }

        // Sort by date
        data_points.sort_by(|a, b| a.date.cmp(&b.date));

        Ok(data_points)
    }

    /// Parse aggregated CSV content (multiple tickers in one file)
    fn parse_aggregated_csv_content(&self, content: &str) -> anyhow::Result<HashMap<String, Vec<StockDataPoint>>> {
        let mut reader = csv::Reader::from_reader(content.as_bytes());
        let mut ticker_data: HashMap<String, Vec<StockDataPoint>> = HashMap::new();

        for result in reader.deserialize() {
            let raw_data: RawStockData = result?;

            let data_point = raw_data.to_stock_data_point()?;

            ticker_data
                .entry(raw_data.ticker.clone())
                .or_insert_with(Vec::new)
                .push(data_point);
        }

        // Sort each ticker's data by date
        for data_points in ticker_data.values_mut() {
            data_points.sort_by(|a, b| a.date.cmp(&b.date));
        }

        Ok(ticker_data)
    }

    /// Check if cached file exists and is not expired (1 hour expiry)
    async fn load_from_cache(&self, cache_file: &PathBuf, data_type: &str) -> anyhow::Result<Option<String>> {
        if !cache_file.exists() {
            return Ok(None);
        }

        // Check file age (1 hour expiry)
        let metadata = fs::metadata(cache_file)?;
        let file_age = metadata.modified()?.elapsed()?;

        if file_age > std::time::Duration::from_secs(3600) {
            self.logger.debug(&format!("Cache expired for {}, removing", data_type));
            let _ = fs::remove_file(cache_file); // Ignore errors
            return Ok(None);
        }

        // Read cached content
        match fs::read_to_string(cache_file) {
            Ok(content) => {
                self.logger.debug(&format!("Cache hit for {}", data_type));
                Ok(Some(content))
            }
            Err(e) => {
                self.logger.warn(&format!("Failed to read cache for {}: {}", data_type, e));
                Ok(None)
            }
        }
    }

    /// Save content to cache file
    async fn save_to_cache(&self, cache_file: &PathBuf, content: &str) -> anyhow::Result<()> {
        let mut file = File::create(cache_file)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
        Ok(())
    }

    /// Clear all cached files
    pub fn clear_cache(&self) -> anyhow::Result<()> {
        if self.cache_dir.exists() {
            let entries = fs::read_dir(&self.cache_dir)?;
            let mut cleared_count = 0;

            for entry in entries {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    fs::remove_file(entry.path())?;
                    cleared_count += 1;
                }
            }

            self.logger.info(&format!("Cleared {} cached files", cleared_count));
        }

        Ok(())
    }

    /// Get cache directory path
    pub fn get_cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> anyhow::Result<CacheStats> {
        let mut total_files = 0;
        let mut total_size = 0u64;
        let mut oldest_file: Option<SystemTime> = None;
        let mut newest_file: Option<SystemTime> = None;

        if self.cache_dir.exists() {
            let entries = fs::read_dir(&self.cache_dir)?;

            for entry in entries {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    total_files += 1;

                    let metadata = entry.metadata()?;
                    total_size += metadata.len();

                    let modified = metadata.modified()?;

                    if oldest_file.is_none() || modified < oldest_file.unwrap() {
                        oldest_file = Some(modified);
                    }

                    if newest_file.is_none() || modified > newest_file.unwrap() {
                        newest_file = Some(modified);
                    }
                }
            }
        }

        Ok(CacheStats {
            total_files,
            total_size_bytes: total_size,
            cache_dir: self.cache_dir.clone(),
            oldest_file,
            newest_file,
        })
    }
}

impl Default for CSVDataService {
    fn default() -> Self {
        Self::new().expect("Failed to create CSVDataService")
    }
}

/// Cache statistics structure
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_files: usize,
    pub total_size_bytes: u64,
    pub cache_dir: PathBuf,
    pub oldest_file: Option<SystemTime>,
    pub newest_file: Option<SystemTime>,
}