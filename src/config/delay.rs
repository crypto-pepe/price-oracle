use duration_string::DurationString;
use serde::{Deserialize, Serialize};
use slog_extlog_derive::SlogValue;

#[derive(Debug, Clone, Deserialize, Serialize, SlogValue)]
pub struct DelayConfig {
    pub request: DurationString,
    pub batch: DurationString,
}
