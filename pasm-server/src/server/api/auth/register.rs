use axum::{
    body::Body,
    extract::State,
    http::{header::AUTHORIZATION, Request, StatusCode},
    response::IntoResponse,
};

use crate::types::{db::Db, state::PasmState};

/// Registers a new user with the Bearer token supplied in the request.
///
/// PostgreSQL generates the user ID and associates it with the supplied
/// authentication key.
pub async fn call(
    State(state): State<PasmState>,
    req: Request<Body>, // mut so we can add extensions
) -> impl IntoResponse {
    let db = &state.db;
    let token = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    let Some(uid) = token else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    println!("registered user!");
    db.register_auth(uid).await.into_response()
}
