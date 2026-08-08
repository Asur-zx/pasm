use wasm_bindgen::prelude::*;

/// Encrypts and serializes entry JSON using the given passkey.
#[wasm_bindgen]
pub fn encrypt_entry(json: &str, passkey: &str) -> Result<String, JsValue> {
    let details: pasm_core::types::detail::Details =
        serde_json::from_str(json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    pasm_core::utils::serialize::serialize_entry(&details, passkey)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Decrypts and deserializes an encrypted entry string.
#[wasm_bindgen]
pub fn decrypt_entry(encrypted: &str, passkey: &str) -> Result<String, JsValue> {
    let details = pasm_core::utils::deserialize::deserialize_entry(encrypted, passkey)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    serde_json::to_string(&details).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Derives the API key from a master password (same algorithm as the CLI).
#[wasm_bindgen]
pub fn derive_api_key(password: &str) -> String {
    pasm_core::utils::crypto::derive_api_key(password)
}

/// Derives the encryption key from a master password.
#[wasm_bindgen]
pub fn derive_encr_key(password: &str) -> String {
    pasm_core::utils::crypto::derive_key(password, "pasm-encr")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_api_key_returns_64_char_hex() {
        let key = derive_api_key("test_password");
        assert_eq!(key.len(), 64);
        assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn derive_api_key_is_deterministic() {
        let a = derive_api_key("hello");
        let b = derive_api_key("hello");
        assert_eq!(a, b);
    }

    #[test]
    fn derive_api_key_empty_password() {
        let key = derive_api_key("");
        assert_eq!(key.len(), 64);
    }

    #[test]
    fn derive_encr_key_returns_64_char_hex() {
        let key = derive_encr_key("test_password");
        assert_eq!(key.len(), 64);
        assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn derive_encr_key_differs_from_api_key() {
        let ak = derive_api_key("mypass");
        let ek = derive_encr_key("mypass");
        assert_ne!(ak, ek);
    }

    #[test]
    fn derive_encr_key_empty_password() {
        let key = derive_encr_key("");
        assert_eq!(key.len(), 64);
    }

    #[test]
    fn derive_keys_differ_for_different_passwords() {
        let a = derive_api_key("alice");
        let b = derive_api_key("bob");
        assert_ne!(a, b);
    }
}
