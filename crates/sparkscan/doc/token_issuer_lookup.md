Convert between issuer public keys and their token identifiers/addresses.

This endpoint allows bidirectional lookup between token issuers and their tokens, useful for:
- Token issuer identification
- Reverse token lookups
- Token validation and verification
- Issuer portfolio analysis

Accepts up to 100 items per request for efficient batch processing.

## Parameters

- `network`: Network to use (REGTEST or MAINNET)
- `request_body`: TokenIssuerLookupRequest containing pubkeys and/or tokens to lookup

## Example

> **Note**: The public keys and other values used in this example are for documentation testing purposes.

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
    
    // Lookup tokens by issuer public keys
    let request = sparkscan::types::TokenIssuerLookupRequest {
        pubkeys: Some(vec![
            "033840d530e74bc6691d6c3c7cca5353309894190c62d72f909cca98879d65a4e1".to_string()
        ]),
        tokens: None,
    };

    let response = client
        .token_issuer_lookup_v1_tokens_issuer_lookup_post()
        .network("MAINNET")
        .body(request)
        .send()
        .await
        .unwrap();

    for result in &response.results {
        if let Some(token_id) = &result.token_identifier {
            println!("Pubkey {} issues token {}", result.pubkey.as_ref().unwrap_or(&"".to_string()), token_id);
        }
    }
});
```

## Returns

`TokenIssuerLookupResponse` containing:
- `results`: Array of lookup results with pubkey, token_identifier, and token_address mappings
