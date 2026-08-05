use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageEnvelope {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub body: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_body: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, Value>,
}

impl MessageEnvelope {
    pub fn new(body: impl Into<String>) -> Self {
        Self {
            title: None,
            body: body.into(),
            html_body: None,
            metadata: BTreeMap::new(),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_html_body(mut self, html_body: impl Into<String>) -> Self {
        self.html_body = Some(html_body.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::MessageEnvelope;

    #[test]
    fn serializes_without_empty_metadata() {
        let envelope = MessageEnvelope::new("body").with_title("title");

        let value = serde_json::to_value(&envelope).unwrap();

        assert_eq!(value, json!({ "title": "title", "body": "body" }));
    }

    #[test]
    fn serializes_with_html_body() {
        let envelope = MessageEnvelope::new("body")
            .with_title("title")
            .with_html_body("<p>body</p>");

        let value = serde_json::to_value(&envelope).unwrap();

        assert_eq!(
            value,
            json!({ "title": "title", "body": "body", "html_body": "<p>body</p>" })
        );
    }

    #[test]
    fn round_trips_with_metadata() {
        let mut envelope = MessageEnvelope::new("body");
        envelope
            .metadata
            .insert("severity".to_string(), json!("warn"));

        let value = serde_json::to_value(&envelope).unwrap();
        let decoded: MessageEnvelope = serde_json::from_value(value).unwrap();

        assert_eq!(decoded.metadata["severity"], json!("warn"));
    }

    #[test]
    fn round_trips_with_html_body() {
        let envelope = MessageEnvelope::new("body").with_html_body("<p>body</p>");

        let value = serde_json::to_value(&envelope).unwrap();
        let decoded: MessageEnvelope = serde_json::from_value(value).unwrap();

        assert_eq!(decoded.html_body.as_deref(), Some("<p>body</p>"));
    }
}
