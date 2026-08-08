use serde::{Deserialize, Serialize};

/// A generic key-value request/response type used in API payloads.
///
/// Used for:
/// - Creating entries (`{ "key": "<name>", "value": "<encrypted>" }`)
/// - Amending entries (same format)
/// - Response payloads from `list_entries`
///
/// # Fields
/// * `key` - The entry name (e.g., `"entry:github"`) or empty string for auth ops
/// * `value` - The encrypted entry data or auth key value
#[derive(Serialize, Deserialize)]
pub struct RequestData {
    pub key: String,
    pub value: String,
}
