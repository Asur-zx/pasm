use axum::{extract::State, response::IntoResponse, Extension, Json};

use crate::types::{db::Db, state::PasmState};

/// This function finds all entries in the database and returns the Vector of Json contents
pub async fn call(
    Extension(auth_key): Extension<String>,
    State(state): State<PasmState>,
) -> impl IntoResponse {
    let db = &state.db;

    let user_id = match db.get_user_id_by_authkey(&auth_key).await {
        Ok(user_id) => user_id,
        Err(err) => return err.into_response(),
    };

    let result = match db.list_entries(&user_id).await {
        Ok(entries) => entries,
        Err(err) => return err.into_response(),
    };

    Json(result).into_response()
}
