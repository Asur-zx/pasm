use magic_crypt::{new_magic_crypt, MagicCryptTrait};

/// Encrypts a plaintext string to base64 using AES-256 via MagicCrypt.
///
/// The encryption is deterministic given the same input and key only if
/// MagicCrypt uses a fixed IV. In practice, MagicCrypt generates a random
/// IV per encryption, so the same data encrypts to different ciphertexts.
///
/// # Arguments
/// * `data` - The plaintext string to encrypt
/// * `password` - The AES-256 key (used as MagicCrypt password)
///
/// # Returns
/// A base64-encoded ciphertext string. The operation is infallible
/// (MagicCrypt does not report encryption errors).
pub fn encrypt_string(data: String, password: String) -> String {
    let mcrypt = new_magic_crypt!(password.as_str(), 256);
    mcrypt.encrypt_str_to_base64(&data)
}
