/**
 * Vectorized MA Score Engine
 * High-performance vectorized calculations for MA score analysis
 * Similar to vectorized-money-flow.ts but for moving average score calculations
 */

use crate::{
    models::{
        ma_score::{MAScoreProcessConfig, MAScorePerformanceMetrics, MAScoreTickerData},
        StockDataPoint,
    },
    utils::{
        matrix_utils::{vectorize_ticker_data, calculate_ma_score_matrix, extract_ma_score_for_date, extract_ma_values},
        Timer, Logger,
    },
};
use std::collections::HashMap;
use rayon::prelude::*;

/// Sequential (non-parallel) MA Score calculation for small incremental updates
/// Avoids Rayon overhead when processing very few dates (<=3)
pub fn calculate_multiple_dates_sequential_ma_score(
    ticker_data: &HashMap<String, Vec<StockDataPoint>>,
    selected_tickers: &[String],
    date_range: &[String],
    config: &MAScoreProcessConfig,
) -> (HashMap<String, Vec<MAScoreTickerData>>, MAScorePerformanceMetrics) {
    let logger = Logger::new("SEQUENTIAL_MA_SCORE");
    let timer = Timer::start("sequential MA score calculation");

    logger.debug(&format!(
        "Starting sequential MA score calculation for {} tickers, {} dates",
        selected_tickers.len(),
        date_range.len()
    ));

    // Sort dates to ensure consistent calculations (chronological order)
    let mut sorted_date_range = date_range.to_vec();
    sorted_date_range.sort();

    // STEP 1: Vectorize ticker data into matrix format (same as parallel version)
    let ticker_matrix = vectorize_ticker_data(ticker_data, selected_tickers, &sorted_date_range);

    // STEP 2: Calculate MA scores for entire matrix in single vectorized operation
    let ma_score_matrix = calculate_ma_score_matrix(&ticker_matrix);

    // STEP 3: Extract results for each date (SEQUENTIAL - no Rayon)
    logger.debug("⚡ SEQUENTIAL: Processing dates without parallel overhead");

    let date_results: Vec<(String, Vec<MAScoreTickerData>)> = sorted_date_range
        .iter() // SEQUENTIAL: Regular iterator instead of par_iter()
        .map(|date| {
            let single_date_results = extract_ma_score_for_date(&ma_score_matrix, date);

            // Convert to expected MAScoreTickerData format (SEQUENTIAL)
            let date_ticker_data: Vec<MAScoreTickerData> = single_date_results
                .into_iter() // SEQUENTIAL: Regular iterator instead of into_par_iter()
                .map(|result| {
                let mut ma10_scores = HashMap::new();
                let mut ma20_scores = HashMap::new();
                let mut ma50_scores = HashMap::new();

                // Only include valid scores (non-zero and finite)
                if result.ma10_value > 0.0 && result.ma10_score.is_finite() {
                    ma10_scores.insert(date.clone(), result.ma10_score);
                }
                if result.ma20_value > 0.0 && result.ma20_score.is_finite() {
                    ma20_scores.insert(date.clone(), result.ma20_score);
                }
                if result.ma50_value > 0.0 && result.ma50_score.is_finite() {
                    ma50_scores.insert(date.clone(), result.ma50_score);
                }

                // Create debug data with actual MA values and current price
                let mut debug_data = HashMap::new();
                debug_data.insert(date.clone(), crate::models::ma_score::MAScoreDebugData {
                    current_price: result.close_price,
                    ma10_value: if result.ma10_value > 0.0 && result.ma10_value.is_finite() {
                        Some(result.ma10_value)
                    } else { None },
                    ma20_value: if result.ma20_value > 0.0 && result.ma20_value.is_finite() {
                        Some(result.ma20_value)
                    } else { None },
                    ma50_value: if result.ma50_value > 0.0 && result.ma50_value.is_finite() {
                        Some(result.ma50_value)
                    } else { None },
                });

                MAScoreTickerData {
                    ticker: result.ticker.clone(),
                    name: result.ticker.clone(), // Use ticker as name for now
                    market_cap: 0.0, // Not used in current calculations
                    ma10_scores,
                    ma20_scores,
                    ma50_scores,
                    trend_score: 0.0, // Simplified for performance
                    consecutive_days_above_ma: 0, // Simplified for performance
                    consecutive_days_below_ma: 0, // Simplified for performance
                    debug_data: Some(debug_data),
                }
            })
            .filter(|ticker_data| {
                // Only include tickers with at least one valid score
                !ticker_data.ma10_scores.is_empty() ||
                !ticker_data.ma20_scores.is_empty() ||
                !ticker_data.ma50_scores.is_empty()
            })
            .collect();

            // Return tuple of (date, ticker_data) for sequential collection
            (date.clone(), date_ticker_data)
        })
        .collect();

    // Convert sequential results to HashMap
    let mut results = HashMap::new();
    for (date, date_ticker_data) in date_results {
        if !date_ticker_data.is_empty() {
            results.insert(date, date_ticker_data);
        }
    }

    let elapsed = timer.elapsed_ms();

    let metrics = MAScorePerformanceMetrics {
        calculation_time: elapsed,
        ticker_count: selected_tickers.len(),
        date_count: date_range.len(),
        calculation_count: selected_tickers.len() * date_range.len(),
        ma_period: config.default_ma_period,
    };

    logger.info(&format!(
        "Sequential MA score calculation completed: {} dates, {} tickers in {:.1}ms",
        results.len(),
        selected_tickers.len(),
        elapsed
    ));

    (results, metrics)
}

/// Calculate multiple dates with vectorized operations (MA Score version)
/// Exactly matches the TypeScript calculateMultipleDatesVectorized function for MA scores
pub fn calculate_multiple_dates_vectorized_ma_score(
    ticker_data: &HashMap<String, Vec<StockDataPoint>>,
    selected_tickers: &[String],
    date_range: &[String],
    config: &MAScoreProcessConfig,
    excluded_tickers: &[String],
) -> (HashMap<String, Vec<MAScoreTickerData>>, MAScorePerformanceMetrics) {
    let logger = Logger::new("VECTORIZED_MA_SCORE");
    let timer = Timer::start("vectorized MA score calculation");

    logger.debug(&format!(
        "Starting vectorized MA score calculation for {} tickers, {} dates",
        selected_tickers.len(),
        date_range.len()
    ));

    // Filter out excluded tickers
    let filtered_tickers: Vec<String> = selected_tickers
        .iter()
        .filter(|ticker| !excluded_tickers.contains(ticker))
        .cloned()
        .collect();

    // Sort dates to ensure consistent calculations (chronological order)
    let mut sorted_date_range = date_range.to_vec();
    sorted_date_range.sort();

    // STEP 1: Vectorize ticker data into matrix format (same as money flow)
    // OPTIMIZATION: Only vectorize data for requested dates, not all historical data
    let ticker_matrix = vectorize_ticker_data(ticker_data, &filtered_tickers, &sorted_date_range);

    // STEP 2: Calculate MA scores for entire matrix in single vectorized operation
    let ma_score_matrix = calculate_ma_score_matrix(&ticker_matrix);

    // STEP 3: Extract results for each date (RAYON PARALLEL)
    logger.debug("⚡ RAYON: Processing dates in parallel for maximum speed");

    let date_results: Vec<(String, Vec<MAScoreTickerData>)> = sorted_date_range
        .par_iter() // RAYON: Parallel processing across all CPU cores
        .map(|date| {
            let single_date_results = extract_ma_score_for_date(&ma_score_matrix, date);

            // Convert to expected MAScoreTickerData format (RAYON PARALLEL)
            let date_ticker_data: Vec<MAScoreTickerData> = single_date_results
                .into_par_iter() // RAYON: Parallel ticker processing within each date
                .map(|result| {
                let mut ma10_scores = HashMap::new();
                let mut ma20_scores = HashMap::new();
                let mut ma50_scores = HashMap::new();

                // Only include valid scores (non-zero and finite)
                if result.ma10_value > 0.0 && result.ma10_score.is_finite() {
                    ma10_scores.insert(date.clone(), result.ma10_score);
                }
                if result.ma20_value > 0.0 && result.ma20_score.is_finite() {
                    ma20_scores.insert(date.clone(), result.ma20_score);
                }
                if result.ma50_value > 0.0 && result.ma50_score.is_finite() {
                    ma50_scores.insert(date.clone(), result.ma50_score);
                }

                // Create debug data with actual MA values and current price
                let mut debug_data = HashMap::new();
                debug_data.insert(date.clone(), crate::models::ma_score::MAScoreDebugData {
                    current_price: result.close_price,
                    ma10_value: if result.ma10_value > 0.0 && result.ma10_value.is_finite() {
                        Some(result.ma10_value)
                    } else { None },
                    ma20_value: if result.ma20_value > 0.0 && result.ma20_value.is_finite() {
                        Some(result.ma20_value)
                    } else { None },
                    ma50_value: if result.ma50_value > 0.0 && result.ma50_value.is_finite() {
                        Some(result.ma50_value)
                    } else { None },
                });

                MAScoreTickerData {
                    ticker: result.ticker.clone(),
                    name: result.ticker.clone(), // Use ticker as name for now
                    market_cap: 0.0, // Not used in current calculations
                    ma10_scores,
                    ma20_scores,
                    ma50_scores,
                    trend_score: 0.0, // Simplified for performance
                    consecutive_days_above_ma: 0, // Simplified for performance
                    consecutive_days_below_ma: 0, // Simplified for performance
                    debug_data: Some(debug_data),
                }
            })
            .filter(|ticker_data| {
                // Only include tickers with at least one valid score
                !ticker_data.ma10_scores.is_empty() ||
                !ticker_data.ma20_scores.is_empty() ||
                !ticker_data.ma50_scores.is_empty()
            })
            .collect();

            // Return tuple of (date, ticker_data) for parallel collection
            (date.clone(), date_ticker_data)
        })
        .collect();

    // Convert parallel results to HashMap
    let mut results = HashMap::new();
    for (date, date_ticker_data) in date_results {
        if !date_ticker_data.is_empty() {
            results.insert(date, date_ticker_data);
        }
    }

    let elapsed = timer.elapsed_ms();

    let metrics = MAScorePerformanceMetrics {
        calculation_time: elapsed,
        ticker_count: selected_tickers.len(),
        date_count: date_range.len(),
        calculation_count: selected_tickers.len() * date_range.len(),
        ma_period: config.default_ma_period,
    };

    logger.info(&format!(
        "Vectorized MA score calculation completed: {} dates, {} tickers in {:.1}ms",
        results.len(),
        selected_tickers.len(),
        elapsed
    ));

    // Debug output matching TypeScript format
    if let Some(latest_date) = date_range.last() {
        let complete_time = chrono::Utc::now();
        tracing::info!(
            "[{}] 🚀 [MA_SCORE] DEBUG:CLI MA score calculation completed",
            complete_time.format("%Y-%m-%d %H:%M:%S UTC")
        );
        tracing::info!(
            "[{}] 🚀 [MA_SCORE] DEBUG:CLI Latest MA score data for date {}:",
            complete_time.format("%Y-%m-%d %H:%M:%S UTC"),
            latest_date
        );
        let key_tickers = ["CTG", "VCB", "BID", "TCB"];

        if let Some(latest_ma_data) = results.get(latest_date) {
            for ticker_name in key_tickers.iter() {
                if let Some(ticker_ma) = latest_ma_data.iter().find(|t| t.ticker == *ticker_name) {
                    let ma20_score = ticker_ma.ma20_scores.get(latest_date).unwrap_or(&0.0);

                    // Get close price and MA20 value from debug_data (calculated MA values)
                    let (close_price, ma20_value) = if let Some(debug_data) = &ticker_ma.debug_data {
                        if let Some(debug_info) = debug_data.get(latest_date) {
                            (debug_info.current_price, debug_info.ma20_value)
                        } else {
                            (0.0, None)
                        }
                    } else {
                        (0.0, None)
                    };

                    let close_str = if close_price > 0.0 {
                        format!("{:.1}k", close_price / 1000.0)
                    } else {
                        "N/A".to_string()
                    };

                    let ma20_str = if let Some(ma20) = ma20_value {
                        if ma20 > 0.0 {
                            format!("{:.1}k", ma20 / 1000.0)
                        } else {
                            "N/A".to_string()
                        }
                    } else {
                        "N/A".to_string()
                    };

                    tracing::info!(
                        "[{}] 🚀 [MA_SCORE] DEBUG:CLI   {}: ma20_score={:.2}, close={}, ma20={}",
                        complete_time.format("%Y-%m-%d %H:%M:%S UTC"),
                        ticker_name,
                        ma20_score,
                        close_str,
                        ma20_str
                    );
                }
            }
        }
    }

    (results, metrics)
}

/// Calculate MA scores for all uncalculated dates in current range
/// Matches the TypeScript MAScoreCalculator.calculateForCurrentRange method
pub fn calculate_for_current_range(
    ticker_data: &HashMap<String, Vec<StockDataPoint>>,
    all_tickers: &[String],
    date_range: &[String],
    config: &MAScoreProcessConfig,
    excluded_tickers: &[String],
) -> (HashMap<String, Vec<MAScoreTickerData>>, MAScorePerformanceMetrics) {
    let logger = Logger::new("MA_SCORE_CALCULATOR");

    logger.info(&format!(
        "Starting MA score calculation for current range: {} tickers, {} dates",
        all_tickers.len(),
        date_range.len()
    ));

    // Filter out excluded tickers (like VNINDEX)
    let filtered_tickers: Vec<String> = all_tickers
        .iter()
        .filter(|ticker| !excluded_tickers.contains(ticker))
        .cloned()
        .collect();

    // Build ticker data map from cache
    let mut filtered_ticker_data = HashMap::new();
    let mut missing_tickers = 0;

    for ticker in &filtered_tickers {
        if let Some(data) = ticker_data.get(ticker) {
            if !data.is_empty() {
                filtered_ticker_data.insert(ticker.clone(), data.clone());
            } else {
                missing_tickers += 1;
            }
        } else {
            missing_tickers += 1;
        }
    }

    logger.info(&format!(
        "Available data: {} tickers, {} missing",
        filtered_ticker_data.len(),
        missing_tickers
    ));

    if filtered_ticker_data.len() < 5 {
        logger.warn("Too few tickers for reliable MA score calculation");
        return (
            HashMap::new(),
            MAScorePerformanceMetrics {
                calculation_time: 0.0,
                ticker_count: 0,
                date_count: 0,
                calculation_count: 0,
                ma_period: 20,
            }
        );
    }

    // Calculate MA scores using vectorized operations
    calculate_multiple_dates_vectorized_ma_score(
        &filtered_ticker_data,
        &filtered_tickers,
        date_range,
        config,
        &[], // excluded_tickers (already filtered)
    )
}

/// Calculate MA scores for specific dates only (incremental calculation)
/// Matches the TypeScript MAScoreCalculator.calculateForDates method
pub fn calculate_for_dates(
    ticker_data: &HashMap<String, Vec<StockDataPoint>>,
    all_tickers: &[String],
    dates: &[String],
    config: &MAScoreProcessConfig,
    excluded_tickers: &[String],
) -> (HashMap<String, Vec<MAScoreTickerData>>, MAScorePerformanceMetrics) {
    let logger = Logger::new("MA_SCORE_CALCULATOR");

    if dates.is_empty() {
        return (
            HashMap::new(),
            MAScorePerformanceMetrics {
                calculation_time: 0.0,
                ticker_count: 0,
                date_count: 0,
                calculation_count: 0,
                ma_period: 20,
            }
        );
    }

    logger.info(&format!(
        "Starting incremental MA score calculation: {} dates",
        dates.len()
    ));

    // OPTIMIZATION: For small incremental updates (<=10 dates), use lightweight calculation
    // to avoid processing the entire 2680-date dataset
    if dates.len() <= 10 {
        logger.info(&format!(
            "Using lightweight calculation for {} dates (avoiding full matrix processing)",
            dates.len()
        ));

        let timer = Timer::start("lightweight incremental MA score");

        // Filter out excluded tickers (like VNINDEX)
        let filtered_tickers: Vec<String> = all_tickers
            .iter()
            .filter(|ticker| !excluded_tickers.contains(ticker))
            .cloned()
            .collect();

        // Build ticker data map from cache
        let mut filtered_ticker_data = HashMap::new();
        for ticker in &filtered_tickers {
            if let Some(data) = ticker_data.get(ticker) {
                if !data.is_empty() {
                    filtered_ticker_data.insert(ticker.clone(), data.clone());
                }
            }
        }

        // CRITICAL FIX: For MA calculations, we need historical context (50+ dates)
        // but only want to update the requested dates in the final result
        let ma_buffer_days = 60; // Buffer for MA50 + extra safety margin

        // Get date range that includes enough historical context for MA calculations
        let mut calculation_date_range = Vec::new();

        // Sort all available dates to find the historical context needed
        let mut all_dates: Vec<String> = filtered_ticker_data
            .values()
            .next()
            .map(|ticker_data| ticker_data.iter().map(|p| p.time.clone()).collect())
            .unwrap_or_default();
        all_dates.sort();

        // Find the earliest requested date
        let earliest_requested = dates.iter().min().cloned().unwrap_or_default();

        // Include historical context: find position of earliest requested date and go back ma_buffer_days
        if let Some(earliest_pos) = all_dates.iter().position(|d| d == &earliest_requested) {
            let start_pos = earliest_pos.saturating_sub(ma_buffer_days);

            // Take from start_pos to the end of requested date range
            let latest_requested = dates.iter().max().cloned().unwrap_or_default();
            if let Some(latest_pos) = all_dates.iter().position(|d| d == &latest_requested) {
                calculation_date_range = all_dates[start_pos..=latest_pos].to_vec();
            }
        }

        // Fallback: if we couldn't find proper range, use requested dates (will be inaccurate but won't crash)
        if calculation_date_range.is_empty() {
            calculation_date_range = dates.to_vec();
        }

        logger.info(&format!(
            "MA Score incremental: Using {} dates for calculation (including {} buffer days) to update {} requested dates",
            calculation_date_range.len(),
            ma_buffer_days,
            dates.len()
        ));

        // Filter ticker data to include the calculation date range (with historical context)
        let mut date_filtered_ticker_data = HashMap::new();
        let calc_dates_set: std::collections::HashSet<&String> = calculation_date_range.iter().collect();

        for (ticker, ticker_points) in &filtered_ticker_data {
            let filtered_points: Vec<StockDataPoint> = ticker_points
                .iter()
                .filter(|point| calc_dates_set.contains(&point.time))
                .cloned()
                .collect();

            if !filtered_points.is_empty() {
                date_filtered_ticker_data.insert(ticker.clone(), filtered_points);
            }
        }

        // For very small incremental updates, use sequential processing to avoid Rayon overhead
        let (results, metrics) = if dates.len() <= 3 {
            logger.info("Using sequential processing for very small incremental update");
            calculate_multiple_dates_sequential_ma_score(
                &date_filtered_ticker_data,
                &filtered_tickers,
                &calculation_date_range, // Use extended range for calculation
                config,
            )
        } else {
            logger.info("Using Rayon parallel processing for incremental update");
            calculate_multiple_dates_vectorized_ma_score(
                &date_filtered_ticker_data,
                &filtered_tickers,
                &calculation_date_range, // Use extended range for calculation
                config,
                &[], // excluded_tickers (already filtered)
            )
        };

        // CRITICAL: Filter results to only return the requested dates
        let requested_dates_set: std::collections::HashSet<&String> = dates.iter().collect();
        let mut filtered_results = HashMap::new();
        let original_result_count = results.len();

        for (date, ticker_data) in results {
            if requested_dates_set.contains(&date) {
                filtered_results.insert(date, ticker_data);
            }
        }

        logger.debug(&format!(
            "Incremental calculation: Computed {} dates, returning {} requested dates",
            original_result_count,
            filtered_results.len()
        ));

        let (results, mut metrics) = (filtered_results, metrics);

        let elapsed = timer.elapsed_ms();
        metrics.calculation_time = elapsed;

        logger.info(&format!(
            "Lightweight incremental calculation completed in {:.1}ms for {} dates",
            elapsed, dates.len()
        ));

        return (results, metrics);
    }

    // For larger incremental updates, use the full vectorized approach
    logger.info(&format!(
        "Using full vectorized calculation for {} dates",
        dates.len()
    ));

    // Filter out excluded tickers (like VNINDEX)
    let filtered_tickers: Vec<String> = all_tickers
        .iter()
        .filter(|ticker| !excluded_tickers.contains(ticker))
        .cloned()
        .collect();

    // Build ticker data map from cache
    let mut filtered_ticker_data = HashMap::new();
    for ticker in &filtered_tickers {
        if let Some(data) = ticker_data.get(ticker) {
            if !data.is_empty() {
                filtered_ticker_data.insert(ticker.clone(), data.clone());
            }
        }
    }

    // Calculate MA scores using vectorized operations
    calculate_multiple_dates_vectorized_ma_score(
        &filtered_ticker_data,
        &filtered_tickers,
        dates,
        config,
        &[], // excluded_tickers (already filtered)
    )
}

/// Update MA values (ma10, ma20, ma50) for all tickers
/// Matches the TypeScript MAScoreCalculator.updateMAValuesInStockData method
pub fn update_ma_values_in_stock_data(
    ticker_data: &HashMap<String, Vec<StockDataPoint>>,
    all_tickers: &[String],
    date_range: &[String],
) -> HashMap<String, Vec<(String, Option<f64>, Option<f64>, Option<f64>)>> {
    let logger = Logger::new("MA_VALUES");
    let timer = Timer::start("MA values update");

    logger.info(&format!(
        "Updating MA values in stock data for {} tickers, {} dates",
        all_tickers.len(),
        date_range.len()
    ));

    // Build ticker data map from cache
    let mut filtered_ticker_data = HashMap::new();
    for ticker in all_tickers {
        if let Some(data) = ticker_data.get(ticker) {
            if !data.is_empty() {
                filtered_ticker_data.insert(ticker.clone(), data.clone());
            }
        }
    }

    if date_range.is_empty() {
        logger.warn("No trading dates available for MA value updates");
        return HashMap::new();
    }

    // Extract MA values using vectorized calculation
    let ma_values_map = extract_ma_values(&filtered_ticker_data, all_tickers, date_range);

    let elapsed = timer.elapsed_ms();
    logger.info(&format!(
        "MA values updated in {:.1}ms for {} tickers",
        elapsed,
        all_tickers.len()
    ));

    ma_values_map
}

/// Update MA values for excluded tickers (like VNINDEX)
/// Handles tickers that need MA values calculated on their full date range
pub fn update_ma_values_for_excluded_tickers(
    ticker_data: &HashMap<String, Vec<StockDataPoint>>,
    excluded_tickers: &[String],
    all_trading_dates: &[String],
) -> HashMap<String, Vec<(String, Option<f64>, Option<f64>, Option<f64>)>> {
    let logger = Logger::new("MA_VALUES");
    let timer = Timer::start("excluded ticker MA values");

    logger.info(&format!(
        "Updating MA values for excluded tickers: {}",
        excluded_tickers.join(", ")
    ));

    let mut excluded_ticker_data = HashMap::new();

    // Get data for each excluded ticker
    for ticker in excluded_tickers {
        if let Some(data) = ticker_data.get(ticker) {
            if !data.is_empty() {
                excluded_ticker_data.insert(ticker.clone(), data.clone());
                logger.debug(&format!(
                    "Processing excluded ticker: {} with {} data points",
                    ticker,
                    data.len()
                ));
            } else {
                logger.warn(&format!("Excluded ticker not found: {}", ticker));
            }
        }
    }

    if excluded_ticker_data.is_empty() {
        logger.warn("No excluded tickers with data found");
        return HashMap::new();
    }

    if all_trading_dates.is_empty() {
        logger.warn("No trading dates available for excluded ticker MA calculation");
        return HashMap::new();
    }

    logger.info(&format!(
        "Calculating MA values for {} excluded tickers using {} trading dates",
        excluded_ticker_data.len(),
        all_trading_dates.len()
    ));

    // Extract MA values using all trading dates
    let ma_values_map = extract_ma_values(&excluded_ticker_data, excluded_tickers, all_trading_dates);

    let elapsed = timer.elapsed_ms();
    logger.info(&format!(
        "Excluded ticker MA values updated in {:.1}ms for {} tickers",
        elapsed,
        excluded_ticker_data.len()
    ));

    ma_values_map
}

/// Check if MA score calculation is needed
pub fn is_calculation_needed(
    uncalculated_dates: &[String],
    changed_dates: &[String],
) -> bool {
    let total_needing_calculation = {
        let mut all_dates = uncalculated_dates.to_vec();
        all_dates.extend_from_slice(changed_dates);
        all_dates.sort();
        all_dates.dedup();
        all_dates.len()
    };

    total_needing_calculation > 0
}

/// Get calculation statistics
pub fn get_calculation_stats(
    uncalculated_dates: &[String],
    changed_dates: &[String],
    calculated_dates_count: usize,
) -> (Vec<String>, Vec<String>, usize, bool) {
    let needs_calculation = is_calculation_needed(uncalculated_dates, changed_dates);

    (
        uncalculated_dates.to_vec(),
        changed_dates.to_vec(),
        calculated_dates_count,
        needs_calculation,
    )
}

/// Determine if full recalculation is needed vs incremental
pub fn should_recalculate_all(
    total_trading_dates: usize,
    uncalculated_count: usize,
    changed_count: usize,
) -> bool {
    // Strategy: If more than 50% of trading dates need calculation, do full recalculation
    let needs_calculation_count = uncalculated_count + changed_count;
    let recalculation_threshold = 5.max(total_trading_dates / 2);

    needs_calculation_count >= recalculation_threshold
}