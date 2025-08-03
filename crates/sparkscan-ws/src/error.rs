//! Error types for the SparkScan WebSocket client.

use thiserror::Error;

/// The main error type for SparkScan WebSocket operations.
#[derive(Error, Debug)]
pub enum SparkScanWsError {
    /// WebSocket connection error
    #[error("WebSocket connection error: {0}")]
    ConnectionError(String),

    /// WebSocket subscription error
    #[error("WebSocket subscription error: {0}")]
    SubscriptionError(String),

    /// Message serialization error
    #[error("Message serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Unknown message type received
    #[error("Unknown message type: {message_type}")]
    UnknownMessageType { 
        /// The unknown message type that was received
        message_type: String 
    },

    /// Invalid message format
    #[error("Invalid message format: {0}")]
    InvalidMessageFormat(String),

    /// Client not connected
    #[error("Client is not connected")]
    NotConnected,

    /// Subscription not found
    #[error("Subscription not found: {topic}")]
    SubscriptionNotFound { 
        /// The topic that was not found
        topic: String 
    },

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Authentication error
    #[error("Authentication error: {0}")]
    AuthError(String),

    /// Rate limit error
    #[error("Rate limit exceeded")]
    RateLimitError,

    /// Generic error
    #[error("SparkScan WebSocket error: {0}")]
    Generic(#[from] anyhow::Error),
}

/// Result type alias for SparkScan WebSocket operations.
pub type Result<T> = std::result::Result<T, SparkScanWsError>;

impl SparkScanWsError {
    /// Create a new connection error.
    pub fn connection<T: Into<String>>(msg: T) -> Self {
        Self::ConnectionError(msg.into())
    }

    /// Create a new subscription error.
    pub fn subscription<T: Into<String>>(msg: T) -> Self {
        Self::SubscriptionError(msg.into())
    }

    /// Create a new unknown message type error.
    pub fn unknown_message_type<T: Into<String>>(message_type: T) -> Self {
        Self::UnknownMessageType {
            message_type: message_type.into(),
        }
    }

    /// Create a new invalid message format error.
    pub fn invalid_format<T: Into<String>>(msg: T) -> Self {
        Self::InvalidMessageFormat(msg.into())
    }

    /// Create a new subscription not found error.
    pub fn subscription_not_found<T: Into<String>>(topic: T) -> Self {
        Self::SubscriptionNotFound {
            topic: topic.into(),
        }
    }

    /// Create a new configuration error.
    pub fn config<T: Into<String>>(msg: T) -> Self {
        Self::ConfigError(msg.into())
    }

    /// Create a new authentication error.
    pub fn auth<T: Into<String>>(msg: T) -> Self {
        Self::AuthError(msg.into())
    }
}