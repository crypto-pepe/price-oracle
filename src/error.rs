use std::num::ParseFloatError;

use prost::EncodeError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("config load failed: {0}")]
    ConfigLoadError(#[from] config::ConfigError),
    #[error("provider error: {0}")]
    ProviderError(String),
    #[error("collector error: {0}")]
    CollectorError(String),
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("parse float error: {0}")]
    ParseFloatError(#[from] ParseFloatError),
    #[error("encoded error: {0}")]
    EncodeError(#[from] EncodeError),
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}
