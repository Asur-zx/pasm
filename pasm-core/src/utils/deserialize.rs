use crate::types::detail::Details;
use crate::types::error::PasmResult;
use crate::utils::decrypt::decrypt_string;

/// Decrypts and deserializes an encrypted string into a `Details` struct.
///
/// This is the reverse of `serialize_entry`:
/// 1. Decrypts the base64 ciphertext using `decrypt_string`
/// 2. Parses the resulting JSON into a `Details` struct
///
/// # Arguments
/// * `data` - The base64-encoded, JSON-serialized, encrypted entry data
/// * `passkey` - The AES-256 key used for decryption
///
/// # Returns
/// `Ok(Details)` on success, `Err(PasmResult)` if decryption or JSON parsing fails.
pub fn deserialize_entry(data: &str, passkey: &str) -> Result<Details, PasmResult> {
    let decrypted = decrypt_string(data, passkey)?;
    serde_json::from_str(&decrypted).map_err(|e| PasmResult::DeserializationError {
        err: format!("{e}"),
    })
}
