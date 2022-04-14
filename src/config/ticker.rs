use serde::{Deserialize, Serialize};
use slog_extlog_derive::SlogValue;

#[derive(Debug, Clone, Deserialize, Serialize, SlogValue)]
pub struct Ticker {
    pub ticker: String,
    pub alias: String,
    pub inverted: bool, 
}
