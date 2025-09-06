Root endpoint for API health check and basic connectivity testing.

Returns a simple response to verify API connectivity, useful for:
- API health monitoring
- Connection testing
- Service availability checks
- Basic endpoint validation

## Example

> **Note**: This example demonstrates basic API connectivity testing.

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
        .root_get()
        .send()
        .await
        .unwrap();

    println!("API is responding: {:?}", response);
});
```

## Returns

Basic JSON response indicating the API is operational.
