use std::time::Instant;

use redis::aio::ConnectionManager;

use crate::types::db::PgDb;

/// Auth info inserted into request extensions by the auth middleware.
/// Allows downstream middleware (e.g. admin_auth) to read roles
/// without an extra DB query.
#[derive(Clone, Debug)]
pub struct AuthInfo {
    pub token: String,
    pub roles: Vec<String>,
}

/// Axum application state holding the database handle and server metadata.
#[derive(Clone)]
pub struct PasmState {
    pub db: PgDb,
    pub started_at: Instant,
    pub redis: ConnectionManager,
}
