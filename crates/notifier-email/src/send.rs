use crate::core::{
    DeliveryChannel, DeliveryResult, MessageEnvelope, NotifierError,
};
use lettre::AsyncTransport;

use super::{
    EmailChannel,
    message::{build_message, build_transport},
};

impl DeliveryChannel for EmailChannel {
    async fn deliver(
        &self,
        _http_client: &reqwest::Client,
        message: &MessageEnvelope,
    ) -> Result<DeliveryResult, NotifierError> {
        let email = build_message(self, message)?;
        let transport = build_transport(self)?;
        transport
            .send(email)
            .await
            .map_err(|error| NotifierError::Transport {
                provider: "email",
                message: error.to_string(),
            })?;

        Ok(DeliveryResult { http_status: None })
    }
}
