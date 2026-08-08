use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::types::state::{AuthInfo, PasmState};

/// Admin authorisation middleware — must run **after** `auth::call`.
///
/// Reads the `AuthInfo` extension injected by the auth middleware and
/// returns `403 Forbidden` if the user does not have the `"admin"` role.
pub async fn call(State(_state): State<PasmState>, req: Request<Body>, next: Next) -> Response {
    let is_admin = req
        .extensions()
        .get::<AuthInfo>()
        .map(|i| i.roles.contains(&"admin".to_string()))
        .unwrap_or(false);

    if is_admin {
        next.run(req).await
    } else {
        (
            StatusCode::FORBIDDEN,
            "admin access required — current user does not have the admin role",
        )
            .into_response()
    }
}
