Construct a new client with an existing reqwest::Client, allowing more control over its configuration.

`baseurl` is the base URL provided to the internal reqwest::Client, and should include a scheme and hostname,
as well as port and a path stem if applicable.

## Usage with api.sparkscan.io

For most use cases with `api.sparkscan.io`, consider using `new_with_api_key` instead for simpler setup.

For advanced configurations, you can manually configure the reqwest::Client:

> **Note**: This example shows advanced client configuration. For simple API key setup, use `new_with_api_key`.

```rust
use sparkscan::Client;

// Advanced configuration example
let client = reqwest::ClientBuilder::new()
    .timeout(std::time::Duration::from_secs(30))
    .build()
    .unwrap();

let sparkscan = Client::new_with_client("https://api.sparkscan.io", client);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Parameters

- `baseurl`: The base URL for the API (e.g., "<https://api.sparkscan.io>")
- `client`: A configured reqwest::Client instance
