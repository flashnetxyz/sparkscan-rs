Get all token holdings for a specific address.

Returns detailed information about all tokens held by an address, including:
- Token metadata (name, ticker, decimals)
- Balance amounts and USD values
- Issuer information
- Supply constraints and freezability

## Parameters

- `address`: The address to get token holdings for
- `network`: Network to use (REGTEST or MAINNET)

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
    let tokens = client
        .get_address_tokens_v1_address_address_tokens_get()
        .address("sp1pgssyv42njtxa7kkgvnukk2xnuwpg96n5mxm4985lvhe6sxgavl902js39la8k")
        .network("MAINNET")
        .send()
        .await
        .unwrap();

    println!("Total token value: ${:.2}", tokens.total_value_usd);
    for token in &tokens.tokens {
        println!("{} ({}): {} - ${:.2}", 
            token.name,
            token.ticker,
            token.balance,
            token.value_usd
        );
    }
});
```

## Returns

`AddressTokensResponse` containing:
- Address and public key information
- Total USD value of all token holdings
- Array of individual token holdings with full metadata
