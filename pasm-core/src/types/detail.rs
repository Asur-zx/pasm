use serde::{Deserialize, Serialize};

/// User detail
///
/// Implements Clone and Debug
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Details {
    pub name: String,
    pub site: String,
    pub uname: String,
    pub pword: String,
    pub note: String,
}
