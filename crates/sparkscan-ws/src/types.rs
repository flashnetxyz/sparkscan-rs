//! Type definitions for SparkScan WebSocket messages.
//! 
//! This module contains the generated types from JSON schemas and helper
//! functions for message dispatching.

use serde::{Deserialize, Serialize};

// Include the generated types from build.rs
include!(concat!(env!("OUT_DIR"), "/types.rs"));

/// Enumeration of all possible SparkScan WebSocket message types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum SparkScanMessage {
    /// Balance update message
    #[serde(rename = "balance")]
    Balance(BalancePayload),
    
    /// Token balance update message
    #[serde(rename = "token_balance")]
    TokenBalance(TokenBalancePayload),
    
    /// Token price update message
    #[serde(rename = "token_price")]
    TokenPrice(TokenPricePayload),
    
    /// Token information update message
    #[serde(rename = "token")]
    Token(TokenPayload),
    
    /// Transaction update message
    #[serde(rename = "transaction")]
    Transaction(TransactionPayload),
}

impl SparkScanMessage {
    /// Get the message type as a string.
    pub fn message_type(&self) -> &'static str {
        match self {
            SparkScanMessage::Balance(_) => "balance",
            SparkScanMessage::TokenBalance(_) => "token_balance",
            SparkScanMessage::TokenPrice(_) => "token_price",
            SparkScanMessage::Token(_) => "token",
            SparkScanMessage::Transaction(_) => "transaction",
        }
    }

    /// Get the network from the message if available.
    pub fn network(&self) -> Option<String> {
        match self {
            SparkScanMessage::Balance(data) => Some(format!("{:?}", data.network)),
            SparkScanMessage::TokenBalance(data) => Some(format!("{:?}", data.network)),
            SparkScanMessage::TokenPrice(data) => Some(format!("{:?}", data.network)),
            SparkScanMessage::Token(data) => Some(format!("{:?}", data.network)),
            SparkScanMessage::Transaction(data) => Some(format!("{:?}", data.network)),
        }
    }
}

/// Topic names for WebSocket subscriptions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Topic {
    /// Balance updates for all addresses
    Balances,
    /// Balance updates for a specific address
    AddressBalance(String),
    /// Token balance updates
    TokenBalances,
    /// Token balance updates for a specific address
    AddressTokenBalance(String),
    /// Token balance updates for a specific token
    TokenBalance(String),
    /// Token price updates
    TokenPrices,
    /// Token price updates for a specific token
    TokenPrice(String),
    /// Token information updates
    Tokens,
    /// Token information for a specific token
    Token(String),
    /// Transaction updates
    Transactions,
    /// Transaction updates for a specific address
    AddressTransactions(String),
    /// Custom topic
    Custom(String),
}

impl Topic {
    /// Convert topic to string for subscription.
    pub fn as_str(&self) -> String {
        match self {
            Topic::Balances => "balances".to_string(),
            Topic::AddressBalance(address) => format!("balances:{}", address),
            Topic::TokenBalances => "token_balances".to_string(),
            Topic::AddressTokenBalance(address) => format!("token_balances:{}", address),
            Topic::TokenBalance(token) => format!("token_balance:{}", token),
            Topic::TokenPrices => "token_prices".to_string(),
            Topic::TokenPrice(token) => format!("token_price:{}", token),
            Topic::Tokens => "tokens".to_string(),
            Topic::Token(token) => format!("token:{}", token),
            Topic::Transactions => "transactions".to_string(),
            Topic::AddressTransactions(address) => format!("transactions:{}", address),
            Topic::Custom(topic) => topic.clone(),
        }
    }

    /// Parse a topic string into a Topic enum.
    pub fn from_str(topic: &str) -> Self {
        if let Some(address) = topic.strip_prefix("balances:") {
            Topic::AddressBalance(address.to_string())
        } else if let Some(address) = topic.strip_prefix("token_balances:") {
            Topic::AddressTokenBalance(address.to_string())
        } else if let Some(token) = topic.strip_prefix("token_balance:") {
            Topic::TokenBalance(token.to_string())
        } else if let Some(token) = topic.strip_prefix("token_price:") {
            Topic::TokenPrice(token.to_string())
        } else if let Some(token) = topic.strip_prefix("token:") {
            Topic::Token(token.to_string())
        } else if let Some(address) = topic.strip_prefix("transactions:") {
            Topic::AddressTransactions(address.to_string())
        } else {
            match topic {
                "balances" => Topic::Balances,
                "token_balances" => Topic::TokenBalances,
                "token_prices" => Topic::TokenPrices,
                "tokens" => Topic::Tokens,
                "transactions" => Topic::Transactions,
                _ => Topic::Custom(topic.to_string()),
            }
        }
    }
}

/// Helper function to try parsing a message based on expected topic type.
pub fn parse_message_for_topic(topic: &Topic, data: &[u8]) -> crate::error::Result<SparkScanMessage> {
    use crate::error::SparkScanWsError;

    match topic {
        Topic::Balances | Topic::AddressBalance(_) => {
            let payload: BalancePayload = serde_json::from_slice(data)?;
            Ok(SparkScanMessage::Balance(payload))
        }
        Topic::TokenBalances | Topic::AddressTokenBalance(_) | Topic::TokenBalance(_) => {
            let payload: TokenBalancePayload = serde_json::from_slice(data)?;
            Ok(SparkScanMessage::TokenBalance(payload))
        }
        Topic::TokenPrices | Topic::TokenPrice(_) => {
            let payload: TokenPricePayload = serde_json::from_slice(data)?;
            Ok(SparkScanMessage::TokenPrice(payload))
        }
        Topic::Tokens | Topic::Token(_) => {
            let payload: TokenPayload = serde_json::from_slice(data)?;
            Ok(SparkScanMessage::Token(payload))
        }
        Topic::Transactions | Topic::AddressTransactions(_) => {
            let payload: TransactionPayload = serde_json::from_slice(data)?;
            Ok(SparkScanMessage::Transaction(payload))
        }
        Topic::Custom(_) => {
            // For custom topics, try to parse as a generic SparkScanMessage
            serde_json::from_slice(data)
                .map_err(|e| SparkScanWsError::SerializationError(e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_parsing() {
        assert_eq!(Topic::from_str("balances"), Topic::Balances);
        assert_eq!(
            Topic::from_str("balances:sp1abc123"),
            Topic::AddressBalance("sp1abc123".to_string())
        );
        assert_eq!(Topic::from_str("token_balances"), Topic::TokenBalances);
        assert_eq!(Topic::from_str("custom_topic"), Topic::Custom("custom_topic".to_string()));
    }

    #[test]
    fn test_topic_to_string() {
        assert_eq!(Topic::Balances.as_str(), "balances");
        assert_eq!(
            Topic::AddressBalance("sp1abc123".to_string()).as_str(),
            "balances:sp1abc123"
        );
        assert_eq!(Topic::TokenBalances.as_str(), "token_balances");
        assert_eq!(
            Topic::Custom("custom".to_string()).as_str(),
            "custom"
        );
    }
}