use super::delay::DelayConfig;
use super::ticker::Ticker;
use serde::{Deserialize, Serialize};
use slog_extlog_derive::SlogValue;

#[derive(Debug, Clone, Deserialize, Serialize, SlogValue)]
pub struct CollectorConfig {
    pub kind: String,
    pub enabled: bool,
    pub endpoint: String,
    pub delay: DelayConfig,
    pub tickers: Vec<Ticker>,
}
