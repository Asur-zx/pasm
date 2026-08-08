use axum::{extract::State, response::IntoResponse, Extension};

use crate::types::{db::Db, state::PasmState};

/// Removes another user and all their data from the database.
///
/// This admin-only endpoint deletes the user's authentication key and their
/// encrypted password entries.
pub async fn call(
    Extension(auth_key): Extension<String>,
    State(state): State<PasmState>,
) -> impl IntoResponse {
    let db = &state.db;

    println!("removed user!");
    db.remove_user(&auth_key).await.into_response()
}
