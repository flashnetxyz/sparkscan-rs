Get the latest transaction ID for each Bitcoin address in a batch request.

Efficiently retrieves the most recent transaction for multiple Bitcoin addresses, useful for:
- Wallet synchronization
- Address monitoring
- Transaction history updates
- Batch address validation

Accepts up to 100 Bitcoin addresses per request.

## Parameters

- `network`: Network to use (REGTEST or MAINNET)
- `addresses`: Array of Bitcoin addresses (max 100)

## Example

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
    
    let addresses = vec![
        "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string(),
    ];

    let response = client
        .get_addresses_latest_txid_v1_bitcoin_addresses_latest_txid_post()
        .network("MAINNET")
        .body(addresses)
        .send()
        .await
        .unwrap();

    for (address, txid) in response.iter() {
        match txid {
            Some(id) => println!("Address {}: latest TX {}", address, id),
            None => println!("Address {}: no transactions found", address),
        }
    }
});
```

## Returns

Dictionary mapping each Bitcoin address to its latest transaction ID (or null if no transactions found).
