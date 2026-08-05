use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NtfyChannel {
    pub base_url: String,
    pub topic: String,
    pub auth_token: Option<String>,
}
