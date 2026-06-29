use anyhow::{Context, Result, anyhow};
use base64::Engine;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac, digest::KeyInit};
use reqwest::{
    StatusCode,
    header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::Sha256;
use std::collections::BTreeMap;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelProvider {
    Ntfy,
    GenericWebhook,
    Dingtalk,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NtfyChannelConfig {
    pub base_url: String,
    pub topic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenericWebhookChannelConfig {
    pub url: String,
    #[serde(default)]
    pub extra_headers: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DingtalkChannelConfig {
    pub webhook_url: String,
    #[serde(default)]
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "channel_type", rename_all = "snake_case")]
pub enum ChannelConfig {
    Ntfy(NtfyChannelConfig),
    GenericWebhook(GenericWebhookChannelConfig),
    Dingtalk(DingtalkChannelConfig),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NtfyChannelSecret {
    pub auth_token: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenericWebhookChannelSecret {
    pub bearer_token: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DingtalkChannelSecret {
    pub secret: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "channel_type", rename_all = "snake_case")]
pub enum ChannelSecret {
    Ntfy(NtfyChannelSecret),
    GenericWebhook(GenericWebhookChannelSecret),
    Dingtalk(DingtalkChannelSecret),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationChannel {
    pub name: String,
    pub provider: ChannelProvider,
    pub config: ChannelConfig,
    pub secret: Option<ChannelSecret>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRuleRef {
    pub id: i64,
    pub name: String,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationStockRef {
    pub ts_code: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSignalRef {
    pub message: String,
    pub raw_message: String,
    pub metric_value: Option<f64>,
    pub threshold_value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMessage {
    pub event_id: i64,
    pub triggered_at: DateTime<Utc>,
    pub title: String,
    pub rule: NotificationRuleRef,
    pub stock: NotificationStockRef,
    pub signal: NotificationSignalRef,
}

#[derive(Debug, Clone)]
pub struct DeliveryResult {
    pub delivery_status: String,
    pub http_status: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct Notifier {
    http_client: reqwest::Client,
}

impl Notifier {
    pub fn new(http_client: reqwest::Client) -> Self {
        Self { http_client }
    }

    pub async fn send(
        &self,
        channel: &NotificationChannel,
        message: &NotificationMessage,
    ) -> Result<DeliveryResult> {
        match channel.provider {
            ChannelProvider::Ntfy => self.send_ntfy(channel, message).await,
            ChannelProvider::GenericWebhook => self.send_generic_webhook(channel, message).await,
            ChannelProvider::Dingtalk => self.send_dingtalk(channel, message).await,
        }
    }

    async fn send_ntfy(
        &self,
        channel: &NotificationChannel,
        message: &NotificationMessage,
    ) -> Result<DeliveryResult> {
        let config = match &channel.config {
            ChannelConfig::Ntfy(config) => config,
            _ => return Err(anyhow!("channel config does not match ntfy type")),
        };
        let secret = match channel.secret.as_ref() {
            Some(ChannelSecret::Ntfy(secret)) => secret.auth_token.as_deref(),
            Some(_) => return Err(anyhow!("channel secret does not match ntfy type")),
            None => None,
        };

        let mut request = self.http_client.post(format!(
            "{}/{}",
            config.base_url.trim_end_matches('/'),
            config.topic
        ));
        request = request.header("Title", &message.title);
        if let Some(token) = secret {
            request = request.bearer_auth(token);
        }
        let response = request
            .body(format_ntfy_body(message))
            .send()
            .await
            .context("failed to send ntfy request")?;
        ensure_success_status("ntfy", response.status())?;
        Ok(DeliveryResult {
            delivery_status: "delivered".to_string(),
            http_status: Some(response.status().as_u16() as i32),
        })
    }

    async fn send_generic_webhook(
        &self,
        channel: &NotificationChannel,
        message: &NotificationMessage,
    ) -> Result<DeliveryResult> {
        let config = match &channel.config {
            ChannelConfig::GenericWebhook(config) => config,
            _ => {
                return Err(anyhow!(
                    "channel config does not match generic_webhook type"
                ));
            }
        };
        let secret = match channel.secret.as_ref() {
            Some(ChannelSecret::GenericWebhook(secret)) => secret.bearer_token.as_deref(),
            Some(_) => {
                return Err(anyhow!(
                    "channel secret does not match generic_webhook type"
                ));
            }
            None => None,
        };
        validate_http_url(&config.url)?;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if let Some(token) = secret {
            let value = HeaderValue::from_str(&format!("Bearer {token}"))
                .context("failed to encode webhook bearer token header")?;
            headers.insert(AUTHORIZATION, value);
        }
        for (key, value) in &config.extra_headers {
            if is_reserved_header(key) {
                return Err(anyhow!(
                    "extra_headers cannot override reserved header `{key}`"
                ));
            }
            let header_name = HeaderName::from_bytes(key.as_bytes())
                .with_context(|| format!("invalid webhook header name `{key}`"))?;
            let header_value = HeaderValue::from_str(value)
                .with_context(|| format!("invalid webhook header value for `{key}`"))?;
            headers.insert(header_name, header_value);
        }

        let response = self
            .http_client
            .post(&config.url)
            .headers(headers)
            .json(&generic_webhook_payload(message))
            .send()
            .await
            .context("failed to send generic webhook request")?;
        ensure_success_status("generic_webhook", response.status())?;
        Ok(DeliveryResult {
            delivery_status: "delivered".to_string(),
            http_status: Some(response.status().as_u16() as i32),
        })
    }

    async fn send_dingtalk(
        &self,
        channel: &NotificationChannel,
        message: &NotificationMessage,
    ) -> Result<DeliveryResult> {
        let config = match &channel.config {
            ChannelConfig::Dingtalk(config) => config,
            _ => return Err(anyhow!("channel config does not match dingtalk type")),
        };
        let secret = match channel.secret.as_ref() {
            Some(ChannelSecret::Dingtalk(secret)) => secret.secret.as_deref(),
            Some(_) => return Err(anyhow!("channel secret does not match dingtalk type")),
            None => None,
        };
        validate_http_url(&config.webhook_url)?;

        let mut url = config.webhook_url.clone();
        if let Some(secret) = secret {
            let timestamp = Utc::now().timestamp_millis().to_string();
            let sign = sign_dingtalk(&timestamp, secret)?;
            let separator = if url.contains('?') { "&" } else { "?" };
            url.push_str(separator);
            url.push_str(&format!("timestamp={timestamp}&sign={sign}"));
        }

        let response = self
            .http_client
            .post(url)
            .json(&json!({
                "msgtype": "text",
                "text": {
                    "content": format_dingtalk_body(message, &config.keywords)?,
                }
            }))
            .send()
            .await
            .context("failed to send dingtalk webhook request")?;
        let status = response.status();
        ensure_success_status("dingtalk", status)?;
        let body: DingtalkResponse = response
            .json()
            .await
            .context("failed to decode dingtalk webhook response")?;
        if body.errcode != 0 {
            return Err(anyhow!(
                "dingtalk webhook rejected request with errcode {}: {}",
                body.errcode,
                body.errmsg
            ));
        }
        Ok(DeliveryResult {
            delivery_status: "delivered".to_string(),
            http_status: Some(status.as_u16() as i32),
        })
    }
}

#[derive(Debug, Deserialize)]
struct DingtalkResponse {
    errcode: i64,
    errmsg: String,
}

fn generic_webhook_payload(message: &NotificationMessage) -> Value {
    json!({
        "event_id": message.event_id,
        "triggered_at": message.triggered_at.to_rfc3339(),
        "title": message.title,
        "rule": {
            "id": message.rule.id,
            "name": message.rule.name,
            "mode": message.rule.mode,
        },
        "stock": {
            "ts_code": message.stock.ts_code,
            "name": message.stock.name,
        },
        "signal": {
            "message": message.signal.message,
            "raw_message": message.signal.raw_message,
            "metric_value": message.signal.metric_value,
            "threshold_value": message.signal.threshold_value,
        }
    })
}

fn format_ntfy_body(message: &NotificationMessage) -> String {
    format!(
        "{}\n{}\n{}",
        message.stock.ts_code,
        message.signal.message,
        message.triggered_at.to_rfc3339()
    )
}

fn format_dingtalk_body(message: &NotificationMessage, keywords: &[String]) -> Result<String> {
    if keywords.len() > 10 {
        return Err(anyhow!("dingtalk keywords cannot exceed 10"));
    }
    let stock_name = message.stock.name.as_deref().unwrap_or("-");
    let keyword_prefix = keywords
        .iter()
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>();
    Ok(format!(
        "{}股票: {} {}\n规则: {} ({})\n信号: {}\n当前值: {}\n阈值: {}\n触发时间: {}",
        if keyword_prefix.is_empty() {
            String::new()
        } else {
            format!("{} ", keyword_prefix.join(" "))
        },
        message.stock.ts_code,
        stock_name,
        message.rule.name,
        message.rule.mode,
        message.signal.message,
        message
            .signal
            .metric_value
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string()),
        message
            .signal
            .threshold_value
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string()),
        message.triggered_at.to_rfc3339()
    ))
}

fn sign_dingtalk(timestamp: &str, secret: &str) -> Result<String> {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).context("invalid dingtalk secret")?;
    mac.update(format!("{timestamp}\n{secret}").as_bytes());
    let encoded = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
    Ok(urlencoding::encode(&encoded).into_owned())
}

fn ensure_success_status(provider: &str, status: StatusCode) -> Result<()> {
    if status.is_success() {
        Ok(())
    } else {
        Err(anyhow!(
            "{provider} request failed with status {}",
            status.as_u16()
        ))
    }
}

fn validate_http_url(url: &str) -> Result<()> {
    let parsed = reqwest::Url::parse(url).with_context(|| format!("invalid url `{url}`"))?;
    match parsed.scheme() {
        "http" | "https" => Ok(()),
        other => Err(anyhow!("unsupported url scheme `{other}`")),
    }
}

fn is_reserved_header(header: &str) -> bool {
    header.eq_ignore_ascii_case("content-type") || header.eq_ignore_ascii_case("authorization")
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::{
        ChannelConfig, ChannelProvider, DeliveryResult, DingtalkChannelConfig,
        GenericWebhookChannelConfig, NotificationChannel, NotificationMessage, NotificationRuleRef,
        NotificationSignalRef, NotificationStockRef, generic_webhook_payload, is_reserved_header,
        sign_dingtalk,
    };

    fn sample_message() -> NotificationMessage {
        NotificationMessage {
            event_id: 1,
            triggered_at: chrono::Utc
                .with_ymd_and_hms(2026, 5, 7, 12, 34, 56)
                .unwrap(),
            title: "Stock alert 600519.SH".to_string(),
            rule: NotificationRuleRef {
                id: 2,
                name: "price".to_string(),
                mode: "simple".to_string(),
            },
            stock: NotificationStockRef {
                ts_code: "600519.SH".to_string(),
                name: Some("贵州茅台".to_string()),
            },
            signal: NotificationSignalRef {
                message: "latest_price >= 1500".to_string(),
                raw_message: "latest_price >= 1500".to_string(),
                metric_value: Some(1501.0),
                threshold_value: Some(1500.0),
            },
        }
    }

    #[test]
    fn dingtalk_sign_is_non_empty() {
        let sign = sign_dingtalk("1715000000000", "SECabc").unwrap();
        assert!(!sign.is_empty());
    }

    #[test]
    fn reserved_headers_are_detected_case_insensitively() {
        assert!(is_reserved_header("Content-Type"));
        assert!(is_reserved_header("authorization"));
        assert!(!is_reserved_header("x-cloudiful-notifier"));
    }

    #[test]
    fn generic_payload_has_expected_shape() {
        let payload = generic_webhook_payload(&sample_message());
        assert_eq!(payload["rule"]["mode"], "simple");
        assert_eq!(payload["stock"]["ts_code"], "600519.SH");
        assert_eq!(payload["signal"]["metric_value"], 1501.0);
    }

    #[test]
    fn config_types_are_constructible() {
        let _ = GenericWebhookChannelConfig {
            url: "https://example.com".to_string(),
            extra_headers: Default::default(),
        };
        let _ = DingtalkChannelConfig {
            webhook_url: "https://oapi.dingtalk.com/robot/send?access_token=abc".to_string(),
            keywords: vec!["监控报警".to_string()],
        };
        let _ = NotificationChannel {
            name: "ops".to_string(),
            provider: ChannelProvider::GenericWebhook,
            config: ChannelConfig::GenericWebhook(GenericWebhookChannelConfig {
                url: "https://example.com".to_string(),
                extra_headers: Default::default(),
            }),
            secret: None,
            enabled: true,
        };
        let _ = DeliveryResult {
            delivery_status: "delivered".to_string(),
            http_status: Some(200),
        };
    }
}
