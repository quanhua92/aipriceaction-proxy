use crate::config::SharedTokenConfig;
use crate::data_structures::{LastInternalUpdate, SharedData, SharedReputation};
use crate::vci::OhlcvData;
use axum::{
    extract::{ConnectInfo, State, Json},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use std::net::SocketAddr;
use std::time::Duration;

pub async fn get_all_tickers_handler(State(state): State<SharedData>) -> impl IntoResponse {
    let data = state.lock().await;
    (StatusCode::OK, Json(data.clone()))
}

pub async fn internal_gossip_handler(
    State(data_state): State<SharedData>,
    State(token_state): State<SharedTokenConfig>,
    State(last_update_state): State<LastInternalUpdate>,
    headers: HeaderMap,
    Json(payload): Json<OhlcvData>,
) -> impl IntoResponse {
    let auth_header = headers.get("Authorization").and_then(|h| h.to_str().ok());
    let token_is_valid = match auth_header {
        Some(header) => {
            let token = header.trim_start_matches("Bearer ");
            token == token_state.primary || token == token_state.secondary
        }
        None => false,
    };

    if !token_is_valid {
        return (StatusCode::UNAUTHORIZED, "Invalid or missing token").into_response();
    }

    *last_update_state.lock().await = std::time::Instant::now();

    let mut data_guard = data_state.lock().await;
    if let Some(symbol) = &payload.symbol {
        let entry = data_guard.entry(symbol.clone()).or_default();
        if entry.last().map_or(true, |last| payload.time > last.time) {
            entry.push(payload);
            entry.sort_by_key(|d| d.time);
        }
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
    let mut reputation_guard = reputation_state.lock().await;
    let actor = reputation_guard.entry(source_ip).or_default();

    if actor.status == crate::data_structures::ActorStatus::Banned {
        return (StatusCode::FORBIDDEN, "Source IP is banned").into_response();
    }

    let last_internal_update = last_update_state.lock().await;
    if last_internal_update.elapsed() > Duration::from_secs(300) {
        return (StatusCode::SERVICE_UNAVAILABLE, "System is running on untrusted data").into_response();
    }

    let mut data_guard = data_state.lock().await;
    if let Some(symbol) = &payload.symbol {
        if let Some(entry) = data_guard.get(symbol.as_str()) {
            if let Some(last_data) = entry.last() {
                let price_change_percent = (payload.close - last_data.close).abs() / last_data.close;
                if price_change_percent > 0.10 {
                    actor.failed_updates += 1;
                    if actor.failed_updates > 5 {
                        actor.status = crate::data_structures::ActorStatus::Banned;
                    }
                    return (StatusCode::BAD_REQUEST, "Implausible price change").into_response();
                }
            }
        }
        
        actor.successful_updates += 1;
        let entry = data_guard.entry(symbol.clone()).or_default();
        entry.push(payload);
        entry.sort_by_key(|d| d.time);
    }

    (StatusCode::OK, "OK").into_response()
}