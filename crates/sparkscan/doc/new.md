Create a new client with default configuration.

This method creates a client using the default reqwest::Client configuration with:
- User-Agent header set to `sparkscan-rs/{version}`
- 15-second connect and request timeouts (non-WASM targets)
- Default headers for optimal API interaction

**Important**: For production use with api.sparkscan.io, you should use `new_with_api_key` instead to configure the required `x-api-key` header. This method is mainly for development and testing.

## Parameters

- `baseurl`: The base URL for the API (e.g., "<https://api.sparkscan.io>")

## Example

> **Note**: This example shows basic client creation for development/testing purposes.

```rust
use sparkscan::Client;

// For development or testing
let client = Client::new("https://api.sparkscan.io");
```

## See Also

- `new_with_api_key` - For production use with API keys (recommended)
- `new_with_client` - For advanced custom client configuration
