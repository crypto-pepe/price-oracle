# price-oracle

Ticker volume-weighted average price oracle

# Configuration

| env               | default | description                                                        |
| ----------------- | ------- | ------------------------------------------------------------------ |
| `RUST_LOG_FORMAT` | `json`  | logging format (`plain` or `json`)                                 |
| `RUST_LOG`        | `error` | [log level](https://docs.rs/slog-envlogger/latest/slog_envlogger/) |
| `CONFIG_PATH`     | `None`  | config file path                                                   |

default config.yaml

```
collectors:
  - kind: binance
    enabled: true
    endpoint: "https://api.binance.com"
    delay:
      batch: 5s
      request: 100ms
    tickers:
      - ticker: "BTCUSDT"
        alias: "BTC"
        inverted: false
      - ticker: "ETHUSDT"
        alias: "ETH"
        inverted: false
      - ticker: "WAVESUSDT"
        alias: "WAVES"
        inverted: false 
  - kind: bitfinex
    enabled: true
    endpoint: "https://api-pub.bitfinex.com"
    delay:
      batch: 5s
      request: 100ms
    tickers:
      - ticker: "BTCUSD"
        alias: "BTC"
        inverted: false
      - ticker: "ETHUSD"
        alias: "ETH"
        inverted: false

oracle:
  delay: 5s
  ttl: 1m

providers:
  p2p:
    - topic: pepe:prices
      endpoint: "http://localhost:3000"
```

### `Collector`

| fieled            | type       | description                               |
| ------------------| ---------- | ----------------------------------------- |
| `kind`            | `string`   | kind of collector `binance` or `bitfinex` |
| `enabled`         | `bool`     | enable/disable collector                  |
| `endpoint`        | `string`   | collect endpoint                          |
| `delay.batch`     | `duration` | timeout between batch requests            |
| `delay.request`   | `duration` | timeout between requests in batch         |
| `tickers.ticker`  | `string`   | collected tickers pair (BTCUSD, etc)      |
| `tickers.alias`   | `string`   | ticker pair alias to display              |
| `tickers.inverted`| `bool`     | if price should be reciprocal (1/x)       |
