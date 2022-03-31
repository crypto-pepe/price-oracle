use crate::provider::MarketData;
use crate::Error;
use chrono::Utc;
use std::collections::HashMap;
use std::time::Duration;

pub struct PriceOracle {
    ttl: Duration,
    prices_map: HashMap<String, Vec<MarketData>>,
}

impl PriceOracle {
    pub fn new(ttl: &Duration) -> Self {
        PriceOracle {
            prices_map: HashMap::new(),
            ttl: ttl.clone(),
        }
    }

    pub fn consume(&mut self, market_data: &MarketData) {
        match self.prices_map.get_mut(&market_data.ticker) {
            Some(prices) => {
                match prices
                    .iter_mut()
                    .find(|price| price.provider == market_data.provider)
                {
                    Some(price_record) => {
                        price_record.price = market_data.price.clone();
                        price_record.volume = market_data.volume.clone();
                        price_record.timestamp = market_data.timestamp;
                    }
                    None => prices.push(market_data.clone()),
                };
            }
            None => {
                self.prices_map
                    .insert(market_data.ticker.clone(), vec![market_data.clone()]);
            }
        };
    }

    pub fn collect(&self) -> Result<Vec<MarketData>, Error> {
        let mut result: Vec<MarketData> = vec![];
        for (_, prices) in self.prices_map.iter() {
            let filtered_prices: Vec<MarketData> = prices
                .iter()
                .filter(|price| {
                    price.timestamp > Utc::now().timestamp() - (self.ttl.as_secs() as i64)
                })
                .map(|price| price.clone())
                .collect();

            let avg_price: Option<MarketData> =
                filtered_prices.iter().fold(None, |avg, price| match avg {
                    Some(mut avg_price) => {
                        if price.timestamp > avg_price.timestamp {
                            avg_price.timestamp = price.timestamp;
                            avg_price.provider = price.provider.clone();
                        }

                        avg_price.price += price.price.clone() * price.volume.clone();
                        avg_price.volume += price.volume.clone();
                        Some(avg_price)
                    }
                    None => Some(price.clone()),
                });

            if let Some(mut data) = avg_price {
                if filtered_prices.len() > 1 {
                    data.price = data.price / data.volume.clone();
                }

                result.push(data);
            }
        }

        Ok(result)
    }
}
