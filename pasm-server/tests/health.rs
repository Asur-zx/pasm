use pasm_server::types::health::HealthResponse;

#[test]
fn health_response_serializes_to_json() {
    let health = HealthResponse {
        status: "ok",
        version: "0.1.0",
        uptime_seconds: 42,
        database: "connected",
        timestamp: 1000,
    };
    let json = serde_json::to_string(&health).unwrap();
    assert!(json.contains(r#""status":"ok""#));
    assert!(json.contains(r#""version":"0.1.0""#));
    assert!(json.contains(r#""uptime_seconds":42"#));
    assert!(json.contains(r#""database":"connected""#));
    assert!(json.contains(r#""timestamp":1000"#));
}

#[test]
fn health_response_degraded_status() {
    let health = HealthResponse {
        status: "degraded",
        version: "0.1.0",
        uptime_seconds: 0,
        database: "disconnected",
        timestamp: 0,
    };
    let json = serde_json::to_string(&health).unwrap();
    assert!(json.contains(r#""status":"degraded""#));
    assert!(json.contains(r#""database":"disconnected""#));
}

#[test]
fn health_response_contains_all_keys() {
    let health = HealthResponse {
        status: "ok",
        version: "1.0.0",
        uptime_seconds: 12345,
        database: "connected",
        timestamp: 67890,
    };
    let json = serde_json::to_value(&health).unwrap();
    let map = json.as_object().unwrap();
    assert!(map.contains_key("status"));
    assert!(map.contains_key("version"));
    assert!(map.contains_key("uptime_seconds"));
    assert!(map.contains_key("database"));
    assert!(map.contains_key("timestamp"));
    assert_eq!(map["status"], "ok");
    assert_eq!(map["version"], "1.0.0");
    assert_eq!(map["uptime_seconds"], 12345);
    assert_eq!(map["database"], "connected");
    assert_eq!(map["timestamp"], 67890);
}
