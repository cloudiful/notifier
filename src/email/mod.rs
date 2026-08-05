mod config;
mod message;
mod send;
#[cfg(test)]
mod transport;

pub use config::{EmailChannel, EmailTlsMode};

#[cfg(test)]
mod tests;
