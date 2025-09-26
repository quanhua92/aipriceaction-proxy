use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Ticker groups by sector (from GitHub JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerGroups {
    #[serde(rename = "CHUNG_KHOAN")]
    pub securities: Vec<String>,
    #[serde(rename = "NGAN_HANG")]
    pub banking: Vec<String>,
    #[serde(rename = "BAT_DONG_SAN")]
    pub real_estate: Vec<String>,
    #[serde(rename = "CONG_NGHE")]
    pub technology: Vec<String>,
    #[serde(rename = "BAN_LE")]
    pub retail: Vec<String>,
    #[serde(rename = "THEP")]
    pub steel: Vec<String>,
    #[serde(rename = "HOA_CHAT")]
    pub chemicals: Vec<String>,
    #[serde(rename = "THUC_PHAM")]
    pub food_beverage: Vec<String>,
    #[serde(rename = "NONG_NGHIEP")]
    pub agriculture: Vec<String>,
    #[serde(rename = "VAN_TAI")]
    pub transportation: Vec<String>,
    #[serde(rename = "VLXD")]
    pub building_materials: Vec<String>,
    #[serde(rename = "XAY_DUNG")]
    pub construction: Vec<String>,
    #[serde(flatten)]
    pub other_sectors: HashMap<String, Vec<String>>,
}

impl TickerGroups {
    /// Get all unique tickers from all sectors
    pub fn get_all_tickers(&self) -> Vec<String> {
        let mut all_tickers = Vec::new();

        // Add tickers from known sectors
        all_tickers.extend(self.securities.iter().cloned());
        all_tickers.extend(self.banking.iter().cloned());
        all_tickers.extend(self.real_estate.iter().cloned());
        all_tickers.extend(self.technology.iter().cloned());
        all_tickers.extend(self.retail.iter().cloned());
        all_tickers.extend(self.steel.iter().cloned());
        all_tickers.extend(self.chemicals.iter().cloned());
        all_tickers.extend(self.food_beverage.iter().cloned());
        all_tickers.extend(self.agriculture.iter().cloned());
        all_tickers.extend(self.transportation.iter().cloned());
        all_tickers.extend(self.building_materials.iter().cloned());
        all_tickers.extend(self.construction.iter().cloned());

        // Add tickers from other sectors
        for sector_tickers in self.other_sectors.values() {
            all_tickers.extend(sector_tickers.iter().cloned());
        }

        // Remove duplicates and sort
        all_tickers.sort();
        all_tickers.dedup();

        all_tickers
    }

    /// Get tickers for a specific sector
    pub fn get_sector_tickers(&self, sector_key: &str) -> Option<&Vec<String>> {
        match sector_key {
            "CHUNG_KHOAN" => Some(&self.securities),
            "NGAN_HANG" => Some(&self.banking),
            "BAT_DONG_SAN" => Some(&self.real_estate),
            "CONG_NGHE" => Some(&self.technology),
            "BAN_LE" => Some(&self.retail),
            "THEP" => Some(&self.steel),
            "HOA_CHAT" => Some(&self.chemicals),
            "THUC_PHAM" => Some(&self.food_beverage),
            "NONG_NGHIEP" => Some(&self.agriculture),
            "VAN_TAI" => Some(&self.transportation),
            "VLXD" => Some(&self.building_materials),
            "XAY_DUNG" => Some(&self.construction),
            _ => self.other_sectors.get(sector_key),
        }
    }

    /// Find which sector a ticker belongs to
    pub fn find_ticker_sector(&self, ticker: &str) -> Option<String> {
        if ticker == "VNINDEX" {
            return None; // VNINDEX is not part of any sector
        }

        // Check known sectors
        if self.securities.contains(&ticker.to_string()) {
            return Some("CHUNG_KHOAN".to_string());
        }
        if self.banking.contains(&ticker.to_string()) {
            return Some("NGAN_HANG".to_string());
        }
        if self.real_estate.contains(&ticker.to_string()) {
            return Some("BAT_DONG_SAN".to_string());
        }
        if self.technology.contains(&ticker.to_string()) {
            return Some("CONG_NGHE".to_string());
        }
        if self.retail.contains(&ticker.to_string()) {
            return Some("BAN_LE".to_string());
        }
        if self.steel.contains(&ticker.to_string()) {
            return Some("THEP".to_string());
        }
        if self.chemicals.contains(&ticker.to_string()) {
            return Some("HOA_CHAT".to_string());
        }
        if self.food_beverage.contains(&ticker.to_string()) {
            return Some("THUC_PHAM".to_string());
        }
        if self.agriculture.contains(&ticker.to_string()) {
            return Some("NONG_NGHIEP".to_string());
        }
        if self.transportation.contains(&ticker.to_string()) {
            return Some("VAN_TAI".to_string());
        }
        if self.building_materials.contains(&ticker.to_string()) {
            return Some("VLXD".to_string());
        }
        if self.construction.contains(&ticker.to_string()) {
            return Some("XAY_DUNG".to_string());
        }

        // Check other sectors
        for (sector_key, sector_tickers) in &self.other_sectors {
            if sector_tickers.contains(&ticker.to_string()) {
                return Some(sector_key.clone());
            }
        }

        None
    }

    /// Get display name for sector
    pub fn get_sector_display_name(sector_key: &str) -> &'static str {
        match sector_key {
            "CHUNG_KHOAN" => "Securities",
            "NGAN_HANG" => "Banking",
            "BAT_DONG_SAN" => "Real Estate",
            "CONG_NGHE" => "Technology",
            "BAN_LE" => "Retail",
            "THEP" => "Steel",
            "HOA_CHAT" => "Chemicals",
            "THUC_PHAM" => "Food & Beverage",
            "NONG_NGHIEP" => "Agriculture",
            "VAN_TAI" => "Transportation",
            "VLXD" => "Building Materials",
            "XAY_DUNG" => "Construction",
            _ => "Other",
        }
    }

    /// Get total number of tickers
    pub fn total_tickers(&self) -> usize {
        self.get_all_tickers().len()
    }

    /// Get number of sectors
    pub fn total_sectors(&self) -> usize {
        12 + self.other_sectors.len() // 12 known sectors + other sectors
    }
}

/// Ticker information (company name and market cap)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerInfo {
    pub company_name: String,
    pub market_cap: f64,
}

/// Full ticker info data (HashMap of ticker -> info)
pub type TickerInfoData = HashMap<String, TickerInfo>;

/// Data request for ticker fetching
#[derive(Debug, Clone)]
pub struct DataRequest {
    pub tickers: Vec<String>,
    pub date_range_config: Option<super::DateRangeConfig>,
    pub priority: RequestPriority,
    pub caller: Option<String>,
}

impl DataRequest {
    pub fn new(tickers: Vec<String>) -> Self {
        Self {
            tickers,
            date_range_config: None,
            priority: RequestPriority::Normal,
            caller: None,
        }
    }

    pub fn with_range(mut self, config: super::DateRangeConfig) -> Self {
        self.date_range_config = Some(config);
        self
    }

    pub fn with_priority(mut self, priority: RequestPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_caller(mut self, caller: String) -> Self {
        self.caller = Some(caller);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestPriority {
    High,
    Normal,
    Low,
}

impl RequestPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            RequestPriority::High => "high",
            RequestPriority::Normal => "normal",
            RequestPriority::Low => "low",
        }
    }
}

/// CSV request specifically for batching
#[derive(Debug, Clone)]
pub struct CSVRequest {
    pub tickers: Vec<String>,
    pub date_range_config: super::DateRangeConfig,
    pub priority: RequestPriority,
}

impl CSVRequest {
    pub fn new(tickers: Vec<String>, date_range_config: super::DateRangeConfig) -> Self {
        Self {
            tickers,
            date_range_config,
            priority: RequestPriority::Normal,
        }
    }

    pub fn with_priority(mut self, priority: RequestPriority) -> Self {
        self.priority = priority;
        self
    }
}