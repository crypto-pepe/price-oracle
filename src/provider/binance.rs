use super::{MarketData, Provider};
use crate::config::ProviderConfig;
use crate::error::Error;
use async_trait::async_trait;
use bigdecimal::BigDecimal;
use pepe_log::error;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{thread::sleep, time::Duration};
use tokio::sync::mpsc::Sender;

const BINANCE_PROVIDER_NAME: &str = "binance";
const USD_TICKER: &str = "USDT";

#[derive(Debug, Serialize, Deserialize)]
struct Response24h {
    #[serde(rename(deserialize = "weightedAvgPrice"))]
    weighted_avg_price: BigDecimal,
    #[serde(rename(deserialize = "quoteVolume"))]
    quote_volume: BigDecimal,
}

#[derive(Debug, Clone)]
pub struct BinanceProvider {
    endpoint: String,
    tickers: Vec<String>,
    batch_delay: Duration,
    request_delay: Duration,
    client: Client,
}

impl BinanceProvider {
    pub fn new(config: &ProviderConfig) -> Self {
        BinanceProvider {
            endpoint: config.endpoint.clone(),
            tickers: config.tickers.clone(),
            batch_delay: config.delay.batch.into(),
            request_delay: config.delay.request.into(),
            client: Client::new(),
        }
    }

    async fn get_market_data(&self, ticker: String) -> Result<MarketData, Error> {
        let url = format!(
            "{}/api/v3/ticker/24hr?symbol={}{}",
            self.endpoint,
            ticker,
            USD_TICKER.to_string()
        );
        let res: Response24h = self.client.get(url).send().await?.json().await?;

        Ok(MarketData {
            provider: BINANCE_PROVIDER_NAME.to_string(),
            ticker,
            price: res.weighted_avg_price,
            quote_volume: res.quote_volume,
        })
    }
}

#[async_trait]
impl Provider for BinanceProvider {
    async fn produce(&self, tx: Sender<MarketData>) {
        loop {
            for ticker in &self.tickers {
                match self.get_market_data(ticker.clone()).await {
                    Ok(market_data) => {
                        match tx.send(market_data).await {
                            Ok(_) => {}
                            Err(e) => error!("can't push market from binance to channel: {:?}", e),
                        };
                    }
                    Err(e) => {
                        error!("can't get market from binance: {:?}", e);
                    }
                };

                sleep(self.request_delay)
            }
            sleep(self.batch_delay)
        }
    }
}
