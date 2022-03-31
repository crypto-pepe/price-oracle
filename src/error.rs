use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("config load failed: {0}")]
    ConfigLoadError(#[from] config::ConfigError),
    #[error("provider error: {0}")]
    ProviderError(String),
    // #[error("tokio send error: {0}")]
    // SendError(#[from] SendError),
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    // #[error("IO error: {0}")]
    // IOError(#[from] std::io::Error),
    // #[error("anyhow error: {0}")]
    // AnyhowError(#[from] anyhow::Error),
    // #[error("join error: {0}")]
    // TokioJoinError(#[from] tokio::task::JoinError),
    // #[error("serde json error: {0}")]
    // SerdeJSONError(#[from] serde_json::error::Error),
}
