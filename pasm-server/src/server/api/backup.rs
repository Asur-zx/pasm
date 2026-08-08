use std::{
    fs::OpenOptions,
    io::Write,
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension};

use crate::types::{db::Db, state::PasmState};

/// Returns a backup filename like `{user_id}_{unix_timestamp}.json`.
fn make_backup_filename(user_id: &str, timestamp: u64) -> String {
    format!("{user_id}_{timestamp}.json")
}

/// Creates a JSON dump of all encrypted entries for the authenticated user.
///
/// Writes to `/tmp/pasm/backups/<user_id>_<unix_timestamp>.json` and
/// returns the file path along with entry count and size.
pub async fn call(
    Extension(auth_key): Extension<String>,
    State(state): State<PasmState>,
) -> impl IntoResponse {
    let db = &state.db;

    let user_id = match db.get_user_id_by_authkey(&auth_key).await {
        Ok(id) => id,
        Err(err) => return err.into_response(),
    };

    let entries = match db.list_entries(&user_id).await {
        Ok(e) => e,
        Err(err) => return err.into_response(),
    };

    let backup_dir = "/tmp/pasm/backups";
    if let Err(e) = std::fs::create_dir_all(backup_dir) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to create backup directory: {e}"),
        )
            .into_response();
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let file_name = make_backup_filename(&user_id, timestamp);
    let backup_path = std::path::Path::new(backup_dir).join(&file_name);

    let json = match serde_json::to_string_pretty(&entries) {
        Ok(j) => j,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to serialize entries: {e}"),
            )
                .into_response();
        }
    };

    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);

    let mut file = match options.open(&backup_path) {
        Ok(file) => file,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to create backup file: {e}"),
            )
                .into_response();
        }
    };

    if let Err(e) = file.write_all(json.as_bytes()) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to write backup file: {e}"),
        )
            .into_response();
    }

    let path_str = backup_path.to_string_lossy().to_string();
    let size = json.len();
    let count = entries.len();

    (
        StatusCode::OK,
        format!("Backup created: {path_str} ({count} entries, {size} bytes)"),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_backup_filename_basic() {
        assert_eq!(make_backup_filename("user-123", 1000), "user-123_1000.json");
    }

    #[test]
    fn make_backup_filename_with_uuid() {
        let uid = "550e8400-e29b-41d4-a716-446655440000";
        let name = make_backup_filename(uid, 987654321);
        assert_eq!(name, "550e8400-e29b-41d4-a716-446655440000_987654321.json");
    }

    #[test]
    fn make_backup_filename_zero_timestamp() {
        assert_eq!(make_backup_filename("admin", 0), "admin_0.json");
    }
}
