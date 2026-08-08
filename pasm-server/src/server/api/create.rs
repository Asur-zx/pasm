use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};

use crate::types::{db::Db, state::PasmState};
use pasm_core::types::entry::RequestData;

/// This function creates new entry in the database
/// if key already exists returns Error:409, `CONFLICT`
pub async fn call(
    State(state): State<PasmState>,
    Extension(auth_key): Extension<String>,
    Json(payload): Json<RequestData>,
) -> impl IntoResponse {
    let db = &state.db;

    let user_id = match db.get_user_id_by_authkey(&auth_key).await {
        Ok(id) => id,
        Err(err) => return err.into_response(),
    };

    if let Err(err) = db.add_entry(&user_id, &payload.key, &payload.value).await {
        return err.into_response();
    }

    (StatusCode::CREATED, "Entry added").into_response()
}
