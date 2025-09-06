Get the latest transactions across the entire network.

Returns recent transactions from all addresses, useful for:
- Network activity monitoring
- Real-time transaction feeds
- Market activity analysis
- System health monitoring

## Parameters

- `network`: Network to use (REGTEST or MAINNET)
- `limit`: Maximum number of transactions to return (1-500, default: 10)
- `offset`: Number of transactions to skip for pagination (default: 0)
- `from_timestamp`: Return transactions created at or after this ISO timestamp (optional)
- `to_timestamp`: Return transactions created at or before this ISO timestamp (optional)

## Example

> **Note**: The values used in this example are for documentation testing purposes.

```rust
use sparkscan::{Client, reqwest};

tokio_test::block_on(async {
    // Configure client with API key
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("x-api-key", std::env::var("X_API_KEY").unwrap_or("test".to_string()).parse().unwrap());

    let http_client = reqwest::ClientBuilder::new()
        .default_headers(headers)
        .build()
        .unwrap();

    let client = Client::new_with_client("https://api.sparkscan.io", http_client);
    let response = client
        .get_latest_transactions_v1_tx_latest_get()
        .network("MAINNET")
        .limit(10)
        .send()
        .await
        .unwrap();

    for tx in response.iter() {
        println!("Latest TX: {} - {} - ${:.2}", 
            tx.id, 
            tx.type_, 
            tx.value_usd
        );
    }
});
```

## Time Filtering

Use timestamp parameters to get transactions within specific time ranges:

```rust
use chrono::{DateTime, Utc};

// Time filtering example (reuse client from above)
let from = "2024-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
let to = "2024-01-02T00:00:00Z".parse::<DateTime<Utc>>().unwrap();

// Note: client would be defined in a real application
// let response = client
//     .get_latest_transactions_v1_tx_latest_get()
//     .network("MAINNET")
//     .from_timestamp(from)
//     .to_timestamp(to)
//     .send()
//     .await
//     .unwrap();
```

## Returns

Array of `LatestNetworkTransactionItem` containing recent network transactions with full metadata.
