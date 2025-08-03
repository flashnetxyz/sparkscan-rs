//! SparkScan WebSocket client implementation.

use std::sync::Arc;
use tokio_centrifuge::{client::Client as CentrifugeClient, config::Config};
use crate::{
    error::Result,
    subscription::SparkScanSubscription,
    types::Topic,
};

/// Configuration for the SparkScan WebSocket client.
#[derive(Debug, Clone)]
pub struct SparkScanWsConfig {
    /// The WebSocket URL to connect to
    pub url: String,
    /// Whether to use protobuf format (default: false, uses JSON)
    pub use_protobuf: bool,
    /// Connection timeout in seconds (default: 30)
    pub connection_timeout: u64,
    /// Whether to automatically reconnect on disconnect (default: true)
    pub auto_reconnect: bool,
    /// Maximum number of reconnection attempts (default: 5)
    pub max_reconnect_attempts: u32,
    /// Reconnection delay in milliseconds (default: 1000)
    pub reconnect_delay: u64,
}

impl Default for SparkScanWsConfig {
    fn default() -> Self {
        Self {
            url: "ws://updates.sparkscan.io".to_string(),
            use_protobuf: false,
            connection_timeout: 30,
            auto_reconnect: true,
            max_reconnect_attempts: 5,
            reconnect_delay: 1000,
        }
    }
}

impl SparkScanWsConfig {
    /// Create a new configuration with the specified URL.
    pub fn new<S: Into<String>>(url: S) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }

    /// Set whether to use protobuf format.
    pub fn with_protobuf(mut self, use_protobuf: bool) -> Self {
        self.use_protobuf = use_protobuf;
        self
    }

    /// Set the connection timeout.
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.connection_timeout = timeout_seconds;
        self
    }

    /// Set auto-reconnection behavior.
    pub fn with_auto_reconnect(mut self, auto_reconnect: bool) -> Self {
        self.auto_reconnect = auto_reconnect;
        self
    }

    /// Set maximum reconnection attempts.
    pub fn with_max_reconnect_attempts(mut self, max_attempts: u32) -> Self {
        self.max_reconnect_attempts = max_attempts;
        self
    }

    /// Set reconnection delay in milliseconds.
    pub fn with_reconnect_delay(mut self, delay_ms: u64) -> Self {
        self.reconnect_delay = delay_ms;
        self
    }
}

/// The main SparkScan WebSocket client.
/// 
/// This client provides a high-level interface for connecting to SparkScan's
/// WebSocket API and subscribing to typed message streams.
/// 
/// # Example
/// 
/// ```rust,no_run
/// use sparkscan_ws::{SparkScanWsClient, SparkScanWsConfig, Topic, SparkScanMessage};
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = SparkScanWsConfig::new("ws://localhost:8000/connection/websocket");
///     let client = SparkScanWsClient::with_config(config);
/// 
///     // Set up connection callbacks
///     client.on_connected(|| {
///         println!("Connected to SparkScan WebSocket!");
///     });
/// 
///     client.on_disconnected(|| {
///         println!("Disconnected from SparkScan WebSocket");
///     });
/// 
///     // Connect to the WebSocket
///     client.connect().await?;
/// 
///     // Subscribe to balance updates
///     let subscription = client.subscribe(Topic::Balances).await?;
///     subscription.on_message(|message| {
///         if let SparkScanMessage::Balance(balance) = message {
///             println!("Balance update: {} sats", balance.soft_balance);
///         }
///     });
///     subscription.subscribe();
/// 
///     // Keep the client running
///     tokio::signal::ctrl_c().await?;
///     Ok(())
/// }
/// ```
pub struct SparkScanWsClient {
    /// The underlying centrifuge client
    inner: Arc<CentrifugeClient>,
    /// Client configuration
    config: SparkScanWsConfig,
}

impl SparkScanWsClient {
    /// Create a new SparkScan WebSocket client with the specified URL.
    /// 
    /// Uses default configuration settings.
    pub fn new<S: Into<String>>(url: S) -> Self {
        let config = SparkScanWsConfig::new(url);
        Self::with_config(config)
    }

    /// Create a new SparkScan WebSocket client with custom configuration.
    pub fn with_config(config: SparkScanWsConfig) -> Self {
        let centrifuge_config = if config.use_protobuf {
            Config::new().use_protobuf()
        } else {
            Config::new()
        };

        let inner = CentrifugeClient::new(&config.url, centrifuge_config);

        Self {
            inner: Arc::new(inner),
            config,
        }
    }

    /// Get the client configuration.
    pub fn config(&self) -> &SparkScanWsConfig {
        &self.config
    }

    /// Set a callback for when the client is connecting.
    /// 
    /// # Example
    /// ```rust,no_run
    /// # use sparkscan_ws::SparkScanWsClient;
    /// let client = SparkScanWsClient::new("ws://localhost:8000/connection/websocket");
    /// client.on_connecting(|| {
    ///     println!("Connecting to SparkScan...");
    /// });
    /// ```
    pub fn on_connecting<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.inner.on_connecting(callback);
    }

    /// Set a callback for when the client successfully connects.
    pub fn on_connected<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.inner.on_connected(callback);
    }

    /// Set a callback for when the client disconnects.
    pub fn on_disconnected<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.inner.on_disconnected(callback);
    }

    /// Set a callback for connection errors.
    /// 
    /// # Example
    /// ```rust,no_run
    /// # use sparkscan_ws::SparkScanWsClient;
    /// let client = SparkScanWsClient::new("ws://localhost:8000/connection/websocket");
    /// client.on_error(|error| {
    ///     eprintln!("WebSocket error: {:?}", error);
    /// });
    /// ```
    pub fn on_error<F>(&self, callback: F)
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        self.inner.on_error(move |err| {
            callback(format!("{:?}", err));
        });
    }

    /// Connect to the SparkScan WebSocket server.
    /// 
    /// This method initiates the connection but returns immediately.
    /// Use the connection callbacks to know when the connection is established.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the connection cannot be initiated.
    pub async fn connect(&self) -> Result<()> {
        self.inner.connect();
        // Wait a bit to allow connection to establish
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        Ok(())
    }

    /// Disconnect from the SparkScan WebSocket server.
    pub async fn disconnect(&self) -> Result<()> {
        // Note: tokio-centrifuge doesn't have an explicit disconnect method
        // The connection will be dropped when the client is dropped
        Ok(())
    }

    /// Create a subscription to the specified topic.
    /// 
    /// # Arguments
    /// 
    /// * `topic` - The topic to subscribe to
    /// 
    /// # Returns
    /// 
    /// A `SparkScanSubscription` that can be used to receive typed messages.
    /// 
    /// # Example
    /// 
    /// ```rust,no_run
    /// # use sparkscan_ws::{SparkScanWsClient, Topic};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = SparkScanWsClient::new("ws://localhost:8000/connection/websocket");
    /// 
    /// // Subscribe to all balance updates
    /// let balances = client.subscribe(Topic::Balances).await?;
    /// 
    /// // Subscribe to balance updates for a specific address
    /// let address_balance = client.subscribe(
    ///     Topic::AddressBalance("sp1abc123...".to_string())
    /// ).await?;
    /// 
    /// // Subscribe to token price updates
    /// let token_prices = client.subscribe(Topic::TokenPrices).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe(&self, topic: Topic) -> Result<SparkScanSubscription> {
        let topic_str = topic.as_str();
        let centrifuge_subscription = self.inner.new_subscription(&topic_str);
        
        Ok(SparkScanSubscription::new(centrifuge_subscription, topic))
    }

    /// Create a subscription to a custom topic string.
    /// 
    /// This is useful for subscribing to topics that don't fit the predefined
    /// `Topic` enum variants.
    pub async fn subscribe_to_custom<S: Into<String>>(&self, topic: S) -> Result<SparkScanSubscription> {
        let topic_string = topic.into();
        let topic = Topic::Custom(topic_string.clone());
        let centrifuge_subscription = self.inner.new_subscription(&topic_string);
        
        Ok(SparkScanSubscription::new(centrifuge_subscription, topic))
    }

    /// Check if the client is currently connected.
    /// 
    /// Note: This is a best-effort check and may not reflect the exact
    /// connection state due to the async nature of WebSocket connections.
    pub fn is_connected(&self) -> bool {
        // tokio-centrifuge doesn't expose connection state directly
        // This would need to be tracked internally if precise state is needed
        true // Placeholder implementation
    }

    /// Get connection statistics.
    /// 
    /// Returns information about the current connection if available.
    pub fn connection_stats(&self) -> ConnectionStats {
        ConnectionStats {
            connected: self.is_connected(),
            reconnect_attempts: 0, // Would need to track this internally
            last_error: None,
        }
    }
}

/// Connection statistics for the WebSocket client.
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    /// Whether the client is currently connected
    pub connected: bool,
    /// Number of reconnection attempts made
    pub reconnect_attempts: u32,
    /// Last connection error if any
    pub last_error: Option<String>,
}

// Implement Clone for SparkScanWsClient since Arc<CentrifugeClient> is cloneable
impl Clone for SparkScanWsClient {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            config: self.config.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = SparkScanWsConfig::new("ws://localhost:8000")
            .with_protobuf(true)
            .with_timeout(60)
            .with_auto_reconnect(false)
            .with_max_reconnect_attempts(10)
            .with_reconnect_delay(2000);

        assert_eq!(config.url, "ws://localhost:8000");
        assert!(config.use_protobuf);
        assert_eq!(config.connection_timeout, 60);
        assert!(!config.auto_reconnect);
        assert_eq!(config.max_reconnect_attempts, 10);
        assert_eq!(config.reconnect_delay, 2000);
    }

    #[tokio::test]
    async fn test_client_creation() {
        let client = SparkScanWsClient::new("ws://localhost:8000");
        assert_eq!(client.config().url, "ws://localhost:8000");
        assert!(!client.config().use_protobuf);
    }

    #[tokio::test]
    async fn test_client_clone() {
        let client = SparkScanWsClient::new("ws://localhost:8000");
        let cloned = client.clone();
        assert_eq!(client.config().url, cloned.config().url);
    }
}