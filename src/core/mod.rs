mod channel;
mod error;
mod message;
mod result;
#[cfg(any(feature = "dingtalk", feature = "webhook", feature = "ntfy"))]
mod util;

pub use channel::DeliveryChannel;
pub use error::NotifierError;
pub use message::MessageEnvelope;
pub use result::DeliveryResult;
#[cfg(feature = "webhook")]
pub use util::is_reserved_header;
#[cfg(any(feature = "dingtalk", feature = "webhook", feature = "ntfy"))]
pub use util::{ensure_success_status, validate_http_url};
