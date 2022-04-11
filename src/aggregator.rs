use crate::collector::MarketData;
use crate::Error;
use bigdecimal::BigDecimal;
use chrono::Utc;
use std::collections::HashMap;
use std::time::Duration;

pub struct PriceAggregator {
    ttl: Duration,
    prices_map: HashMap<String, Vec<MarketData>>,
}

impl PriceAggregator {
    pub fn new(ttl: &Duration) -> Self {
        PriceAggregator {
            prices_map: HashMap::new(),
            ttl: *ttl,
        }
    }

    // consume market data (insert/update)
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

    // collect avg result
    pub fn aggregate(&self) -> Result<Vec<MarketData>, Error> {
        match self
            .prices_map
            .values()
            .cloned()
            .map(|prices| -> Option<MarketData> {
                let filtered_prices = prices
                    .iter()
                    .filter(|price| {
                        price.timestamp > Utc::now().timestamp() - (self.ttl.as_secs() as i64)
                    })
                    .cloned()
                    .collect::<Vec<_>>();

                if filtered_prices.len() == 1 {
                    filtered_prices.get(0).cloned()
                } else {
                    let volume: BigDecimal = filtered_prices
                        .iter()
                        .map(|price| price.volume.clone())
                        .sum();

                    filtered_prices.iter().fold(None, |avg, data| match avg {
                        Some(mut avg_price) => {
                            if data.timestamp > avg_price.timestamp {
                                avg_price.timestamp = data.timestamp;
                            }
                            avg_price.price +=
                                data.volume.clone() / volume.clone() * data.price.clone();
                            avg_price.volume += data.volume.clone();
                            Some(avg_price)
                        }
                        None => Some(MarketData {
                            provider: data.provider.clone(),
                            ticker: data.ticker.clone(),
                            price: data.volume.clone() / volume.clone() * data.price.clone(),
                            volume: data.volume.clone(),
                            timestamp: data.timestamp,
                        }),
                    })
                }
            })
            .collect::<Option<Vec<_>>>()
        {
            Some(prices) => Ok(prices),
            None => Ok(vec![]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PriceAggregator;
    use crate::collector::MarketData;
    use bigdecimal::{BigDecimal, FromPrimitive};
    use chrono::Utc;
    use std::str::FromStr;
    use std::time::Duration;

    #[test]
    fn empty() {
        let ttl = Duration::from_secs(1);
        assert!(PriceAggregator::new(&ttl).aggregate().unwrap().len() == 0);
    }

    #[test]
    fn old() {
        let ttl = Duration::from_secs(60);
        let mut aggregator = PriceAggregator::new(&ttl);
        let binance = MarketData {
            provider: "binance".to_string(),
            ticker: "BTC".to_string(),
            price: BigDecimal::from_f64(40000.0).unwrap(),
            volume: BigDecimal::from_f64(10.0).unwrap(),
            timestamp: Utc::now().timestamp() - 120,
        };
        let bitfinex = MarketData {
            provider: "bitfinex".to_string(),
            ticker: "BTC".to_string(),
            price: BigDecimal::from_f64(40000.0).unwrap(),
            volume: BigDecimal::from_f64(10.0).unwrap(),
            timestamp: Utc::now().timestamp(),
        };

        aggregator.consume(&binance);
        aggregator.consume(&bitfinex);
        let result = aggregator.aggregate().unwrap();
        assert_eq!(result.len(), 1);
        assert!(vec![bitfinex.clone()].iter().eq(result.iter()));
    }

    #[test]
    fn weighted_average() {
        let ttl = Duration::from_secs(60);
        let mut aggregator = PriceAggregator::new(&ttl);
        let binance = MarketData {
            provider: "binance".to_string(),
            ticker: "BTC".to_string(),
            price: BigDecimal::from_str("42580.02").unwrap(),
            volume: BigDecimal::from_str("18555.70986").unwrap(),
            timestamp: Utc::now().timestamp(),
        };
        let bitfinex = MarketData {
            provider: "bitfinex".to_string(),
            ticker: "BTC".to_string(),
            price: BigDecimal::from_f64(42562.0).unwrap(),
            volume: BigDecimal::from_f64(1929.42519104).unwrap(),
            timestamp: Utc::now().timestamp() - 1,
        };

        aggregator.consume(&binance);
        aggregator.consume(&bitfinex);
        let result = aggregator.aggregate().unwrap();
        assert_eq!(result.len(), 1);
        let data = result.get(0).unwrap();
        assert_eq!(data.provider, binance.provider);
        assert_eq!(data.ticker, binance.ticker);
        assert_eq!(data.volume, BigDecimal::from_str("20485.13505104").unwrap());
        assert_eq!(data.price, BigDecimal::from_str("42578.3227574943531715796949408716305856862537106332767936007037519948040614906956044719693742682527181817900000000000").unwrap());
        assert_eq!(data.timestamp, binance.timestamp);
    }
}
