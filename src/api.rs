use crate::config::SharedTokenConfig;
use crate::data_structures::{LastInternalUpdate, SharedData, SharedReputation, SharedTickerGroups, SharedHealthStats};
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
use tracing::{info, debug, warn, error, instrument};
use chrono::NaiveDate;

// Define the struct to hold the query parameters.
// `symbol` will hold all values passed for the "symbol" key.
#[derive(Debug, Deserialize)]
pub struct TickerParams {
    symbol: Option<Vec<String>>,
    start_date: Option<String>,
    end_date: Option<String>,
    all: Option<bool>,
}

#[instrument(skip(state))]
pub async fn get_all_tickers_handler(
    State(state): State<SharedData>,
    Query(params): Query<TickerParams>
) -> impl IntoResponse {
    debug!("Received request for tickers with params: {:?}", params);
    
    let data = state.lock().await;
    
    // Parse date filters
    let start_date_filter = match &params.start_date {
        Some(date_str) => {
            match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                Ok(date) => Some(date.and_hms_opt(0, 0, 0).unwrap().and_utc()),
                Err(_) => {
                    warn!(start_date = %date_str, "Invalid start_date format, expected YYYY-MM-DD");
                    return (StatusCode::BAD_REQUEST, Json("Invalid start_date format. Expected YYYY-MM-DD")).into_response();
                }
            }
        }
        None => None,
    };

    let end_date_filter = match &params.end_date {
        Some(date_str) => {
            match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                Ok(date) => Some(date.and_hms_opt(23, 59, 59).unwrap().and_utc()),
                Err(_) => {
                    warn!(end_date = %date_str, "Invalid end_date format, expected YYYY-MM-DD");
                    return (StatusCode::BAD_REQUEST, Json("Invalid end_date format. Expected YYYY-MM-DD")).into_response();
                }
            }
        }
        None => None,
    };

    // If no date filters provided and all=true is not set, default to last day only
    let use_last_day_only = start_date_filter.is_none() && end_date_filter.is_none() && !params.all.unwrap_or(false);
    
    // Filter data by symbols first
    let symbol_filtered_data = match params.symbol {
        Some(symbols) if !symbols.is_empty() => {
            // Filter data to only include requested symbols
            let mut filtered = std::collections::HashMap::new();
            for symbol in symbols {
                if let Some(ticker_data) = data.get(&symbol) {
                    filtered.insert(symbol, ticker_data.clone());
                }
            }
            filtered
        }
        _ => {
            // Return all data if no symbols specified or empty vector
            data.clone()
        }
    };

    // Apply date filtering
    let mut date_filtered_data = std::collections::HashMap::new();
    for (symbol, ticker_data) in symbol_filtered_data {
        let filtered_data: Vec<_> = if use_last_day_only {
            // Return only the most recent data point
            ticker_data.into_iter().rev().take(1).collect()
        } else {
            // Filter by date range
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
    
    let symbol_count = date_filtered_data.len();
    let symbols: Vec<_> = date_filtered_data.keys().cloned().collect();
    let total_data_points: usize = date_filtered_data.values().map(|v| v.len()).sum();
    
    if use_last_day_only {
        info!(symbol_count, symbols = ?symbols, total_data_points, "Returning ticker data (last day only)");
    } else if params.all.unwrap_or(false) && start_date_filter.is_none() && end_date_filter.is_none() {
        info!(symbol_count, symbols = ?symbols, total_data_points, "Returning all ticker data (all=true)");
    } else {
        info!(symbol_count, symbols = ?symbols, total_data_points, start_date = ?params.start_date, end_date = ?params.end_date, "Returning ticker data with date filters");
    }
    
    let mut headers = HeaderMap::new();
    headers.insert(CACHE_CONTROL, "max-age=30".parse().unwrap());
    (StatusCode::OK, headers, Json(date_filtered_data)).into_response()
}

#[instrument(skip(data_state, token_state, last_update_state, headers), fields(symbol = %payload.symbol.as_deref().unwrap_or("unknown")))]
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

#[instrument(skip(data_state, reputation_state, last_update_state), fields(source_ip = %addr.ip(), symbol = %payload.symbol.as_deref().unwrap_or("unknown")))]
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

#[instrument(skip(state))]
pub async fn get_ticker_groups_handler(State(state): State<SharedTickerGroups>) -> impl IntoResponse {
    debug!("Received request for ticker groups");
    
    let group_count = state.0.len();
    let group_names: Vec<_> = state.0.keys().cloned().collect();
    
    info!(group_count, groups = ?group_names, "Returning ticker groups");
    (StatusCode::OK, Json(state.0.clone()))
}

#[instrument(skip(health_state, data_state))]
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