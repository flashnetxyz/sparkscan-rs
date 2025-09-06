Get transaction history for a specific token.

Returns all transactions involving a specific token, including:
- Token transfers between addresses
- Minting and burning operations
- Multi-input/output transactions
- Transaction metadata and counterparty information

## Parameters

- `identifier`: Token identifier (64-char hex) or Bech32 token address
- `network`: Network to use (REGTEST or MAINNET)
- `limit`: Maximum number of transactions to return (1-100, default: 25)
- `offset`: Number of transactions to skip for pagination (default: 0)

## Example

> **Note**: The token identifier and other values used in this example are for documentation testing purposes.

```rust
use sparkscan::Client;

tokio_test::block_on(async {
    // Create client with API key from environment
    let client = Client::new_with_api_key(
        "https://api.sparkscan.io",
        &std::env::var("X_API_KEY").unwrap_or("test".to_string())
    );
    let response = client
        .get_token_transactions_v1_tokens_identifier_transactions_get()
        .identifier("06f620d132bcef1a1bf6f132351a6436334a89332d4965b6acecf13b78156094")
        .network("MAINNET")
        .limit(25)
        .send()
        .await
        .unwrap();

    println!("Total transactions: {}", response.meta.total_items);
    for tx in &response.data {
        println!("TX: {} - {} tokens (${:.2})", 
            tx.id,
            tx.amount,
            tx.value_usd
        );
    }
});
```

## Returns

`TokenTransactionsResponse` containing:
- Pagination metadata (total count, limit, offset)
- Array of token transaction details with amounts, parties, and metadata
