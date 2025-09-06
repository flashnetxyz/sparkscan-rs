Get paginated transaction history for a specific address.

Retrieves all transactions associated with an address, including:
- Spark transfers (incoming/outgoing)
- Bitcoin deposits and withdrawals  
- Lightning payments
- Token transfers, mints, and burns
- Multi-token operations

## Parameters

- `address`: The address to get transactions for
- `network`: Network to use (REGTEST or MAINNET)
- `limit`: Maximum number of transactions to return (1-100, default: 25)
- `offset`: Number of transactions to skip for pagination (default: 0)

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
        .get_address_transactions_v1_address_address_transactions_get()
        .address("sp1pgssyv42njtxa7kkgvnukk2xnuwpg96n5mxm4985lvhe6sxgavl902js39la8k")
        .network("MAINNET")
        .limit(50)
        .offset(0)
        .send()
        .await
        .unwrap();

    for transaction in &response.data {
        println!("TX: {} - {} - ${:.2}", 
            transaction.id, 
            transaction.type_, 
            transaction.value_usd
        );
    }
});
```

## Returns

`AddressTransactionsResponse` containing:
- Pagination metadata (total count, limit, offset)
- Array of transaction details with counterparty information
- USD values and status for each transaction
