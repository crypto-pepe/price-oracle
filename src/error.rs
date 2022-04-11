use std::num::ParseFloatError;

use prost::EncodeError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("config load failed: {0}")]
    ConfigLoad(#[from] config::ConfigError),
    #[error("provider error: {0}")]
    Provider(String),
    #[error("collector error: {0}")]
    Collector(String),
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("parse float error: {0}")]
    ParseFloat(#[from] ParseFloatError),
    #[error("encoded error: {0}")]
    Encode(#[from] EncodeError),
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
}
