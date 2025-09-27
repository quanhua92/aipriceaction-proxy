use crate::config::SharedTokenConfig;
use crate::data_structures::{LastInternalUpdate, SharedData, SharedReputation, SharedTickerGroups, SharedHealthStats, SharedEnhancedData, EnhancedTickerResponse, TickerResponseMeta};
use crate::vci::OhlcvData;
use axum::{
    extract::{ConnectInfo, State, Json},
    http::{HeaderMap, StatusCode, header::CACHE_CONTROL},
    response::{IntoResponse, Response},
};
use axum_extra::extract::Query;
use serde::Deserialize;
use std::net::SocketAddr;
use std::time::Duration;
use tracing::{info, debug, warn, error};

// Define the struct to hold the query parameters.
// `symbol` will hold all values passed for the "symbol" key.
#[derive(Debug, Deserialize)]
pub struct TickerParams {
    symbol: Option<Vec<String>>,
    start_date: Option<String>,
    end_date: Option<String>,
    all: Option<bool>,
    format: Option<String>,  // "json" or "csv"
}

pub async fn get_all_tickers_handler(
    State(state): State<SharedData>,
    State(enhanced_state): State<SharedEnhancedData>,
    Query(params): Query<TickerParams>
) -> impl IntoResponse {
    debug!("Received request for tickers with params: {:?}", params);

    let format = params.format.as_deref().unwrap_or("json");

    // Try to get enhanced data first
    let enhanced_data = enhanced_state.lock().await;

    if !enhanced_data.is_empty() {
        // Use enhanced data with calculations
        let filtered_data = filter_enhanced_data(&enhanced_data, &params);

        let symbol_count = filtered_data.len();
        let symbols: Vec<_> = filtered_data.keys().cloned().collect();
        let total_data_points: usize = filtered_data.values().map(|v| v.len()).sum();

        info!(symbol_count, symbols = ?symbols, total_data_points, format, "Returning enhanced ticker data with calculations");

        match format {
            "csv" => {
                // Return CSV format
                let csv_content = format_enhanced_data_as_csv(filtered_data);
                let mut headers = HeaderMap::new();
                headers.insert("content-type", "text/csv".parse().unwrap());
                headers.insert(CACHE_CONTROL, "max-age=30".parse().unwrap());
                (StatusCode::OK, headers, csv_content).into_response()
            }
            _ => {
                // Return JSON format with metadata
                let response = EnhancedTickerResponse {
                    meta: TickerResponseMeta {
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        calculated: true,
                        error: None,
                    },
                    data: filtered_data,
                };

                let mut headers = HeaderMap::new();
                headers.insert(CACHE_CONTROL, "max-age=30".parse().unwrap());
                (StatusCode::OK, headers, Json(response)).into_response()
            }
        }
    } else {
        // Fallback to regular OHLCV data if enhanced data is not available
        info!("Enhanced data not available, falling back to regular OHLCV data");

        let data = state.lock().await;
        let filtered_data = filter_ohlcv_data(&data, &params);

        let symbol_count = filtered_data.len();
        let symbols: Vec<_> = filtered_data.keys().cloned().collect();
        let total_data_points: usize = filtered_data.values().map(|v| v.len()).sum();

        info!(symbol_count, symbols = ?symbols, total_data_points, format, "Returning fallback OHLCV data");

        match format {
            "csv" => {
                // Return CSV format for OHLCV data
                let csv_content = format_ohlcv_data_as_csv(filtered_data);
                let mut headers = HeaderMap::new();
                headers.insert("content-type", "text/csv".parse().unwrap());
                headers.insert(CACHE_CONTROL, "max-age=30".parse().unwrap());
                (StatusCode::OK, headers, csv_content).into_response()
            }
            _ => {
                // Return JSON format
                let mut headers = HeaderMap::new();
                headers.insert(CACHE_CONTROL, "max-age=30".parse().unwrap());
                (StatusCode::OK, headers, Json(filtered_data)).into_response()
            }
        }
    }
}

pub async fn internal_gossip_handler(
    State(data_state): State<SharedData>,
    State(token_state): State<SharedTokenConfig>,
    State(last_update_state): State<LastInternalUpdate>,
    headers: HeaderMap,
    Json(payload): Json<OhlcvData>,
) -> impl IntoResponse {
    debug!("Received internal gossip request");
    
    let auth_header = headers.get("Authorization").and_then(|h| h.to_str().ok());
    let token_is_valid = match auth_header {
        Some(header) => {
            let token = header.trim_start_matches("Bearer ");
            let is_primary = token == token_state.primary;
            let is_secondary = token == token_state.secondary;
            debug!(has_auth_header = true, is_primary, is_secondary, "Token validation");
            is_primary || is_secondary
        }
        None => {
            debug!(has_auth_header = false, "No authorization header provided");
            false
        }
    };

    if !token_is_valid {
        warn!("Unauthorized internal gossip attempt");
        return (StatusCode::UNAUTHORIZED, "Invalid or missing token").into_response();
    }

    *last_update_state.lock().await = std::time::Instant::now();
    debug!("Updated last internal update timestamp");

    let mut data_guard = data_state.lock().await;
    if let Some(symbol) = &payload.symbol {
        let entry = data_guard.entry(symbol.clone()).or_default();
        let should_update = entry.last().map_or(true, |last| payload.time > last.time);
        
        if should_update {
            entry.push(payload.clone());
            entry.sort_by_key(|d| d.time);
            info!(symbol, close_price = payload.close, volume = payload.volume, "Updated symbol data from internal gossip");
        } else {
            debug!(symbol, "Received older data, skipping update");
        }
    } else {
        warn!("Received gossip payload without symbol");
    }

    (StatusCode::OK, "OK").into_response()
}

pub async fn public_gossip_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(data_state): State<SharedData>,
    State(reputation_state): State<SharedReputation>,
    State(last_update_state): State<LastInternalUpdate>,
    Json(payload): Json<OhlcvData>,
) -> Response {
    let source_ip = addr.ip();
    debug!("Received public gossip request");
    
    let mut reputation_guard = reputation_state.lock().await;
    let actor = reputation_guard.entry(source_ip).or_default();

    debug!(
        successful_updates = actor.successful_updates,
        failed_updates = actor.failed_updates,
        status = ?actor.status,
        "Actor reputation status"
    );

    if actor.status == crate::data_structures::ActorStatus::Banned {
        warn!("Rejected request from banned IP");
        return (StatusCode::FORBIDDEN, "Source IP is banned").into_response();
    }

    let last_internal_update = last_update_state.lock().await;
    let time_since_update = last_internal_update.elapsed();
    debug!(time_since_last_internal_update = ?time_since_update, "Checking system trust status");
    
    if time_since_update > Duration::from_secs(300) {
        warn!(time_since_update = ?time_since_update, "System running on untrusted data too long");
        return (StatusCode::SERVICE_UNAVAILABLE, "System is running on untrusted data").into_response();
    }

    let mut data_guard = data_state.lock().await;
    if let Some(symbol) = &payload.symbol {
        if let Some(entry) = data_guard.get(symbol.as_str()) {
            if let Some(last_data) = entry.last() {
                let price_change_percent = (payload.close - last_data.close).abs() / last_data.close;
                debug!(
                    symbol,
                    old_price = last_data.close,
                    new_price = payload.close,
                    price_change_percent,
                    "Price change validation"
                );
                
                if price_change_percent > 0.10 {
                    actor.failed_updates += 1;
                    warn!(
                        symbol,
                        price_change_percent,
                        failed_updates = actor.failed_updates,
                        "Implausible price change detected"
                    );
                    
                    if actor.failed_updates > 5 {
                        actor.status = crate::data_structures::ActorStatus::Banned;
                        error!("Banning IP due to repeated implausible data");
                    }
                    return (StatusCode::BAD_REQUEST, "Implausible price change").into_response();
                }
            }
        }
        
        actor.successful_updates += 1;
        let entry = data_guard.entry(symbol.clone()).or_default();
        entry.push(payload.clone());
        entry.sort_by_key(|d| d.time);
        
        info!(
            symbol,
            close_price = payload.close,
            volume = payload.volume,
            successful_updates = actor.successful_updates,
            "Accepted public gossip data"
        );
    } else {
        warn!("Received public gossip payload without symbol");
        actor.failed_updates += 1;
    }

    (StatusCode::OK, "OK").into_response()
}

pub async fn get_ticker_groups_handler(State(state): State<SharedTickerGroups>) -> impl IntoResponse {
    debug!("Received request for ticker groups");
    
    let group_count = state.0.len();
    let group_names: Vec<_> = state.0.keys().cloned().collect();
    
    info!(group_count, groups = ?group_names, "Returning ticker groups");
    (StatusCode::OK, Json(state.0.clone()))
}

pub async fn health_handler(
    State(health_state): State<SharedHealthStats>,
    State(data_state): State<SharedData>,
) -> impl IntoResponse {
    debug!("Received request for health stats");
    
    let mut health_stats = health_state.lock().await.clone();
    
    // Calculate current memory usage dynamically
    {
        let data_guard = data_state.lock().await;
        let memory_bytes = crate::data_structures::estimate_memory_usage(&*data_guard);
        let memory_mb = memory_bytes as f64 / (1024.0 * 1024.0);
        let memory_percent = (memory_bytes as f64 / crate::data_structures::MAX_MEMORY_BYTES as f64) * 100.0;
        
        health_stats.memory_usage_bytes = memory_bytes;
        health_stats.memory_usage_mb = memory_mb;
        health_stats.memory_usage_percent = memory_percent;
        health_stats.active_tickers_count = data_guard.len();
    }
    
    info!(
        is_office_hours = health_stats.is_office_hours,
        current_interval_secs = health_stats.current_interval_secs,
        active_tickers = health_stats.active_tickers_count,
        total_tickers = health_stats.total_tickers_count,
        memory_mb = format!("{:.2}", health_stats.memory_usage_mb),
        memory_percent = format!("{:.1}%", health_stats.memory_usage_percent),
        iteration_count = health_stats.iteration_count,
        "Returning health stats"
    );
    
    (StatusCode::OK, Json(health_stats))
}

fn filter_enhanced_data(
    enhanced_data: &crate::data_structures::EnhancedInMemoryData,
    params: &TickerParams,
) -> crate::data_structures::EnhancedInMemoryData {
    use std::collections::HashMap;

    let mut filtered_data = HashMap::new();

    // Filter by symbols first
    let target_symbols: Option<std::collections::HashSet<String>> = params.symbol
        .as_ref()
        .map(|symbols| symbols.iter().cloned().collect());

    for (symbol, ticker_data) in enhanced_data.iter() {
        // Skip if specific symbols are requested and this symbol is not in the list
        if let Some(ref target_set) = target_symbols {
            if !target_set.contains(symbol) {
                continue;
            }
        }

        let mut filtered_points = ticker_data.clone();

        // Apply date filtering
        if let Some(ref start_date_str) = params.start_date {
            filtered_points.retain(|point| point.date >= *start_date_str);
        }

        if let Some(ref end_date_str) = params.end_date {
            filtered_points.retain(|point| point.date <= *end_date_str);
        }

        // If no date filters and all=false, return only latest point
        if params.start_date.is_none() && params.end_date.is_none() && !params.all.unwrap_or(false) {
            if let Some(latest_point) = filtered_points.last() {
                filtered_points = vec![latest_point.clone()];
            }
        }

        if !filtered_points.is_empty() {
            filtered_data.insert(symbol.clone(), filtered_points);
        }
    }

    filtered_data
}

fn filter_ohlcv_data(
    ohlcv_data: &crate::data_structures::InMemoryData,
    params: &TickerParams,
) -> crate::data_structures::InMemoryData {
    use chrono::NaiveDate;
    use std::collections::HashMap;

    // Parse date filters
    let start_date_filter = params.start_date.as_ref().and_then(|date_str| {
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .ok()
            .map(|date| date.and_hms_opt(0, 0, 0).unwrap().and_utc())
    });

    let end_date_filter = params.end_date.as_ref().and_then(|date_str| {
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .ok()
            .map(|date| date.and_hms_opt(23, 59, 59).unwrap().and_utc())
    });

    let use_last_day_only = start_date_filter.is_none() && end_date_filter.is_none() && !params.all.unwrap_or(false);

    // Filter data by symbols first
    let symbol_filtered_data = match &params.symbol {
        Some(symbols) if !symbols.is_empty() => {
            let mut filtered = HashMap::new();
            for symbol in symbols {
                if let Some(ticker_data) = ohlcv_data.get(symbol) {
                    filtered.insert(symbol.clone(), ticker_data.clone());
                }
            }
            filtered
        }
        _ => ohlcv_data.clone(),
    };

    // Apply date filtering
    let mut date_filtered_data = HashMap::new();
    for (symbol, ticker_data) in symbol_filtered_data {
        let filtered_data: Vec<_> = if use_last_day_only {
            ticker_data.into_iter().rev().take(1).collect()
        } else {
            ticker_data.into_iter()
                .filter(|ohlcv| {
                    let time_matches_start = start_date_filter.map_or(true, |start| ohlcv.time >= start);
                    let time_matches_end = end_date_filter.map_or(true, |end| ohlcv.time <= end);
                    time_matches_start && time_matches_end
                })
                .collect()
        };

        if !filtered_data.is_empty() {
            date_filtered_data.insert(symbol, filtered_data);
        }
    }

    date_filtered_data
}

fn format_enhanced_data_as_csv(data: crate::data_structures::EnhancedInMemoryData) -> String {
    use std::io::Write;

    let mut csv_content = Vec::new();

    // Write header
    writeln!(
        csv_content,
        "date,symbol,open,high,low,close,volume,ma10,ma20,ma50,moneyFlow,af,df,ts,score10,score20,score50"
    ).unwrap();

    // Write data rows
    for (symbol, ticker_data) in data {
        for point in ticker_data {
            writeln!(
                csv_content,
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                point.date,
                symbol,
                point.open,
                point.high,
                point.low,
                point.close,
                point.volume,
                format_optional_f64(point.ma10),
                format_optional_f64(point.ma20),
                format_optional_f64(point.ma50),
                format_optional_f64(point.money_flow),
                format_optional_f64(point.af),
                format_optional_f64(point.df),
                format_optional_f64(point.ts),
                format_optional_f64(point.score10),
                format_optional_f64(point.score20),
                format_optional_f64(point.score50),
            ).unwrap();
        }
    }

    String::from_utf8(csv_content).unwrap()
}

fn format_ohlcv_data_as_csv(data: crate::data_structures::InMemoryData) -> String {
    use std::io::Write;

    let mut csv_content = Vec::new();

    // Write header for OHLCV data (without enhanced calculations)
    writeln!(
        csv_content,
        "date,symbol,open,high,low,close,volume"
    ).unwrap();

    // Write data rows
    for (symbol, ohlcv_data) in data {
        for ohlcv in ohlcv_data {
            writeln!(
                csv_content,
                "{},{},{},{},{},{},{}",
                ohlcv.time,
                symbol,
                ohlcv.open,
                ohlcv.high,
                ohlcv.low,
                ohlcv.close,
                ohlcv.volume,
            ).unwrap();
        }
    }

    String::from_utf8(csv_content).unwrap()
}

fn format_optional_f64(value: Option<f64>) -> String {
    match value {
        Some(v) => v.to_string(),
        None => String::new(),
    }
}