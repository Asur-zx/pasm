use axum::{extract::State, response::IntoResponse, Extension, Json};

use crate::types::{db::Db, state::PasmState};
use pasm_core::types::entry::RequestData;

/// Updates another user's authentication key.
///
/// This admin-only endpoint replaces the existing auth key with a new one
/// provided in the payload.
pub async fn call(
    State(state): State<PasmState>,
    Extension(auth_key): Extension<String>,
    Json(payload): Json<RequestData>,
) -> impl IntoResponse {
    let db = &state.db;
    let new_auth = &payload.value;

    println!("updated user!");
    db.update_auth(&auth_key, new_auth).await.into_response()
}
