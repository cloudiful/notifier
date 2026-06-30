use base64::Engine;
use hmac::{Hmac, Mac, digest::KeyInit};
use sha2::Sha256;

use cloudiful_notifier_core::NotifierError;

type HmacSha256 = Hmac<Sha256>;

pub fn sign(timestamp: &str, secret: &str) -> Result<String, NotifierError> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).map_err(|error| {
        NotifierError::InvalidSecret {
            provider: "dingtalk",
            message: error.to_string(),
        }
    })?;
    mac.update(format!("{timestamp}\n{secret}").as_bytes());
    let encoded = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
    Ok(urlencoding::encode(&encoded).into_owned())
}
