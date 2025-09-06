Create a new client with default configuration.

This method creates a client using the default reqwest::Client configuration with:
- User-Agent header set to `sparkscan-rs/{version}`
- 15-second connect and request timeouts (non-WASM targets)
- Default headers for optimal API interaction

**Important**: For production use with api.sparkscan.io, you must use `new_with_client` instead to configure the required `x-api-key` header. This method is mainly fordevelopment and testing.

## Parameters

- `baseurl`: The base URL for the API (e.g., "<https://api.sparkscan.io>")

## Example

```rust
use sparkscan::Client;

// For development or testing
let client = Client::new("https://api.sparkscan.io");
```

## See Also

- `new_with_client` - For custom client configuration including API keys
