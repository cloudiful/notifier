use crate::{DeliveryResult, MessageEnvelope, NotifierError};

#[allow(async_fn_in_trait)]
pub trait DeliveryChannel {
    async fn deliver(
        &self,
        http_client: &reqwest::Client,
        message: &MessageEnvelope,
    ) -> Result<DeliveryResult, NotifierError>;
}
