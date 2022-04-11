use serde::{Deserialize, Serialize};
use slog_extlog_derive::SlogValue;

#[derive(Debug, Clone, Deserialize, Serialize, SlogValue)]
pub struct ProvidersConfig {
    pub p2p: Vec<P2PProxyProviderConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, SlogValue)]
pub struct P2PProxyProviderConfig {
    pub topic: String,
    pub endpoint: String,
}
