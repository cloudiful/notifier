use cloudiful_notifier_core::{
    DeliveryChannel, DeliveryResult, MessageEnvelope, NotifierError, ensure_success_status,
    validate_http_url,
};

use crate::NtfyChannel;

impl DeliveryChannel for NtfyChannel {
    async fn deliver(
        &self,
        http_client: &reqwest::Client,
        message: &MessageEnvelope,
    ) -> Result<DeliveryResult, NotifierError> {
        validate_http_url(&self.base_url)?;

        let mut request = http_client.post(format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            self.topic
        ));
        if let Some(title) = message.title.as_ref() {
            request = request.header("Title", title);
        }
        if let Some(token) = self.auth_token.as_deref() {
            request = request.bearer_auth(token);
        }

        let response = request
            .body(message.body.clone())
            .send()
            .await
            .map_err(|error| NotifierError::HttpRequest {
                provider: "ntfy",
                message: error.to_string(),
            })?;
        ensure_success_status("ntfy", response.status())?;

        Ok(DeliveryResult {
            http_status: Some(response.status().as_u16()),
        })
    }
}
