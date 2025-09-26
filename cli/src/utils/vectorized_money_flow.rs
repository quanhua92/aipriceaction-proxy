/**
 * Vectorized Money Flow Engine for Rust
 * High-performance vectorized calculations for money flow analysis
 * Direct port of TypeScript vectorized-money-flow.ts
 */

use crate::models::StockDataPoint;
use crate::utils::matrix_utils::{
    vectorize_ticker_data, calculate_money_flow_matrix, calculate_daily_totals,
    calculate_flow_percentages, apply_vnindex_volume_scaling, calculate_trend_scores, SingleDateDebugInfo,
    MoneyFlowMatrix
};
use crate::utils::money_flow_utils::{
    MoneyFlowTickerData, VolumeData, DebugData, PerformanceMetrics, calculate_vnindex_volume_scaling,
    MultipleDatesResult, SingleDateResult
};
use std::collections::HashMap;
use std::time::Instant;
use rayon::prelude::*;

/// Main vectorized money flow calculation engine
/// Replaces nested loops with vectorized operations for massive performance gains
pub fn calculate_multiple_dates_vectorized(
    ticker_data: &HashMap<String, Vec<StockDataPoint>>,
    selected_tickers: &[String],
    date_range: &[String],
    vnindex_data: Option<&[StockDataPoint]>,
    vnindex_volume_weighting: bool,
    _directional_colors: bool,
) -> MultipleDatesResult {
    let start_time = Instant::now();

    if selected_tickers.is_empty() || date_range.is_empty() {
        return MultipleDatesResult {
            results: HashMap::new(),
            metrics: PerformanceMetrics {
                vectorized_time: start_time.elapsed().as_millis() as f64,
                traditional_time: None,
                speedup_factor: None,
                ticker_count: 0,
                date_count: 0,
                calculation_count: 0,
            },
        };
    }

    // CRITICAL FIX: Sort dates chronologically (oldest first) for consistent matrix calculations
    let mut sorted_date_range = date_range.to_vec();
    sorted_date_range.sort(); // Sort ascending (oldest first) to match matrix expectations

    // STEP 1: Vectorize ticker data into matrix format
    let ticker_matrix = vectorize_ticker_data(ticker_data, selected_tickers, &sorted_date_range);

    // STEP 2: Calculate money flows for entire matrix in single operation
    let money_flow_matrix = calculate_money_flow_matrix(&ticker_matrix, Some(ticker_data), true);

    // STEP 3: Calculate daily totals using vectorized sum for both flow types
    let activity_daily_totals = calculate_daily_totals(&money_flow_matrix, "activity");
    let dollar_daily_totals = calculate_daily_totals(&money_flow_matrix, "dollar");

    // STEP 4: Convert to percentages (vectorized operation) for both flow types
    let activity_percentages = calculate_flow_percentages(
        &money_flow_matrix,
        &activity_daily_totals,
        "activity",
    );
    let dollar_percentages = calculate_flow_percentages(
        &money_flow_matrix,
        &dollar_daily_totals,
        "dollar",
    );

    // STEP 5: Calculate VNINDEX volume scaling
    let vnindex_volume_scaling = calculate_vnindex_volume_scaling(
        vnindex_data,
        &sorted_date_range,
        vnindex_volume_weighting,
    );

    // STEP 6: Apply volume scaling to signed flows for both flow types (vectorized)
    let scaled_signed_activity_flows = apply_vnindex_volume_scaling(
        &money_flow_matrix.activity_flows,
        &vnindex_volume_scaling,
        &money_flow_matrix.date_index,
        &sorted_date_range,
        selected_tickers.len(),
    );

    let scaled_signed_dollar_flows = apply_vnindex_volume_scaling(
        &money_flow_matrix.dollar_flows,
        &vnindex_volume_scaling,
        &money_flow_matrix.date_index,
        &sorted_date_range,
        selected_tickers.len(),
    );

    // STEP 7: Calculate trend scores (vectorized) - use activity percentages for trend score consistency
    let trend_scores = calculate_trend_scores(
        &activity_percentages,
        selected_tickers.len(),
        sorted_date_range.len(),
    );

    // STEP 8: Pre-allocate results to avoid repeated allocations
    let now = chrono::Utc::now();
    tracing::info!(
        "[{}] 🚀 [MONEY_FLOW] OPTIMIZATION: Pre-allocating data structures for {} dates x {} tickers",
        now.format("%Y-%m-%d %H:%M:%S UTC"),
        sorted_date_range.len(),
        selected_tickers.len()
    );

    let mut results = HashMap::with_capacity(sorted_date_range.len());
    let num_dates = sorted_date_range.len();

    // Process dates in parallel to maximize CPU usage
    let date_results: Vec<(String, Vec<MoneyFlowTickerData>)> = sorted_date_range
        .par_iter()
        .enumerate()
        .map(|(date_idx, date)| {
        // Extract activity flow data
        let activity_results = extract_single_date_data_with_flows(
            &money_flow_matrix,
            &activity_percentages,
            &scaled_signed_activity_flows,
            date,
            date_idx,
            num_dates,
        );

        // Extract dollar flow data
        let dollar_results = extract_single_date_data_with_flows(
            &money_flow_matrix,
            &dollar_percentages,
            &scaled_signed_dollar_flows,
            date,
            date_idx,
            num_dates,
        );

        let ticker_data_array: Vec<MoneyFlowTickerData> = activity_results
            .into_par_iter()
            .zip(dollar_results.into_par_iter())
            .enumerate()
            .map(|(index, (activity_item, dollar_item))| {
                // VERIFY: Ticker order consistency between activity and dollar results
                if activity_item.ticker != dollar_item.ticker {
                    eprintln!(
                        "🚨 TICKER ORDER MISMATCH at index {}: expected {}, got {}",
                        index, activity_item.ticker, dollar_item.ticker
                    );
                }

                // Calculate signed percentages for both flow types
                let multiplier = activity_item.debug_info.as_ref()
                    .map(|debug| debug.multiplier)
                    .unwrap_or_else(|| {
                        if activity_item.signed_flow >= 0.0 { 1.0 } else { -1.0 }
                    });

                let flow_sign = if multiplier >= 0.0 { 1.0 } else { -1.0 };

                // DEBUG: Check for sign inconsistencies
                let original_activity_sign = if activity_item.signed_flow >= 0.0 { 1.0 } else { -1.0 };
                let original_dollar_sign = if dollar_item.signed_flow >= 0.0 { 1.0 } else { -1.0 };
                if original_activity_sign != original_dollar_sign {
                    println!(
                        "🔄 MONEY FLOW SIGN INCONSISTENCY DETECTED ({}): activity={}, dollar={}, fixed={}",
                        activity_item.ticker,
                        original_activity_sign,
                        original_dollar_sign,
                        flow_sign
                    );
                }

                let activity_signed_percentage = flow_sign * activity_item.flow_percentage.abs();
                let dollar_signed_percentage = flow_sign * dollar_item.flow_percentage.abs();

                // For backward compatibility, use activity as default signed_percentage
                let signed_percentage = activity_signed_percentage;

                // FIXED: Find correct trend score by ticker name, not by sorted position
                let original_ticker_index = selected_tickers
                    .iter()
                    .position(|t| t == &activity_item.ticker)
                    .unwrap_or(0);
                let correct_trend_score = trend_scores.get(original_ticker_index).copied().unwrap_or(0.0);

                // Create debug data if available
                let debug_data = activity_item.debug_info.map(|debug_info| {
                    let mut debug_map = HashMap::new();
                    debug_map.insert(date.clone(), DebugData {
                        effective_low: debug_info.effective_low,
                        effective_high: debug_info.effective_high,
                        effective_range: debug_info.effective_range,
                        multiplier: debug_info.multiplier,
                        is_limit_move: debug_info.is_limit_move,
                        prev_close: debug_info.prev_close,
                        price_change_percent: debug_info.price_change_percent,
                    });
                    debug_map
                });

                MoneyFlowTickerData {
                    ticker: activity_item.ticker.clone(),
                    name: activity_item.ticker.clone(),
                    market_cap: 0.0,
                    daily_data: HashMap::from([(date.clone(), activity_item.flow_percentage)]),
                    signed_flow_data: HashMap::from([(date.clone(), activity_item.signed_flow)]),
                    signed_percentage_data: HashMap::from([(date.clone(), signed_percentage)]),
                    activity_flow_data: HashMap::from([(date.clone(), activity_signed_percentage)]),
                    dollar_flow_data: HashMap::from([(date.clone(), dollar_signed_percentage)]),
                    volume_data: HashMap::from([(date.clone(), VolumeData {
                        volume: activity_item.volume,
                        change: 0.0,
                    })]),
                    trend_score: correct_trend_score,
                    debug_data,
                }
            })
            .collect();

            (date.clone(), ticker_data_array)
        })
        .collect();

    // Convert parallel results back to HashMap
    for (date, ticker_data_array) in date_results {
        results.insert(date, ticker_data_array);
    }

    let end_time = Instant::now();

    // Debug output matching TypeScript format
    let complete_time = chrono::Utc::now();
    tracing::info!(
        "[{}] 🚀 [MONEY_FLOW] DEBUG:CLI Money flow calculation completed",
        complete_time.format("%Y-%m-%d %H:%M:%S UTC")
    );
    if let Some(latest_date) = sorted_date_range.last() {
        tracing::info!(
            "[{}] 🚀 [MONEY_FLOW] DEBUG:CLI Latest money flow data for date {}:",
            complete_time.format("%Y-%m-%d %H:%M:%S UTC"),
            latest_date
        );
        let key_tickers = ["CTG", "VCB", "BID", "TCB"];

        if let Some(latest_results) = results.get(latest_date) {
            for ticker_name in key_tickers.iter() {
                if let Some(ticker_result) = latest_results.iter().find(|t| t.ticker == *ticker_name) {
                    let money_flow = ticker_result.signed_percentage_data.get(latest_date).unwrap_or(&0.0);
                    let volume_data = ticker_result.volume_data.get(latest_date);
                    let volume = volume_data.map(|v| v.volume).unwrap_or(0.0);
                    let activity_flow = ticker_result.activity_flow_data.get(latest_date).unwrap_or(&0.0);
                    let dollar_flow = ticker_result.dollar_flow_data.get(latest_date).unwrap_or(&0.0);

                    // Get close price from the original ticker_data HashMap parameter
                    let close_price = if let Some(stock_data) = ticker_data.get(*ticker_name) {
                        stock_data.iter()
                            .find(|point| point.time == *latest_date)
                            .map(|point| point.close)
                            .unwrap_or(0.0)
                    } else { 0.0 };

                    tracing::info!(
                        "[{}] 🚀 [MONEY_FLOW] DEBUG:CLI   {}: money_flow={:.2}%, volume={:.1}M, close={:.1}k, trend_score={:.2}, AF={:.2}, DF={:.2}",
                        complete_time.format("%Y-%m-%d %H:%M:%S UTC"),
                        ticker_name,
                        money_flow,
                        volume / 1_000_000.0,
                        close_price / 1000.0,
                        ticker_result.trend_score,
                        activity_flow,
                        dollar_flow
                    );
                }
            }
        }
    }

    let final_time = chrono::Utc::now();
    tracing::info!(
        "[{}] 🔚 [MONEY_FLOW] RETURN: calculate_multiple_dates_vectorized completed successfully",
        final_time.format("%Y-%m-%d %H:%M:%S UTC")
    );

    MultipleDatesResult {
        results,
        metrics: PerformanceMetrics {
            vectorized_time: end_time.duration_since(start_time).as_millis() as f64,
            traditional_time: None,
            speedup_factor: None,
            ticker_count: selected_tickers.len(),
            date_count: sorted_date_range.len(),
            calculation_count: selected_tickers.len() * sorted_date_range.len(),
        },
    }
}

/// Optimized single-date money flow calculation
/// Uses vectorized operations even for single date to maintain performance
pub fn calculate_single_date_vectorized(
    ticker_data: &HashMap<String, Vec<StockDataPoint>>,
    selected_tickers: &[String],
    target_date: &str,
    vnindex_data: Option<&[StockDataPoint]>,
    vnindex_volume_weighting: bool,
    _directional_colors: bool,
) -> SingleDateResult {
    let start_time = Instant::now();

    if selected_tickers.is_empty() {
        return SingleDateResult {
            result: vec![],
            metrics: PerformanceMetrics {
                vectorized_time: start_time.elapsed().as_millis() as f64,
                traditional_time: None,
                speedup_factor: None,
                ticker_count: 0,
                date_count: 1,
                calculation_count: 0,
            },
        };
    }

    // Create single-date range for vectorized processing
    let date_range = vec![target_date.to_string()];

    // Use the multiple dates function with single date
    let multiple_result = calculate_multiple_dates_vectorized(
        ticker_data,
        selected_tickers,
        &date_range,
        vnindex_data,
        vnindex_volume_weighting,
        _directional_colors,
    );

    let result = multiple_result.results
        .get(target_date)
        .cloned()
        .unwrap_or_default();

    let end_time = Instant::now();

    SingleDateResult {
        result,
        metrics: PerformanceMetrics {
            vectorized_time: end_time.duration_since(start_time).as_millis() as f64,
            traditional_time: None,
            speedup_factor: None,
            ticker_count: selected_tickers.len(),
            date_count: 1,
            calculation_count: selected_tickers.len(),
        },
    }
}

/// Extract single date data from vectorized results with flows
fn extract_single_date_data_with_flows(
    matrix: &MoneyFlowMatrix,
    percentages: &[f64],
    flows: &[f64],
    _target_date: &str,
    date_idx: usize,
    num_dates: usize,
) -> Vec<ExtractedSingleDateResult> {
    let mut results = Vec::new();

    for (ticker_idx, ticker) in matrix.tickers.iter().enumerate() {
        let index = ticker_idx * num_dates + date_idx;

        let debug_info = matrix.debug_info.as_ref().map(|debug| SingleDateDebugInfo {
            effective_low: debug.effective_lows[index],
            effective_high: debug.effective_highs[index],
            effective_range: debug.effective_ranges[index],
            multiplier: debug.multipliers[index],
            is_limit_move: debug.is_limit_moves[index],
            prev_close: debug.prev_closes[index],
            price_change_percent: debug.price_change_percents[index],
        });

        results.push(ExtractedSingleDateResult {
            ticker: ticker.clone(),
            flow_percentage: percentages[index],
            signed_flow: flows[index], // Use the provided flows (activity or dollar)
            volume: matrix.volumes[index],
            debug_info,
        });
    }

    results
}

/// Extracted single date result structure
#[derive(Debug, Clone)]
struct ExtractedSingleDateResult {
    ticker: String,
    flow_percentage: f64,
    signed_flow: f64,
    volume: f64,
    debug_info: Option<SingleDateDebugInfo>,
}