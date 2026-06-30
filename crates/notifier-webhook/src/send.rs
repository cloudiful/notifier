use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};

use crate::core::{
    DeliveryChannel, DeliveryResult, MessageEnvelope, NotifierError, ensure_success_status,
    is_reserved_header, validate_http_url,
};

use super::WebhookChannel;

impl DeliveryChannel for WebhookChannel {
    async fn deliver(
        &self,
        http_client: &reqwest::Client,
        message: &MessageEnvelope,
    ) -> Result<DeliveryResult, NotifierError> {
        validate_http_url(&self.url)?;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if let Some(token) = self.bearer_token.as_deref() {
            let value = HeaderValue::from_str(&format!("Bearer {token}")).map_err(|error| {
                NotifierError::InvalidHeaderValue {
                    header: "authorization".to_string(),
                    message: error.to_string(),
                }
            })?;
            headers.insert(AUTHORIZATION, value);
        }

        for (key, value) in &self.extra_headers {
            if is_reserved_header(key) {
                return Err(NotifierError::ReservedHeader {
                    header: key.to_string(),
                });
            }

            let header_name = HeaderName::from_bytes(key.as_bytes()).map_err(|error| {
                NotifierError::InvalidHeaderName {
                    header: key.to_string(),
                    message: error.to_string(),
                }
            })?;
            let header_value = HeaderValue::from_str(value).map_err(|error| {
                NotifierError::InvalidHeaderValue {
                    header: key.to_string(),
                    message: error.to_string(),
                }
            })?;
            headers.insert(header_name, header_value);
        }

        let response = http_client
            .post(&self.url)
            .headers(headers)
            .json(message)
            .send()
            .await
            .map_err(|error| NotifierError::HttpRequest {
                provider: "webhook",
                message: error.to_string(),
            })?;
        ensure_success_status("webhook", response.status())?;

        Ok(DeliveryResult {
            http_status: Some(response.status().as_u16()),
        })
    }
}
