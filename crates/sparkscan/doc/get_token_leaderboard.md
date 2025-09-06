Get the top tokens ranked by market cap and activity.

Returns a leaderboard of tokens sorted by various metrics, useful for:
- Token market analysis
- Investment research
- Popular token discovery
- Market cap tracking

## Parameters

- `network`: Network to use (REGTEST or MAINNET)
- `limit`: Maximum number of tokens to return (1-100, default: 25)
- `offset`: Number of tokens to skip for pagination (default: 0)
- `after_updated_at`: Return only tokens updated after this timestamp (optional)

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
    let leaderboard = client
        .get_token_leaderboard_v1_stats_leaderboard_tokens_get()
        .network("MAINNET")
        .limit(10)
        .send()
        .await
        .unwrap();

    println!("Total tokens: {}", leaderboard.total_tokens);
    for token in &leaderboard.leaderboard {
        println!("#{}: {} ({}) - Market Cap: ${:.2}", 
            token.rank,
            token.name,
            token.ticker,
            token.market_cap_usd
        );
    }
});
```

## Returns

`TokenLeaderboardResponse` containing:
- `total_tokens`: Total number of tokens in the system
- `leaderboard`: Array of ranked token entries with full metadata including market cap, volume, and holder count
