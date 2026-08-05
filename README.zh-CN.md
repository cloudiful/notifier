# cloudiful-notifier

[English](README.md)

`cloudiful-notifier` 0.3 是一个异步通知传输库，提供 ntfy、通用 JSON
Webhook、钉钉机器人和 SMTP Email 渠道。

## 功能

- 统一的 `MessageEnvelope`、`Notifier`、投递结果和错误类型。
- 钉钉文本与 Markdown 消息、Webhook 签名和响应校验。
- ntfy、通用 Webhook 和 SMTP Email 传输。
- provider feature 可选，减少不需要的依赖。

## 使用

只启用钉钉：

```toml
[dependencies]
cloudiful-notifier = { version = "0.3", default-features = false, features = ["dingtalk"] }
```

钉钉默认发送文本消息。设置
`DingtalkChannel.message_type = DingtalkMessageType::Markdown` 后，envelope
标题映射为 Markdown 标题，正文保持原样发送；Markdown 消息必须提供非空标题。

```rust
use cloudiful_notifier::{
    DingtalkChannel, DingtalkMessageType, MessageEnvelope, Notifier,
};

let channel = DingtalkChannel {
    webhook_url: "https://example.com/robot/send?access_token=...".to_string(),
    secret: None,
    keywords: Vec::new(),
    message_type: DingtalkMessageType::Markdown,
};
let message = MessageEnvelope::new("## 完成").with_title("任务完成");
let notifier = Notifier::new(reqwest::Client::new());
# let _ = notifier.send(&channel, &message).await;
```

provider 实现只负责传输，不负责应用设置、模板、规则、限流或 MCP 工具。
