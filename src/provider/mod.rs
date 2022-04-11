use std::sync::Arc;

use crate::{collector::MarketData, config::ProvidersConfig, error::Error};
use async_trait::async_trait;

use self::p2p::P2PProvider;

mod p2p;

#[async_trait]
pub trait Provider: Send + Sync {
    async fn send(&self, prices: &Vec<MarketData>) -> Result<(), Error>;
}

pub fn init_providers(config: &ProvidersConfig) -> Result<Vec<Arc<dyn Provider>>, Error> {
    config
        .p2p
        .iter()
        .map(|config| -> Result<Arc<dyn Provider>, Error> {
            Ok(Arc::new(P2PProvider::new(config)))
        })
        .collect::<Result<Vec<Arc<dyn Provider>>, Error>>()
}
