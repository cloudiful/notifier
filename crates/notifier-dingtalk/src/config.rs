use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DingtalkChannel {
    pub webhook_url: String,
    pub secret: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
}
