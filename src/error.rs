use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("config load failed: {0}")]
    ConfigLoadError(#[from] config::ConfigError),
    #[error("provider error: {0}")]
    ProviderError(String),
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}
