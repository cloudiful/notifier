use crate::EmailTlsMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TransportKind {
    Relay,
    StartTlsRelay,
    Plain,
}

pub(crate) fn transport_kind(tls_mode: EmailTlsMode) -> TransportKind {
    match tls_mode {
        EmailTlsMode::ImplicitTls => TransportKind::Relay,
        EmailTlsMode::StartTls => TransportKind::StartTlsRelay,
        EmailTlsMode::Plain => TransportKind::Plain,
    }
}
