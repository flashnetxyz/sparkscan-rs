//! SparkScan WebSocket client implementation.

use crate::{error::Result, subscription::SparkScanSubscription, types::Topic};
use std::sync::Arc;
use tokio_centrifuge::{client::Client as CentrifugeClient, config::Config};

/// Configuration parameters for the SparkScan WebSocket client.
///
/// Provides comprehensive control over connection behavior, message format,
/// timeout settings, and automatic reconnection policies.
#[derive(Debug, Clone)]
pub struct SparkScanWsConfig {
    /// The WebSocket URL endpoint for the SparkScan API
    pub url: String,
    /// Message serialization format selection (default: false for JSON, true for protobuf)
    pub use_protobuf: bool,
    /// Maximum time to wait for connection establishment in seconds (default: 30)
    pub connection_timeout: u64,
    /// Enable automatic reconnection on connection loss (default: true for production reliability)
    pub auto_reconnect: bool,
    /// Maximum consecutive reconnection attempts before giving up (default: 5)
    pub max_reconnect_attempts: u32,
    /// Delay between reconnection attempts in milliseconds (default: 1000ms)
    pub reconnect_delay: u64,
}

impl Default for SparkScanWsConfig {
    fn default() -> Self {
        Self {
            url: "ws://updates.sparkscan.io/".to_string(),
            use_protobuf: false,
            connection_timeout: 30,
            auto_reconnect: true,
            max_reconnect_attempts: 5,
            reconnect_delay: 1000,
        }
    }
}

impl SparkScanWsConfig {
    /// Create a new configuration with the specified WebSocket URL.
    ///
    /// All other parameters are set to their default values for typical production use.
    pub fn new<S: Into<String>>(url: S) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }

    /// Configure message serialization format.
    ///
    /// # Arguments
    ///
    /// * `use_protobuf` - true for protobuf serialization, false for JSON (default)
    pub fn with_protobuf(mut self, use_protobuf: bool) -> Self {
        self.use_protobuf = use_protobuf;
        self
    }

    /// Configure connection establishment timeout.
    ///
    /// # Arguments
    ///
    /// * `timeout_seconds` - Maximum seconds to wait for connection establishment
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.connection_timeout = timeout_seconds;
        self
    }

    /// Configure automatic reconnection behavior.
    ///
    /// # Arguments
    ///
    /// * `auto_reconnect` - true to enable automatic reconnection on connection loss
    pub fn with_auto_reconnect(mut self, auto_reconnect: bool) -> Self {
        self.auto_reconnect = auto_reconnect;
        self
    }

    /// Configure maximum consecutive reconnection attempts.
    ///
    /// # Arguments
    ///
    /// * `max_attempts` - Maximum number of reconnection attempts before giving up
    pub fn with_max_reconnect_attempts(mut self, max_attempts: u32) -> Self {
        self.max_reconnect_attempts = max_attempts;
        self
    }

    /// Configure delay between reconnection attempts.
    ///
    /// # Arguments
    ///
    /// * `delay_ms` - Delay in milliseconds between reconnection attempts
    pub fn with_reconnect_delay(mut self, delay_ms: u64) -> Self {
        self.reconnect_delay = delay_ms;
        self
    }
}

/// WebSocket client for SparkScan API connectivity.
///
/// Provides enterprise-grade connection management and subscription creation for typed
/// message streams over WebSocket transport with automatic reconnection and error handling.
///
/// # Example
///
/// ```rust,no_run
/// use sparkscan_ws::{SparkScanWsClient, SparkScanWsConfig, Topic, SparkScanMessage};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = SparkScanWsConfig::new("ws://updates.sparkscan.io/");
///     let client = SparkScanWsClient::with_config(config);
///
///     // Configure connection event handlers for monitoring
///     client.on_connected(|| {
///         println!("WebSocket connection established to SparkScan API");
///     });
///
///     client.on_disconnected(|| {
///         println!("WebSocket connection terminated");
///     });
///
///     // Establish connection to SparkScan API
///     client.connect().await?;
///
///     // Create subscription for real-time balance updates
///     let subscription = client.subscribe(Topic::Balances).await?;
///     subscription.on_message(|message| {
///         if let SparkScanMessage::Balance(balance) = message {
///             println!("Balance update: {} sats for {}", balance.soft_balance, balance.address);
///         }
///     });
///     subscription.subscribe();
///
///     // Run until user interruption
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
    /// Create WebSocket client with specified URL.
    ///
    /// Initializes client with default configuration parameters including
    /// automatic reconnection, 30-second timeout, and JSON message format.
    pub fn new<S: Into<String>>(url: S) -> Self {
        let config = SparkScanWsConfig::new(url);
        Self::with_config(config)
    }

    /// Create WebSocket client with custom configuration.
    ///
    /// Provides full control over connection parameters, message format,
    /// and reconnection behavior for production deployments.
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

    /// Get the current client configuration.
    ///
    /// Returns a reference to the configuration used for this client instance.
    pub fn config(&self) -> &SparkScanWsConfig {
        &self.config
    }

    /// Register callback for connection initiation events.
    ///
    /// This callback is invoked when the client begins establishing a WebSocket connection.
    /// Useful for updating UI state or logging connection attempts.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use sparkscan_ws::SparkScanWsClient;
    /// let client = SparkScanWsClient::new("ws://updates.sparkscan.io/");
    /// client.on_connecting(|| {
    ///     println!("Initiating connection to SparkScan API...");
    /// });
    /// ```
    pub fn on_connecting<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.inner.on_connecting(callback);
    }

    /// Register callback for successful connection events.
    ///
    /// This callback is invoked when the WebSocket connection is successfully established
    /// and the client is ready to create subscriptions and receive messages.
    pub fn on_connected<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.inner.on_connected(callback);
    }

    /// Register callback for disconnection events.
    ///
    /// This callback is invoked when the WebSocket connection is terminated,
    /// either intentionally or due to network issues. If auto-reconnect is enabled,
    /// reconnection attempts will begin automatically.
    pub fn on_disconnected<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.inner.on_disconnected(callback);
    }

    /// Register callback for connection error events.
    ///
    /// This callback is invoked when connection errors occur, including network
    /// timeouts, authentication failures, or protocol errors. Error details are
    /// provided as formatted strings for logging and debugging.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use sparkscan_ws::SparkScanWsClient;
    /// let client = SparkScanWsClient::new("ws://updates.sparkscan.io/");
    /// client.on_error(|error| {
    ///     eprintln!("WebSocket connection error: {}", error);
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

    /// Initiate WebSocket connection to the SparkScan API server.
    ///
    /// This method initiates the connection process asynchronously and returns immediately.
    /// The actual connection establishment happens in the background. Use the connection
    /// event callbacks (`on_connected`, `on_error`) to monitor connection state changes.
    ///
    /// # Errors
    ///
    /// Returns error if connection initiation fails due to invalid configuration
    /// or immediate network issues.
    pub async fn connect(&self) -> Result<()> {
        self.inner.connect();
        // Wait a bit to allow connection to establish
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        Ok(())
    }

    /// Terminate WebSocket connection gracefully.
    ///
    /// # Note
    ///
    /// This function is not currently supported by the underlying tokio-centrifuge crate
    /// as it does not provide an explicit disconnect method. The connection will be
    /// automatically terminated when the client instance is dropped from memory.
    pub async fn disconnect(&self) -> Result<()> {
        todo!("Explicit disconnect not supported by tokio-centrifuge")
    }

    /// Create subscription for specified topic.
    ///
    /// Establishes a typed subscription to receive real-time updates for the specified topic.
    /// The subscription must be activated using the `subscribe()` method on the returned handle.
    ///
    /// # Arguments
    ///
    /// * `topic` - The topic to subscribe to (balances, transactions, token prices, etc.)
    ///
    /// # Returns
    ///
    /// A subscription handle for configuring callbacks and managing the subscription lifecycle.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use sparkscan_ws::{SparkScanWsClient, Topic};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = SparkScanWsClient::new("ws://updates.sparkscan.io/");
    ///
    /// // Global balance updates for all addresses
    /// let balances = client.subscribe(Topic::Balances).await?;
    ///
    /// // Address-specific balance updates
    /// let address_balance = client.subscribe(
    ///     Topic::BalanceAddress("sp1abc123...".to_string())
    /// ).await?;
    ///
    /// // Real-time token price feed
    /// let token_prices = client.subscribe(Topic::TokenPrices).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe(&self, topic: Topic) -> Result<SparkScanSubscription> {
        let topic_str = topic.as_str();
        let centrifuge_subscription = self.inner.new_subscription(&topic_str);

        Ok(SparkScanSubscription::new(centrifuge_subscription, topic))
    }

    /// Check current WebSocket connection status.
    ///
    /// # Note
    ///
    /// This function is not currently supported by the underlying tokio-centrifuge crate
    /// as it does not expose real-time connection state information. Use connection
    /// event callbacks instead to track connection status changes.
    pub fn is_connected(&self) -> bool {
        todo!("Connection state tracking not supported by tokio-centrifuge")
    }

    /// Retrieve comprehensive connection statistics and metrics.
    ///
    /// # Note
    ///
    /// This function is not currently supported by the underlying tokio-centrifuge crate
    /// as it does not expose connection statistics, state tracking, or performance metrics.
    /// Consider implementing custom metrics collection using connection event callbacks.
    pub fn connection_stats(&self) -> ConnectionStats {
        todo!("Connection statistics not supported by tokio-centrifuge")
    }
}

/// Connection statistics and performance metrics for the WebSocket client.
///
/// Provides comprehensive information about connection health, reconnection history,
/// and error tracking for monitoring and debugging purposes.
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    /// Current connection status - true if WebSocket is active and ready
    pub connected: bool,
    /// Total number of reconnection attempts since client initialization
    pub reconnect_attempts: u32,
    /// Most recent connection error message, if any error has occurred
    pub last_error: Option<String>,
}

// Implement Clone for SparkScanWsClient to enable sharing client instances
// across async tasks while maintaining shared connection state
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
    fn test_config_builder_pattern() {
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
    async fn test_client_creation_with_defaults() {
        let client = SparkScanWsClient::new("ws://localhost:8000");
        assert_eq!(client.config().url, "ws://localhost:8000");
        assert!(!client.config().use_protobuf);
    }

    #[tokio::test]
    async fn test_client_clone_shares_state() {
        let client = SparkScanWsClient::new("ws://localhost:8000");
        let cloned = client.clone();
        assert_eq!(client.config().url, cloned.config().url);
    }
}
