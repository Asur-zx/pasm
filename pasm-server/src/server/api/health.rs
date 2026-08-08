use std::time::{SystemTime, UNIX_EPOCH};

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use crate::types::{db::Db, health::HealthResponse, state::PasmState};

pub async fn call(State(state): State<PasmState>) -> impl IntoResponse {
    let db_status = match state.db.ping().await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    let uptime = state.started_at.elapsed().as_secs();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let health = HealthResponse {
        status: if db_status == "connected" {
            "ok"
        } else {
            "degraded"
        },
        version: env!("CARGO_PKG_VERSION"),
        uptime_seconds: uptime,
        database: db_status,
        timestamp,
    };

    let status_code = if health.status == "ok" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status_code, Json(health)).into_response()
}
