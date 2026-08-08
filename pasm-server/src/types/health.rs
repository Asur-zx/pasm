use serde::Serialize;

/// Response body returned by `GET /health`.
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
    pub uptime_seconds: u64,
    pub database: &'static str,
    pub timestamp: u64,
}
