use crate::config::SharedTokenConfig;
use crate::data_structures::{LastInternalUpdate, SharedData, SharedReputation, SharedTickerGroups};
use crate::vci::OhlcvData;
use axum::{
    extract::{ConnectInfo, State, Json},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use std::net::SocketAddr;
use std::time::Duration;
use tracing::{info, debug, warn, error, instrument};

#[instrument(skip(state))]
pub async fn get_all_tickers_handler(State(state): State<SharedData>) -> impl IntoResponse {
    debug!("Received request for all tickers");
    
    let data = state.lock().await;
    let symbol_count = data.len();
    let symbols: Vec<_> = data.keys().cloned().collect();
    
    info!(symbol_count, symbols = ?symbols, "Returning ticker data");
    (StatusCode::OK, Json(data.clone()))
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