use super::{MarketData, Provider};
use crate::config::ProviderConfig;
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
const USD_TICKER: &str = "USD";

#[derive(Debug, Clone)]
pub struct BitfinexProvider {
    endpoint: String,
    tickers: Vec<String>,
    batch_delay: Duration,
    request_delay: Duration,
    client: Client,
}

impl BitfinexProvider {
    pub fn new(config: &ProviderConfig) -> Self {
        BitfinexProvider {
            endpoint: config.endpoint.clone(),
            tickers: config.tickers.clone(),
            batch_delay: config.delay.batch.into(),
            request_delay: config.delay.request.into(),
            client: Client::new(),
        }
    }

    async fn get_market_data(&self, ticker: String) -> Result<MarketData, Error> {
        let url = format!(
            "{}/v2/ticker/t{}{}",
            self.endpoint,
            ticker,
            USD_TICKER.to_string()
        );
        let res: Vec<f64> = self.client.get(url).send().await?.json().await?;
        let price = res
            .get(6)
            .ok_or(Error::ProviderError(String::from("can't decode price")))
            .map(|num| BigDecimal::from_f64(num.clone()))?
            .ok_or(Error::ProviderError(String::from(
                "can't decode price format",
            )))?;
        let volume = res
            .get(7)
            .ok_or(Error::ProviderError(String::from("can't decode volume")))
            .map(|num| BigDecimal::from_f64(num.clone()))?
            .ok_or(Error::ProviderError(String::from(
                "can't decode volume format",
            )))?;

        Ok(MarketData {
            provider: BITFINEX_PROVIDER_NAME.to_string(),
            ticker,
            price,
            volume: volume,
            timestamp: Utc::now().timestamp(),
        })
    }
}

#[async_trait]
impl Provider for BitfinexProvider {
    async fn produce(&self, tx: Sender<MarketData>) {
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
