use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebhookChannel {
    pub url: String,
    pub bearer_token: Option<String>,
    #[serde(default)]
    pub extra_headers: BTreeMap<String, String>,
}
