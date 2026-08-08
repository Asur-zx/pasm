use pasm_server::server::api::auth::session::{RefreshRequest, SessionResponse};

#[test]
fn session_response_serializes_to_json() {
    let resp = SessionResponse {
        session_token: "abc-123".to_string(),
        refresh_token: "xyz-789".to_string(),
        expires_in: 86400,
    };
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains(r#""session_token":"abc-123""#));
    assert!(json.contains(r#""refresh_token":"xyz-789""#));
    assert!(json.contains(r#""expires_in":86400"#));
}

#[test]
fn session_response_contains_all_keys() {
    let resp = SessionResponse {
        session_token: "sess".to_string(),
        refresh_token: "refr".to_string(),
        expires_in: 3600,
    };
    let val = serde_json::to_value(&resp).unwrap();
    let map = val.as_object().unwrap();
    assert!(map.contains_key("session_token"));
    assert!(map.contains_key("refresh_token"));
    assert!(map.contains_key("expires_in"));
}

#[test]
fn refresh_request_deserializes_from_json() {
    let json = r#"{"refresh_token":"my-refresh-token"}"#;
    let req: RefreshRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.refresh_token, "my-refresh-token");
}

#[test]
fn refresh_request_empty_token() {
    let json = r#"{"refresh_token":""}"#;
    let req: RefreshRequest = serde_json::from_str(json).unwrap();
    assert!(req.refresh_token.is_empty());
}

#[test]
fn refresh_request_missing_field_fails() {
    let json = r#"{}"#;
    let result: Result<RefreshRequest, _> = serde_json::from_str(json);
    assert!(result.is_err());
}
