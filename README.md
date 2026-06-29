# cloudiful-notifier

`cloudiful-notifier` is a small async delivery library for outbound notifications.

Current providers:

- `ntfy`
- generic JSON webhooks
- DingTalk custom robots

The crate focuses on transport concerns only:

- provider config and secret types
- payload shaping
- request signing and header handling
- uniform delivery results

It intentionally does not own application-specific concepts such as user settings,
notification templates, rule storage, or MCP tool surfaces.

## Example

```rust
use chrono::Utc;
use cloudiful_notifier::{
    ChannelConfig, ChannelProvider, NotificationChannel, NotificationMessage,
    NotificationRuleRef, NotificationSignalRef, NotificationStockRef, Notifier,
    NtfyChannelConfig,
};

let channel = NotificationChannel {
    name: "ops".to_string(),
    provider: ChannelProvider::Ntfy,
    config: ChannelConfig::Ntfy(NtfyChannelConfig {
        base_url: "https://ntfy.sh".to_string(),
        topic: "alerts".to_string(),
    }),
    secret: None,
    enabled: true,
};

let message = NotificationMessage {
    event_id: 1,
    triggered_at: Utc::now(),
    title: "Stock alert 600519.SH".to_string(),
    rule: NotificationRuleRef {
        id: 2,
        name: "price break".to_string(),
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
};

let client = reqwest::Client::new();
let notifier = Notifier::new(client);
# let _ = (&channel, &message, &notifier);
```
