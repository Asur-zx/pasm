use argon2::Argon2;
use sha2::{Digest, Sha256};

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Derives a 256-bit key from a password using Argon2id.
///
/// Uses Argon2id with:
/// - Memory cost: 19456 KiB (~19 MiB)
/// - Time cost: 2 iterations
/// - Parallelism: 1
/// - Salt: first 16 bytes of SHA-256(context)
///
/// The salt is deterministic from the context string so the same
/// password always produces the same key for a given purpose.
pub fn derive_key(password: &str, context: &str) -> String {
    let params = argon2::Params::new(19_456, 2, 1, Some(32)).expect("invalid Argon2 parameters");
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let salt = Sha256::digest(context.as_bytes());
    let mut hash = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), &salt[..16], &mut hash)
        .expect("Argon2 hashing failed");
    hex_encode(&hash)
}

/// Full API key derivation from the master password.
///
/// Equivalent to `derive_key(password, "pasm-auth")`.
pub fn derive_api_key(password: &str) -> String {
    derive_key(password, "pasm-auth")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_encode_returns_proper_hex_string() {
        assert_eq!(hex_encode(&[0x00]), "00");
        assert_eq!(hex_encode(&[0xff]), "ff");
        assert_eq!(hex_encode(&[0xde, 0xad]), "dead");
        assert_eq!(hex_encode(&[0x0f, 0x1e]), "0f1e");
        assert_eq!(hex_encode(b"hello"), "68656c6c6f");
    }

    #[test]
    fn hex_encode_empty_slice() {
        assert_eq!(hex_encode(&[]), "");
    }
}
