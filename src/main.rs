use crate::aggregator::PriceAggregator;
use crate::collector::{MarketData, MarketDataVec};
use crate::provider::init_providers;
use crate::{collector::init_collectors, config::AppConfig, error::Error};
use futures::future::try_join_all;
use pepe_config::load;
use pepe_log::{error, info};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tokio::time::sleep;

mod aggregator;
mod collector;
mod config;
mod error;
mod provider;

const DEFAULT_CONFIG_PATH: &str = include_str!("../config.yaml");

#[tokio::main]
async fn main() -> Result<(), Error> {
    info!("start application");

    let app_config: AppConfig = load(DEFAULT_CONFIG_PATH, ::config::FileFormat::Yaml)?;
    info!("config loaded"; "config" => &app_config);

    let collectors = init_collectors(&app_config.collectors)?;
    let providers = init_providers(&app_config.providers)?;
    let price_oracle = Arc::new(RwLock::new(PriceAggregator::new(
        &app_config.oracle.ttl.into(),
    )));

    let (tx, mut rx) = mpsc::channel::<MarketData>(100);
    for collector in &collectors {
        let sender = tx.clone();
        let collector = collector.clone();

        tokio::spawn(async move {
            collector.clone().collect(sender).await;
        });
    }

    let price_oracle_consumer = price_oracle.clone();
    tokio::spawn(async move {
        loop {
            sleep(app_config.oracle.delay.into()).await;
            let oracle = price_oracle_consumer.read().await;
            match oracle.aggregate() {
                Ok(prices) => {
                    info!("new market data"; "prices" => MarketDataVec{prices: prices.clone()});

                    if let Err(e) = try_join_all(
                        providers
                            .iter()
                            .map(|provider| provider.send(&prices))
                            .collect::<Vec<_>>(),
                    )
                    .await
                    {
                        error!("can't feed prices: {}", e)
                    };
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
