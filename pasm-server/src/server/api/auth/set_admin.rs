use axum::{extract::State, response::IntoResponse, Json};

use crate::types::{db::Db, state::PasmState};
use pasm_core::types::entry::RequestData;

/// Promotes a user to admin by setting their roles to `['user', 'admin']`.
///
/// The caller must already be authenticated as admin (enforced by the
/// `admin_auth` middleware). The target user is identified by their
/// auth key in the `value` field of the JSON payload.
pub async fn call(
    State(state): State<PasmState>,
    Json(payload): Json<RequestData>,
) -> impl IntoResponse {
    let db = &state.db;
    db.set_admin_role(&payload.value).await.into_response()
}
