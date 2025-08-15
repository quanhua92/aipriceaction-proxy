use reqwest::{Client, Error as ReqwestError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use tokio::time::sleep;
use chrono::{DateTime, NaiveDate, TimeZone, Utc};

#[derive(Debug)]
pub enum TcbsError {
    Http(ReqwestError),
    Serialization(serde_json::Error),
    InvalidInterval(String),
    InvalidResponse(String),
    RateLimit,
    NoData,
}

impl From<ReqwestError> for TcbsError {
    fn from(error: ReqwestError) -> Self {
        TcbsError::Http(error)
    }
}

impl From<serde_json::Error> for TcbsError {
    fn from(error: serde_json::Error) -> Self {
        TcbsError::Serialization(error)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OhlcvData {
    pub time: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
    pub symbol: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyOverview {
    pub ticker: String,
    pub exchange: Option<String>,
    pub industry: Option<String>,
    pub company_type: Option<String>,
    pub no_shareholders: Option<u32>,
    pub foreign_percent: Option<f64>,
    pub outstanding_share: Option<f64>, // TCBS returns in millions
    pub issue_share: Option<f64>,
    pub established_year: Option<u32>,
    pub no_employees: Option<u32>,
    pub stock_rating: Option<String>,
    pub short_name: Option<String>,
    pub website: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyProfile {
    pub symbol: String,
    pub company_profile: Option<String>,
    pub business_overview: Option<String>,
    pub business_strategy: Option<String>,
    pub business_advantage: Option<String>,
    pub company_promise: Option<String>,
    pub business_risk: Option<String>,
    pub key_developments: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareholderInfo {
    pub share_holder: String,
    pub share_own_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfficerInfo {
    pub officer_name: String,
    pub officer_position: String,
    pub officer_own_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyInfo {
    pub symbol: String,
    pub overview: Option<CompanyOverview>,
    pub profile: Option<CompanyProfile>,
    pub shareholders: Vec<ShareholderInfo>,
    pub officers: Vec<OfficerInfo>,
    pub market_cap: Option<f64>,
    pub current_price: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialStatement {
    pub period: String,
    pub data: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialInfo {
    pub symbol: String,
    pub period: String,
    pub balance_sheet: Option<Vec<FinancialStatement>>,
    pub income_statement: Option<Vec<FinancialStatement>>,
    pub cash_flow: Option<Vec<FinancialStatement>>,
    pub ratios: Option<Vec<FinancialStatement>>,
}

pub struct TcbsClient {
    client: Client,
    base_url: String,
    rate_limit_per_minute: u32,
    request_timestamps: Vec<SystemTime>,
    user_agents: Vec<String>,
    random_agent: bool,
}

impl TcbsClient {
    pub fn new(random_agent: bool, rate_limit_per_minute: u32) -> Result<Self, TcbsError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        let user_agents = vec![
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:120.0) Gecko/20100101 Firefox/120.0".to_string(),
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.3 Safari/605.1.15".to_string(),
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0".to_string(),
        ];

        Ok(TcbsClient {
            client,
            base_url: "https://apipubaws.tcbs.com.vn".to_string(),
            rate_limit_per_minute,
            request_timestamps: Vec::new(),
            user_agents,
            random_agent,
        })
    }

    fn get_interval_value(&self, interval: &str) -> Result<String, TcbsError> {
        let interval_map = HashMap::from([
            ("1m", "1"),
            ("5m", "5"),
            ("15m", "15"),
            ("30m", "30"),
            ("1H", "60"),
            ("1D", "D"),
            ("1W", "W"),
            ("1M", "M"),
        ]);

        interval_map.get(interval)
            .map(|s| s.to_string())
            .ok_or_else(|| TcbsError::InvalidInterval(interval.to_string()))
    }

    fn get_index_mapping(&self, symbol: &str) -> String {
        match symbol {
            "VNINDEX" => "VNINDEX".to_string(),
            "HNXINDEX" => "HNXIndex".to_string(),
            "UPCOMINDEX" => "UPCOM".to_string(),
            _ => symbol.to_string(),
        }
    }

    fn get_user_agent(&self) -> String {
        if self.random_agent {
            use rand::seq::SliceRandom;
            self.user_agents.choose(&mut rand::thread_rng())
                .unwrap_or(&self.user_agents[0])
                .clone()
        } else {
            self.user_agents[0].clone()
        }
    }

    async fn enforce_rate_limit(&mut self) {
        let current_time = SystemTime::now();
        
        // Remove timestamps older than 1 minute
        self.request_timestamps.retain(|&timestamp| {
            current_time.duration_since(timestamp).unwrap_or(Duration::from_secs(0)) < Duration::from_secs(60)
        });

        // If we're at the rate limit, wait
        if self.request_timestamps.len() >= self.rate_limit_per_minute as usize {
            if let Some(&oldest_request) = self.request_timestamps.first() {
                let wait_time = Duration::from_secs(60) - current_time.duration_since(oldest_request).unwrap_or(Duration::from_secs(0));
                if !wait_time.is_zero() {
                    sleep(wait_time + Duration::from_millis(100)).await;
                }
            }
        }

        self.request_timestamps.push(current_time);
    }

    async fn make_request(&mut self, url: &str, params: Option<&[(&str, &str)]>) -> Result<Value, TcbsError> {
        const MAX_RETRIES: u32 = 5;
        
        for attempt in 0..MAX_RETRIES {
            self.enforce_rate_limit().await;

            if attempt > 0 {
                let delay = Duration::from_secs_f64(2.0_f64.powi(attempt as i32 - 1) + rand::random::<f64>());
                let delay = delay.min(Duration::from_secs(60));
                sleep(delay).await;
            }

            let user_agent = self.get_user_agent();
            let mut request = self.client
                .get(url)
                .header("Accept", "application/json, text/plain, */*")
                .header("Accept-Language", "en-US,en;q=0.9,vi-VN;q=0.8,vi;q=0.7")
                .header("Accept-Encoding", "gzip, deflate, br")
                .header("Connection", "keep-alive")
                .header("Cache-Control", "no-cache")
                .header("Pragma", "no-cache")
                .header("DNT", "1")
                .header("Sec-Fetch-Dest", "empty")
                .header("Sec-Fetch-Mode", "cors")
                .header("Sec-Fetch-Site", "cross-site")
                .header("sec-ch-ua", "\"Not_A Brand\";v=\"8\", \"Chromium\";v=\"120\", \"Google Chrome\";v=\"120\"")
                .header("sec-ch-ua-mobile", "?0")
                .header("sec-ch-ua-platform", "\"Windows\"")
                .header("User-Agent", user_agent)
                .header("Referer", "https://www.tcbs.com.vn/")
                .header("Origin", "https://www.tcbs.com.vn");

            if let Some(query_params) = params {
                request = request.query(query_params);
            }

            let response = request.send().await;

            match response {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        match resp.json::<Value>().await {
                            Ok(data) => return Ok(data),
                            Err(_) => continue,
                        }
                    } else if status == 403 || status == 429 || status.is_server_error() {
                        continue;
                    } else if status.is_client_error() {
                        break;
                    } else {
                        continue;
                    }
                }
                Err(_) => continue,
            }
        }

        Err(TcbsError::InvalidResponse("Max retries exceeded".to_string()))
    }

    fn camel_to_snake(&self, name: &str) -> String {
        let mut result = String::new();
        let mut chars = name.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch.is_uppercase() && !result.is_empty() {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
        }

        result
    }

    pub async fn get_history(
        &mut self,
        symbol: &str,
        start: &str,
        end: Option<&str>,
        interval: &str,
        count_back: u32,
    ) -> Result<Vec<OhlcvData>, TcbsError> {
        let interval_value = self.get_interval_value(interval)?;
        let mapped_symbol = self.get_index_mapping(symbol);

        let start_time = NaiveDate::parse_from_str(start, "%Y-%m-%d")
            .map_err(|_| TcbsError::InvalidResponse("Invalid start date".to_string()))?;
        
        let end_time = match end {
            Some(date) => NaiveDate::parse_from_str(date, "%Y-%m-%d")
                .map_err(|_| TcbsError::InvalidResponse("Invalid end date".to_string()))?,
            None => Utc::now().date_naive(),
        };

        if end_time < start_time {
            return Err(TcbsError::InvalidResponse("End date cannot be earlier than start date".to_string()));
        }

        let end_timestamp = end_time.and_hms_opt(23, 59, 59).unwrap().and_utc().timestamp();

        // Determine asset type and endpoint
        let (asset_type, base_path) = if symbol.contains("F2") {
            ("derivative", "futures-insight")
        } else {
            ("stock", "stock-insight")
        };

        let endpoint = if matches!(interval, "1D" | "1W" | "1M") {
            "bars-long-term"
        } else {
            "bars"
        };

        let url = format!("{}/{}/v2/stock/{}", self.base_url, base_path, endpoint);
        let params = &[
            ("resolution", interval_value.as_str()),
            ("ticker", &mapped_symbol),
            ("type", asset_type),
            ("to", &end_timestamp.to_string()),
            ("countBack", &count_back.to_string()),
        ];


        let response_data = self.make_request(&url, Some(params)).await?;

        let data = response_data.get("data").ok_or(TcbsError::NoData)?;

        let mut result = Vec::new();

        if let Some(data_array) = data.as_array() {
            // TCBS format: list of objects with tradingDate, open, high, low, close, volume
            if data_array.is_empty() {
                return Err(TcbsError::NoData);
            }

            for item in data_array {
                if let Some(trading_date) = item.get("tradingDate").and_then(|v| v.as_str()) {
                    let date_part = if trading_date.contains('T') {
                        trading_date.split('T').next().unwrap()
                    } else {
                        trading_date
                    };

                    let naive_date = NaiveDate::parse_from_str(date_part, "%Y-%m-%d")
                        .map_err(|_| TcbsError::InvalidResponse("Invalid trading date format".to_string()))?;

                    if naive_date >= start_time.into() {
                        let time = Utc.from_utc_datetime(&naive_date.and_hms_opt(0, 0, 0).unwrap());

                        result.push(OhlcvData {
                            time,
                            open: item.get("open").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            high: item.get("high").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            low: item.get("low").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            close: item.get("close").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            volume: item.get("volume").and_then(|v| v.as_u64()).unwrap_or(0),
                            symbol: Some(symbol.to_string()),
                        });
                    }
                }
            }
        } else {
            // VCI-style format with parallel arrays
            let required_keys = ["t", "o", "h", "l", "c", "v"];
            for key in &required_keys {
                if !data.get(key).is_some() {
                    return Err(TcbsError::InvalidResponse(format!("Missing key: {}", key)));
                }
            }

            let times = data["t"].as_array().ok_or(TcbsError::InvalidResponse("Invalid times".to_string()))?;
            let opens = data["o"].as_array().ok_or(TcbsError::InvalidResponse("Invalid opens".to_string()))?;
            let highs = data["h"].as_array().ok_or(TcbsError::InvalidResponse("Invalid highs".to_string()))?;
            let lows = data["l"].as_array().ok_or(TcbsError::InvalidResponse("Invalid lows".to_string()))?;
            let closes = data["c"].as_array().ok_or(TcbsError::InvalidResponse("Invalid closes".to_string()))?;
            let volumes = data["v"].as_array().ok_or(TcbsError::InvalidResponse("Invalid volumes".to_string()))?;

            let length = times.len();
            if [opens.len(), highs.len(), lows.len(), closes.len(), volumes.len()].iter().any(|&len| len != length) {
                return Err(TcbsError::InvalidResponse("Inconsistent array lengths".to_string()));
            }

            for i in 0..length {
                let timestamp = times[i].as_i64().ok_or(TcbsError::InvalidResponse("Invalid timestamp".to_string()))?;
                let time = DateTime::<Utc>::from_timestamp(timestamp, 0).ok_or(TcbsError::InvalidResponse("Invalid timestamp".to_string()))?;

                if time.date_naive() >= start_time {
                    result.push(OhlcvData {
                        time,
                        open: opens[i].as_f64().unwrap_or(0.0),
                        high: highs[i].as_f64().unwrap_or(0.0),
                        low: lows[i].as_f64().unwrap_or(0.0),
                        close: closes[i].as_f64().unwrap_or(0.0),
                        volume: volumes[i].as_u64().unwrap_or(0),
                        symbol: Some(symbol.to_string()),
                    });
                }
            }
        }

        result.sort_by(|a, b| a.time.cmp(&b.time));
        Ok(result)
    }

    // pub async fn get_batch_history(
    //     &mut self,
    //     symbols: &[String],
    //     start: &str,
    //     end: Option<&str>,
    //     interval: &str,
    //     count_back: u32,
    // ) -> Result<HashMap<String, Option<Vec<OhlcvData>>>, TcbsError> {
    //     if symbols.is_empty() {
    //         return Err(TcbsError::InvalidResponse("Symbols list cannot be empty".to_string()));
    //     }

    //     println!("Fetching batch data for {} symbols: {}", symbols.len(), symbols.join(", "));
    //     println!("Date range: {} to {} [{}] (count_back={})", 
    //              start, end.unwrap_or("now"), interval, count_back);

    //     let mut results = HashMap::new();
    //     let mut successful_count = 0;

    //     // Process each symbol sequentially with rate limiting (TCBS doesn't support true batch requests)
    //     for (i, symbol) in symbols.iter().enumerate() {
    //         println!("Processing {} ({}/{})", symbol, i + 1, symbols.len());

    //         // Add small delay between requests
    //         if i > 0 {
    //             sleep(Duration::from_millis(500)).await;
    //         }

    //         match self.get_history(symbol, start, end, interval, count_back).await {
    //             Ok(mut data) => {
    //                 if !data.is_empty() {
    //                     // Add symbol column for identification
    //                     for item in &mut data {
    //                         item.symbol = Some(symbol.clone());
    //                     }
    //                     results.insert(symbol.clone(), Some(data));
    //                     successful_count += 1;
    //                     println!("✅ {}: {} data points", symbol, results[symbol].as_ref().unwrap().len());
    //                 } else {
    //                     results.insert(symbol.clone(), None);
    //                     println!("❌ {}: No data", symbol);
    //                 }
    //             }
    //             Err(e) => {
    //                 results.insert(symbol.clone(), None);
    //                 println!("❌ {}: Error - {:?}", symbol, e);
    //             }
    //         }
    //     }

    //     println!("Successfully fetched data for {}/{} symbols", successful_count, symbols.len());
    //     Ok(results)
    // }

    pub async fn overview(&mut self, symbol: &str) -> Result<CompanyOverview, TcbsError> {
        let url = format!("{}/tcanalysis/v1/ticker/{}/overview", self.base_url, symbol.to_uppercase());


        let response_data = self.make_request(&url, None).await?;

        let mut overview: CompanyOverview = serde_json::from_value(response_data)?;
        overview.ticker = symbol.to_uppercase();

        Ok(overview)
    }

    pub async fn profile(&mut self, symbol: &str) -> Result<CompanyProfile, TcbsError> {
        let url = format!("{}/tcanalysis/v1/company/{}/overview", self.base_url, symbol.to_uppercase());


        let response_data = self.make_request(&url, None).await?;

        // Clean HTML content if needed (simplified version without BeautifulSoup)
        let clean_html = |text: &str| -> String {
            // Simple HTML tag removal - in production you'd want a proper HTML parser
            let re = regex::Regex::new(r"<[^>]*>").unwrap();
            re.replace_all(text, "").replace('\n', " ")
        };

        let profile = CompanyProfile {
            symbol: symbol.to_uppercase(),
            company_profile: response_data.get("companyProfile").and_then(|v| v.as_str()).map(clean_html),
            business_overview: response_data.get("businessOverview").and_then(|v| v.as_str()).map(clean_html),
            business_strategy: response_data.get("businessStrategy").and_then(|v| v.as_str()).map(clean_html),
            business_advantage: response_data.get("businessAdvantage").and_then(|v| v.as_str()).map(clean_html),
            company_promise: response_data.get("companyPromise").and_then(|v| v.as_str()).map(clean_html),
            business_risk: response_data.get("businessRisk").and_then(|v| v.as_str()).map(clean_html),
            key_developments: response_data.get("keyDevelopments").and_then(|v| v.as_str()).map(clean_html),
        };

        Ok(profile)
    }

    pub async fn shareholders(&mut self, symbol: &str) -> Result<Vec<ShareholderInfo>, TcbsError> {
        let url = format!("{}/tcanalysis/v1/company/{}/large-share-holders", self.base_url, symbol.to_uppercase());


        let response_data = self.make_request(&url, None).await?;

        let shareholders_array = response_data.get("listShareHolder")
            .and_then(|v| v.as_array())
            .ok_or(TcbsError::NoData)?;

        let mut shareholders = Vec::new();
        for shareholder in shareholders_array {
            if let (Some(name), Some(percentage)) = (
                shareholder.get("name").and_then(|v| v.as_str()),
                shareholder.get("ownPercent").and_then(|v| v.as_f64())
            ) {
                shareholders.push(ShareholderInfo {
                    share_holder: name.to_string(),
                    share_own_percent: percentage,
                });
            }
        }

        Ok(shareholders)
    }

    pub async fn officers(&mut self, symbol: &str) -> Result<Vec<OfficerInfo>, TcbsError> {
        let url = format!("{}/tcanalysis/v1/company/{}/key-officers", self.base_url, symbol.to_uppercase());


        let response_data = self.make_request(&url, None).await?;

        let officers_array = response_data.get("listKeyOfficer")
            .and_then(|v| v.as_array())
            .ok_or(TcbsError::NoData)?;

        let mut officers = Vec::new();
        for officer in officers_array {
            if let (Some(name), Some(position)) = (
                officer.get("name").and_then(|v| v.as_str()),
                officer.get("position").and_then(|v| v.as_str())
            ) {
                let percentage = officer.get("ownPercent").and_then(|v| v.as_f64());
                officers.push(OfficerInfo {
                    officer_name: name.to_string(),
                    officer_position: position.to_string(),
                    officer_own_percent: percentage,
                });
            }
        }

        // Sort by ownership percentage
        officers.sort_by(|a, b| {
            b.officer_own_percent.unwrap_or(0.0).partial_cmp(&a.officer_own_percent.unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(officers)
    }

    pub async fn get_current_price(&mut self, symbol: &str) -> Result<Option<f64>, TcbsError> {
        let url = format!("{}/stock-insight/v1/stock/second-tc-price", self.base_url);
        let symbol_upper = symbol.to_uppercase();
        let params = &[("tickers", symbol_upper.as_str())];

        let response_data = self.make_request(&url, Some(params)).await?;

        let data = response_data.get("data")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .ok_or(TcbsError::NoData)?;

        let current_price = data.get("cp").and_then(|v| v.as_f64());
        Ok(current_price)
    }

    async fn make_financial_request(&mut self, url: &str, params: &[(&str, &str)]) -> Result<Value, TcbsError> {
        // Use direct HTTP request like Python does for financial endpoints
        self.enforce_rate_limit().await;
        
        let user_agent = self.get_user_agent();
        let request = self.client
            .get(url)
            .header("Accept", "application/json, text/plain, */*")
            .header("Accept-Language", "en-US,en;q=0.9,vi-VN;q=0.8,vi;q=0.7")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("Connection", "keep-alive")
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            .header("DNT", "1")
            .header("Sec-Fetch-Dest", "empty")
            .header("Sec-Fetch-Mode", "cors")
            .header("Sec-Fetch-Site", "cross-site")
            .header("sec-ch-ua", "\"Not_A Brand\";v=\"8\", \"Chromium\";v=\"120\", \"Google Chrome\";v=\"120\"")
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", "\"Windows\"")
            .header("User-Agent", user_agent)
            .header("Referer", "https://www.tcbs.com.vn/")
            .header("Origin", "https://www.tcbs.com.vn")
            .timeout(Duration::from_secs(30))
            .query(params);

        let response = request.send().await?;
        
        if response.status().is_success() {
            let data = response.json::<Value>().await?;
            Ok(data)
        } else {
            Err(TcbsError::Http(reqwest::Error::from(
                response.error_for_status().unwrap_err()
            )))
        }
    }

    pub async fn company_info(&mut self, symbol: &str) -> Result<CompanyInfo, TcbsError> {

        let mut company_info = CompanyInfo {
            symbol: symbol.to_uppercase(),
            overview: None,
            profile: None,
            shareholders: Vec::new(),
            officers: Vec::new(),
            market_cap: None,
            current_price: None,
        };

        // Get company overview
        match self.overview(symbol).await {
            Ok(overview) => company_info.overview = Some(overview),
            Err(_) => company_info.overview = None,
        }

        sleep(Duration::from_millis(500)).await;

        // Get company profile
        match self.profile(symbol).await {
            Ok(profile) => company_info.profile = Some(profile),
            Err(_) => company_info.profile = None,
        }

        sleep(Duration::from_millis(500)).await;

        // Get shareholders
        match self.shareholders(symbol).await {
            Ok(shareholders) => company_info.shareholders = shareholders,
            Err(_) => company_info.shareholders = Vec::new(),
        }

        sleep(Duration::from_millis(500)).await;

        // Get officers
        match self.officers(symbol).await {
            Ok(officers) => company_info.officers = officers,
            Err(_) => company_info.officers = Vec::new(),
        }

        // Calculate market cap if we have the data
        if let Some(ref overview) = company_info.overview {
            if let Some(outstanding_share) = overview.outstanding_share {
                match self.get_current_price(symbol).await {
                    Ok(Some(current_price)) => {
                        // TCBS returns outstanding shares in millions
                        let shares_actual = outstanding_share * 1_000_000.0;
                        let market_cap = shares_actual * current_price;

                        company_info.market_cap = Some(market_cap);
                        company_info.current_price = Some(current_price);

                    }
                    Ok(None) => {
                        company_info.market_cap = None;
                        company_info.current_price = None;
                    }
                    Err(_) => {
                        company_info.market_cap = None;
                        company_info.current_price = None;
                    }
                }
            }
        }

        Ok(company_info)
    }

    pub async fn financial_info(&mut self, symbol: &str, period: &str) -> Result<FinancialInfo, TcbsError> {
        let period_value = match period {
            "quarter" => "1",  // Python uses "1" as string for quarter
            "year" => "0",     // Python uses "0" as string for year
            _ => "1",
        };

        let mut financial_info = FinancialInfo {
            symbol: symbol.to_uppercase(),
            period: period.to_string(),
            balance_sheet: None,
            income_statement: None,
            cash_flow: None,
            ratios: None,
        };

        // Get balance sheet data - using direct request like Python
        let bs_url = format!("{}/tcanalysis/v1/finance/{}/balance_sheet", self.base_url, symbol.to_uppercase());
        let params = &[("yearly", period_value), ("isAll", "true")];

        match self.make_financial_request(&bs_url, params).await {
            Ok(data) => {
                if let Some(bs_array) = data.as_array() {
                    let mut statements = Vec::new();
                    for item in bs_array {
                        let year = item.get("year").and_then(|v| v.as_str()).unwrap_or("");
                        let quarter = item.get("quarter").and_then(|v| v.as_str()).unwrap_or("");
                        let period_str = if period == "quarter" && !quarter.is_empty() {
                            format!("{}-Q{}", year, quarter)
                        } else {
                            year.to_string()
                        };

                        let mut data_map = HashMap::new();
                        for (key, value) in item.as_object().unwrap() {
                            if let Some(num_value) = value.as_f64() {
                                let snake_case_key = self.camel_to_snake(key);
                                data_map.insert(snake_case_key, num_value);
                            }
                        }

                        statements.push(FinancialStatement {
                            period: period_str,
                            data: data_map,
                        });
                    }
                    financial_info.balance_sheet = Some(statements);
                }
            }
            Err(_) => financial_info.balance_sheet = None,
        }

        sleep(Duration::from_millis(500)).await;

        // Get income statement data - using direct request like Python
        let is_url = format!("{}/tcanalysis/v1/finance/{}/income_statement", self.base_url, symbol.to_uppercase());
        match self.make_financial_request(&is_url, params).await {
            Ok(data) => {
                if let Some(is_array) = data.as_array() {
                    let mut statements = Vec::new();
                    for item in is_array {
                        let year = item.get("year").and_then(|v| v.as_str()).unwrap_or("");
                        let quarter = item.get("quarter").and_then(|v| v.as_str()).unwrap_or("");
                        let period_str = if period == "quarter" && !quarter.is_empty() {
                            format!("{}-Q{}", year, quarter)
                        } else {
                            year.to_string()
                        };

                        let mut data_map = HashMap::new();
                        for (key, value) in item.as_object().unwrap() {
                            if let Some(num_value) = value.as_f64() {
                                let snake_case_key = self.camel_to_snake(key);
                                data_map.insert(snake_case_key, num_value);
                            }
                        }

                        statements.push(FinancialStatement {
                            period: period_str,
                            data: data_map,
                        });
                    }
                    financial_info.income_statement = Some(statements);
                }
            }
            Err(_) => financial_info.income_statement = None,
        }

        sleep(Duration::from_millis(500)).await;

        // Get cash flow data - using direct request like Python
        let cf_url = format!("{}/tcanalysis/v1/finance/{}/cash_flow", self.base_url, symbol.to_uppercase());
        match self.make_financial_request(&cf_url, params).await {
            Ok(data) => {
                if let Some(cf_array) = data.as_array() {
                    let mut statements = Vec::new();
                    for item in cf_array {
                        let year = item.get("year").and_then(|v| v.as_str()).unwrap_or("");
                        let quarter = item.get("quarter").and_then(|v| v.as_str()).unwrap_or("");
                        
                        let mut data_map = HashMap::new();
                        for (key, value) in item.as_object().unwrap() {
                            if let Some(num_value) = value.as_f64() {
                                let snake_case_key = self.camel_to_snake(key);
                                data_map.insert(snake_case_key, num_value);
                            }
                        }

                        statements.push(FinancialStatement {
                            period: format!("{}-{}", year, quarter),
                            data: data_map,
                        });
                    }
                    financial_info.cash_flow = Some(statements);
                }
            }
            Err(_) => financial_info.cash_flow = None,
        }

        sleep(Duration::from_millis(500)).await;

        // Get financial ratios - using direct request like Python
        let ratios_url = format!("{}/tcanalysis/v1/finance/{}/financialratio", self.base_url, symbol.to_uppercase());
        match self.make_financial_request(&ratios_url, params).await {
            Ok(data) => {
                if let Some(ratios_array) = data.as_array() {
                    let mut statements = Vec::new();
                    for item in ratios_array {
                        let year = item.get("year").and_then(|v| v.as_str()).unwrap_or("");
                        let quarter = item.get("quarter").and_then(|v| v.as_str()).unwrap_or("");
                        let period_str = if period == "quarter" && !quarter.is_empty() {
                            format!("{}-Q{}", year, quarter)
                        } else {
                            year.to_string()
                        };

                        let mut data_map = HashMap::new();
                        for (key, value) in item.as_object().unwrap() {
                            if let Some(num_value) = value.as_f64() {
                                let snake_case_key = self.camel_to_snake(key);
                                data_map.insert(snake_case_key, num_value);
                            }
                        }

                        statements.push(FinancialStatement {
                            period: period_str,
                            data: data_map,
                        });
                    }
                    financial_info.ratios = Some(statements);
                }
            }
            Err(_) => financial_info.ratios = None,
        }

        Ok(financial_info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tcbs_client_creation() {
        let client = TcbsClient::new(true, 6);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_interval_mapping() {
        let client = TcbsClient::new(false, 6).unwrap();
        assert_eq!(client.get_interval_value("1D").unwrap(), "D");
        assert_eq!(client.get_interval_value("1H").unwrap(), "60");
        assert!(client.get_interval_value("invalid").is_err());
    }

    #[test]
    fn test_camel_to_snake() {
        let client = TcbsClient::new(false, 6).unwrap();
        assert_eq!(client.camel_to_snake("camelCase"), "camel_case");
        assert_eq!(client.camel_to_snake("PascalCase"), "pascal_case");
        assert_eq!(client.camel_to_snake("simple"), "simple");
    }
}