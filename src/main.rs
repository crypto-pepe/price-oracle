use crate::{
    config::AppConfig,
    error::Error,
    price_oracle::PriceOracle,
    provider::{BinanceProvider, BitfinexProvider, MarketData, Provider},
};
use pepe_config::load;
use pepe_log::{error, info};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tokio::time::sleep;

mod config;
mod error;
mod price_oracle;
mod provider;

const DEFAULT_CONFIG_PATH: &str = include_str!("../config.yaml");

#[tokio::main]
async fn main() -> Result<(), Error> {
    info!("start application");

    // load config from file
    let app_config: AppConfig = load(DEFAULT_CONFIG_PATH, ::config::FileFormat::Yaml)?;
    info!("config loaded"; "config" => &app_config);

    // init providers from configs
    let providers = app_config
        .providers
        .iter()
        .filter(|provider_config| provider_config.enabled)
        .map(|provider_config| -> Result<Arc<dyn Provider>, Error> {
            match provider_config.kind.as_str() {
                "binance" => Ok(Arc::new(BinanceProvider::new(provider_config))),
                "bitfinex" => Ok(Arc::new(BitfinexProvider::new(provider_config))),
                _ => Err(Error::ProviderError(String::from("unsupported kind"))),
            }
        })
        .collect::<Result<Vec<Arc<dyn Provider>>, Error>>()?;

    let price_oracle = Arc::new(RwLock::new(PriceOracle::new(
        &app_config.price_oracle.ttl.into(),
    )));
    let (tx, mut rx) = mpsc::channel::<MarketData>(100);
    // start providers
    for provider_wrapper in &providers {
        let sender = tx.clone();
        let provider = provider_wrapper.clone();

        tokio::spawn(async move {
            provider.clone().produce(sender).await;
        });
    }

    let price_oracle_consumer = price_oracle.clone();
    tokio::spawn(async move {
        loop {
            sleep(app_config.price_oracle.delay.into()).await;

            let oracle = price_oracle_consumer.read().await;
            match oracle.collect() {
                Ok(prices) => {
                    for price in prices.iter() {
                        info!("provider market data"; "price" => &price);
                    }
                    // match swarm.behaviour_mut().gossipsub.publish(topic.clone(), []) {
                    //     Ok(id) => {
                    //         debug!("publish data success"; "id" => id.to_string());
                    //     }
                    //     Err(e) => {
                    //         error!("can't collect prices: {}", e);
                    //     }
                    // };
                }
                Err(e) => error!("can't collect prices: {}", e),
            };
        }
    });

    loop {
        tokio::select! {
            data = rx.recv() => {
                match data {
                Some(market_data) => {
                    let mut oracle = price_oracle.write().await;
                    oracle.consume(&market_data);
                }
                None => {
                    error!("can't get market data from channel");
                    break;
                }
                }
            }
        }
    }

    Ok(())
}
