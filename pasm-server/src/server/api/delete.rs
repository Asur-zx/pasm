use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension,
};

use crate::types::{db::Db, state::PasmState};

/// This function deletes specified entry in the database
/// if key doesnot exist return Error:404, `NOT_FOUND`
pub async fn call(
    Path(name): Path<String>,
    Extension(auth_key): Extension<String>,
    State(state): State<PasmState>,
) -> impl IntoResponse {
    let db = &state.db;

    let user_id = match db.get_user_id_by_authkey(&auth_key).await {
        Ok(id) => id,
        Err(err) => return err.into_response(),
    };

    if let Err(err) = db.remove_entry(&user_id, &name).await {
        return err.into_response();
    }

    (StatusCode::OK, "Entry deleted").into_response()
}
