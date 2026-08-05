mod config;
mod send;
mod signing;

pub use config::{DingtalkChannel, DingtalkMessageType};

#[cfg(test)]
mod tests;
