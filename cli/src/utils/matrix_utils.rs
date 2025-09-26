use crate::models::{StockDataPoint, MAScoreMatrix, SingleDateMAScoreResult};
use std::collections::HashMap;

/// 3D Matrix structure for ticker data: [tickers, dates, OHLCV]
/// Index 0: Open, Index 1: High, Index 2: Low, Index 3: Close, Index 4: Volume
#[derive(Debug, Clone)]
pub struct TickerDataMatrix {
    pub data: Vec<f64>,
    pub shape: (usize, usize, usize), // [tickers, dates, ohlcv_fields]
    pub ticker_index: HashMap<String, usize>,
    pub date_index: HashMap<String, usize>,
    pub tickers: Vec<String>,
    pub dates: Vec<String>,
}

/// Money flow calculation results matrix
#[derive(Debug, Clone)]
pub struct MoneyFlowMatrix {
    /// Activity flows: Multiplier × Volume - measures trading activity intensity & conviction
    pub activity_flows: Vec<f64>,
    /// Dollar flows: Multiplier × Close × Volume - measures estimated economic value with direction
    pub dollar_flows: Vec<f64>,
    /// Absolute activity flows for percentage calculations
    pub absolute_activity_flows: Vec<f64>,
    /// Absolute dollar flows for percentage calculations
    pub absolute_dollar_flows: Vec<f64>,
    /// Volume data
    pub volumes: Vec<f64>,
    /// Close price data for dollar flow calculations
    pub closes: Vec<f64>,
    /// Legacy flows field for backward compatibility
    pub flows: Vec<f64>, // Same as activity_flows
    /// Legacy absolute_flows field for backward compatibility
    pub absolute_flows: Vec<f64>, // Same as absolute_activity_flows
    pub shape: (usize, usize), // [tickers, dates]
    pub ticker_index: HashMap<String, usize>,
    pub date_index: HashMap<String, usize>,
    pub tickers: Vec<String>,
    pub dates: Vec<String>,
    /// Debug information
    pub debug_info: Option<MoneyFlowDebugInfo>,
}

/// Debug information for money flow calculations
#[derive(Debug, Clone)]
pub struct MoneyFlowDebugInfo {
    pub effective_lows: Vec<f64>,
    pub effective_highs: Vec<f64>,
    pub effective_ranges: Vec<f64>,
    pub multipliers: Vec<f64>,
    pub is_limit_moves: Vec<bool>,
    pub prev_closes: Vec<f64>,
    pub price_change_percents: Vec<f64>,
}

/// Convert ticker data object to vectorized 3D matrix format
/// This eliminates object property lookups during calculations
pub fn vectorize_ticker_data(
    ticker_data: &HashMap<String, Vec<StockDataPoint>>,
    tickers: &[String],
    date_range: &[String],
) -> TickerDataMatrix {
    let num_tickers = tickers.len();
    let num_dates = date_range.len();
    let num_fields = 5; // OHLCV

    // Pre-allocate matrix data - much faster than dynamic arrays
    let mut data = vec![0.0; num_tickers * num_dates * num_fields];

    // Create lookup maps for O(1) access
    let mut ticker_index = HashMap::new();
    let mut date_index = HashMap::new();

    for (i, ticker) in tickers.iter().enumerate() {
        ticker_index.insert(ticker.clone(), i);
    }
    for (i, date) in date_range.iter().enumerate() {
        date_index.insert(date.clone(), i);
    }

    // Fill matrix with ticker data
    let empty_vec = Vec::new();
    for (t, ticker) in tickers.iter().enumerate() {
        let ticker_data_points = ticker_data.get(ticker).unwrap_or(&empty_vec);

        // Create date lookup for this ticker's data
        let mut ticker_date_map = HashMap::new();
        for point in ticker_data_points {
            ticker_date_map.insert(point.time.clone(), point);
        }

        for (d, date) in date_range.iter().enumerate() {
            // Calculate base index for this ticker-date combination
            let base_index = t * num_dates * num_fields + d * num_fields;

            if let Some(data_point) = ticker_date_map.get(date) {
                // Store OHLCV data in contiguous memory
                data[base_index + 0] = data_point.open;
                data[base_index + 1] = data_point.high;
                data[base_index + 2] = data_point.low;
                data[base_index + 3] = data_point.close;
                data[base_index + 4] = data_point.volume as f64;
            } else {
                // Fill missing data with zeros
                data[base_index + 0] = 0.0;
                data[base_index + 1] = 0.0;
                data[base_index + 2] = 0.0;
                data[base_index + 3] = 0.0;
                data[base_index + 4] = 0.0;
            }
        }
    }

    TickerDataMatrix {
        data,
        shape: (num_tickers, num_dates, num_fields),
        ticker_index,
        date_index,
        tickers: tickers.to_vec(),
        dates: date_range.to_vec(),
    }
}

/// Vectorized money flow calculation: ((close - low) - (high - close)) / (high - low) * volume
/// Processes entire matrix in single pass - massive performance gain
pub fn calculate_money_flow_matrix(
    matrix: &TickerDataMatrix,
    _ticker_data: Option<&HashMap<String, Vec<StockDataPoint>>>,
    include_debug_info: bool,
) -> MoneyFlowMatrix {
    let (num_tickers, num_dates, num_fields) = matrix.shape;
    let data = &matrix.data;

    // Pre-allocate result arrays
    let mut activity_flows = vec![0.0; num_tickers * num_dates];
    let mut dollar_flows = vec![0.0; num_tickers * num_dates];
    let mut absolute_activity_flows = vec![0.0; num_tickers * num_dates];
    let mut absolute_dollar_flows = vec![0.0; num_tickers * num_dates];
    let mut volumes = vec![0.0; num_tickers * num_dates];
    let mut closes = vec![0.0; num_tickers * num_dates];

    // Pre-allocate debug arrays if needed
    let mut debug_info = if include_debug_info {
        Some(MoneyFlowDebugInfo {
            effective_lows: vec![0.0; num_tickers * num_dates],
            effective_highs: vec![0.0; num_tickers * num_dates],
            effective_ranges: vec![0.0; num_tickers * num_dates],
            multipliers: vec![0.0; num_tickers * num_dates],
            is_limit_moves: vec![false; num_tickers * num_dates],
            prev_closes: vec![0.0; num_tickers * num_dates],
            price_change_percents: vec![0.0; num_tickers * num_dates],
        })
    } else {
        None
    };

    // Optimized calculation with performance logging
    let now = chrono::Utc::now();
    tracing::info!(
        "[{}] 🚀 [MONEY_FLOW] RAYON: Money flow calculation for {} tickers x {} dates",
        now.format("%Y-%m-%d %H:%M:%S UTC"),
        num_tickers,
        num_dates
    );

    for t in 0..num_tickers {
        for d in 0..num_dates {
            let base_index = t * num_dates * num_fields + d * num_fields;
            let result_index = t * num_dates + d;

            // Extract OHLCV values
            let _open = data[base_index + 0];
            let high = data[base_index + 1];
            let low = data[base_index + 2];
            let close = data[base_index + 3];
            let volume = data[base_index + 4];

            // Calculate money flow multiplier and flows
            let (multiplier, activity_flow, dollar_flow) = if high == low || volume == 0.0 {
                // No price movement or volume - no money flow
                (0.0, 0.0, 0.0)
            } else {
                // Williams %R-based money flow calculation
                let money_flow_multiplier = ((close - low) - (high - close)) / (high - low);
                let activity_flow_val = money_flow_multiplier * volume;
                let dollar_flow_val = money_flow_multiplier * close * volume;
                (money_flow_multiplier, activity_flow_val, dollar_flow_val)
            };

            // Store results
            activity_flows[result_index] = activity_flow;
            dollar_flows[result_index] = dollar_flow;
            absolute_activity_flows[result_index] = activity_flow.abs();
            absolute_dollar_flows[result_index] = dollar_flow.abs();
            volumes[result_index] = volume;
            closes[result_index] = close;

            // Store debug info if requested
            if let Some(ref mut debug) = debug_info {
                debug.multipliers[result_index] = multiplier;
                debug.effective_lows[result_index] = low;
                debug.effective_highs[result_index] = high;
                debug.effective_ranges[result_index] = high - low;
                debug.is_limit_moves[result_index] = (high - low) < 0.001; // Very small range

                // Calculate previous close and price change for debug
                if d > 0 {
                    let prev_index = t * num_dates * num_fields + (d - 1) * num_fields;
                    let prev_close = data[prev_index + 3];
                    debug.prev_closes[result_index] = prev_close;
                    debug.price_change_percents[result_index] =
                        if prev_close > 0.0 { ((close - prev_close) / prev_close) * 100.0 } else { 0.0 };
                } else {
                    debug.prev_closes[result_index] = close;
                    debug.price_change_percents[result_index] = 0.0;
                }
            }
        }
    }

    MoneyFlowMatrix {
        flows: activity_flows.clone(), // Legacy compatibility
        absolute_flows: absolute_activity_flows.clone(), // Legacy compatibility
        activity_flows,
        dollar_flows,
        absolute_activity_flows,
        absolute_dollar_flows,
        volumes,
        closes,
        shape: (num_tickers, num_dates),
        ticker_index: matrix.ticker_index.clone(),
        date_index: matrix.date_index.clone(),
        tickers: matrix.tickers.clone(),
        dates: matrix.dates.clone(),
        debug_info,
    }
}

/// Calculate daily totals using vectorized sum for both flow types
pub fn calculate_daily_totals(matrix: &MoneyFlowMatrix, flow_type: &str) -> Vec<f64> {
    let (num_tickers, num_dates) = matrix.shape;
    let mut daily_totals = vec![0.0; num_dates];

    let flows = match flow_type {
        "activity" => &matrix.absolute_activity_flows,
        "dollar" => &matrix.absolute_dollar_flows,
        _ => &matrix.absolute_activity_flows, // Default to activity
    };

    for d in 0..num_dates {
        let mut total = 0.0;
        for t in 0..num_tickers {
            let index = t * num_dates + d;
            total += flows[index];
        }
        daily_totals[d] = total;
    }

    daily_totals
}

/// Convert to percentages (vectorized operation) for both flow types
pub fn calculate_flow_percentages(
    matrix: &MoneyFlowMatrix,
    daily_totals: &[f64],
    flow_type: &str,
) -> Vec<f64> {
    let (num_tickers, num_dates) = matrix.shape;
    let mut percentages = vec![0.0; num_tickers * num_dates];

    let flows = match flow_type {
        "activity" => &matrix.absolute_activity_flows,
        "dollar" => &matrix.absolute_dollar_flows,
        _ => &matrix.absolute_activity_flows, // Default to activity
    };

    for t in 0..num_tickers {
        for d in 0..num_dates {
            let index = t * num_dates + d;
            let flow_value = flows[index];
            let total = daily_totals[d];

            percentages[index] = if total > 0.0 {
                (flow_value / total) * 100.0
            } else {
                0.0
            };
        }
    }

    percentages
}

/// Apply VNINDEX volume scaling to signed flows (vectorized)
pub fn apply_vnindex_volume_scaling(
    flows: &[f64],
    vnindex_volume_scaling: &HashMap<String, f64>,
    _date_index: &HashMap<String, usize>,
    date_range: &[String],
    num_tickers: usize,
) -> Vec<f64> {
    let mut scaled_flows = flows.to_vec();
    let num_dates = date_range.len();

    for (d, date) in date_range.iter().enumerate() {
        let scaling_factor = vnindex_volume_scaling.get(date).unwrap_or(&1.0);

        for t in 0..num_tickers {
            let index = t * num_dates + d;
            scaled_flows[index] *= scaling_factor;
        }
    }

    scaled_flows
}

/// Calculate trend scores (vectorized) - matches TypeScript sophisticated algorithm
pub fn calculate_trend_scores(
    activity_percentages: &[f64],
    num_tickers: usize,
    num_dates: usize,
) -> Vec<f64> {
    let mut trend_scores = vec![0.0; num_tickers];

    for t in 0..num_tickers {
        // Debug first few tickers to see the data format
        if t < 4 {
            let sample_recent: Vec<f64> = ((num_dates - 5.min(num_dates))..num_dates)
                .map(|d| {
                    let index = t * num_dates + d;
                    activity_percentages[index]
                })
                .collect();
            let debug_time = chrono::Utc::now();
            tracing::info!(
                "[{}] 🔍 [MONEY_FLOW] TICKER {} DATA: recent_5_newest={:?}",
                debug_time.format("%Y-%m-%d %H:%M:%S UTC"),
                t,
                sample_recent
            );
        }

        // Calculate trend score using TypeScript's algorithm directly on the matrix data
        trend_scores[t] = if num_dates >= 7 {
            calculate_sophisticated_trend_score_matrix(activity_percentages, t, num_dates)
        } else {
            // For periods < 7 days, use simple average of POSITIVE values only (matches TypeScript)
            let mut total = 0.0;
            let mut valid_days = 0;
            for d in 0..num_dates {
                let index = t * num_dates + d;
                let value = activity_percentages[index];
                if value > 0.0 {
                    total += value;
                    valid_days += 1;
                }
            }
            if valid_days > 0 { total / valid_days as f64 } else { 0.0 }
        };
    }

    trend_scores
}

/// Matrix-based trend score calculation exactly matching TypeScript algorithm
fn calculate_sophisticated_trend_score_matrix(
    activity_percentages: &[f64],
    ticker_index: usize,
    num_dates: usize,
) -> f64 {
    // Exact TypeScript period calculation logic
    let recent_days = std::cmp::min(14, (num_dates as f64 * 0.3).floor() as usize);
    let older_days = std::cmp::min(14, (num_dates as f64 * 0.3).floor() as usize);
    let older_start_index = std::cmp::max(
        recent_days + 7,
        num_dates - older_days - recent_days,
    );

    // Calculate recent average (TypeScript: indices 0 to recentDays = newest dates)
    // In Rust oldest-first data, newest dates are at the END (last recent_days elements)
    let mut recent_total = 0.0;
    for d in (num_dates - recent_days)..num_dates {
        let index = ticker_index * num_dates + d;
        recent_total += activity_percentages[index];
    }
    let recent_average = recent_total / recent_days as f64;

    // Calculate older average (TypeScript: indices olderStartIndex to olderStartIndex + olderDays)
    // In Rust oldest-first data, we need to map TypeScript indices to Rust indices
    // TypeScript olderStartIndex -> Rust: num_dates - 1 - olderStartIndex
    let rust_older_start = num_dates - 1 - older_start_index - older_days + 1;
    let mut older_total = 0.0;
    for d in rust_older_start..(rust_older_start + older_days) {
        if d < num_dates {
            let index = ticker_index * num_dates + d;
            older_total += activity_percentages[index];
        }
    }
    let older_average = if rust_older_start + older_days <= num_dates {
        older_total / older_days as f64
    } else {
        0.0
    };

    // Calculate trend factor exactly like TypeScript
    let trend_factor = if older_average > 0.0 {
        let factor = (recent_average - older_average) / older_average;
        factor.clamp(-1.0, 1.0) // TypeScript: Math.max(-1, Math.min(1, trendFactor))
    } else {
        0.0
    };

    // Calculate acceleration factor exactly like TypeScript
    let acceleration_factor = if recent_days >= 7 {
        let first_half_days = recent_days / 2; // TypeScript: Math.floor(recentDays / 2)
        let second_half_days = recent_days - first_half_days;

        // TypeScript: firstHalf = older dates within recent (indices firstHalfDays to recentDays)
        // In Rust oldest-first data: firstHalf = earlier part of recent period
        let mut first_half_total = 0.0;
        for d in (num_dates - recent_days)..(num_dates - second_half_days) {
            let index = ticker_index * num_dates + d;
            first_half_total += activity_percentages[index];
        }
        let first_half_avg = first_half_total / first_half_days as f64;

        // TypeScript: secondHalf = newer dates (indices 0 to secondHalfDays)
        // In Rust oldest-first data: secondHalf = last part of recent period
        let mut second_half_total = 0.0;
        for d in (num_dates - second_half_days)..num_dates {
            let index = ticker_index * num_dates + d;
            second_half_total += activity_percentages[index];
        }
        let second_half_avg = second_half_total / second_half_days as f64;

        if first_half_avg > 0.0 {
            let factor = (second_half_avg - first_half_avg) / first_half_avg;
            factor.clamp(-0.5, 0.5) * 0.3 // TypeScript: Math.max(-0.5, Math.min(0.5, accelerationFactor)) * 0.3
        } else {
            0.0
        }
    } else {
        0.0
    };

    // Final trend score formula exactly like TypeScript
    let period_gain = recent_average * (1.0 + trend_factor * 0.5 + acceleration_factor);
    let final_score = period_gain.max(0.0); // TypeScript: Math.max(0, periodGain)

    // Debug output for first few tickers
    static TICKER_DEBUG_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let count = TICKER_DEBUG_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    if count < 4 {
        let debug_time = chrono::Utc::now();
        tracing::info!(
            "[{}] 🔍 [MONEY_FLOW] MATRIX TREND DEBUG TICKER {}: recent_days={}, older_days={}, older_start={}, recent_avg={:.4}, older_avg={:.4}, trend_factor={:.4}, accel={:.4}, period_gain={:.4}, final={:.4}",
            debug_time.format("%Y-%m-%d %H:%M:%S UTC"),
            count,
            recent_days,
            older_days,
            older_start_index,
            recent_average,
            older_average,
            trend_factor,
            acceleration_factor,
            period_gain,
            final_score
        );
    }

    final_score
}

/// Sophisticated trend score calculation exactly matching TypeScript algorithm
#[allow(dead_code)]
fn calculate_sophisticated_trend_score(percentages: &[f64]) -> f64 {
    let num_dates = percentages.len();

    // Exact TypeScript period calculation logic
    let recent_days = std::cmp::min(14, (num_dates as f64 * 0.3).floor() as usize);
    let older_days = std::cmp::min(14, (num_dates as f64 * 0.3).floor() as usize);
    let older_start_index = std::cmp::max(
        recent_days + 7,
        num_dates - older_days - recent_days,
    );

    // FIXED: Calculate recent average - after reverse(), data is newest-first like TypeScript
    // So we take the FIRST recent_days elements (indices 0 to recent_days)
    let recent_total: f64 = percentages.iter().take(recent_days).sum();
    let recent_average = recent_total / recent_days as f64;

    // FIXED: Calculate older average - now that data is newest-first like TypeScript
    // TypeScript: olderStartIndex to olderStartIndex + olderDays
    let older_total: f64 = if older_start_index + older_days <= num_dates {
        percentages[older_start_index..older_start_index + older_days].iter().sum()
    } else {
        0.0
    };
    let older_average = if older_start_index + older_days <= num_dates {
        older_total / older_days as f64
    } else {
        0.0
    };

    // Calculate trend factor exactly like TypeScript
    let trend_factor = if older_average > 0.0 {
        let factor = (recent_average - older_average) / older_average;
        factor.clamp(-1.0, 1.0)
    } else {
        0.0
    };

    // FIXED: Calculate acceleration factor - now data is newest-first like TypeScript
    let acceleration_factor = if recent_days >= 7 {
        let first_half_days = recent_days / 2; // TypeScript: Math.floor(recentDays / 2)
        let second_half_days = recent_days - first_half_days;

        // TypeScript logic: firstHalf = older dates within recent period (indices firstHalfDays to recentDays)
        // Since we have newest-first data, firstHalf = elements at positions [first_half_days..recent_days]
        let first_half_total: f64 = percentages[first_half_days..recent_days].iter().sum();
        let first_half_avg = first_half_total / first_half_days as f64;

        // TypeScript logic: secondHalf = newer dates (indices 0 to secondHalfDays)
        // Since we have newest-first data, this matches exactly
        let second_half_total: f64 = percentages.iter().take(second_half_days).sum();
        let second_half_avg = second_half_total / second_half_days as f64;

        if first_half_avg > 0.0 {
            let factor = (second_half_avg - first_half_avg) / first_half_avg;
            factor.clamp(-0.5, 0.5) * 0.3 // TypeScript: clamp to [-0.5, 0.5] then * 0.3
        } else {
            0.0
        }
    } else {
        0.0
    };

    // Final trend score formula exactly like TypeScript
    let period_gain = recent_average * (1.0 + trend_factor * 0.5 + acceleration_factor);
    let final_score = period_gain.max(0.0); // TypeScript: Math.max(0, periodGain)

    // Debug output for trend score calculation (only for first few tickers)
    if num_dates > 100 {
        static TICKER_DEBUG_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let count = TICKER_DEBUG_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if count < 4 {
            let sample_recent: Vec<f64> = percentages.iter().take(recent_days).cloned().collect();
            let now = chrono::Utc::now();
            tracing::info!(
                "[{}] 🔍 [MATRIX] TREND DEBUG TICKER {}: len={}, recent_days={}, older_days={}, recent_avg={:.4}, older_avg={:.4}, trend_factor={:.4}, accel={:.4}, period_gain={:.4}, final={:.4}",
                now.format("%Y-%m-%d %H:%M:%S UTC"),
                count, num_dates, recent_days, older_days, recent_average, older_average, trend_factor, acceleration_factor, period_gain, final_score
            );
            tracing::info!(
                "[{}] 🔍 [MATRIX] Recent data sample (first {} elements): {:?}",
                now.format("%Y-%m-%d %H:%M:%S UTC"),
                recent_days.min(10), &sample_recent[..recent_days.min(10)]
            );
        }
    }

    final_score
}

/// Extract single date data from vectorized results
pub fn extract_single_date_data(
    matrix: &MoneyFlowMatrix,
    percentages: &[f64],
    target_date: &str,
) -> Vec<SingleDateResult> {
    let mut results = Vec::new();

    if let Some(&date_idx) = matrix.date_index.get(target_date) {
        let num_dates = matrix.dates.len();

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

            results.push(SingleDateResult {
                ticker: ticker.clone(),
                flow_percentage: percentages[index],
                signed_flow: matrix.flows[index], // Use activity flows for signed flow
                volume: matrix.volumes[index],
                debug_info,
            });
        }
    }

    results
}

/// Single date result structure
#[derive(Debug, Clone)]
pub struct SingleDateResult {
    pub ticker: String,
    pub flow_percentage: f64,
    pub signed_flow: f64,
    pub volume: f64,
    pub debug_info: Option<SingleDateDebugInfo>,
}

/// Single date debug information
#[derive(Debug, Clone)]
pub struct SingleDateDebugInfo {
    pub effective_low: f64,
    pub effective_high: f64,
    pub effective_range: f64,
    pub multiplier: f64,
    pub is_limit_move: bool,
    pub prev_close: f64,
    pub price_change_percent: f64,
}

/// Calculate MA scores for multiple periods (10, 20, 50) using vectorized operations
/// This is significantly faster than the nested loop approach
pub fn calculate_ma_score_matrix(ticker_matrix: &TickerDataMatrix) -> MAScoreMatrix {
    let (num_tickers, num_dates, _num_fields) = ticker_matrix.shape;
    let total_size = num_tickers * num_dates;

    // Pre-allocate arrays for vectorized operations
    let mut ma10_scores = vec![0.0; total_size];
    let mut ma10_values = vec![0.0; total_size];
    let mut ma20_scores = vec![0.0; total_size];
    let mut ma20_values = vec![0.0; total_size];
    let mut ma50_scores = vec![0.0; total_size];
    let mut ma50_values = vec![0.0; total_size];
    let mut closes = vec![0.0; total_size];

    // Extract close prices first (vectorized copy)
    for t in 0..num_tickers {
        for d in 0..num_dates {
            let base_index = t * num_dates * 5 + d * 5; // 5 = OHLCV fields
            let result_index = t * num_dates + d;

            closes[result_index] = ticker_matrix.data[base_index + 3]; // Close price
        }
    }

    // Calculate moving averages for each period
    calculate_moving_averages(&closes, num_tickers, num_dates, 10, &mut ma10_values);
    calculate_moving_averages(&closes, num_tickers, num_dates, 20, &mut ma20_values);
    calculate_moving_averages(&closes, num_tickers, num_dates, 50, &mut ma50_values);

    // Calculate MA scores: ((price - ma) / ma) * 100 (optimized sequential)
    for i in 0..total_size {
        let close_price = closes[i];

        // MA10 Score
        let ma10 = ma10_values[i];
        ma10_scores[i] = if ma10 > 0.0 && ma10.is_finite() {
            ((close_price - ma10) / ma10) * 100.0
        } else {
            0.0
        };

        // MA20 Score
        let ma20 = ma20_values[i];
        ma20_scores[i] = if ma20 > 0.0 && ma20.is_finite() {
            ((close_price - ma20) / ma20) * 100.0
        } else {
            0.0
        };

        // MA50 Score
        let ma50 = ma50_values[i];
        ma50_scores[i] = if ma50 > 0.0 && ma50.is_finite() {
            ((close_price - ma50) / ma50) * 100.0
        } else {
            0.0
        };
    }

    MAScoreMatrix {
        ma10_scores,
        ma10_values,
        ma20_scores,
        ma20_values,
        ma50_scores,
        ma50_values,
        closes,
        shape: (num_tickers, num_dates),
        ticker_index: ticker_matrix.ticker_index.clone(),
        date_index: ticker_matrix.date_index.clone(),
        tickers: ticker_matrix.tickers.clone(),
        dates: ticker_matrix.dates.clone(),
    }
}

/// Calculate moving averages for a specific period using vectorized operations
fn calculate_moving_averages(
    closes: &[f64],
    num_tickers: usize,
    num_dates: usize,
    period: usize,
    ma_values: &mut [f64],
) {
    for t in 0..num_tickers {
        for d in 0..num_dates {
            let result_index = t * num_dates + d;

            if d + 1 < period {
                // Not enough data for MA calculation
                ma_values[result_index] = 0.0;
                continue;
            }

            // Calculate MA for this ticker and date
            let start_idx = if d + 1 >= period { d + 1 - period } else { 0 };
            let mut sum = 0.0;
            let mut count = 0;

            for i in start_idx..=d {
                let price_index = t * num_dates + i;
                let close_price = closes[price_index];
                if close_price > 0.0 && close_price.is_finite() {
                    sum += close_price;
                    count += 1;
                }
            }

            ma_values[result_index] = if count >= period {
                sum / count as f64
            } else {
                0.0
            };
        }
    }
}

/// Extract MA Score data for a specific date (vectorized)
pub fn extract_ma_score_for_date(
    ma_score_matrix: &MAScoreMatrix,
    date_str: &str,
) -> Vec<SingleDateMAScoreResult> {
    let mut results = Vec::new();

    if let Some(&date_idx) = ma_score_matrix.date_index.get(date_str) {
        let (_num_tickers, num_dates) = ma_score_matrix.shape;

        for (ticker_idx, ticker) in ma_score_matrix.tickers.iter().enumerate() {
            let index = ticker_idx * num_dates + date_idx;

            results.push(SingleDateMAScoreResult {
                ticker: ticker.clone(),
                ma10_score: ma_score_matrix.ma10_scores[index],
                ma10_value: ma_score_matrix.ma10_values[index],
                ma20_score: ma_score_matrix.ma20_scores[index],
                ma20_value: ma_score_matrix.ma20_values[index],
                ma50_score: ma_score_matrix.ma50_scores[index],
                ma50_value: ma_score_matrix.ma50_values[index],
                close_price: ma_score_matrix.closes[index],
            });
        }
    }

    results
}

/// Extract MA values (ma10, ma20, ma50) for updating stock data
pub fn extract_ma_values(
    ticker_data: &HashMap<String, Vec<StockDataPoint>>,
    selected_tickers: &[String],
    date_range: &[String],
) -> HashMap<String, Vec<(String, Option<f64>, Option<f64>, Option<f64>)>> {
    // Vectorize the ticker data
    let ticker_matrix = vectorize_ticker_data(ticker_data, selected_tickers, date_range);

    // Calculate MA score matrix (includes MA values)
    let ma_score_matrix = calculate_ma_score_matrix(&ticker_matrix);

    let mut results = HashMap::new();

    for ticker in selected_tickers {
        if let Some(&ticker_idx) = ma_score_matrix.ticker_index.get(ticker) {
            let mut ticker_ma_values = Vec::new();

            for (date_idx, date) in date_range.iter().enumerate() {
                let matrix_index = ticker_idx * date_range.len() + date_idx;

                let ma10 = ma_score_matrix.ma10_values[matrix_index];
                let ma20 = ma_score_matrix.ma20_values[matrix_index];
                let ma50 = ma_score_matrix.ma50_values[matrix_index];

                ticker_ma_values.push((
                    date.clone(),
                    if ma10 > 0.0 && ma10.is_finite() { Some(ma10) } else { None },
                    if ma20 > 0.0 && ma20.is_finite() { Some(ma20) } else { None },
                    if ma50 > 0.0 && ma50.is_finite() { Some(ma50) } else { None },
                ));
            }

            results.insert(ticker.clone(), ticker_ma_values);
        }
    }

    results
}