//! Configuration sources (priority: high → low):
//!   1. CLI flags (set via `set_*` functions before first `*()` call)
//!   2. Environment variables (`PASM_*`)
//!   3. Config file (`~/.config/pasm/config.toml`)
//!   4. Hardcoded defaults

use std::path::PathBuf;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

const DEFAULT_SERVER_URL: &str = "http://localhost:3000";
const DEFAULT_SERVER_ADDR: &str = "0.0.0.0:3000";
const DEFAULT_MAX_CONNECTIONS: u32 = 5;
const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1:6379";

/// Mirrors the on-disk `config.toml` structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TomlConfig {
    pub server_url: Option<String>,
    pub server_addr: Option<String>,
    pub database_url: Option<String>,
    pub redis_url: Option<String>,
    pub max_connections: Option<u32>,
}

// ── config file path ──────────────────────────────────────────

/// Returns the resolved config file path.
///
/// Priority: `PASM_CONFIG` env var → `$HOME/.config/pasm/config.toml`.
pub fn config_path() -> PathBuf {
    if let Ok(p) = std::env::var("PASM_CONFIG") {
        return PathBuf::from(p);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".config/pasm/config.toml")
}

/// Overrides the config file path (used by `--config` flag).
pub fn set_config_path(path: &str) {
    std::env::set_var("PASM_CONFIG", path);
}

// ── config file I/O ──────────────────────────────────────────

/// Reads the config file from the standard path, returning `None` when
/// the file does not exist or cannot be parsed.
pub fn load_toml() -> Option<TomlConfig> {
    let path = config_path();
    let content = std::fs::read_to_string(path).ok()?;
    toml::from_str(&content).ok()
}

/// Writes a `TomlConfig` to the standard config path, creating parent
/// directories as needed.
pub fn write_toml(config: &TomlConfig) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("failed to create config dir: {e}"))?;
    }
    let data =
        toml::to_string_pretty(config).map_err(|e| format!("failed to serialize config: {e}"))?;
    std::fs::write(&path, data).map_err(|e| format!("failed to write config: {e}"))?;
    Ok(())
}

// ── server URL (client) ──────────────────────────────────────

/// Returns the server URL the client should connect to.
///
/// Priority: `--addr` / `set_server_url` → `PASM_SERVER_URL` env →
/// config file → `http://localhost:3000`.
pub fn server_url() -> &'static str {
    static URL: OnceLock<String> = OnceLock::new();
    URL.get_or_init(|| {
        if let Ok(url) = std::env::var("PASM_SERVER_URL") {
            return url;
        }
        if let Some(cfg) = load_toml() {
            if let Some(url) = cfg.server_url {
                return url;
            }
        }
        DEFAULT_SERVER_URL.to_string()
    })
}

/// Overrides the server URL by setting the `PASM_SERVER_URL` env var.
/// Must be called before the first call to [`server_url`].
pub fn set_server_url(addr: &str) {
    std::env::set_var("PASM_SERVER_URL", addr);
}

// ── Redis URL (server) ────────────────────────────────────────

/// Returns the Redis connection string.
///
/// Priority: `PASM_REDIS_URL` env → config file → `redis://127.0.0.1:6379`.
pub fn redis_url() -> String {
    if let Ok(url) = std::env::var("PASM_REDIS_URL") {
        return url;
    }
    if let Some(cfg) = load_toml() {
        if let Some(url) = cfg.redis_url {
            return url;
        }
    }
    DEFAULT_REDIS_URL.to_string()
}

// ── server bind address (server) ─────────────────────────────

/// Returns the address the server should bind to.
///
/// Priority: `--addr` / `PASM_SERVER_ADDR` env → config file →
/// `0.0.0.0:3000`.
pub fn server_addr() -> String {
    if let Ok(addr) = std::env::var("PASM_SERVER_ADDR") {
        return addr;
    }
    if let Some(cfg) = load_toml() {
        if let Some(addr) = cfg.server_addr {
            return addr;
        }
    }
    DEFAULT_SERVER_ADDR.to_string()
}

// ── database URL (server) ────────────────────────────────────

/// Returns the PostgreSQL connection string.
///
/// Priority: `PASM_DATABASE_URL` env → config file.
/// Panics if neither provides a value.
pub fn database_url() -> String {
    if let Ok(url) = std::env::var("PASM_DATABASE_URL") {
        return url;
    }
    if let Some(cfg) = load_toml() {
        if let Some(url) = cfg.database_url {
            return url;
        }
    }
    panic!("PASM_DATABASE_URL must be set (e.g. postgres://user:pass@localhost/pasm)")
}

// ── connection pool size (server) ───────────────────────────

/// Returns max database pool connections.
///
/// Priority: config file → default (5).
pub fn max_connections() -> u32 {
    if let Some(cfg) = load_toml() {
        if let Some(n) = cfg.max_connections {
            return n;
        }
    }
    DEFAULT_MAX_CONNECTIONS
}
