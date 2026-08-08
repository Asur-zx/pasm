use pasm_core::utils::crypto::{derive_api_key, derive_key};

#[test]
fn derive_key_returns_64_char_hex() {
    let key = derive_key("my_password", "test");
    assert_eq!(key.len(), 64);
    assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn derive_key_is_deterministic() {
    let a = derive_key("password123", "app");
    let b = derive_key("password123", "app");
    assert_eq!(a, b);
}

#[test]
fn derive_key_differs_for_different_passwords() {
    let a = derive_key("alice", "ctx");
    let b = derive_key("bob", "ctx");
    assert_ne!(a, b);
}

#[test]
fn derive_key_differs_for_different_contexts() {
    let a = derive_key("mypass", "auth");
    let b = derive_key("mypass", "encr");
    assert_ne!(a, b);
}

#[test]
fn derive_key_empty_password() {
    let key = derive_key("", "ctx");
    assert_eq!(key.len(), 64);
}

#[test]
fn derive_key_unicode_password() {
    let key = derive_key("pässwörd🔥", "ctx");
    assert_eq!(key.len(), 64);
}

#[test]
fn derive_api_key_matches_derive_key_with_pasm_auth() {
    let pw = "my_pass";
    assert_eq!(derive_api_key(pw), derive_key(pw, "pasm-auth"));
}

#[test]
fn derive_api_key_returns_64_char_hex() {
    let key = derive_api_key("admin");
    assert_eq!(key.len(), 64);
    assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
}
