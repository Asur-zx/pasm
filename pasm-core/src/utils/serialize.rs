use crate::types::detail::Details;
use crate::types::error::PasmResult;
use crate::utils::encrypt::encrypt_string;

/// Serializes and encrypts a `Details` struct for storage on the server.
///
/// The process is:
/// 1. Serializes `details` to a JSON string
/// 2. Encrypts the JSON string with `encrypt_string` using `passkey`
///
/// The resulting string is safe to transmit over HTTP or store in the database.
///
/// # Arguments
/// * `details` - The entry details to encrypt
/// * `passkey` - The AES-256 key used for encryption
///
/// # Returns
/// `Ok(String)` containing the base64-encoded ciphertext, or
/// `Err(PasmResult::SerializationError)` if JSON serialization fails.
pub fn serialize_entry(details: &Details, passkey: &str) -> Result<String, PasmResult> {
    let detail_str =
        serde_json::to_string(details).map_err(|e| PasmResult::SerializationError {
            err: format!("{e}"),
        })?;
    Ok(encrypt_string(detail_str, passkey.to_string()))
}
