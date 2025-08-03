//! # SparkScan WebSocket SDK
//! 
//! A high-level, type-safe WebSocket client for the SparkScan API.
//! 
//! This crate provides a WebSocket client that automatically handles message
//! deserialization using types generated from JSON schemas. It's built on top
//! of `tokio-centrifuge` for reliable WebSocket connectivity.
//! 
//! ## Features
//! 
//! - **Type Safety**: Messages are automatically parsed into strongly-typed Rust structs
//! - **Topic-based Subscriptions**: Subscribe to specific data streams (balances, transactions, etc.)
//! - **Automatic Reconnection**: Built-in reconnection logic for robust connectivity
//! - **Async/Await Support**: Full async/await support using Tokio
//! - **Flexible Configuration**: Customizable connection settings and behavior
//! 
//! ## Quick Start
//! 
//! ```rust,no_run
//! use sparkscan_ws::{SparkScanWsClient, Topic, SparkScanMessage};
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a client
//!     let client = SparkScanWsClient::new("ws://localhost:8000/connection/websocket");
//! 
//!     // Set up connection callbacks
//!     client.on_connected(|| println!("Connected!"));
//!     client.on_error(|err| eprintln!("Error: {}", err));
//! 
//!     // Connect
//!     client.connect().await?;
//! 
//!     // Subscribe to balance updates
//!     let subscription = client.subscribe(Topic::Balances).await?;
//!     subscription.on_message(|message| {
//!         match message {
//!             SparkScanMessage::Balance(balance) => {
//!                 println!("Balance: {} sats for address {}", 
//!                          balance.soft_balance, balance.address);
//!             }
//!             _ => println!("Received other message type"),
//!         }
//!     });
//!     subscription.subscribe();
//! 
//!     // Keep running
//!     tokio::signal::ctrl_c().await?;
//!     Ok(())
//! }
//! ```
//! 
//! ## Available Topics
//! 
//! The SDK supports subscribing to various data streams:
//! 
//! - **Balances**: Account balance updates (`Topic::Balances`)
//! - **Token Balances**: Token-specific balance updates (`Topic::TokenBalances`)  
//! - **Token Prices**: Token price updates (`Topic::TokenPrices`)
//! - **Tokens**: Token metadata updates (`Topic::Tokens`)
//! - **Transactions**: Transaction updates (`Topic::Transactions`)
//! 
//! You can also subscribe to filtered streams for specific addresses or tokens:
//! 
//! ```rust,no_run
//! # use sparkscan_ws::Topic;
//! // Balance updates for a specific address
//! let topic = Topic::AddressBalance("sp1abc123...".to_string());
//! 
//! // Token price updates for a specific token
//! let topic = Topic::TokenPrice("btkn1def456...".to_string());
//! ```
//! 
//! ## Message Types
//! 
//! All messages are parsed into the `SparkScanMessage` enum:
//! 
//! ```rust,no_run
//! use sparkscan_ws::SparkScanMessage;
//! 
//! match message {
//!     SparkScanMessage::Balance(balance) => {
//!         // Handle balance update
//!         println!("Address: {}", balance.address);
//!         println!("Balance: {} sats", balance.soft_balance);
//!     }
//!     SparkScanMessage::Transaction(tx) => {
//!         // Handle transaction update  
//!         println!("Transaction: {}", tx.id);
//!         println!("Amount: {} sats", tx.amount_sats);
//!     }
//!     SparkScanMessage::TokenPrice(price) => {
//!         // Handle token price update
//!         println!("Token: {}", price.address);
//!         println!("Price: {} sats", price.price_sats);
//!     }
//!     // ... other message types
//!     _ => {}
//! }
//! ```
//! 
//! ## Error Handling
//! 
//! The SDK provides comprehensive error handling through the `SparkScanWsError` type:
//! 
//! ```rust,no_run
//! use sparkscan_ws::{SparkScanWsError, Result};
//! 
//! async fn handle_connection() -> Result<()> {
//!     let client = SparkScanWsClient::new("ws://updates.sparkscan.io");
//!     
//!     match client.connect().await {
//!         Ok(_) => println!("Connected successfully"),
//!         Err(SparkScanWsError::ConnectionError(msg)) => {
//!             eprintln!("Failed to connect: {}", msg);
//!         }
//!         Err(e) => eprintln!("Other error: {}", e),
//!     }
//!     
//!     Ok(())
//! }
//! ```
//! 
//! ## Configuration
//! 
//! Customize client behavior using `SparkScanWsConfig`:
//! 
//! ```rust,no_run
//! use sparkscan_ws::{SparkScanWsClient, SparkScanWsConfig};
//! 
//! let config = SparkScanWsConfig::new("ws://updates.sparkscan.io")
//!     .with_protobuf(true)              // Use protobuf format
//!     .with_timeout(60)                 // 60 second timeout
//!     .with_auto_reconnect(true)        // Enable auto-reconnection
//!     .with_max_reconnect_attempts(10)  // Max 10 reconnect attempts
//!     .with_reconnect_delay(2000);      // 2 second delay between attempts
//! 
//! let client = SparkScanWsClient::with_config(config);
//! ```

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod client;
pub mod error;
pub mod subscription;

// Allow missing docs for the types module since it contains generated code
#[allow(missing_docs)]
pub mod types;



// Re-export main types for convenience
pub use client::{SparkScanWsClient, SparkScanWsConfig, ConnectionStats};
pub use error::{SparkScanWsError, Result};
pub use subscription::{SparkScanSubscription, SubscriptionManager};
pub use types::{SparkScanMessage, Topic};

// Re-export generated types
pub use types::{
    BalancePayload,
    TokenBalancePayload, 
    TokenPricePayload,
    TokenPayload,
    TransactionPayload,
};

/// The current version of the SparkScan WebSocket SDK.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default WebSocket URL for SparkScan mainnet.
pub const DEFAULT_MAINNET_URL: &str = "wss://ws.sparkscan.io/connection/websocket";

/// Default WebSocket URL for SparkScan regtest.
pub const DEFAULT_REGTEST_URL: &str = "wss://regtest-ws.sparkscan.io/connection/websocket";

/// Prelude module containing the most commonly used types.
/// 
/// This module is designed to be glob-imported for convenience:
/// 
/// ```rust,no_run
/// use sparkscan_ws::prelude::*;
/// 
/// // Now you can use the main types directly
/// let client = SparkScanWsClient::new("ws://localhost:8000");
/// let topic = Topic::Balances;
/// ```
pub mod prelude {
    pub use crate::{
        SparkScanWsClient,
        SparkScanWsConfig,
        SparkScanSubscription,
        SparkScanMessage,
        Topic,
        Result,
        SparkScanWsError,
        BalancePayload,
        TokenBalancePayload,
        TokenPricePayload,
        TokenPayload,
        TransactionPayload,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test] 
    fn test_default_urls() {
        assert!(DEFAULT_MAINNET_URL.starts_with("wss://"));
        assert!(DEFAULT_REGTEST_URL.starts_with("wss://"));
    }
}