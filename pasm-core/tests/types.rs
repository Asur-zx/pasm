use http::StatusCode;
use pasm_core::types::detail::Details;
use pasm_core::types::entry::RequestData;
use pasm_core::types::error::PasmResult;

#[test]
fn details_serde_round_trip() {
    let details = Details {
        name: "test".to_string(),
        site: "example.com".to_string(),
        uname: "user".to_string(),
        pword: "pass".to_string(),
        note: "note".to_string(),
    };
    let json = serde_json::to_string(&details).unwrap();
    let parsed: Details = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.name, details.name);
    assert_eq!(parsed.site, details.site);
    assert_eq!(parsed.uname, details.uname);
    assert_eq!(parsed.pword, details.pword);
    assert_eq!(parsed.note, details.note);
}

#[test]
fn details_empty_fields_serde() {
    let details = Details {
        name: "empty".to_string(),
        site: String::new(),
        uname: String::new(),
        pword: String::new(),
        note: String::new(),
    };
    let json = serde_json::to_string(&details).unwrap();
    let parsed: Details = serde_json::from_str(&json).unwrap();
    assert!(parsed.site.is_empty());
}

#[test]
fn request_data_serde_round_trip() {
    let data = RequestData {
        key: "entry:github".to_string(),
        value: "encrypted_value_here".to_string(),
    };
    let json = serde_json::to_string(&data).unwrap();
    let parsed: RequestData = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.key, data.key);
    assert_eq!(parsed.value, data.value);
}

#[test]
fn pasm_result_display_database_error() {
    let err = PasmResult::DatabaseError {
        err: "connection refused".to_string(),
    };
    assert_eq!(format!("{err}"), "database error: connection refused");
}

#[test]
fn pasm_result_display_encryption_error() {
    let err = PasmResult::EncryptionError {
        err: "key invalid".to_string(),
    };
    assert_eq!(format!("{err}"), "encryption error: key invalid");
}

#[test]
fn pasm_result_display_decryption_error() {
    let err = PasmResult::DecryptionError {
        err: "bad ciphertext".to_string(),
    };
    assert_eq!(format!("{err}"), "decryption error: bad ciphertext");
}

#[test]
fn pasm_result_display_serialization_error() {
    let err = PasmResult::SerializationError {
        err: "invalid utf-8".to_string(),
    };
    assert_eq!(format!("{err}"), "serialization error: invalid utf-8");
}

#[test]
fn pasm_result_display_deserialization_error() {
    let err = PasmResult::DeserializationError {
        err: "missing field".to_string(),
    };
    assert_eq!(format!("{err}"), "deserialization error: missing field");
}

#[test]
fn pasm_result_display_server_status() {
    let err = PasmResult::ServerStatus(StatusCode::NOT_FOUND, "entry not found".to_string());
    assert_eq!(format!("{err}"), "entry not found");
}
