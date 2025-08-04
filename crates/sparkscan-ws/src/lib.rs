//! # SparkScan WebSocket SDK
//! 
//! Type-safe WebSocket client implementation for SparkScan API integration.
//! 
//! Provides automatic JSON schema-based message deserialization and topic-based
//! subscription management built on tokio-centrifuge.
//! 
//! ## Features
//! 
//! - Type-safe message parsing from JSON schemas
//! - Topic-based subscription routing
//! - Configurable reconnection handling
//! - Async runtime compatibility
//! - Error propagation and handling
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
//!     // Register connection event handlers
//!     client.on_connected(|| println!("WebSocket connected"));
//!     client.on_error(|err| eprintln!("Connection error: {}", err));
//! 
//!     // Establish connection
//!     client.connect().await?;
//! 
//!     // Create subscription for balance topic
//!     let subscription = client.subscribe(Topic::Balances).await?;
//!     subscription.on_message(|message| {
//!         match message {
//!             SparkScanMessage::Balance(balance) => {
//!                 println!("Balance update: {} sats for {}", 
//!                          balance.soft_balance, balance.address);
//!             }
//!             _ => println!("Unexpected message type received"),
//!         }
//!     });
//!     subscription.subscribe();
//! 
//!     // Run until interrupted
//!     tokio::signal::ctrl_c().await?;
//!     Ok(())
//! }
//! ```
//! 
//! ## Topic Types
//! 
//! Available subscription topics:
//! 
//! - `Topic::Balances` - Account balance updates
//! - `Topic::TokenBalances` - Token balance updates  
//! - `Topic::TokenPrices` - Token price feeds
//! - `Topic::Tokens` - Token metadata updates
//! - `Topic::Transactions` - Transaction updates
//! 
//! Filtered subscriptions for specific identifiers:
//! 
//! ```rust,no_run
//! # use sparkscan_ws::Topic;
//! // Address-specific balance subscription
//! let topic = Topic::AddressBalance("sp1abc123...".to_string());
//! 
//! // Token-specific price subscription
//! let topic = Topic::TokenPrice("btkn1def456...".to_string());
//! ```
//! 
//! ## Message Processing
//! 
//! Messages are deserialized into typed enum variants:
//! 
//! ```rust,no_run
//! use sparkscan_ws::SparkScanMessage;
//! 
//! fn handle_message(message: SparkScanMessage) {
//!     match message {
//!         SparkScanMessage::Balance(balance) => {
//!             // Process balance update
//!             println!("Address: {}", balance.address);
//!             println!("Balance: {} sats", balance.soft_balance);
//!         }
//!         SparkScanMessage::Transaction(tx) => {
//!             // Process transaction update  
//!             println!("Transaction ID: {}", tx.id);
//!             if let Some(amount) = &tx.amount_sats {
//!                 println!("Amount: {} sats", amount);
//!             }
//!         }
//!         SparkScanMessage::TokenPrice(price) => {
//!             // Process price update
//!             println!("Token: {}", price.address);
//!             println!("Price: {:?} sats", price.price_sats);
//!         }
//!         _ => {} // Handle other message types
//!     }
//! }
//! ```
//! 
//! ## Error Types
//! 
//! Error handling through structured error types:
//! 
//! ```rust,no_run
//! use sparkscan_ws::{SparkScanWsClient, SparkScanWsError, Result};
//! 
//! async fn handle_connection() -> Result<()> {
//!     let client = SparkScanWsClient::new("ws://updates.sparkscan.io");
//!     
//!     match client.connect().await {
//!         Ok(_) => println!("Connection established"),
//!         Err(SparkScanWsError::ConnectionError(msg)) => {
//!             eprintln!("Connection failed: {}", msg);
//!         }
//!         Err(e) => eprintln!("Error: {}", e),
//!     }
//!     
//!     Ok(())
//! }
//! ```
//! 
//! ## Client Configuration
//! 
//! Configure client connection parameters:
//! 
//! ```rust,no_run
//! use sparkscan_ws::{SparkScanWsClient, SparkScanWsConfig};
//! 
//! let config = SparkScanWsConfig::new("ws://updates.sparkscan.io")
//!     .with_protobuf(true)              // Enable protobuf message format
//!     .with_timeout(60)                 // Set connection timeout (seconds)
//!     .with_auto_reconnect(true)        // Enable automatic reconnection
//!     .with_max_reconnect_attempts(10)  // Limit reconnection attempts
//!     .with_reconnect_delay(2000);      // Delay between attempts (ms)
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

/// Prelude module for convenient type imports.
/// 
/// Provides glob import access to commonly used types:
/// 
/// ```rust,no_run
/// use sparkscan_ws::prelude::*;
/// 
/// // All primary types available
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