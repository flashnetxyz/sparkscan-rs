Construct a new client with an existing reqwest::Client, allowing more control over its configuration.

`baseurl` is the base URL provided to the internal reqwest::Client, and should include a scheme and hostname,
as well as port and a path stem if applicable.

## Usage with api.sparkscan.io

When using the official SparkScan API at `api.sparkscan.io`, you must configure the `x-api-key` header:

```rust
use sparkscan::reqwest;
use sparkscan::Client;

let mut headers = reqwest::header::HeaderMap::new();
headers.insert("x-api-key", "your-api-key-here".parse().unwrap());

let client = reqwest::ClientBuilder::new()
    .default_headers(headers)
    .build()
    .unwrap();

let sparkscan = Client::new_with_client("https://api.sparkscan.io", client);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Parameters

- `baseurl`: The base URL for the API (e.g., "<https://api.sparkscan.io>")
- `client`: A configured reqwest::Client instance
