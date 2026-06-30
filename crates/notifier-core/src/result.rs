#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeliveryResult {
    pub http_status: Option<u16>,
}
