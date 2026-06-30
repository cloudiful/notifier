use crate::NotifierError;

pub fn ensure_success_status(
    provider: &'static str,
    status: reqwest::StatusCode,
) -> Result<(), NotifierError> {
    if status.is_success() {
        Ok(())
    } else {
        Err(NotifierError::HttpStatus {
            provider,
            status: status.as_u16(),
        })
    }
}

pub fn validate_http_url(url: &str) -> Result<(), NotifierError> {
    let parsed = reqwest::Url::parse(url).map_err(|error| NotifierError::InvalidUrl {
        url: url.to_string(),
        message: error.to_string(),
    })?;
    match parsed.scheme() {
        "http" | "https" => Ok(()),
        other => Err(NotifierError::UnsupportedUrlScheme {
            scheme: other.to_string(),
        }),
    }
}

pub fn is_reserved_header(header: &str) -> bool {
    header.eq_ignore_ascii_case("content-type") || header.eq_ignore_ascii_case("authorization")
}
