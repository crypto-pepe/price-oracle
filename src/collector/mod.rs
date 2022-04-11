use crate::{config::CollectorConfig, error::Error};
use async_trait::async_trait;
use bigdecimal::BigDecimal;
use serde::Serialize;
use slog_extlog_derive::SlogValue;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

mod binance;
mod bitfinex;

#[derive(Debug, Clone, Serialize, SlogValue, PartialEq)]
pub struct MarketData {
    pub provider: String,
    pub ticker: String,
    pub price: BigDecimal,
    pub volume: BigDecimal,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, SlogValue)]
pub struct MarketDataVec {
    pub prices: Vec<MarketData>,
}

#[async_trait]
pub trait MarketDataCollector: Send + Sync {
    async fn collect(&self, tx: Sender<MarketData>);
}

pub fn init_collectors(
    config: &Vec<CollectorConfig>,
) -> Result<Vec<Arc<dyn MarketDataCollector>>, Error> {
    config
        .iter()
        .filter(|provider_config| provider_config.enabled)
        .map(
            |collector_config| -> Result<Arc<dyn MarketDataCollector>, Error> {
                match collector_config.kind.as_str() {
                    "binance" => Ok(Arc::new(binance::BinanceMarketDataCollector::new(
                        collector_config,
                    ))),
                    "bitfinex" => Ok(Arc::new(bitfinex::BitfinexMarketDataCollector::new(
                        collector_config,
                    ))),
                    _ => Err(Error::CollectorError(String::from("unsupported kind"))),
                }
            },
        )
        .collect::<Result<Vec<Arc<dyn MarketDataCollector>>, Error>>()
}
