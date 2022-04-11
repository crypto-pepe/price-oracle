use self::oracle::Price;
use super::Provider;
use crate::Error;
use crate::{config::P2PProxyProviderConfig, provider::MarketData};
use async_trait::async_trait;
use bigdecimal::ToPrimitive;
use prost::Message;
use reqwest::{Client, StatusCode};
use serde::Serialize;

pub mod oracle {
    include!(concat!(env!("OUT_DIR"), "/oracle.rs"));
}

const P2P_PUBSUB_PUBLISH_URL: &str = "/pubsub/publish";

#[derive(Debug, Clone, Serialize)]
pub struct P2PRequest {
    topic: String,
    data: String,
}

#[derive(Debug, Clone)]
pub struct P2PProvider {
    endpoint: String,
    topic: String,
    client: Client,
}

impl P2PProvider {
    pub fn new(config: &P2PProxyProviderConfig) -> Self {
        P2PProvider {
            endpoint: format!(
                "{}{}",
                config.endpoint.clone(),
                P2P_PUBSUB_PUBLISH_URL.to_string()
            ),
            topic: config.topic.clone(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl Provider for P2PProvider {
    async fn send(&self, prices: &Vec<MarketData>) -> Result<(), Error> {
        let mut binary_data = vec![];
        oracle::Prices {
            prices: prices
                .iter()
                .map(|data| {
                    Ok(Price {
                        ticker: data.ticker.clone(),
                        price: data
                            .price
                            .to_f64()
                            .ok_or(Error::ProviderError("can't encode price".to_string()))?,
                        timestamp: data.timestamp,
                    })
                })
                .collect::<Result<_, Error>>()?,
        }
        .encode(&mut binary_data)?;

        match self
            .client
            .post(self.endpoint.clone())
            .json(&P2PRequest {
                topic: self.topic.clone(),
                data: base64::encode(binary_data),
            })
            .send()
            .await?
            .status()
        {
            StatusCode::OK => Ok(()),
            status_code => Err(Error::ProviderError(format!(
                "can't feed data to proxy, status code: {}",
                status_code
            ))),
        }
    }
}
