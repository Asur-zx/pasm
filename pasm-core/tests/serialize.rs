use pasm_core::types::detail::Details;
use pasm_core::utils::crypto::derive_key;
use pasm_core::utils::deserialize::deserialize_entry;
use pasm_core::utils::serialize::serialize_entry;

fn make_details() -> Details {
    Details {
        name: "github".to_string(),
        site: "github.com".to_string(),
        uname: "user@example.com".to_string(),
        pword: "s3cret!".to_string(),
        note: "My GitHub account".to_string(),
    }
}

#[test]
fn serialize_then_deserialize_round_trip() {
    let key = derive_key("test", "serde");
    let details = make_details();
    let serialized = serialize_entry(&details, &key).unwrap();
    let deserialized = deserialize_entry(&serialized, &key).unwrap();
    assert_eq!(deserialized.name, details.name);
    assert_eq!(deserialized.site, details.site);
    assert_eq!(deserialized.uname, details.uname);
    assert_eq!(deserialized.pword, details.pword);
    assert_eq!(deserialized.note, details.note);
}

#[test]
fn deserialize_wrong_key_fails() {
    let key = derive_key("correct", "serde");
    let wrong_key = derive_key("wrong", "serde");
    let details = make_details();
    let serialized = serialize_entry(&details, &key).unwrap();
    let result = deserialize_entry(&serialized, &wrong_key);
    assert!(result.is_err());
}

#[test]
fn serialize_empty_fields() {
    let key = derive_key("pw", "serde");
    let details = Details {
        name: "empty".to_string(),
        site: String::new(),
        uname: String::new(),
        pword: String::new(),
        note: String::new(),
    };
    let serialized = serialize_entry(&details, &key).unwrap();
    let deserialized = deserialize_entry(&serialized, &key).unwrap();
    assert!(deserialized.site.is_empty());
    assert!(deserialized.note.is_empty());
}

#[test]
fn serialize_unicode_fields() {
    let key = derive_key("pw", "serde");
    let details = Details {
        name: "🔥".to_string(),
        site: "café.fr".to_string(),
        uname: "üser".to_string(),
        pword: "päss".to_string(),
        note: "日本語".to_string(),
    };
    let serialized = serialize_entry(&details, &key).unwrap();
    let deserialized = deserialize_entry(&serialized, &key).unwrap();
    assert_eq!(deserialized.name, "🔥");
    assert_eq!(deserialized.site, "café.fr");
    assert_eq!(deserialized.note, "日本語");
}
