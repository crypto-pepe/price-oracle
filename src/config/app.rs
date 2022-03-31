use super::ProviderConfig;
use serde::{Deserialize, Serialize};
use slog_extlog_derive::SlogValue;

#[derive(Debug, Clone, Deserialize, Serialize, SlogValue)]
pub struct AppConfig {
    pub providers: Vec<ProviderConfig>,
}
