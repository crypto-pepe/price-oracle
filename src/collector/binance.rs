use super::{MarketData, MarketDataCollector};
use crate::config::{CollectorConfig, Ticker};
use crate::error::Error;
use async_trait::async_trait;
use bigdecimal::BigDecimal;
use chrono::Utc;
use pepe_log::error;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::{sync::mpsc::Sender, time::sleep};

const BINANCE_PROVIDER_NAME: &str = "binance";

#[derive(Debug, Serialize, Deserialize)]
struct Response24h {
    #[serde(rename(deserialize = "lastPrice"))]
    last_price: BigDecimal,
    #[serde(rename(deserialize = "quoteVolume"))]
    volume: BigDecimal,
}

#[derive(Debug, Clone)]
pub struct BinanceMarketDataCollector {
    endpoint: String,
    tickers: Vec<Ticker>,
    batch_delay: Duration,
    request_delay: Duration,
    client: Client,
}

impl BinanceMarketDataCollector {
    pub fn new(config: &CollectorConfig) -> Self {
        BinanceMarketDataCollector {
            endpoint: config.endpoint.clone(),
            tickers: config.tickers.clone(),
            batch_delay: config.delay.batch.into(),
            request_delay: config.delay.request.into(),
            client: Client::new(),
        }
    }

    async fn get_market_data(&self, ticker: Ticker) -> Result<MarketData, Error> {
        let url = format!(
            "{}/api/v3/ticker/24hr?symbol={}",
            self.endpoint, ticker.ticker
        );
        let res: Response24h = self.client.get(url).send().await?.json().await?;

        Ok(MarketData {
            provider: BINANCE_PROVIDER_NAME.to_string(),
            ticker: ticker.alias,
            price: if ticker.inverted{res.last_price.inverse()} else {res.last_price},
            volume: res.volume,
            timestamp: Utc::now().timestamp(),
        })
    }
}

#[async_trait]
impl MarketDataCollector for BinanceMarketDataCollector {
    async fn collect(&self, tx: Sender<MarketData>) {
        loop {
            for ticker in &self.tickers {
                match self.get_market_data(ticker.clone()).await {
                    Ok(market_data) => {
                        match tx.send(market_data).await {
                            Ok(_) => {}
                            Err(e) => error!("can't push market from binance to channel: {}", e),
                        };
                    }
                    Err(e) => {
                        error!("can't get market from binance: {}", e);
                    }
                };

                sleep(self.request_delay).await
            }
            sleep(self.batch_delay).await
        }
    }
}
