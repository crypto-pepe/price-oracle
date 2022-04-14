use super::{MarketData, MarketDataCollector};
use crate::config::{CollectorConfig, Ticker};
use crate::error::Error;
use async_trait::async_trait;
use bigdecimal::BigDecimal;
use bigdecimal::FromPrimitive;
use chrono::Utc;
use pepe_log::error;
use reqwest::Client;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;

const BITFINEX_PROVIDER_NAME: &str = "bitfinex";

#[derive(Debug, Clone)]
pub struct BitfinexMarketDataCollector {
    endpoint: String,
    tickers: Vec<Ticker>,
    batch_delay: Duration,
    request_delay: Duration,
    client: Client,
}

impl BitfinexMarketDataCollector {
    pub fn new(config: &CollectorConfig) -> Self {
        BitfinexMarketDataCollector {
            endpoint: config.endpoint.clone(),
            tickers: config.tickers.clone(),
            batch_delay: config.delay.batch.into(),
            request_delay: config.delay.request.into(),
            client: Client::new(),
        }
    }

    async fn get_market_data(&self, ticker: Ticker) -> Result<MarketData, Error> {
        let url = format!("{}/v2/ticker/t{}", self.endpoint, ticker.ticker);
        let res: Vec<f64> = self.client.get(url).send().await?.json().await?;
        let price = res
            .get(6)
            .ok_or_else(|| Error::Provider(String::from("can't decode price")))
            .map(|num| BigDecimal::from_f64(*num))?
            .ok_or_else(|| Error::Collector(String::from("can't decode price format")))?;
        let volume = res
            .get(7)
            .ok_or_else(|| Error::Provider(String::from("can't decode volume")))
            .map(|num| BigDecimal::from_f64(*num))?
            .ok_or_else(|| Error::Collector(String::from("can't decode volume format")))?;
        Ok(MarketData {
            provider: BITFINEX_PROVIDER_NAME.to_string(),
            ticker: if ticker.alias.trim().is_empty() {
                ticker.ticker
            } else {
                ticker.alias
            },
            price: if ticker.inverted {
                price.inverse()
            } else {
                price
            },
            volume,
            timestamp: Utc::now().timestamp(),
        })
    }
}

#[async_trait]
impl MarketDataCollector for BitfinexMarketDataCollector {
    async fn collect(&self, tx: Sender<MarketData>) {
        loop {
            for ticker in &self.tickers {
                match self.get_market_data(ticker.clone()).await {
                    Ok(market_data) => {
                        match tx.send(market_data).await {
                            Ok(_) => {}
                            Err(e) => error!("can't push market from bitfinex to channel: {}", e),
                        };
                    }
                    Err(e) => {
                        error!("can't get market from bitfinex: {}", e);
                    }
                };

                sleep(self.request_delay).await
            }
            sleep(self.batch_delay).await
        }
    }
}
