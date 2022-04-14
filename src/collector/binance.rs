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
            ticker: if ticker.alias.trim().is_empty() {
                ticker.ticker
            } else {
                ticker.alias
            },
            price: if ticker.inverted {
                res.last_price.inverse()
            } else {
                res.last_price
            },
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

#[test]
fn test_inverse() {
    use::bigdecimal::*;
    use std::str::FromStr;

    let vals = vec![
        ("100", "0.01"),
        ("2", "0.5"),
        (".2", "5"),
        ("3.141592653", "0.3183098862435492205742690218851870990799646487459493049686604293188738877535183744268834079171116523"),
    ];
    for &(x, y) in vals.iter() {
        let a = BigDecimal::from_str(x).unwrap();
        let i = a.inverse();
        let b = BigDecimal::from_str(y).unwrap();
        assert_eq!(i, b);
        assert_eq!(BigDecimal::from(1)/&a, b);
        assert_eq!(i.inverse(), a);
        // assert_eq!(a.scale, b.scale, "scale mismatch ({} != {}", a, b);
    }
}