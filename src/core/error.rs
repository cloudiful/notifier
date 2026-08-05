use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotifierError {
    InvalidUrl {
        url: String,
        message: String,
    },
    UnsupportedUrlScheme {
        scheme: String,
    },
    InvalidHeaderName {
        header: String,
        message: String,
    },
    InvalidHeaderValue {
        header: String,
        message: String,
    },
    ReservedHeader {
        header: String,
    },
    InvalidSecret {
        provider: &'static str,
        message: String,
    },
    HttpRequest {
        provider: &'static str,
        message: String,
    },
    Transport {
        provider: &'static str,
        message: String,
    },
    HttpStatus {
        provider: &'static str,
        status: u16,
    },
    ProviderRejected {
        provider: &'static str,
        code: String,
        message: String,
    },
    InvalidMessage {
        provider: &'static str,
        message: String,
    },
    ResponseDecode {
        provider: &'static str,
        message: String,
    },
}

impl Display for NotifierError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidUrl { url, message } => {
                write!(f, "invalid url `{url}`: {message}")
            }
            Self::UnsupportedUrlScheme { scheme } => {
                write!(f, "unsupported url scheme `{scheme}`")
            }
            Self::InvalidHeaderName { header, message } => {
                write!(f, "invalid header name `{header}`: {message}")
            }
            Self::InvalidHeaderValue { header, message } => {
                write!(f, "invalid header value for `{header}`: {message}")
            }
            Self::ReservedHeader { header } => {
                write!(f, "reserved header `{header}` cannot be overridden")
            }
            Self::InvalidSecret { provider, message } => {
                write!(f, "{provider} secret is invalid: {message}")
            }
            Self::HttpRequest { provider, message } => {
                write!(f, "{provider} request failed: {message}")
            }
            Self::Transport { provider, message } => {
                write!(f, "{provider} transport failed: {message}")
            }
            Self::HttpStatus { provider, status } => {
                write!(f, "{provider} request failed with status {status}")
            }
            Self::ProviderRejected {
                provider,
                code,
                message,
            } => {
                write!(f, "{provider} rejected request with code {code}: {message}")
            }
            Self::InvalidMessage { provider, message } => {
                write!(f, "{provider} message is invalid: {message}")
            }
            Self::ResponseDecode { provider, message } => {
                write!(f, "failed to decode {provider} response: {message}")
            }
        }
    }
}

impl std::error::Error for NotifierError {}

#[cfg(test)]
mod tests {
    use super::NotifierError;

    #[test]
    fn renders_invalid_url_error() {
        let error = NotifierError::InvalidUrl {
            url: "ftp://example.com".to_string(),
            message: "relative URL without a base".to_string(),
        };

        assert!(error.to_string().contains("invalid url"));
    }

    #[test]
    fn renders_provider_rejected_error() {
        let error = NotifierError::ProviderRejected {
            provider: "dingtalk",
            code: "310000".to_string(),
            message: "signature error".to_string(),
        };

        assert!(error.to_string().contains("dingtalk rejected request"));
    }
}
