use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use colored::Colorize;
use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use serde::{Deserialize, Serialize};

use crate::client::curl::requests;
use crate::client::input::prompts::prompt_hidden;
use pasm_core::utils::config;
use pasm_core::utils::crypto;

const CONFIG_DIR: &str = ".config/pasm";
const HASH_FILE: &str = "master.hash";
const SESSION_FILE: &str = "session";
const VERIFY_PLAINTEXT: &str = "pasm::verify";

/// Data stored in the session JSON file.
///
/// Fields:
/// - `api_key`: Bearer token sent with API requests (derived from master password)
/// - `encr_key`: AES-256 key for entry encryption/decryption (derived from master password)
#[derive(Serialize, Deserialize)]
struct SessionData {
    api_key: String,
    encr_key: String,
}

/// Returns the `$HOME/.config/pasm` directory path.
///
/// Falls back to `/tmp/.config/pasm` if `$HOME` is unset.
fn config_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(CONFIG_DIR)
}

/// Returns the path to the master password hash file.
fn hash_path() -> PathBuf {
    config_dir().join(HASH_FILE)
}

/// Returns the path to the session file.
fn session_path() -> PathBuf {
    config_dir().join(SESSION_FILE)
}

/// Returns whether the master password hash file exists on disk.
fn password_exists() -> bool {
    hash_path().exists()
}

/// Returns whether the session file exists on disk.
fn session_exists() -> bool {
    session_path().exists()
}

/// Sets Unix permissions to 0600 (owner read/write only) on the given path.
///
/// This is a no-op on non-Unix platforms.
///
/// # Arguments
/// * `path` - Path to the file whose permissions to restrict
fn set_restricted_permissions(path: &std::path::Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(path) {
            let mut perms = meta.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(path, perms).ok();
        }
    }
}

/// Stores an encrypted verification of the master password to disk.
///
/// Encrypts the known plaintext `"pasm::verify"` using the password as the
/// encryption key, then writes the base64 ciphertext to the hash file.
/// The file is created with 0600 permissions. This can be used later to
/// verify the password without storing a password-equivalent hash.
///
/// # Arguments
/// * `password` - The master password to store a verification for
///
/// # Errors
/// Returns an error string if the config directory cannot be created or
/// the hash file cannot be written.
fn store_password_hash(password: &str) -> Result<(), String> {
    fs::create_dir_all(config_dir()).map_err(|e| format!("failed to create config dir: {e}"))?;
    let mcrypt = new_magic_crypt!(password, 256);
    let hash = mcrypt.encrypt_str_to_base64(VERIFY_PLAINTEXT);
    fs::write(hash_path(), hash.as_bytes()).map_err(|e| format!("failed to write hash: {e}"))?;
    set_restricted_permissions(&hash_path());
    Ok(())
}

/// Verifies a password against the stored hash file.
///
/// Decrypts the hash file content using the provided password and checks
/// whether the decrypted plaintext matches the expected verification string.
///
/// # Arguments
/// * `password` - The password to verify
///
/// # Returns
/// `true` if the password matches, `false` if the hash file is missing,
/// the password is wrong, or decryption fails for any reason.
fn verify_password(password: &str) -> bool {
    let hash = match fs::read_to_string(hash_path()) {
        Ok(h) => h.trim().to_string(),
        Err(_) => return false,
    };
    let mcrypt = new_magic_crypt!(password, 256);
    match mcrypt.decrypt_base64_to_string(&hash) {
        Ok(s) => s == VERIFY_PLAINTEXT,
        Err(_) => false,
    }
}

/// Writes the session keys to the session JSON file with 0600 permissions.
///
/// # Arguments
/// * `api_key` - The API key (Bearer token) to store
/// * `encr_key` - The encryption key to store
///
/// # Errors
/// Returns an error string if serialization or file I/O fails.
fn store_session(api_key: &str, encr_key: &str) -> Result<(), String> {
    fs::create_dir_all(config_dir()).map_err(|e| format!("failed to create config dir: {e}"))?;
    let data = SessionData {
        api_key: api_key.to_string(),
        encr_key: encr_key.to_string(),
    };
    let json = serde_json::to_string(&data).map_err(|e| format!("session serialization: {e}"))?;
    fs::write(session_path(), json.as_bytes())
        .map_err(|e| format!("failed to write session: {e}"))?;
    set_restricted_permissions(&session_path());
    Ok(())
}

/// Deletes the session file from disk.
///
/// This does not invalidate the server-side API key; it just removes
/// local credentials, requiring the user to log in again.
fn destroy_session() -> Result<(), String> {
    if session_path().exists() {
        fs::remove_file(session_path()).map_err(|e| format!("failed to remove session: {e}"))
    } else {
        Ok(())
    }
}

/// Reads and returns the session keys from the session file, if it exists.
///
/// # Returns
/// `Some((api_key, encr_key))` if the session file exists and is valid JSON,
/// `None` otherwise.
pub fn get_session_keys() -> Option<(String, String)> {
    let path = session_path();
    if !path.exists() {
        return None;
    }
    let content = fs::read_to_string(path).ok()?;
    let data: SessionData = serde_json::from_str(&content).ok()?;
    Some((data.api_key, data.encr_key))
}

/// Logs the user in by creating or verifying the master password and setting up a session.
///
/// **First-time flow:**
/// 1. Displays a password criticality warning
/// 2. Prompts for and confirms a new master password
/// 3. Stores a password verification hash on disk
/// 4. Derives `api_key` and `encr_key` from the password
/// 5. Registers the `api_key` with the server via `POST /auth`
/// 6. Saves the session keys to the session file
///
/// **Subsequent flow:**
/// 1. If a session already exists, offers auto-login (skip re-entering password)
/// 2. Otherwise, prompts for the master password (up to 3 attempts)
/// 3. Verifies the password against the stored hash
/// 4. Re-derives keys and re-creates the session file
///
/// # Returns
/// A status message string indicating success or describing the failure.
pub fn login() -> String {
    if !password_exists() {
        println!(
            "\n{} {}\n{}\n{}\n",
            "⚠".yellow().bold(),
            "MASTER PASSWORD SETUP".yellow().bold(),
            "This password protects ALL your stored credentials."
                .red()
                .bold(),
            "If you lose it, your data CANNOT be recovered."
                .red()
                .bold(),
        );

        let password = prompt_hidden("Create master password: ");
        if password.is_empty() {
            return "password cannot be empty".to_string();
        }
        let confirm = prompt_hidden("Confirm master password: ");
        if password != confirm {
            return "passwords do not match".to_string();
        }

        if let Err(e) = store_password_hash(&password) {
            return format!("failed to store password: {e}");
        }

        // Write default config file on first setup
        let cfg = config::TomlConfig {
            server_url: Some(config::server_url().to_string()),
            server_addr: None,
            database_url: None,
            redis_url: None,
            max_connections: None,
        };
        if let Err(e) = config::write_toml(&cfg) {
            eprintln!("Warning: failed to create config file: {e}");
        }

        let api_key = crypto::derive_api_key(&password);
        let encr_key = crypto::derive_key(&password, "pasm-encr");

        let reg_res = requests::register_auth(&api_key);
        if reg_res.starts_with("Error") {
            eprintln!("Warning: server registration failed. API calls may not work.");
        }

        if let Err(e) = store_session(&api_key, &encr_key) {
            return format!("failed to create session: {e}");
        }

        return "master password created and logged in".to_string();
    }

    if session_exists() {
        let answer = prompt_visible("Auto-login using existing session? [Y/n]: ");
        if !answer.starts_with('n') && !answer.starts_with('N') {
            return "auto-logged in".to_string();
        }
    }

    for attempt in 1..=3 {
        let password = prompt_hidden("Master password: ");
        if verify_password(&password) {
            let api_key = crypto::derive_api_key(&password);
            let encr_key = crypto::derive_key(&password, "pasm-encr");
            if let Err(e) = store_session(&api_key, &encr_key) {
                return format!("failed to create session: {e}");
            }
            return "logged in".to_string();
        }
        if attempt < 3 {
            println!("incorrect password (attempt {attempt}/3)");
        }
    }

    "too many failed attempts".to_string()
}

/// Logs the user out by deleting the local session file.
///
/// The server-side API key remains registered and is not invalidated.
///
/// # Returns
/// A status message string indicating success or failure.
pub fn logout() -> String {
    match destroy_session() {
        Ok(_) => "logged out".to_string(),
        Err(e) => format!("logout failed: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static HOME_LOCK: Mutex<()> = Mutex::new(());

    fn with_temp_home<F>(f: F)
    where
        F: FnOnce(PathBuf),
    {
        let _lock = HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let original = std::env::var("HOME").ok();
        let temp = std::env::temp_dir().join("pasm_cli_test_master");
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&temp).unwrap();
        std::env::set_var("HOME", &temp);
        f(temp.clone());
        match original {
            Some(h) => std::env::set_var("HOME", h),
            None => std::env::remove_var("HOME"),
        }
        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn config_dir_uses_home_env() {
        with_temp_home(|tmp| {
            assert_eq!(config_dir(), tmp.join(".config/pasm"));
        });
    }

    #[test]
    fn config_dir_fallback_when_home_unset() {
        let _lock = HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let original = std::env::var("HOME").ok();
        std::env::remove_var("HOME");
        let dir = config_dir();
        assert_eq!(dir, PathBuf::from("/tmp").join(".config/pasm"));
        if let Some(h) = original {
            std::env::set_var("HOME", h);
        }
    }

    #[test]
    fn hash_path_contains_master_hash() {
        with_temp_home(|tmp| {
            assert_eq!(hash_path(), tmp.join(".config/pasm/master.hash"));
        });
    }

    #[test]
    fn session_path_contains_session() {
        with_temp_home(|tmp| {
            assert_eq!(session_path(), tmp.join(".config/pasm/session"));
        });
    }

    #[test]
    fn password_exists_false_when_no_hash() {
        with_temp_home(|_| {
            assert!(!password_exists());
        });
    }

    #[test]
    fn password_exists_true_after_store() {
        with_temp_home(|_| {
            store_password_hash("mypass").unwrap();
            assert!(password_exists());
        });
    }

    #[test]
    fn store_and_verify_password_roundtrip() {
        with_temp_home(|_| {
            store_password_hash("correct").unwrap();
            assert!(verify_password("correct"));
            assert!(!verify_password("wrong"));
        });
    }

    #[test]
    fn verify_password_returns_false_when_no_hash() {
        with_temp_home(|_| {
            assert!(!verify_password("any"));
        });
    }

    #[test]
    fn session_exists_false_when_no_session() {
        with_temp_home(|_| {
            assert!(!session_exists());
        });
    }

    #[test]
    fn store_session_and_get_keys_roundtrip() {
        with_temp_home(|_| {
            store_session("ak_abc", "ek_xyz").unwrap();
            let keys = get_session_keys();
            assert!(keys.is_some());
            let (api_key, encr_key) = keys.unwrap();
            assert_eq!(api_key, "ak_abc");
            assert_eq!(encr_key, "ek_xyz");
        });
    }

    #[test]
    fn get_session_keys_returns_none_when_no_file() {
        with_temp_home(|_| {
            assert!(get_session_keys().is_none());
        });
    }

    #[test]
    fn destroy_session_removes_session_file() {
        with_temp_home(|_| {
            store_session("ak", "ek").unwrap();
            assert!(session_exists());
            destroy_session().unwrap();
            assert!(!session_exists());
        });
    }

    #[test]
    fn destroy_session_noop_when_no_session() {
        with_temp_home(|_| {
            assert!(destroy_session().is_ok());
        });
    }
}

/// Prompts the user for a yes/no response with visible input.
///
/// # Arguments
/// * `msg` - The prompt message (e.g., `"Auto-login? [Y/n]: "`)
///
/// # Returns
/// The trimmed response string, or `"y"` if the input was empty.
fn prompt_visible(msg: &str) -> String {
    print!("{msg}");
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    input.trim().to_string()
}
