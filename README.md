# cloudiful-notifier

`cloudiful-notifier` is a small async delivery library for outbound notifications.

Current providers:

- `ntfy`
- generic JSON webhooks
- DingTalk custom robots
- SMTP email

The crate focuses on transport concerns only:

- provider config
- provider-neutral message envelopes
- request signing and header handling
- uniform delivery results

It intentionally does not own application-specific concepts such as user settings,
notification templates, rule storage, or MCP tool surfaces.

## Internal layout

- `cloudiful-notifier`: only published crate, with `Notifier`
- internal core module: shared message, result, error, and trait types
- internal `ntfy` module
- internal generic JSON webhook module
- internal DingTalk module
- internal SMTP email module

## Features

Default features include all providers.

```toml
[dependencies]
cloudiful-notifier = { version = "0.2", default-features = false, features = ["webhook"] }
```

Available provider features:

- `ntfy`
- `webhook`
- `dingtalk`
- `email`

## Example

```rust
use cloudiful_notifier::{
    MessageEnvelope, Notifier, WebhookChannel,
};
use serde_json::json;
use std::collections::BTreeMap;

let mut extra_headers = BTreeMap::new();
extra_headers.insert("x-source".to_string(), "pricing".to_string());

let channel = WebhookChannel {
    url: "https://example.com/hooks/alerts".to_string(),
    bearer_token: Some("token-123".to_string()),
    extra_headers,
};

let mut message = MessageEnvelope::new("Threshold exceeded").with_title("Market alert");
message
    .metadata
    .insert("symbol".to_string(), json!("600519.SH"));
message
    .metadata
    .insert("threshold".to_string(), json!(1500.0));

let client = reqwest::Client::new();
let notifier = Notifier::new(client);
# let _ = notifier.send(&channel, &message).await;
```

Webhook payloads use the generic envelope shape:

```json
{
  "title": "Market alert",
  "body": "Threshold exceeded",
  "metadata": {
    "symbol": "600519.SH",
    "threshold": 1500.0
  }
}
```

Text-oriented providers treat the envelope differently:

- `ntfy`: `title` maps to the `Title` header, `body` is sent as-is
- `dingtalk`: message text is `title + "\n" + body` when a title exists
- `email`: `title` maps to `Subject`, `body` is plain text, optional `html_body` adds an HTML alternative part

## Email example

```rust
use cloudiful_notifier::{
    EmailChannel, EmailTlsMode, MessageEnvelope, Notifier,
};

let channel = EmailChannel {
    smtp_host: "smtp.example.com".to_string(),
    smtp_port: Some(587),
    tls_mode: EmailTlsMode::StartTls,
    username: Some("smtp-user".to_string()),
    password: Some("smtp-pass".to_string()),
    from: "Ops <ops@example.com>".to_string(),
    to: vec!["alice@example.com".to_string()],
    reply_to: Some("noreply@example.com".to_string()),
};

let message = MessageEnvelope::new("Threshold exceeded")
    .with_title("Market alert")
    .with_html_body("<p><strong>Threshold exceeded</strong></p>");

let client = reqwest::Client::new();
let notifier = Notifier::new(client);
# let _ = notifier.send(&channel, &message).await;
```

## Publishing

This repository publishes a single crate: `cloudiful-notifier`.
Provider and core implementations stay internal to the package.
