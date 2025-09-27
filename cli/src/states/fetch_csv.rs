use crate::{
    models::{CSVRequest, DateRangeConfig, RequestPriority, TimeRange},
    services::CSVDataService,
    state_machine::{State, StateContext},
    states::{FetchLiveState},
    utils::{Logger, Timer},
};

/// FetchCSVState - Handles CSV data fetching and prerequisites
pub struct FetchCSVState {
    logger: Logger,
    is_initial_load: bool,
    csv_service: CSVDataService,
}

impl FetchCSVState {
    pub fn new() -> Self {
        Self {
            logger: Logger::new("FETCH_CSV"),
            is_initial_load: true,
            csv_service: CSVDataService::new().expect("Failed to create CSV service"),
        }
    }
}

#[async_trait::async_trait]
impl State for FetchCSVState {
    fn name(&self) -> &'static str {
        "FETCH_CSV"
    }

    async fn enter(&mut self, context: &mut StateContext) -> anyhow::Result<()> {
        let now = chrono::Utc::now();
        if self.is_initial_load {
            self.logger.info(&format!(
                "📶 [FETCH_CSV] [{}] Entering CSV fetch state - initial load",
                now.format("%Y-%m-%d %H:%M:%S UTC")
            ));
            self.load_prerequisites(context).await?;
            self.is_initial_load = false;
        } else {
            self.logger.info(&format!(
                "📶 [FETCH_CSV] [{}] Entering CSV fetch state - subsequent load",
                now.format("%Y-%m-%d %H:%M:%S UTC")
            ));
        }
        Ok(())
    }

    async fn exit(&mut self, context: &mut StateContext) -> anyhow::Result<()> {
        let cache_stats = context.get_cache_stats();
        let now = chrono::Utc::now();
        self.logger.info_with_data(
            &format!(
                "🏁 [FETCH_CSV] [{}] Exiting CSV fetch state",
                now.format("%Y-%m-%d %H:%M:%S UTC")
            ),
            format!(
                "total_tickers: {}, cached_tickers: {}, has_vnindex: {}",
                context.get_all_tickers().len(),
                cache_stats.ticker_count,
                context.cache.has_vnindex()
            ),
        );
        Ok(())
    }

    async fn tick(
        &mut self,
        context: &mut StateContext,
    ) -> anyhow::Result<Option<Box<dyn State + Send + Sync>>> {
        // 1. Check if prerequisites are loaded
        if !context.has_prerequisites() {
            self.logger.info("Loading prerequisites...");
            self.load_prerequisites(context).await?;

            // After loading prerequisites, check if we have CSV requests to process
            let csv_requests = context.request_queue.take_csv_requests(5);
            if !csv_requests.is_empty() {
                // Process the requests immediately
                for request in csv_requests {
                    self.process_csv_request(context, &request).await?;
                }

                // If more requests exist, stay in state to process them
                if context.request_queue.has_csv_requests() {
                    return Ok(None); // Stay in current state
                }
            }
        }

        // 2. Process CSV requests from queue
        let csv_requests = context.request_queue.take_csv_requests(5); // Process 5 requests at a time

        if csv_requests.is_empty() {
            // No CSV requests to process, transition to fetch live data
            self.logger.info("No CSV requests pending - transitioning to fetch live data");
            return Ok(Some(Box::new(FetchLiveState::new())));
        }

        // 3. Process CSV requests
        for request in csv_requests {
            self.process_csv_request(context, &request).await?;
        }

        // 4. Continue processing if more requests exist
        if context.request_queue.has_csv_requests() {
            self.logger.debug("More CSV requests to process, staying in state");
            return Ok(None); // Stay in current state
        }

        // 5. All CSV requests processed, fetch live data next
        self.logger.info("All CSV requests processed - transitioning to fetch live data");
        Ok(Some(Box::new(FetchLiveState::new())))
    }
}

impl FetchCSVState {
    /// Load prerequisites (ticker groups and VNINDEX)
    async fn load_prerequisites(&mut self, context: &mut StateContext) -> anyhow::Result<()> {
        // Load ticker groups if not already loaded
        if !context.cache.has_ticker_groups() {
            self.logger.info("Loading ticker groups...");

            match self.csv_service.fetch_ticker_groups().await {
                Ok(ticker_groups) => {
                    let ticker_count = ticker_groups.get_all_tickers().len();
                    context.cache.set_ticker_groups(ticker_groups);
                    self.logger.info(&format!("Ticker groups loaded: {} tickers", ticker_count));
                }
                Err(e) => {
                    self.logger.error(&format!("Failed to load ticker groups: {}", e));
                    return Err(e);
                }
            }
        }

        // Load VNINDEX if not already loaded
        if !context.cache.has_vnindex() {
            // TEMPORARY: Use smaller date range for faster startup
            self.logger.info("Loading VNINDEX data (1Y range for faster startup)...");

            match self.csv_service.fetch_vnindex_1y().await {
                Ok(vnindex_data) => {
                    let point_count = vnindex_data.len();
                    context.cache.set_vnindex(vnindex_data);
                    self.logger.info(&format!("VNINDEX loaded: {} points", point_count));
                }
                Err(e) => {
                    self.logger.warn(&format!("Failed to load VNINDEX 1Y data: {}, falling back to ALL range", e));
                    // Fall back to original method
                    match self.csv_service.fetch_vnindex().await {
                        Ok(vnindex_data) => {
                            let point_count = vnindex_data.len();
                            context.cache.set_vnindex(vnindex_data);
                            self.logger.info(&format!("VNINDEX loaded: {} points", point_count));
                        }
                        Err(e) => {
                            self.logger.error(&format!("Failed to load VNINDEX data: {}", e));
                            return Err(e);
                        }
                    }
                }
            }
        }

        // Set default range and queue initial request if not done
        let cache = context.cache.get_cache();
        if cache.last_requested_range.is_none() {
            self.logger.info("Setting default range to ALL (individual files) and queuing initial load...");

            let default_range = DateRangeConfig::new(TimeRange::All);
            context.cache.set_last_requested_range(default_range.clone());

            // Queue all tickers for initial full historical load (individual CSV files)
            let all_tickers = context.get_all_tickers();
            if !all_tickers.is_empty() {
                let csv_request = CSVRequest::new(all_tickers.clone(), default_range)
                    .with_priority(RequestPriority::High);

                context.request_queue.add_csv_request(csv_request);

                self.logger.info(&format!(
                    "Queued initial CSV request for {} tickers (ALL range - individual files)",
                    all_tickers.len()
                ));
            }
        }

        self.logger.info("Prerequisites loaded successfully");
        Ok(())
    }

    /// Process a single CSV request
    async fn process_csv_request(
        &mut self,
        context: &mut StateContext,
        request: &CSVRequest,
    ) -> anyhow::Result<()> {
        let timer = Timer::start("CSV request processing");

        self.logger.info(&format!(
            "Processing CSV request: {} tickers ({})",
            request.tickers.len(),
            request.date_range_config.range.as_str()
        ));

        // Fetch CSV data for all tickers in the request
        match self.csv_service.fetch_tickers(&request.tickers, &request.date_range_config).await {
            Ok(csv_data) => {
                // Merge data into cache
                let change_count = context.cache.merge_csv_data(csv_data);

                // Update the requested range to match this CSV request
                context.cache.set_last_requested_range(request.date_range_config.clone());

                timer.log_elapsed("FETCH_CSV");

                self.logger.info(&format!(
                    "CSV request completed: {} tickers updated in {:.1}ms",
                    change_count,
                    timer.elapsed_ms()
                ));

                // Log detailed information
                self.logger.info_with_data(
                    "CSV request processed",
                    format!(
                        "requested_tickers: {}, updated_tickers: {}, duration: {:.1}ms, range: {}",
                        request.tickers.len(),
                        change_count,
                        timer.elapsed_ms(),
                        request.date_range_config.range.as_str()
                    ),
                );
            }
            Err(e) => {
                self.logger.error(&format!(
                    "Failed to process CSV request for {} tickers: {}",
                    request.tickers.len(),
                    e
                ));

                // Continue processing other requests instead of failing completely
                // This allows the system to recover from individual ticker failures
            }
        }

        Ok(())
    }
}

impl Default for FetchCSVState {
    fn default() -> Self {
        Self::new()
    }
}