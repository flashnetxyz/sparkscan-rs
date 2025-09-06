Get comprehensive summary information for a Spark address.

Returns detailed information about an address including:
- Balance summary (soft/hard BTC balances)
- Token holdings and their USD values
- Transaction count and total portfolio value
- Associated public key information

## Parameters

- `address`: The Spark address to query
- `network`: Network to use (REGTEST or MAINNET)

## Example

> **Note**: The address and other values used in this example are for documentation testing purposes.

```rust
use sparkscan::Client;

tokio_test::block_on(async {
    // Create client with API key from environment
    let client = Client::new_with_api_key(
        "https://api.sparkscan.io",
        &std::env::var("X_API_KEY").unwrap_or("test".to_string())
    );
    let response = client
        .address_summary_v1_address_address_get()
        .address("sp1pgssyv42njtxa7kkgvnukk2xnuwpg96n5mxm4985lvhe6sxgavl902js39la8k")
        .network("MAINNET")
        .send()
        .await
        .unwrap();

    println!("Address balance: {} sats", response.balance.btc_soft_balance_sats);
    println!("Token count: {}", response.token_count);
});
```

## Returns

`AddressSummaryResponse` containing:
- Balance information (BTC soft/hard balances)
- Total USD value of holdings
- Transaction and token counts
- Optional list of token holdings
