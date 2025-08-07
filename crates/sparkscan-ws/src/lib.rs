//! # SparkScan WebSocket SDK
//!
//! ## Overview
//!
//! A high-performance, type-safe WebSocket client implementation for seamless SparkScan API integration.
//! This SDK provides enterprise-grade real-time data streaming capabilities with automatic schema-based
//! message deserialization and robust subscription management, built on the proven tokio-centrifuge framework.
//!
//! Designed for production environments requiring reliable, low-latency access to SparkScan's real-time
//! data feeds including balance updates, transaction notifications, and token price movements.
//!
//! ## Core Features
//!
//! - **Type-Safe Message Parsing**: Automatic JSON schema-based message deserialization with compile-time type safety
//! - **Topic-Based Subscription Routing**: Granular subscription management for different data streams and filters
//! - **Intelligent Reconnection Handling**: Configurable automatic reconnection with exponential backoff strategies
//! - **Async Runtime Compatibility**: Full tokio async/await support with non-blocking I/O operations
//! - **Comprehensive Error Handling**: Structured error propagation with detailed context and recovery strategies
//! - **Production-Ready Architecture**: Built-in connection monitoring, health checks, and observability hooks
//! - **Performance Optimized**: Efficient message routing and minimal resource overhead for high-throughput applications
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use sparkscan_ws::{SparkScanWsClient, Topic, SparkScanMessage};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a client with default configuration
//!     let client = SparkScanWsClient::new("ws://updates.sparkscan.io/");
//!
//!     // Register connection event handlers for monitoring
//!     client.on_connected(|| println!("WebSocket connected to SparkScan API"));
//!     client.on_error(|err| eprintln!("Connection error: {}", err));
//!
//!     // Establish connection to SparkScan API
//!     client.connect().await?;
//!
//!     // Create subscription for real-time balance updates
//!     let subscription = client.subscribe(Topic::Balances).await?;
//!     subscription.on_message(|message| {
//!         match message {
//!             SparkScanMessage::Balance(balance) => {
//!                 println!("Balance update: {} sats for address {:?}",
//!                          balance.soft_balance, balance.address);
//!             }
//!             _ => println!("Unexpected message type received"),
//!         }
//!     });
//!     subscription.subscribe();
//!
//!     // Run until user interruption
//!     tokio::signal::ctrl_c().await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Topic Types
//!
//! Available subscription topics for real-time data feeds:
//!
//! - `Topic::Balances` - Global account balance updates across all addresses
//! - `Topic::TokenBalances` - Token-specific balance updates and transfers  
//! - `Topic::TokenPrices` - Real-time token price feeds and market data
//! - `Topic::Tokens` - Token metadata updates and new token discoveries
//! - `Topic::Transactions` - Transaction confirmations and status updates
//!
//! Filtered subscriptions for targeted monitoring:
//!
//! ```rust,no_run
//! # use sparkscan_ws::Topic;
//! // Monitor specific address balance changes
//! let topic = Topic::BalanceAddress("sp1abc123...".to_string());
//!
//! // Track specific token price movements
//! let topic = Topic::TokenPriceIdentifier("btkn1def456...".to_string());
//!
//! // Network-specific transaction monitoring
//! let topic = Topic::TransactionNetwork("mainnet".to_string());
//! ```
//!
//! ## Message Processing
//!
//! All incoming messages are automatically deserialized into typed enum variants
//! providing compile-time safety and structured access to real-time data:
//!
//! ```rust,no_run
//! use sparkscan_ws::SparkScanMessage;
//!
//! fn handle_message(message: SparkScanMessage) {
//!     match message {
//!         SparkScanMessage::Balance(balance) => {
//!             // Process real-time balance update
//!             println!("Address: {:?}", balance.address);
//!             println!("Soft Balance: {} sats", balance.soft_balance);
//!             println!("Hard Balance: {} sats", balance.hard_balance);
//!             println!("Network: {}", balance.network);
//!         }
//!         SparkScanMessage::Transaction(tx) => {
//!             // Process transaction confirmation or update  
//!             println!("Transaction ID: {}", tx.id);
//!             println!("Status: {}", tx.status);
//!             if let Some(amount) = &tx.amount_sats {
//!                 println!("Amount: {} sats", amount);
//!             }
//!         }
//!         SparkScanMessage::TokenPrice(price) => {
//!             // Process real-time price update
//!             println!("Token: {:?}", price.address);
//!             println!("Price: {:?} sats", price.price_sats);
//!             println!("Protocol: {:?}", price.protocol);
//!         }
//!         _ => {} // Handle other message types as needed
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
//!     let client = SparkScanWsClient::new("ws://updates.sparkscan.io/");
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
//! Customize client behavior for production environments:
//!
//! ```rust,no_run
//! use sparkscan_ws::{SparkScanWsClient, SparkScanWsConfig};
//!
//! let config = SparkScanWsConfig::new("ws://updates.sparkscan.io/")
//!     .with_protobuf(true)              // Enable protobuf for reduced bandwidth
//!     .with_timeout(60)                 // Extended timeout for slow networks
//!     .with_auto_reconnect(true)        // Maintain connection reliability
//!     .with_max_reconnect_attempts(10)  // Aggressive reconnection policy
//!     .with_reconnect_delay(2000);      // 2-second backoff between attempts
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
pub use client::{ConnectionStats, SparkScanWsClient, SparkScanWsConfig};
pub use error::{Result, SparkScanWsError};
pub use subscription::{SparkScanSubscription, SubscriptionManager};
pub use types::{SparkScanMessage, Topic};

// Re-export generated types
pub use types::{
    balance::BalancePayload,
    token::TokenPayload,
    token_balance::TokenBalancePayload,
    token_price::TokenPricePayload,
    transaction::{TransactionPayload, Type},
};

/// The current version of the SparkScan WebSocket SDK.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default WebSocket URL for SparkScan mainnet API endpoint.
pub const DEFAULT_MAINNET_URL: &str = "ws://updates.sparkscan.io/";

/// Prelude module for convenient type imports.
///
/// Provides glob import access to commonly used types and traits for streamlined development:
///
/// ```rust,no_run
/// use sparkscan_ws::prelude::*;
///
/// // All primary types are immediately available
/// let client = SparkScanWsClient::new("ws://updates.sparkscan.io/");
/// let topic = Topic::Balances;
/// let config = SparkScanWsConfig::default();
/// ```
pub mod prelude {
    pub use crate::{
        BalancePayload, Result, SparkScanMessage, SparkScanSubscription, SparkScanWsClient,
        SparkScanWsConfig, SparkScanWsError, TokenBalancePayload, TokenPayload, TokenPricePayload,
        Topic, TransactionPayload,
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
    fn test_default_mainnet_url_format() {
        assert!(DEFAULT_MAINNET_URL.starts_with("ws://"));
        assert!(DEFAULT_MAINNET_URL.contains("updates.sparkscan.io"));
    }
}
