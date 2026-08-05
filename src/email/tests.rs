use crate::core::{DeliveryChannel, DeliveryResult, MessageEnvelope, NotifierError};

use super::{
    EmailChannel, EmailTlsMode,
    message::{build_message, build_transport},
    transport::{TransportKind, transport_kind},
};

fn sample_channel() -> EmailChannel {
    EmailChannel {
        smtp_host: "smtp.example.com".to_string(),
        smtp_port: Some(2525),
        tls_mode: EmailTlsMode::ImplicitTls,
        username: Some("user".to_string()),
        password: Some("secret".to_string()),
        from: "Ops <ops@example.com>".to_string(),
        to: vec!["Alice <alice@example.com>".to_string()],
        reply_to: Some("noreply@example.com".to_string()),
    }
}

#[test]
fn email_builds_plain_text_message() {
    let message = MessageEnvelope::new("plain body").with_title("Important");

    let email = build_message(&sample_channel(), &message).unwrap();
    let formatted = String::from_utf8(email.formatted()).unwrap();

    assert!(formatted.contains("Subject: Important"));
    assert!(!formatted.contains("multipart/alternative"));
    assert!(formatted.contains("\r\n\r\nplain body"));
}

#[test]
fn email_builds_plain_and_html_message() {
    let message = MessageEnvelope::new("plain body")
        .with_title("Important")
        .with_html_body("<p>html body</p>");

    let email = build_message(&sample_channel(), &message).unwrap();
    let formatted = String::from_utf8(email.formatted()).unwrap();

    assert!(formatted.contains("Subject: Important"));
    assert!(formatted.contains("Content-Type: multipart/alternative;"));
    assert!(formatted.contains("Content-Type: text/plain; charset=utf-8"));
    assert!(formatted.contains("Content-Type: text/html; charset=utf-8"));
    assert!(formatted.contains("plain body"));
    assert!(formatted.contains("<p>html body</p>"));
}

#[test]
fn email_requires_title() {
    let error = build_message(&sample_channel(), &MessageEnvelope::new("plain body")).unwrap_err();

    assert_eq!(
        error,
        NotifierError::InvalidMessage {
            provider: "email",
            message: "email subject requires `title`".to_string(),
        }
    );
}

#[test]
fn email_requires_recipients() {
    let mut channel = sample_channel();
    channel.to.clear();

    let error = build_message(
        &channel,
        &MessageEnvelope::new("plain body").with_title("Important"),
    )
    .unwrap_err();

    assert_eq!(
        error,
        NotifierError::InvalidMessage {
            provider: "email",
            message: "email channel requires at least one `to` recipient".to_string(),
        }
    );
}

#[test]
fn email_rejects_invalid_addresses() {
    let mut channel = sample_channel();
    channel.to = vec!["not-an-email".to_string()];

    let error = build_message(
        &channel,
        &MessageEnvelope::new("plain body").with_title("Important"),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        NotifierError::InvalidMessage {
            provider: "email",
            message,
        } if message.starts_with("invalid `to` address:")
    ));
}

#[test]
fn email_requires_auth_pair() {
    let mut channel = sample_channel();
    channel.password = None;

    let error = build_message(
        &channel,
        &MessageEnvelope::new("plain body").with_title("Important"),
    )
    .unwrap_err();

    assert_eq!(
        error,
        NotifierError::InvalidMessage {
            provider: "email",
            message: "email authentication requires both `username` and `password`".to_string(),
        }
    );
}

#[test]
fn email_tls_modes_map_to_expected_transport_kind() {
    assert_eq!(
        transport_kind(EmailTlsMode::ImplicitTls),
        TransportKind::Relay
    );
    assert_eq!(
        transport_kind(EmailTlsMode::StartTls),
        TransportKind::StartTlsRelay
    );
    assert_eq!(transport_kind(EmailTlsMode::Plain), TransportKind::Plain);
}

#[test]
fn email_build_transport_allows_custom_port() {
    let channel = sample_channel();

    let _transport = build_transport(&channel).unwrap();
}

struct StubEmailChannel;

impl DeliveryChannel for StubEmailChannel {
    async fn deliver(
        &self,
        _http_client: &reqwest::Client,
        _message: &MessageEnvelope,
    ) -> Result<DeliveryResult, NotifierError> {
        Ok(DeliveryResult { http_status: None })
    }
}

#[tokio::test]
async fn facade_like_send_contract_accepts_email_style_channel() {
    let result = StubEmailChannel
        .deliver(
            &reqwest::Client::new(),
            &MessageEnvelope::new("plain body").with_title("Important"),
        )
        .await
        .unwrap();

    assert_eq!(result.http_status, None);
}
