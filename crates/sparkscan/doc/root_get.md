Root endpoint for API health check and basic connectivity testing.

Returns a simple response to verify API connectivity, useful for:
- API health monitoring
- Connection testing
- Service availability checks
- Basic endpoint validation

## Example

> **Note**: This example demonstrates basic API connectivity testing.

```rust
use sparkscan::Client;

tokio_test::block_on(async {
    // Create client with API key from environment
    let client = Client::new_with_api_key(
        "https://api.sparkscan.io",
        &std::env::var("X_API_KEY").unwrap_or("test".to_string())
    );
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
