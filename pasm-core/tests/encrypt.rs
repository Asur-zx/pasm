use pasm_core::utils::crypto::derive_key;
use pasm_core::utils::decrypt::decrypt_string;
use pasm_core::utils::encrypt::encrypt_string;

#[test]
fn encrypt_then_decrypt_round_trip() {
    let key = derive_key("test_password", "test_encr");
    let ciphertext = encrypt_string("hello world".to_string(), key.clone());
    let decrypted = decrypt_string(&ciphertext, &key).unwrap();
    assert_eq!(decrypted, "hello world");
}

#[test]
fn decrypt_with_wrong_key_fails() {
    let key = derive_key("correct", "encr");
    let wrong_key = derive_key("wrong", "encr");
    let ciphertext = encrypt_string("secret data".to_string(), key);
    let result = decrypt_string(&ciphertext, &wrong_key);
    assert!(result.is_err());
}

#[test]
fn decrypt_garbage_fails() {
    let key = derive_key("any", "encr");
    let result = decrypt_string("not-valid-base64!!", &key);
    assert!(result.is_err());
}

#[test]
fn encrypt_unicode() {
    let key = derive_key("pw", "encr");
    let data = "héllo wörld 🔥 中文".to_string();
    let ciphertext = encrypt_string(data.clone(), key.clone());
    let decrypted = decrypt_string(&ciphertext, &key).unwrap();
    assert_eq!(decrypted, data);
}

#[test]
fn encrypt_empty_string() {
    let key = derive_key("pw", "encr");
    let ciphertext = encrypt_string(String::new(), key.clone());
    let decrypted = decrypt_string(&ciphertext, &key).unwrap();
    assert_eq!(decrypted, "");
}
