use axum::{
    extract::State,
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::types::{db::Db, state::PasmState};

const SESSION_TTL: usize = 86400;
const REFRESH_TTL: usize = 604800;

#[derive(Serialize)]
pub struct SessionResponse {
    pub session_token: String,
    pub refresh_token: String,
    pub expires_in: usize,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// Creates a new session from an API key.
///
/// The API key is sent as a Bearer token. On success, returns a
/// session token (24h TTL) and a refresh token (7d TTL).
pub async fn create(
    State(state): State<PasmState>,
    headers: HeaderMap,
) -> Result<Json<SessionResponse>, (StatusCode, &'static str)> {
    let api_key = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "authorization failed — missing or malformed Bearer token",
        ))?;

    let _roles = state
        .db
        .get_user_roles(api_key)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "database error"))?
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "authorization failed — invalid auth key",
        ))?;

    let mut rcon = state.redis.clone();
    let session_token = uuid::Uuid::new_v4().to_string();
    let refresh_token = uuid::Uuid::new_v4().to_string();

    redis::cmd("SET")
        .arg(format!("session:{session_token}"))
        .arg(api_key)
        .arg("EX")
        .arg(SESSION_TTL)
        .query_async::<()>(&mut rcon)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "session creation failed"))?;

    redis::cmd("SET")
        .arg(format!("refresh:{refresh_token}"))
        .arg(api_key)
        .arg("EX")
        .arg(REFRESH_TTL)
        .query_async::<()>(&mut rcon)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "session creation failed"))?;

    Ok(Json(SessionResponse {
        session_token,
        refresh_token,
        expires_in: SESSION_TTL,
    }))
}

/// Exchanges a refresh token for a new session token.
pub async fn refresh(
    State(state): State<PasmState>,
    Json(body): Json<RefreshRequest>,
) -> Result<Json<SessionResponse>, (StatusCode, &'static str)> {
    let mut rcon = state.redis.clone();

    let api_key: Option<String> = redis::cmd("GET")
        .arg(format!("refresh:{}", body.refresh_token))
        .query_async(&mut rcon)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "redis error"))?;

    let api_key = api_key.ok_or((StatusCode::UNAUTHORIZED, "refresh token expired or invalid"))?;

    // Invalidate old refresh token (one-time use)
    redis::cmd("DEL")
        .arg(format!("refresh:{}", body.refresh_token))
        .query_async::<()>(&mut rcon)
        .await
        .ok();

    let session_token = uuid::Uuid::new_v4().to_string();
    let new_refresh_token = uuid::Uuid::new_v4().to_string();

    redis::cmd("SET")
        .arg(format!("session:{session_token}"))
        .arg(&api_key)
        .arg("EX")
        .arg(SESSION_TTL)
        .query_async::<()>(&mut rcon)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "session creation failed"))?;

    redis::cmd("SET")
        .arg(format!("refresh:{new_refresh_token}"))
        .arg(api_key)
        .arg("EX")
        .arg(REFRESH_TTL)
        .query_async::<()>(&mut rcon)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "session creation failed"))?;

    Ok(Json(SessionResponse {
        session_token,
        refresh_token: new_refresh_token,
        expires_in: SESSION_TTL,
    }))
}
