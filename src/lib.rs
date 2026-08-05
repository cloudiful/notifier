mod core;

#[cfg(feature = "dingtalk")]
mod dingtalk;
#[cfg(feature = "email")]
mod email;
#[cfg(feature = "ntfy")]
mod ntfy;
#[cfg(feature = "webhook")]
mod webhook;

pub use core::{DeliveryChannel, DeliveryResult, MessageEnvelope, NotifierError};

#[cfg(feature = "dingtalk")]
pub use dingtalk::DingtalkChannel;
#[cfg(feature = "dingtalk")]
pub use dingtalk::DingtalkMessageType;
#[cfg(feature = "email")]
pub use email::{EmailChannel, EmailTlsMode};
#[cfg(feature = "ntfy")]
pub use ntfy::NtfyChannel;
#[cfg(feature = "webhook")]
pub use webhook::WebhookChannel;

#[derive(Debug, Clone)]
pub struct Notifier {
    http_client: reqwest::Client,
}

impl Notifier {
    pub fn new(http_client: reqwest::Client) -> Self {
        Self { http_client }
    }

    pub async fn send<C: DeliveryChannel>(
        &self,
        channel: &C,
        message: &MessageEnvelope,
    ) -> Result<DeliveryResult, NotifierError> {
        channel.deliver(&self.http_client, message).await
    }
}

#[cfg(test)]
mod tests {
    use super::{DeliveryChannel, DeliveryResult, MessageEnvelope, Notifier, NotifierError};

    struct StubChannel;

    impl DeliveryChannel for StubChannel {
        async fn deliver(
            &self,
            _http_client: &reqwest::Client,
            _message: &MessageEnvelope,
        ) -> Result<DeliveryResult, NotifierError> {
            Ok(DeliveryResult {
                http_status: Some(202),
            })
        }
    }

    #[tokio::test]
    async fn notifier_delegates_to_channel() {
        let notifier = Notifier::new(reqwest::Client::new());
        let message = MessageEnvelope::new("hello");

        let result = notifier.send(&StubChannel, &message).await.unwrap();

        assert_eq!(result.http_status, Some(202));
    }
}
