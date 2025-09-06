Get the list of addresses holding a specific token.

Returns holders of a token with their balances and portfolio percentages, useful for:
- Token distribution analysis
- Whale watching and large holder identification
- Decentralization metrics
- Holder concentration studies

## Parameters

- `identifier`: Token identifier (64-char hex) or Bech32 token address
- `network`: Network to use (REGTEST or MAINNET)
- `limit`: Maximum number of holders to return (1-100, default: 25)
- `offset`: Number of holders to skip for pagination (default: 0)

## Example

> **Note**: The token identifier and other values used in this example are for documentation testing purposes.

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
        .get_token_holders_v1_tokens_identifier_holders_get()
        .identifier("06f620d132bcef1a1bf6f132351a6436334a89332d4965b6acecf13b78156094")
        .network("MAINNET")
        .limit(10)
        .send()
        .await
        .unwrap();

    println!("Total holders: {}", response.meta.total_items);
    for holder in &response.data {
        println!("{}: {} tokens ({:.2}% - ${:.2})", 
            holder.address,
            holder.balance,
            holder.percentage,
            holder.value_usd
        );
    }
});
```

## Returns

`TokenHoldersResponse` containing:
- Pagination metadata (total count, limit, offset)
- Array of holder information with addresses, balances, percentages, and USD values
