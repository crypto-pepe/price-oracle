use std::{sync::Arc, thread::sleep, time::Duration};

use crate::{
    config::AppConfig,
    error::Error,
    provider::{BinanceProvider, BitfinexProvider, MarketData, Provider},
};
use pepe_config::load;
use pepe_log::{error, info};
use tokio::sync::mpsc;

mod config;
mod error;
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

    let (tx, mut rx) = mpsc::channel::<MarketData>(100);
    // start providers
    for provider_wrapper in &providers {
        let sender = tx.clone();
        let provider = provider_wrapper.clone();

        tokio::spawn(async move {
            provider.clone().produce(sender).await;
        });
    }

    loop {
        match rx.recv().await {
            Some(market_data) => {
                info!("provider market data"; "data" => &market_data);
            }
            None => {
                error!("can't get market data from channel");
                break;
            }
        }
    }

    sleep(Duration::from_secs(1));
    Ok(())
}
