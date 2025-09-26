use crate::{
    models::{
        ClientDataCache, DateRangeConfig, MoneyFlowStats,
        StockDataPoint, TickerCacheEntry, TickerGroups,
        ma_score::{MAScoreTickerData, MAScoreProcessConfig, MAScorePerformanceMetrics, MAScoreStats},
    },
    utils::{log_cache, Logger, money_flow_utils::MoneyFlowTickerData},
};
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};

/// Cache manager service for handling all data caching operations
/// Manages in-memory cache and coordinates with file storage in /tmp
#[derive(Debug)]
pub struct CacheManager {
    cache: ClientDataCache,
    cache_version: u64,
    money_flow_version: u64,
    ma_score_version: u64,
    logger: Logger,
}

impl CacheManager {
    pub fn new() -> Self {
        Self {
            cache: ClientDataCache::new(),
            cache_version: 0,
            money_flow_version: 0,
            ma_score_version: 0,
            logger: Logger::new("CACHE"),
        }
    }

    // === Cache Version Management ===

    pub fn get_cache_version(&self) -> u64 {
        self.cache_version
    }

    pub fn get_money_flow_version(&self) -> u64 {
        self.money_flow_version
    }

    pub fn get_ma_score_version(&self) -> u64 {
        self.ma_score_version
    }

    fn trigger_cache_update(&mut self) {
        self.cache_version += 1;
        log_cache(&format!("Cache version updated to {}", self.cache_version));
    }

    fn trigger_money_flow_update(&mut self) {
        self.money_flow_version += 1;
        log_cache(&format!(
            "Money flow version updated to {}",
            self.money_flow_version
        ));
    }

    fn trigger_ma_score_update(&mut self) {
        self.ma_score_version += 1;
        log_cache(&format!(
            "MA score version updated to {}",
            self.ma_score_version
        ));
    }

    // === Initialization Methods ===

    pub fn set_ticker_groups(&mut self, ticker_groups: TickerGroups) {
        let all_tickers = ticker_groups.get_all_tickers();
        self.cache.ticker_groups = Some(ticker_groups);
        self.cache.all_tickers = all_tickers;

        self.logger
            .info(&format!("Updated ticker groups: {} unique tickers", self.cache.all_tickers.len()));
        self.trigger_cache_update();
    }

    pub fn set_vnindex(&mut self, vnindex_data: Vec<StockDataPoint>) {
        // Check if data actually changed
        let data_changed = if let Some(existing_data) = &self.cache.vnindex_data {
            !self.is_data_equal(existing_data, &vnindex_data)
        } else {
            true
        };

        if data_changed {
            self.cache.vnindex_data = Some(vnindex_data.clone());
            self.cache.vnindex_last_updated = Some(Utc::now());

            // Also store VNINDEX in ticker cache
            let cache_entry = TickerCacheEntry::new("VNINDEX".to_string(), vnindex_data.clone(), "all".to_string());
            self.cache.ticker_data.insert("VNINDEX".to_string(), cache_entry);

            self.logger.info(&format!("Updated VNINDEX data: {} points", vnindex_data.len()));
            self.trigger_cache_update();
        }
    }

    pub fn set_last_requested_range(&mut self, range: DateRangeConfig) {
        self.cache.last_requested_range = Some(range);
        self.logger.info("Set last requested range");
    }

    // === Data Access Methods ===

    pub fn has_ticker_groups(&self) -> bool {
        self.cache.ticker_groups.is_some()
    }

    pub fn has_vnindex(&self) -> bool {
        self.cache.vnindex_data.is_some()
    }

    pub fn get_all_tickers(&self) -> Vec<String> {
        self.cache.all_tickers.clone()
    }

    pub fn get_ticker_data(&self, ticker: &str, date_range_config: Option<&DateRangeConfig>) -> Option<Vec<StockDataPoint>> {
        if let Some(entry) = self.cache.ticker_data.get(ticker) {
            if let Some(config) = date_range_config {
                Some(entry.get_data_in_range(config))
            } else {
                Some(entry.data.clone())
            }
        } else {
            None
        }
    }

    pub fn get_vnindex_data(&self, date_range_config: Option<&DateRangeConfig>) -> Option<Vec<StockDataPoint>> {
        if let Some(data) = &self.cache.vnindex_data {
            if let Some(config) = date_range_config {
                Some(StockDataPoint::filter_by_date_range(data.clone(), config))
            } else {
                Some(data.clone())
            }
        } else {
            None
        }
    }

    pub fn get_ticker_count(&self) -> usize {
        self.cache.ticker_data.len()
    }

    pub fn is_initialized(&self) -> bool {
        self.cache.is_initialized
    }

    pub fn get_last_ticker_update(&self) -> Option<DateTime<Utc>> {
        self.cache
            .ticker_data
            .values()
            .map(|entry| entry.last_updated)
            .max()
    }

    pub fn get_last_live_update(&self) -> Option<DateTime<Utc>> {
        // For now, return the last ticker update as a proxy
        // In a full implementation, this would track live data updates separately
        self.get_last_ticker_update()
    }

    // === CSV Data Management ===

    pub fn merge_csv_data(&mut self, csv_data: HashMap<String, Vec<StockDataPoint>>) -> usize {
        let mut change_count = 0;

        for (ticker, data) in csv_data {
            // Check if entry exists first
            let needs_merge = self.cache.ticker_data.contains_key(&ticker);

            if needs_merge {
                // Get existing data for merging
                let existing_data = self.cache.ticker_data.get(&ticker).unwrap().data.clone();
                let merged_data = self.merge_ticker_data(&existing_data, &data);

                // Now update the entry
                if let Some(existing_entry) = self.cache.ticker_data.get_mut(&ticker) {
                    existing_entry.data = merged_data;
                    existing_entry.last_updated = Utc::now();
                }
            } else {
                // New ticker data
                let cache_type = self.detect_cache_type(&data);
                let cache_entry = TickerCacheEntry::new(ticker.clone(), data, cache_type);
                self.cache.ticker_data.insert(ticker, cache_entry);
            }
            change_count += 1;
        }

        if change_count > 0 {
            self.logger.info(&format!("Merged CSV data for {} tickers", change_count));
            self.trigger_cache_update();
            self.mark_csv_update_completed();
        }

        change_count
    }

    pub fn merge_live_data(&mut self, live_data: HashMap<String, Vec<StockDataPoint>>) -> Vec<String> {
        let mut changed_dates = Vec::new();
        let live_data_len = live_data.len();

        for (ticker, new_data) in live_data {
            // Check if entry exists first
            let needs_merge = self.cache.ticker_data.contains_key(&ticker);

            if needs_merge {
                // Get existing data for merging
                let old_data = self.cache.ticker_data.get(&ticker).unwrap().data.clone();
                let merged_data = self.merge_live_with_existing(&old_data, &new_data);

                // Always update the entry with merged data
                if let Some(existing_entry) = self.cache.ticker_data.get_mut(&ticker) {
                    existing_entry.data = merged_data;
                    existing_entry.last_updated = Utc::now();
                }

                // For live data, always treat all dates as changed to force recalculation
                // This ensures we always recalculate money flow and MA scores with latest data
                for point in &new_data {
                    let date_str = point.time.clone();
                    if !changed_dates.contains(&date_str) {
                        changed_dates.push(date_str.clone());

                        // Debug: Log what dates are being marked as changed
                        if changed_dates.len() <= 5 { // Only log first few to avoid spam
                            self.logger.debug(&format!(
                                "Live data marking date as changed: {} (ticker: {})",
                                date_str, ticker
                            ));
                        }
                    }
                }
            } else {
                // New ticker from live data
                let cache_entry = TickerCacheEntry::new(ticker.clone(), new_data.clone(), "live".to_string());
                self.cache.ticker_data.insert(ticker, cache_entry);

                // Record all dates as changed
                for point in &new_data {
                    let date_str = point.time.clone();
                    if !changed_dates.contains(&date_str) {
                        changed_dates.push(date_str);
                    }
                }
            }
        }

        if !changed_dates.is_empty() {
            // Update changed dates in cache
            for date in &changed_dates {
                self.cache.changed_dates.insert(date.clone());

                // CRITICAL: Clear existing money flow and MA score data for changed dates
                // This forces recalculation when values change (simple like Money Flow)
                self.cache.money_flow_data.remove(date);
                self.cache.ma_score_data.remove(date);
            }

            self.logger.info(&format!(
                "Merged live data: {} tickers updated, {} dates changed (clearing cached calculations)",
                live_data_len,
                changed_dates.len()
            ));
            self.trigger_cache_update();
        }

        changed_dates
    }

    // === Money Flow Management ===

    pub fn get_money_flow_data(&self, date: Option<&str>) -> Option<Vec<MoneyFlowTickerData>> {
        if let Some(date_key) = date {
            self.cache.money_flow_data.get(date_key).cloned()
        } else {
            // Return all money flow data (flattened)
            let mut all_data = Vec::new();
            for data_list in self.cache.money_flow_data.values() {
                all_data.extend(data_list.clone());
            }
            if all_data.is_empty() {
                None
            } else {
                Some(all_data)
            }
        }
    }

    pub fn get_money_flow_stats(&self) -> MoneyFlowStats {
        MoneyFlowStats {
            total_dates: self.cache.money_flow_data.len(),
            total_calculations: self.cache.money_flow_data.values().map(|v| v.len()).sum(),
            last_update: self.cache.money_flow_last_updated,
            is_calculating: false, // This would be managed by the state machine
            last_metrics: self.cache.money_flow_metrics.clone(),
            uncalculated_dates: self.get_uncalculated_money_flow_dates(),
        }
    }

    pub fn get_money_flow_date_count(&self) -> usize {
        self.cache.money_flow_data.len()
    }

    pub fn clear_money_flow_cache(&mut self) {
        self.cache.clear_money_flow_cache();
        self.trigger_money_flow_update();
        self.logger.info("Money flow cache cleared");
    }

    fn get_uncalculated_money_flow_dates(&self) -> Vec<String> {
        // Find dates that have ticker data but no money flow calculations
        let mut uncalculated = Vec::new();

        // Get all unique dates from ticker data
        let mut all_dates = HashSet::new();
        for entry in self.cache.ticker_data.values() {
            for point in &entry.data {
                all_dates.insert(point.time.clone());
            }
        }

        // Find dates not in money flow data
        for date in all_dates {
            if !self.cache.money_flow_data.contains_key(&date) {
                uncalculated.push(date);
            }
        }

        uncalculated.sort();
        uncalculated
    }

    // === MA Score Management ===

    pub fn get_ma_score_data(&self, date: Option<&str>, _ma_period: Option<u32>) -> Option<Vec<MAScoreTickerData>> {
        if let Some(date_key) = date {
            self.cache.ma_score_data.get(date_key).cloned()
        } else {
            // Return all MA score data (flattened)
            let mut all_data = Vec::new();
            for data_list in self.cache.ma_score_data.values() {
                all_data.extend(data_list.clone());
            }
            if all_data.is_empty() {
                None
            } else {
                Some(all_data)
            }
        }
    }

    pub fn get_ma_score_stats(&self) -> MAScoreStats {
        MAScoreStats {
            calculated_dates: self.cache.ma_score_data.len(),
            total_calculations: self.cache.ma_score_data.values().map(|v| v.len()).sum(),
            changed_dates: Vec::new(),
            last_calculation: self.cache.ma_score_last_updated,
            last_update: self.cache.ma_score_last_updated,
            calculation_time_ms: self.cache.ma_score_metrics.as_ref().map(|m| m.calculation_time).unwrap_or(0.0),
            is_calculating: false, // This would be managed by the state machine
            last_metrics: self.cache.ma_score_metrics.clone(),
            uncalculated_dates: self.get_uncalculated_ma_score_dates(),
            default_ma_period: self.cache
                .ma_score_config
                .as_ref()
                .map(|c| c.default_ma_period as u32)
                .unwrap_or(20),
        }
    }

    pub fn get_ma_score_date_count(&self) -> usize {
        self.cache.ma_score_data.len()
    }

    pub fn clear_ma_score_cache(&mut self) {
        self.cache.clear_ma_score_cache();
        self.trigger_ma_score_update();
        self.logger.info("MA score cache cleared");
    }

    fn get_uncalculated_ma_score_dates(&self) -> Vec<String> {
        // Similar to money flow, find dates that need MA score calculation
        let mut uncalculated = Vec::new();

        let mut all_dates = HashSet::new();
        for entry in self.cache.ticker_data.values() {
            for point in &entry.data {
                all_dates.insert(point.time.clone());
            }
        }

        for date in all_dates {
            if !self.cache.ma_score_data.contains_key(&date) {
                uncalculated.push(date);
            }
        }

        uncalculated.sort();
        uncalculated
    }

    // === Cache Management ===

    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.cache_version = 0;
        self.money_flow_version = 0;
        self.ma_score_version = 0;
        self.logger.info("Full cache cleared");
    }

    pub fn get_cache(&self) -> &ClientDataCache {
        &self.cache
    }

    fn mark_csv_update_completed(&mut self) {
        self.cache.last_background_update = Some(Utc::now());
        self.cache.is_initialized = true;
    }

    // === Helper Methods ===

    fn merge_ticker_data(&self, existing_data: &[StockDataPoint], new_data: &[StockDataPoint]) -> Vec<StockDataPoint> {
        let mut merged = existing_data.to_vec();

        for new_point in new_data {
            // Find if we have data for this date
            if let Some(existing_index) = merged.iter().position(|p| p.time == new_point.time) {
                // Replace existing data point
                merged[existing_index] = new_point.clone();
            } else {
                // Add new data point and maintain chronological order
                merged.push(new_point.clone());
            }
        }

        // Sort by date to maintain chronological order
        merged.sort_by(|a, b| a.date.cmp(&b.date));
        merged
    }

    fn merge_live_with_existing(&self, existing_data: &[StockDataPoint], live_data: &[StockDataPoint]) -> Vec<StockDataPoint> {
        let mut merged = existing_data.to_vec();

        for live_point in live_data {
            // Check if we have data for this date
            if let Some(existing_index) = merged.iter().position(|p| p.time == live_point.time) {
                let existing_point = &merged[existing_index];

                // CRITICAL DEBUG: Log what's happening during merge
                if live_point.data_changed(existing_point) {
                    self.logger.info(&format!(
                        "🔥 CRITICAL MERGE DEBUG: Replacing existing data for date {}",
                        live_point.time
                    ));
                    self.logger.info(&format!(
                        "   BEFORE: close={:.1}",
                        existing_point.close
                    ));
                    self.logger.info(&format!(
                        "   LIVE:   close={:.1}",
                        live_point.close
                    ));

                    // Use live data directly - MA values are handled by MA score calculations
                    merged[existing_index] = live_point.clone();

                    self.logger.info(&format!(
                        "   AFTER:  close={:.1}",
                        merged[existing_index].close
                    ));
                }
            } else {
                // New data point - check if it's newer than the latest existing point
                if let Some(latest_existing) = merged.last() {
                    if live_point.date > latest_existing.date {
                        self.logger.info(&format!(
                            "🔥 NEW DATA POINT: Adding new date {} (close={:.1})",
                            live_point.time, live_point.close
                        ));
                        merged.push(live_point.clone());
                    }
                } else {
                    // No existing data
                    merged.push(live_point.clone());
                }
            }
        }

        // Sort by date to maintain chronological order
        merged.sort_by(|a, b| a.date.cmp(&b.date));
        merged
    }

    fn detect_cache_type(&self, data: &[StockDataPoint]) -> String {
        // Determine cache type based on data length (similar to TypeScript version)
        if data.len() <= 70 {
            "60d".to_string()
        } else if data.len() <= 200 {
            "180d".to_string()
        } else if data.len() <= 400 {
            "365d".to_string()
        } else {
            "all".to_string()
        }
    }

    fn is_data_equal(&self, data1: &[StockDataPoint], data2: &[StockDataPoint]) -> bool {
        if data1.len() != data2.len() {
            return false;
        }

        for (point1, point2) in data1.iter().zip(data2.iter()) {
            if point1.time != point2.time
                || point1.ticker != point2.ticker
                || (point1.open - point2.open).abs() > 0.01
                || (point1.high - point2.high).abs() > 0.01
                || (point1.low - point2.low).abs() > 0.01
                || (point1.close - point2.close).abs() > 0.01
                || (point1.volume - point2.volume).abs() > 1 {
                return false;
            }
        }

        true
    }

    #[allow(dead_code)]
    fn has_data_changes(&self, old_data: &[StockDataPoint], new_data: &[StockDataPoint]) -> bool {
        for new_point in new_data {
            if let Some(old_point) = old_data.iter().find(|p| p.time == new_point.time) {
                if new_point.data_changed(old_point) {
                    return true;
                }
            } else {
                // New data point
                return true;
            }
        }
        false
    }

    // === Money Flow Cache Methods ===

    pub fn set_money_flow_data(
        &mut self,
        results: HashMap<String, Vec<MoneyFlowTickerData>>,
    ) {
        // Store results in cache
        for (date, ticker_data) in results {
            self.cache.money_flow_data.insert(date, ticker_data);
        }

        self.cache.money_flow_last_updated = Some(Utc::now());
        self.trigger_cache_update();

        self.logger.info(&format!(
            "Money flow data stored: {} dates",
            self.cache.money_flow_data.len()
        ));
    }

    // === MA Score Cache Methods ===

    pub fn set_ma_score_data(
        &mut self,
        results: HashMap<String, Vec<MAScoreTickerData>>,
        config: MAScoreProcessConfig,
        metrics: MAScorePerformanceMetrics,
    ) {
        // Store results in cache
        for (date, ticker_data) in results {
            self.cache.ma_score_data.insert(date.clone(), ticker_data);
            self.cache.ma_score_calculated_dates.insert(date);
        }

        // Update config and metrics
        self.cache.ma_score_config = Some(config);
        self.cache.ma_score_metrics = Some(metrics.clone());
        self.cache.ma_score_last_updated = Some(Utc::now());

        // Clear changed dates as they've been processed
        self.cache.ma_score_changed_dates.clear();

        self.trigger_ma_score_update();

        self.logger.info(&format!(
            "MA score data stored: {} dates, calculation time: {:.1}ms",
            self.cache.ma_score_data.len(),
            metrics.calculation_time
        ));
    }

    pub fn update_ma_score_data(
        &mut self,
        results: HashMap<String, Vec<MAScoreTickerData>>,
        metrics: MAScorePerformanceMetrics,
    ) {
        // Update results in cache
        for (date, ticker_data) in results {
            self.cache.ma_score_data.insert(date.clone(), ticker_data);
            self.cache.ma_score_calculated_dates.insert(date);
        }

        // Update metrics
        self.cache.ma_score_metrics = Some(metrics.clone());
        self.cache.ma_score_last_updated = Some(Utc::now());

        // Clear changed dates as they've been processed
        self.cache.ma_score_changed_dates.clear();

        self.trigger_ma_score_update();

        self.logger.info(&format!(
            "MA score data updated: {} dates, calculation time: {:.1}ms",
            self.cache.ma_score_data.len(),
            metrics.calculation_time
        ));
    }

    pub fn get_dates_needing_ma_score(&self) -> Vec<String> {
        self.get_uncalculated_ma_score_dates()
    }

    pub fn get_all_ma_score_trading_dates_in_range(&self) -> Vec<String> {
        // Get all trading dates from ticker data within the configured range
        let mut all_dates = HashSet::new();

        for ticker_entry in self.cache.ticker_data.values() {
            for point in &ticker_entry.data {
                all_dates.insert(point.time.clone());
            }
        }

        let mut sorted_dates: Vec<String> = all_dates.into_iter().collect();
        sorted_dates.sort();
        sorted_dates
    }

    pub fn get_all_trading_dates_in_range(&self) -> Vec<String> {
        // Get all trading dates from ticker data within the configured range
        let mut all_dates = HashSet::new();

        for ticker_entry in self.cache.ticker_data.values() {
            for point in &ticker_entry.data {
                all_dates.insert(point.time.clone());
            }
        }

        let mut sorted_dates: Vec<String> = all_dates.into_iter().collect();
        sorted_dates.sort();
        sorted_dates
    }

    // MA values are now handled exclusively by the MA score calculation system
    // No longer storing MA values in StockDataPoint to eliminate dual sources
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}