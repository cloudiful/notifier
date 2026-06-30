use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use serde_json::json;

use crate::core::{
    DeliveryChannel, DeliveryResult, MessageEnvelope, NotifierError, ensure_success_status,
    validate_http_url,
};

use super::{DingtalkChannel, signing::sign};

#[derive(Debug, Deserialize)]
struct DingtalkResponse {
    errcode: i64,
    errmsg: String,
}

impl DeliveryChannel for DingtalkChannel {
    async fn deliver(
        &self,
        http_client: &reqwest::Client,
        message: &MessageEnvelope,
    ) -> Result<DeliveryResult, NotifierError> {
        validate_http_url(&self.webhook_url)?;

        let content = format_body(message, &self.keywords)?;
        let mut url = self.webhook_url.clone();
        if let Some(secret) = self.secret.as_deref() {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|error| NotifierError::InvalidMessage {
                    provider: "dingtalk",
                    message: error.to_string(),
                })?
                .as_millis()
                .to_string();
            let signature = sign(&timestamp, secret)?;
            let separator = if url.contains('?') { "&" } else { "?" };
            url.push_str(separator);
            url.push_str(&format!("timestamp={timestamp}&sign={signature}"));
        }

        let response = http_client
            .post(url)
            .json(&json!({
                "msgtype": "text",
                "text": {
                    "content": content,
                }
            }))
            .send()
            .await
            .map_err(|error| NotifierError::HttpRequest {
                provider: "dingtalk",
                message: error.to_string(),
            })?;
        let status = response.status();
        ensure_success_status("dingtalk", status)?;

        let body: DingtalkResponse =
            response
                .json()
                .await
                .map_err(|error| NotifierError::ResponseDecode {
                    provider: "dingtalk",
                    message: error.to_string(),
                })?;
        if body.errcode != 0 {
            return Err(NotifierError::ProviderRejected {
                provider: "dingtalk",
                code: body.errcode.to_string(),
                message: body.errmsg,
            });
        }

        Ok(DeliveryResult {
            http_status: Some(status.as_u16()),
        })
    }
}

fn format_body(message: &MessageEnvelope, keywords: &[String]) -> Result<String, NotifierError> {
    if keywords.len() > 10 {
        return Err(NotifierError::InvalidMessage {
            provider: "dingtalk",
            message: "keywords cannot exceed 10".to_string(),
        });
    }

    let prefix = keywords
        .iter()
        .map(|keyword| keyword.trim())
        .filter(|keyword| !keyword.is_empty())
        .collect::<Vec<_>>();
    let mut lines = Vec::new();
    if let Some(title) = message.title.as_ref() {
        lines.push(title.clone());
    }
    lines.push(message.body.clone());
    let content = lines.join("\n");

    if prefix.is_empty() {
        Ok(content)
    } else {
        Ok(format!("{} {}", prefix.join(" "), content))
    }
}
