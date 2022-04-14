use super::{CollectorConfig, PriceOracleConfig, ProvidersConfig};
use serde::{Deserialize, Serialize};
use slog_extlog_derive::SlogValue;

#[derive(Debug, Clone, Deserialize, Serialize, SlogValue)]
pub struct AppConfig {
    pub providers: ProvidersConfig,
    pub collectors: Vec<CollectorConfig>,
    pub oracle: PriceOracleConfig,
}
