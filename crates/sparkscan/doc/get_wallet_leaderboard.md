Get the top wallets by total value held on the network.

Returns a ranked list of wallets sorted by their total portfolio value, useful for:
- Market analysis and whale watching
- Network wealth distribution insights
- Top holder identification
- Leaderboard applications

## Parameters

- `network`: Network to use (REGTEST or MAINNET)
- `limit`: Maximum number of wallets to return (1-100, default: 25)

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
    let leaderboard = client
        .get_wallet_leaderboard_v1_stats_leaderboard_wallets_get()
        .network("MAINNET")
        .limit(10)
        .send()
        .await
        .unwrap();

    for entry in &leaderboard.leaderboard {
        println!("#{}: {} - {} sats (${:.2})", 
            entry.rank,
            entry.spark_address,
            entry.total_value_sats,
            entry.total_value_usd.unwrap_or(0.0)
        );
    }
});
```

## Returns

`WalletLeaderboard` containing:
- Array of ranked wallet entries with addresses and values
- Current Bitcoin price for USD conversions
