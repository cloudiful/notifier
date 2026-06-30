use crate::core::{MessageEnvelope, NotifierError};
use lettre::{
    Message, Tokio1Executor,
    message::MultiPart,
    transport::smtp::{
        AsyncSmtpTransport,
        authentication::Credentials,
    },
};

use super::{EmailChannel, EmailTlsMode};

pub(crate) type EmailTransport = AsyncSmtpTransport<Tokio1Executor>;

pub(crate) fn build_message(
    channel: &EmailChannel,
    envelope: &MessageEnvelope,
) -> Result<Message, NotifierError> {
    let subject = envelope
        .title
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| invalid_message("email subject requires `title`"))?;
    if channel.to.is_empty() {
        return Err(invalid_message("email channel requires at least one `to` recipient"));
    }
    validate_auth_pair(channel)?;

    let mut builder = Message::builder().from(parse_mailbox(&channel.from, "from")?);
    for recipient in &channel.to {
        builder = builder.to(parse_mailbox(recipient, "to")?);
    }
    if let Some(reply_to) = channel.reply_to.as_deref() {
        builder = builder.reply_to(parse_mailbox(reply_to, "reply_to")?);
    }
    builder = builder.subject(subject);

    if let Some(html_body) = envelope.html_body.as_ref() {
        builder
            .multipart(MultiPart::alternative_plain_html(
                envelope.body.clone(),
                html_body.clone(),
            ))
            .map_err(|error| invalid_message(&format!("failed to build email body: {error}")))
    } else {
        builder
            .body(envelope.body.clone())
            .map_err(|error| invalid_message(&format!("failed to build email body: {error}")))
    }
}

pub(crate) fn build_transport(channel: &EmailChannel) -> Result<EmailTransport, NotifierError> {
    validate_auth_pair(channel)?;

    let mut builder = match channel.tls_mode {
        EmailTlsMode::ImplicitTls => AsyncSmtpTransport::<Tokio1Executor>::relay(&channel.smtp_host)
            .map_err(|error| transport_error(error.to_string()))?,
        EmailTlsMode::StartTls => {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&channel.smtp_host)
                .map_err(|error| transport_error(error.to_string()))?
        }
        EmailTlsMode::Plain => {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(channel.smtp_host.clone())
        }
    };

    if let Some(port) = channel.smtp_port {
        builder = builder.port(port);
    }

    if let (Some(username), Some(password)) = (&channel.username, &channel.password) {
        builder = builder.credentials(Credentials::new(username.clone(), password.clone()));
    }

    Ok(builder.build())
}

fn parse_mailbox(value: &str, field: &str) -> Result<lettre::message::Mailbox, NotifierError> {
    value
        .parse()
        .map_err(|error| invalid_message(&format!("invalid `{field}` address: {error}")))
}

fn validate_auth_pair(channel: &EmailChannel) -> Result<(), NotifierError> {
    match (&channel.username, &channel.password) {
        (Some(_), Some(_)) | (None, None) => Ok(()),
        _ => Err(invalid_message(
            "email authentication requires both `username` and `password`",
        )),
    }
}

fn invalid_message(message: &str) -> NotifierError {
    NotifierError::InvalidMessage {
        provider: "email",
        message: message.to_string(),
    }
}

fn transport_error(message: String) -> NotifierError {
    NotifierError::Transport {
        provider: "email",
        message,
    }
}
