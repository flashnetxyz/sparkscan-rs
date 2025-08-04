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
    /// Balance updates filtered by network
    BalanceNetwork(String),
    /// Balance updates for a specific address
    BalanceAddress(String),
    
    /// Token balance updates for all
    TokenBalances,
    /// Token balance updates filtered by network
    TokenBalanceNetwork(String),
    /// Token balance updates for a specific token identifier
    TokenBalanceIdentifier(String),
    /// Token balance updates for a specific address
    TokenBalanceAddress(String),
    
    /// Token price updates for all
    TokenPrices,
    /// Token price updates filtered by network
    TokenPriceNetwork(String),
    /// Token price updates for a specific token identifier
    TokenPriceIdentifier(String),
    
    /// Transaction updates for all
    Transactions,
    /// Transaction updates filtered by network
    TransactionNetwork(String),
    /// Incoming transaction updates for network and field (address/bitcoin/lightning)
    TransactionIn(String, String),
    /// Outgoing transaction updates for network and field (address/bitcoin/lightning)
    TransactionOut(String, String),
    
    /// Token information updates for all
    Tokens,
    /// Token information for a specific token identifier
    TokenIdentifier(String),
    /// Token information filtered by network
    TokenNetwork(String),
    /// Token information filtered by issuer
    TokenIssuer(String),
}

impl Topic {
    /// Convert topic to string for subscription.
    pub fn as_str(&self) -> String {
        match self {
            Topic::Balances => "balances".to_string(),
            Topic::BalanceNetwork(network) => format!("/balance/network/{}", network),
            Topic::BalanceAddress(address) => format!("/balance/address/{}", address),
            
            Topic::TokenBalances => "token_balances".to_string(),
            Topic::TokenBalanceNetwork(network) => format!("/token_balance/network/{}", network),
            Topic::TokenBalanceIdentifier(identifier) => format!("/token_balance/identifier/{}", identifier),
            Topic::TokenBalanceAddress(address) => format!("/token_balance/address/{}", address),
            
            Topic::TokenPrices => "token_prices".to_string(),
            Topic::TokenPriceNetwork(network) => format!("/token_price/network/{}", network),
            Topic::TokenPriceIdentifier(identifier) => format!("/token_price/identifier/{}", identifier),
            
            Topic::Transactions => "transactions".to_string(),
            Topic::TransactionNetwork(network) => format!("/transaction/network/{}", network),
            Topic::TransactionIn(network, field) => format!("/transaction/in/{}/{}", network, field),
            Topic::TransactionOut(network, field) => format!("/transaction/out/{}/{}", network, field),
            
            Topic::Tokens => "tokens".to_string(),
            Topic::TokenIdentifier(identifier) => format!("/token/identifier/{}", identifier),
            Topic::TokenNetwork(network) => format!("/token/network/{}", network),
            Topic::TokenIssuer(issuer) => format!("/token/issuer/{}", issuer),
        }
    }

    /// Parse a topic string into a Topic enum.
    pub fn from_str(topic: &str) -> Self {
        // Handle basic topics first
        match topic {
            "balances" => return Topic::Balances,
            "token_balances" => return Topic::TokenBalances,
            "token_prices" => return Topic::TokenPrices,
            "transactions" => return Topic::Transactions,
            "tokens" => return Topic::Tokens,
            _ => {}
        }
        
        // Handle path-based topics
        if let Some(rest) = topic.strip_prefix("/balance/network/") {
            Topic::BalanceNetwork(rest.to_string())
        } else if let Some(rest) = topic.strip_prefix("/balance/address/") {
            Topic::BalanceAddress(rest.to_string())
        } else if let Some(rest) = topic.strip_prefix("/token_balance/network/") {
            Topic::TokenBalanceNetwork(rest.to_string())
        } else if let Some(rest) = topic.strip_prefix("/token_balance/identifier/") {
            Topic::TokenBalanceIdentifier(rest.to_string())
        } else if let Some(rest) = topic.strip_prefix("/token_balance/address/") {
            Topic::TokenBalanceAddress(rest.to_string())
        } else if let Some(rest) = topic.strip_prefix("/token_price/network/") {
            Topic::TokenPriceNetwork(rest.to_string())
        } else if let Some(rest) = topic.strip_prefix("/token_price/identifier/") {
            Topic::TokenPriceIdentifier(rest.to_string())
        } else if let Some(rest) = topic.strip_prefix("/transaction/network/") {
            Topic::TransactionNetwork(rest.to_string())
        } else if let Some(rest) = topic.strip_prefix("/transaction/in/") {
            let parts: Vec<&str> = rest.splitn(2, '/').collect();
            if parts.len() == 2 {
                Topic::TransactionIn(parts[0].to_string(), parts[1].to_string())
            } else {
                panic!("Invalid transaction in topic format: {}. Expected /transaction/in/network/field", topic);
            }
        } else if let Some(rest) = topic.strip_prefix("/transaction/out/") {
            let parts: Vec<&str> = rest.splitn(2, '/').collect();
            if parts.len() == 2 {
                Topic::TransactionOut(parts[0].to_string(), parts[1].to_string())
            } else {
                panic!("Invalid transaction out topic format: {}. Expected /transaction/out/network/field", topic);
            }
        } else if let Some(rest) = topic.strip_prefix("/token/identifier/") {
            Topic::TokenIdentifier(rest.to_string())
        } else if let Some(rest) = topic.strip_prefix("/token/network/") {
            Topic::TokenNetwork(rest.to_string())
        } else if let Some(rest) = topic.strip_prefix("/token/issuer/") {
            Topic::TokenIssuer(rest.to_string())
        } else {
            panic!("Unknown topic: {}. Only predefined topics are supported.", topic);
        }
    }
}

/// Helper function to try parsing a message based on expected topic type.
pub fn parse_message_for_topic(topic: &Topic, data: &[u8]) -> crate::error::Result<SparkScanMessage> {
    // Debug: Log the raw data structure to understand the WebSocket message format
    #[cfg(feature = "tracing")]
    {
        if let Ok(raw_str) = std::str::from_utf8(data) {
            tracing::debug!("Raw WebSocket data for topic {:?}: {}", topic, raw_str);
        }
    }

    // First, try to parse as a JSON value
    let json_value: serde_json::Value = serde_json::from_slice(data)?;
    
    // Handle the case where the data itself is a JSON string that needs to be parsed again
    let payload_data = if json_value.is_string() {
        // The entire data is a JSON string, parse it again
        let json_str = json_value.as_str().unwrap();
        serde_json::from_str(json_str)?
    } else if let Some(data_field) = json_value.get("data") {
        // Check if there's a "data" envelope field
        if let Some(data_str) = data_field.as_str() {
            // The data field is a JSON string, parse it again
            serde_json::from_str(data_str)?
        } else {
            // The data field is already a JSON object
            data_field.clone()
        }
    } else {
        // Use the entire JSON value as-is
        json_value
    };

    match topic {
        Topic::Balances | Topic::BalanceNetwork(_) | Topic::BalanceAddress(_) => {
            let payload: BalancePayload = serde_json::from_value(payload_data)?;
            Ok(SparkScanMessage::Balance(payload))
        }
        Topic::TokenBalances | Topic::TokenBalanceNetwork(_) | Topic::TokenBalanceIdentifier(_) | Topic::TokenBalanceAddress(_) => {
            let payload: TokenBalancePayload = serde_json::from_value(payload_data)?;
            Ok(SparkScanMessage::TokenBalance(payload))
        }
        Topic::TokenPrices | Topic::TokenPriceNetwork(_) | Topic::TokenPriceIdentifier(_) => {
            let payload: TokenPricePayload = serde_json::from_value(payload_data)?;
            Ok(SparkScanMessage::TokenPrice(payload))
        }
        Topic::Tokens | Topic::TokenIdentifier(_) | Topic::TokenNetwork(_) | Topic::TokenIssuer(_) => {
            let payload: TokenPayload = serde_json::from_value(payload_data)?;
            Ok(SparkScanMessage::Token(payload))
        }
        Topic::Transactions | Topic::TransactionNetwork(_) | Topic::TransactionIn(_, _) | Topic::TransactionOut(_, _) => {
            let payload: TransactionPayload = serde_json::from_value(payload_data)?;
            Ok(SparkScanMessage::Transaction(payload))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_parsing() {
        // Basic topics
        assert_eq!(Topic::from_str("balances"), Topic::Balances);
        assert_eq!(Topic::from_str("token_balances"), Topic::TokenBalances);
        assert_eq!(Topic::from_str("token_prices"), Topic::TokenPrices);
        assert_eq!(Topic::from_str("transactions"), Topic::Transactions);
        assert_eq!(Topic::from_str("tokens"), Topic::Tokens);
        
        // Balance topics
        assert_eq!(
            Topic::from_str("/balance/network/mainnet"),
            Topic::BalanceNetwork("mainnet".to_string())
        );
        assert_eq!(
            Topic::from_str("/balance/address/sp1abc123"),
            Topic::BalanceAddress("sp1abc123".to_string())
        );
        
        // Token balance topics
        assert_eq!(
            Topic::from_str("/token_balance/network/mainnet"),
            Topic::TokenBalanceNetwork("mainnet".to_string())
        );
        assert_eq!(
            Topic::from_str("/token_balance/identifier/btkn1xyz"),
            Topic::TokenBalanceIdentifier("btkn1xyz".to_string())
        );
        assert_eq!(
            Topic::from_str("/token_balance/address/sp1def456"),
            Topic::TokenBalanceAddress("sp1def456".to_string())
        );
        
        // Transaction topics
        assert_eq!(
            Topic::from_str("/transaction/in/mainnet/sp1abc123"),
            Topic::TransactionIn("mainnet".to_string(), "sp1abc123".to_string())
        );
        assert_eq!(
            Topic::from_str("/transaction/out/mainnet/bitcoin"),
            Topic::TransactionOut("mainnet".to_string(), "bitcoin".to_string())
        );
    }

    #[test]
    fn test_topic_to_string() {
        // Basic topics
        assert_eq!(Topic::Balances.as_str(), "balances");
        assert_eq!(Topic::TokenBalances.as_str(), "token_balances");
        
        // Balance topics  
        assert_eq!(
            Topic::BalanceNetwork("mainnet".to_string()).as_str(),
            "/balance/network/mainnet"
        );
        assert_eq!(
            Topic::BalanceAddress("sp1abc123".to_string()).as_str(),
            "/balance/address/sp1abc123"
        );
        
        // Token balance topics
        assert_eq!(
            Topic::TokenBalanceIdentifier("btkn1xyz".to_string()).as_str(),
            "/token_balance/identifier/btkn1xyz"
        );
        
        // Transaction topics
        assert_eq!(
            Topic::TransactionIn("mainnet".to_string(), "sp1abc123".to_string()).as_str(),
            "/transaction/in/mainnet/sp1abc123"
        );
        assert_eq!(
            Topic::TransactionOut("mainnet".to_string(), "lightning".to_string()).as_str(),
            "/transaction/out/mainnet/lightning"
        );
    }
}