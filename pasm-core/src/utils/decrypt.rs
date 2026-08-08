use magic_crypt::{new_magic_crypt, MagicCryptTrait};

use crate::types::error::PasmResult;

/// Decrypts a base64-encoded ciphertext string using the given password.
///
/// The password is used as the AES-256 key via MagicCrypt. The input `data`
/// must be a base64 string previously produced by `encrypt_string`.
///
/// # Arguments
/// * `data` - The base64-encoded ciphertext to decrypt
/// * `password` - The AES-256 key (used as MagicCrypt password)
///
/// # Returns
/// `Ok(String)` with the decrypted plaintext, or `Err(PasmResult::DecryptionError)`
/// if the data is not valid ciphertext for this key.
pub fn decrypt_string(data: &str, password: &str) -> Result<String, PasmResult> {
    let mcrypt = new_magic_crypt!(password, 256);
    mcrypt
        .decrypt_base64_to_string(data)
        .map_err(|e| PasmResult::DecryptionError {
            err: format!("{e}"),
        })
}
