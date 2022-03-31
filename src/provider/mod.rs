mod binance;
mod bitfinex;

use async_trait::async_trait;
use bigdecimal::BigDecimal;
pub use binance::BinanceProvider;
pub use bitfinex::BitfinexProvider;
use serde::Serialize;
use slog_extlog_derive::SlogValue;
use tokio::sync::mpsc::Sender;

#[derive(Debug, Clone, Serialize, SlogValue)]
pub struct MarketData {
    pub provider: String,
    pub ticker: String,
    pub price: BigDecimal,
    pub quote_volume: BigDecimal,
}

#[async_trait]
pub trait Provider: Send + Sync {
    async fn produce(&self, tx: Sender<MarketData>);
}
