use crate::{
    models::{CSVRequest, DataRequest, DateRangeConfig, RequestPriority},
    utils::{log_consolidation, Logger},
};
use std::collections::VecDeque;

/// Request queue service for managing ticker data requests
/// Handles CSV requests, live data requests, and request consolidation
#[derive(Debug)]
pub struct RequestQueue {
    csv_requests: VecDeque<CSVRequest>,
    data_requests: VecDeque<DataRequest>,
    processed_count: usize,
    logger: Logger,
}

impl RequestQueue {
    pub fn new() -> Self {
        Self {
            csv_requests: VecDeque::new(),
            data_requests: VecDeque::new(),
            processed_count: 0,
            logger: Logger::new("REQUEST_QUEUE"),
        }
    }

    // === CSV Request Management ===

    /// Add a CSV request to the queue
    pub fn add_csv_request(&mut self, request: CSVRequest) {
        self.logger.info(&format!(
            "Adding CSV request: {} tickers, {} range, {} priority",
            request.tickers.len(),
            request.date_range_config.range.as_str(),
            request.priority.as_str()
        ));

        // Insert based on priority (high priority goes first)
        match request.priority {
            RequestPriority::High => self.csv_requests.push_front(request),
            RequestPriority::Normal => self.csv_requests.push_back(request),
            RequestPriority::Low => self.csv_requests.push_back(request),
        }
    }

    /// Create and add a CSV request from basic parameters
    pub fn add_csv_request_simple(
        &mut self,
        tickers: Vec<String>,
        date_range_config: DateRangeConfig,
        priority: RequestPriority,
    ) {
        let request = CSVRequest::new(tickers, date_range_config).with_priority(priority);
        self.add_csv_request(request);
    }

    /// Take up to N CSV requests from the queue (for batch processing)
    pub fn take_csv_requests(&mut self, max_count: usize) -> Vec<CSVRequest> {
        let mut taken = Vec::new();

        for _ in 0..max_count {
            if let Some(request) = self.csv_requests.pop_front() {
                taken.push(request);
            } else {
                break;
            }
        }

        if !taken.is_empty() {
            self.processed_count += taken.len();
            log_consolidation(&format!(
                "Took {} CSV requests from queue for processing",
                taken.len()
            ));
        }

        taken
    }

    /// Check if there are pending CSV requests
    pub fn has_csv_requests(&self) -> bool {
        !self.csv_requests.is_empty()
    }

    /// Get count of pending CSV requests
    pub fn csv_request_count(&self) -> usize {
        self.csv_requests.len()
    }

    // === Data Request Management ===

    /// Add a general data request to the queue
    pub fn add_data_request(&mut self, request: DataRequest) {
        self.logger.info(&format!(
            "Adding data request: {} tickers, {} priority, caller: {}",
            request.tickers.len(),
            request.priority.as_str(),
            request.caller.as_deref().unwrap_or("unknown")
        ));

        // Insert based on priority
        match request.priority {
            RequestPriority::High => self.data_requests.push_front(request),
            RequestPriority::Normal => self.data_requests.push_back(request),
            RequestPriority::Low => self.data_requests.push_back(request),
        }
    }

    /// Take up to N data requests from the queue
    pub fn take_data_requests(&mut self, max_count: usize) -> Vec<DataRequest> {
        let mut taken = Vec::new();

        for _ in 0..max_count {
            if let Some(request) = self.data_requests.pop_front() {
                taken.push(request);
            } else {
                break;
            }
        }

        if !taken.is_empty() {
            self.processed_count += taken.len();
            self.logger.info(&format!("Took {} data requests from queue", taken.len()));
        }

        taken
    }

    /// Check if there are pending data requests
    pub fn has_data_requests(&self) -> bool {
        !self.data_requests.is_empty()
    }

    /// Get count of pending data requests
    pub fn data_request_count(&self) -> usize {
        self.data_requests.len()
    }

    // === Request Consolidation ===

    /// Consolidate multiple data requests into CSV requests
    /// This mimics the consolidation logic from the TypeScript version
    pub fn consolidate_data_requests(&mut self) -> Vec<CSVRequest> {
        let data_requests = self.take_data_requests(10); // Take up to 10 for consolidation
        if data_requests.is_empty() {
            return Vec::new();
        }

        let mut consolidated = Vec::new();
        let mut requests_by_range: std::collections::HashMap<String, Vec<DataRequest>> = std::collections::HashMap::new();

        // Group requests by date range
        for request in data_requests {
            let range_key = if let Some(config) = &request.date_range_config {
                format!("{:?}", config.range) // Use debug format as key
            } else {
                "3M".to_string() // Default range
            };

            requests_by_range.entry(range_key).or_insert_with(Vec::new).push(request);
        }

        // Calculate total requests before consuming the map
        let total_data_requests: usize = requests_by_range.values().map(|v| v.len()).sum();

        // Create consolidated CSV requests
        for (_range_key, requests) in requests_by_range {
            let mut all_tickers = Vec::new();
            let mut highest_priority = RequestPriority::Low;

            // Collect all tickers and determine highest priority
            for request in &requests {
                all_tickers.extend(request.tickers.clone());

                if matches!(request.priority, RequestPriority::High) {
                    highest_priority = RequestPriority::High;
                } else if matches!(request.priority, RequestPriority::Normal) && matches!(highest_priority, RequestPriority::Low) {
                    highest_priority = RequestPriority::Normal;
                }
            }

            // Remove duplicates and sort
            all_tickers.sort();
            all_tickers.dedup();

            // Use the date range config from the first request, or default
            let date_range_config = requests[0]
                .date_range_config
                .clone()
                .unwrap_or_else(|| DateRangeConfig::default_3m());

            let csv_request = CSVRequest::new(all_tickers, date_range_config).with_priority(highest_priority);

            consolidated.push(csv_request);
        }

        if !consolidated.is_empty() {
            let total_requests = consolidated.len();
            let total_tickers: usize = consolidated.iter().map(|r| r.tickers.len()).sum();

            log_consolidation(&format!(
                "Consolidated {} data requests into {} CSV requests with {} total tickers",
                total_data_requests,
                total_requests,
                total_tickers
            ));

            // Add consolidated requests back to CSV queue
            for request in &consolidated {
                self.add_csv_request(request.clone());
            }
        }

        consolidated
    }

    // === Queue Management ===

    /// Clear all requests from queues
    pub fn clear_all(&mut self) {
        let csv_count = self.csv_requests.len();
        let data_count = self.data_requests.len();

        self.csv_requests.clear();
        self.data_requests.clear();

        if csv_count > 0 || data_count > 0 {
            self.logger.info(&format!(
                "Cleared all requests: {} CSV requests, {} data requests",
                csv_count, data_count
            ));
        }
    }

    /// Clear only CSV requests
    pub fn clear_csv_requests(&mut self) {
        let count = self.csv_requests.len();
        self.csv_requests.clear();

        if count > 0 {
            self.logger.info(&format!("Cleared {} CSV requests", count));
        }
    }

    /// Clear only data requests
    pub fn clear_data_requests(&mut self) {
        let count = self.data_requests.len();
        self.data_requests.clear();

        if count > 0 {
            self.logger.info(&format!("Cleared {} data requests", count));
        }
    }

    // === Statistics ===

    /// Get total pending requests (all types)
    pub fn total_pending(&self) -> usize {
        self.csv_requests.len() + self.data_requests.len()
    }

    /// Get total processed requests
    pub fn total_processed(&self) -> usize {
        self.processed_count
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.csv_requests.is_empty() && self.data_requests.is_empty()
    }

    /// Get queue statistics for debugging
    pub fn get_queue_stats(&self) -> QueueStats {
        QueueStats {
            csv_requests: self.csv_requests.len(),
            data_requests: self.data_requests.len(),
            total_pending: self.total_pending(),
            total_processed: self.total_processed(),
            high_priority_csv: self.count_high_priority_csv(),
            high_priority_data: self.count_high_priority_data(),
        }
    }

    fn count_high_priority_csv(&self) -> usize {
        self.csv_requests
            .iter()
            .filter(|r| matches!(r.priority, RequestPriority::High))
            .count()
    }

    fn count_high_priority_data(&self) -> usize {
        self.data_requests
            .iter()
            .filter(|r| matches!(r.priority, RequestPriority::High))
            .count()
    }

    // === Request Prioritization ===

    /// Promote all requests of a specific ticker to high priority
    pub fn promote_ticker_requests(&mut self, ticker: &str) {
        let mut promoted_csv = 0;
        let mut promoted_data = 0;

        // Promote CSV requests
        for request in &mut self.csv_requests {
            if request.tickers.contains(&ticker.to_string()) {
                request.priority = RequestPriority::High;
                promoted_csv += 1;
            }
        }

        // Promote data requests
        for request in &mut self.data_requests {
            if request.tickers.contains(&ticker.to_string()) {
                request.priority = RequestPriority::High;
                promoted_data += 1;
            }
        }

        if promoted_csv > 0 || promoted_data > 0 {
            self.logger.info(&format!(
                "Promoted {} CSV and {} data requests for ticker {}",
                promoted_csv, promoted_data, ticker
            ));

            // Re-sort queues to put high priority items first
            self.resort_queues();
        }
    }

    /// Re-sort queues to ensure high priority items are processed first
    fn resort_queues(&mut self) {
        // Convert to vectors, sort by priority, then back to VecDeque
        let mut csv_vec: Vec<CSVRequest> = self.csv_requests.drain(..).collect();
        csv_vec.sort_by_key(|r| match r.priority {
            RequestPriority::High => 0,
            RequestPriority::Normal => 1,
            RequestPriority::Low => 2,
        });
        self.csv_requests = csv_vec.into();

        let mut data_vec: Vec<DataRequest> = self.data_requests.drain(..).collect();
        data_vec.sort_by_key(|r| match r.priority {
            RequestPriority::High => 0,
            RequestPriority::Normal => 1,
            RequestPriority::Low => 2,
        });
        self.data_requests = data_vec.into();
    }
}

impl Default for RequestQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Queue statistics structure
#[derive(Debug, Clone)]
pub struct QueueStats {
    pub csv_requests: usize,
    pub data_requests: usize,
    pub total_pending: usize,
    pub total_processed: usize,
    pub high_priority_csv: usize,
    pub high_priority_data: usize,
}