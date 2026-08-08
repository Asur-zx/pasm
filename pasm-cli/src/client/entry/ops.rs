use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use colored::Colorize;

use crate::client::{curl::requests, input::prompts};
use pasm_core::types::{detail::Details, entry::RequestData};
use pasm_core::utils::{deserialize::deserialize_entry, serialize::serialize_entry};

/// Creates a new encrypted entry on the server.
///
/// Collects entry details via interactive prompts, encrypts them with
/// `serialize_entry`, and sends the encrypted data to the server via
/// `POST /entry`.
///
/// # Arguments
/// * `api_key` - Bearer token for server API authentication
/// * `encr_key` - AES-256 key used to encrypt the entry data before upload
///
/// # Returns
/// A success message like `"entry 'github' created"`, or the raw error
/// response from the server if creation fails.
pub fn create(api_key: &str, encr_key: &str) -> String {
    let details = prompts::collect_details();
    let encrypted = match serialize_entry(&details, encr_key) {
        Ok(e) => e,
        Err(e) => return e.to_string(),
    };
    let res = requests::create_entry(api_key, &details.name, &encrypted);
    if res.is_empty() || res.contains("Error") {
        return res;
    }
    format!("entry '{}' created", details.name)
}

/// Finds and displays a decrypted entry by name.
///
/// Fetches the encrypted entry from the server via `GET /entry/{name}`,
/// then decrypts and formats it using `deserialize_entry`.
///
/// # Arguments
/// * `api_key` - Bearer token for server API authentication
/// * `encr_key` - AES-256 key used to decrypt the fetched entry data
/// * `name` - The entry name to look up
///
/// # Returns
/// A formatted display string with the decrypted entry details, or an
/// error message if the entry is not found or decryption fails.
pub fn find(api_key: &str, encr_key: &str, name: &str) -> String {
    let res = requests::find_entry(api_key, name);
    let parsed: serde_json::Value = match serde_json::from_str(&res) {
        Ok(v) => v,
        Err(_) => return res,
    };
    let value = match parsed["value"].as_str() {
        Some(v) => v,
        None => return res,
    };
    match deserialize_entry(value, encr_key) {
        Ok(details) => format_details(&details),
        Err(e) => e.to_string(),
    }
}

/// Lists all entries with their decrypted details.
///
/// Fetches all entries from the server via `GET /entries`, then decrypts
/// and formats each one.
///
/// # Arguments
/// * `api_key` - Bearer token for server API authentication
/// * `encr_key` - AES-256 key used to decrypt each entry
///
/// # Returns
/// A formatted string showing all entries separated by headers, or
/// `"(no entries)"` if the list is empty. Decryption failures are noted
/// per-entry rather than aborting the entire listing.
pub fn list(api_key: &str, encr_key: &str) -> String {
    let res = requests::list_entries(api_key);
    let items: Vec<serde_json::Value> = match serde_json::from_str(&res) {
        Ok(v) => v,
        Err(_) => return res,
    };
    if items.is_empty() {
        return "(no entries)".to_string();
    }
    let mut out = String::new();
    for item in &items {
        let name = item["key"].as_str().unwrap_or("?");
        let value = item["value"].as_str().unwrap_or("");
        match deserialize_entry(value, encr_key) {
            Ok(details) => {
                out.push_str(&format!("--- {} ---\n", name.bright_blue()));
                out.push_str(&format_detail_lines(&details));
            }
            Err(e) => {
                out.push_str(&format!("--- {} --- ({})\n", name.bright_blue(), e));
            }
        }
    }
    out.trim_end().to_string()
}

/// Updates (or creates) an encrypted entry on the server.
///
/// Collects entry details via interactive prompts, encrypts them, and
/// sends them to the server via `POST /entry/amend`. Unlike `create`,
/// this will overwrite an existing entry with the same name.
///
/// # Arguments
/// * `api_key` - Bearer token for server API authentication
/// * `encr_key` - AES-256 key used to encrypt the entry data before upload
///
/// # Returns
/// A success message like `"entry 'github' updated"`, or the raw error
/// response from the server if the operation fails.
pub fn amend(api_key: &str, encr_key: &str) -> String {
    let details = prompts::collect_details();
    let encrypted = match serialize_entry(&details, encr_key) {
        Ok(e) => e,
        Err(e) => return e.to_string(),
    };
    let res = requests::amend_entry(api_key, &details.name, &encrypted);
    if res.is_empty() || res.contains("Error") {
        return res;
    }
    format!("entry '{}' updated", details.name)
}

/// Deletes an entry by name.
///
/// Sends `DELETE /entry/{name}` — no encryption/decryption is needed since
/// only the entry name is used.
///
/// # Arguments
/// * `api_key` - Bearer token for server API authentication
/// * `name` - The entry name to delete
///
/// # Returns
/// The server response body.
pub fn delete(api_key: &str, name: &str) -> String {
    requests::delete_entry(api_key, name)
}

/// Returns the local backup directory path (`~/.config/pasm/backups`).
fn backup_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".config/pasm/backups")
}

/// Saves a local copy of all encrypted entries to `~/.config/pasm/backups/`.
///
/// Fetches all entries from the server and writes them as pretty-printed JSON
/// to a timestamped file. The file is saved with 0600 permissions.
///
/// # Arguments
/// * `api_key` - Bearer token for server API authentication
///
/// # Returns
/// `Ok(path_string)` on success, or an error message on failure.
fn save_local_backup(api_key: &str) -> Result<String, String> {
    let res = requests::list_entries(api_key);
    let entries: Vec<RequestData> =
        serde_json::from_str(&res).map_err(|e| format!("failed to parse entries: {e}"))?;

    let dir = backup_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("failed to create backup dir: {e}"))?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let file_name = format!("backup_{timestamp}.json");
    let path = dir.join(&file_name);

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| format!("failed to serialize entries: {e}"))?;

    fs::write(&path, &json).map_err(|e| format!("failed to write backup: {e}"))?;

    // Set 0600 permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(&path) {
            let mut perms = meta.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&path, perms).ok();
        }
    }

    let path_str = path.to_string_lossy().to_string();
    Ok(format!(
        "{path_str} ({count} entries, {size} bytes)",
        count = entries.len(),
        size = json.len()
    ))
}

/// Creates a backup: calls the server-side backup endpoint and also saves a
/// local copy of all entries to `~/.config/pasm/backups/`.
///
/// # Arguments
/// * `api_key` - Bearer token for server API authentication
///
/// # Returns
/// A summary message with server backup path and local backup path.
pub fn backup(api_key: &str) -> String {
    let mut parts: Vec<String> = Vec::new();

    // Server-side backup
    let server_res = requests::backup(api_key);
    if server_res.starts_with("Error") {
        parts.push(format!("Warning: server backup failed — {server_res}"));
    } else {
        parts.push(server_res);
    }

    // Local backup
    match save_local_backup(api_key) {
        Ok(local) => parts.push(format!("Local backup saved: {local}")),
        Err(e) => parts.push(format!("Warning: local backup failed — {e}")),
    }

    parts.join("\n")
}

/// Restores entries from a local backup JSON file into the server.
///
/// Reads the file, parses the `Vec<RequestData>` entries, and amends each
/// one to the server. Entries already present on the server are overwritten.
///
/// # Arguments
/// * `api_key` - Bearer token for server API authentication
/// * `path` - Path to the local backup JSON file
///
/// # Returns
/// A summary message with counts of restored, skipped, and failed entries.
pub fn restore_from_file(api_key: &str, path: &str) -> String {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => return format!("Error: failed to read '{path}': {e}"),
    };

    let entries: Vec<RequestData> = match serde_json::from_str(&content) {
        Ok(e) => e,
        Err(e) => return format!("Error: failed to parse backup file: {e}"),
    };

    if entries.is_empty() {
        return "no entries found in backup file".to_string();
    }

    let mut restored = 0u32;
    let mut errors: Vec<String> = Vec::new();

    for entry in &entries {
        let name = entry.key.strip_prefix("entry:").unwrap_or(&entry.key);
        let res = requests::amend_entry(api_key, name, &entry.value);
        if res.starts_with("Error") {
            errors.push(format!("  {name}: {res}"));
        } else {
            restored += 1;
        }
    }

    let total = entries.len();
    let failed = errors.len();

    let mut msg = format!("Restored {restored}/{total} entries");
    if failed > 0 {
        msg.push_str(&format!("\nFailed ({failed}):\n{}", errors.join("\n")));
    }
    msg
}

#[cfg(test)]
mod tests {
    use super::*;
    use pasm_core::types::detail::Details;

    fn make_details() -> Details {
        Details {
            name: "github".to_string(),
            site: "github.com".to_string(),
            uname: "user@example.com".to_string(),
            pword: "s3cret!".to_string(),
            note: "My account".to_string(),
        }
    }

    #[test]
    fn format_details_contains_field_values() {
        let out = format_details(&make_details());
        assert!(out.contains("github.com"));
        assert!(out.contains("user@example.com"));
        assert!(out.contains("s3cret!"));
        assert!(out.contains("My account"));
    }

    #[test]
    fn format_details_contains_field_labels() {
        let out = format_details(&make_details());
        assert!(out.contains("Name:"));
        assert!(out.contains("Site:"));
        assert!(out.contains("Username:"));
        assert!(out.contains("Password:"));
        assert!(out.contains("Note:"));
    }

    #[test]
    fn format_detail_lines_contains_field_values() {
        let out = format_detail_lines(&make_details());
        assert!(out.contains("github.com"));
        assert!(out.contains("user@example.com"));
        assert!(out.contains("s3cret!"));
        assert!(out.contains("My account"));
    }

    #[test]
    fn format_detail_lines_contains_field_labels() {
        let out = format_detail_lines(&make_details());
        assert!(out.contains("Site:"));
        assert!(out.contains("Username:"));
        assert!(out.contains("Password:"));
        assert!(out.contains("Note:"));
    }

    #[test]
    fn backup_dir_ends_with_correct_suffix() {
        let path = backup_dir();
        let s = path.to_string_lossy();
        assert!(s.ends_with(".config/pasm/backups"));
    }
}

/// Formats a `Details` struct into a multi-line display string for single-entry view.
///
/// Each field is labelled with a green prefix and displayed on its own line.
fn format_details(d: &Details) -> String {
    format!(
        "{} {}\n{} {}\n{} {}\n{} {}\n{} {}",
        "Name:".bright_green(),
        d.name,
        "Site:".bright_green(),
        d.site,
        "Username:".bright_green(),
        d.uname,
        "Password:".bright_green(),
        d.pword,
        "Note:".bright_green(),
        d.note,
    )
}

/// Formats a `Details` struct into an indented sub-list for list-entry view.
///
/// Used within the `list` function to display each entry's non-name fields
/// under a header. Each field is indented with two spaces.
fn format_detail_lines(d: &Details) -> String {
    format!(
        "  {} {}\n  {} {}\n  {} {}\n  {} {}\n",
        "Site:".bright_green(),
        d.site,
        "Username:".bright_green(),
        d.uname,
        "Password:".bright_green(),
        d.pword,
        "Note:".bright_green(),
        d.note,
    )
}
