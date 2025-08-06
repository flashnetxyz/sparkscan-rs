//! Type definitions for SparkScan WebSocket messages.
//!
//! This module contains the generated types from JSON schemas and helper
//! functions for message dispatching.

use serde::{Deserialize, Serialize};
use tokio_centrifuge::utils::decode_json;

// Include the generated types from build.rs
include!(concat!(env!("OUT_DIR"), "/types.rs"));

/// Enumeration of all possible SparkScan WebSocket message types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum SparkScanMessage {
    /// Balance update message
    #[serde(rename = "balance")]
    Balance(balance::BalancePayload),

    /// Token balance update message
    #[serde(rename = "token_balance")]
    TokenBalance(token_balance::TokenBalancePayload),

    /// Token price update message
    #[serde(rename = "token_price")]
    TokenPrice(token_price::TokenPricePayload),

    /// Token information update message
    #[serde(rename = "token")]
    Token(token::TokenPayload),

    /// Transaction update message
    #[serde(rename = "transaction")]
    Transaction(transaction::TransactionPayload),
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
            Topic::TokenBalanceIdentifier(identifier) => {
                format!("/token_balance/identifier/{}", identifier)
            }
            Topic::TokenBalanceAddress(address) => format!("/token_balance/address/{}", address),

            Topic::TokenPrices => "token_prices".to_string(),
            Topic::TokenPriceNetwork(network) => format!("/token_price/network/{}", network),
            Topic::TokenPriceIdentifier(identifier) => {
                format!("/token_price/identifier/{}", identifier)
            }

            Topic::Transactions => "transactions".to_string(),
            Topic::TransactionNetwork(network) => format!("/transaction/network/{}", network),
            Topic::TransactionIn(network, field) => {
                format!("/transaction/in/{}/{}", network, field)
            }
            Topic::TransactionOut(network, field) => {
                format!("/transaction/out/{}/{}", network, field)
            }

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
            panic!(
                "Unknown topic: {}. Only predefined topics are supported.",
                topic
            );
        }
    }
}

/// Extract payload data from potentially nested JSON structures
fn extract_payload_data(json_value: serde_json::Value) -> crate::error::Result<serde_json::Value> {
    // Handle different JSON envelope patterns that Centrifugo/WebSocket servers might use

    // Case 1: Data is a double-encoded JSON string (most common case for Centrifugo)
    if json_value.is_string() {
        let json_str = json_value.as_str().unwrap();
        return serde_json::from_str(json_str)
            .map_err(|e| crate::error::SparkScanWsError::SerializationError(e));
    }

    // Case 2: Data is wrapped in a "data" field
    if let Some(data_field) = json_value.get("data") {
        if data_field.is_string() {
            // Data field contains a JSON string
            let data_str = data_field.as_str().unwrap();
            return serde_json::from_str(data_str)
                .map_err(|e| crate::error::SparkScanWsError::SerializationError(e));
        } else {
            // Data field is already a JSON object
            return Ok(data_field.clone());
        }
    }

    // Case 3: Data is wrapped in a "payload" field
    if let Some(payload_field) = json_value.get("payload") {
        if payload_field.is_string() {
            let payload_str = payload_field.as_str().unwrap();
            return serde_json::from_str(payload_str)
                .map_err(|e| crate::error::SparkScanWsError::SerializationError(e));
        } else {
            return Ok(payload_field.clone());
        }
    }

    // Case 4: Look for message envelope patterns
    if let Some(message_field) = json_value.get("message") {
        if message_field.is_string() {
            let message_str = message_field.as_str().unwrap();
            return serde_json::from_str(message_str)
                .map_err(|e| crate::error::SparkScanWsError::SerializationError(e));
        } else {
            return Ok(message_field.clone());
        }
    }

    // Case 5: Use the entire JSON value as-is (direct payload)
    Ok(json_value)
}

/// Create a fallback TransactionPayload from any JSON, putting unmappable fields into token_io_details
fn create_fallback_transaction_payload(
    json_data: serde_json::Value,
) -> crate::error::Result<transaction::TransactionPayload> {
    let obj = json_data.as_object().ok_or_else(|| {
        crate::error::SparkScanWsError::InvalidMessageFormat("Expected JSON object".to_string())
    })?;

    // Extract required fields with defaults if missing
    let id = obj
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let network = obj
        .get("network")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(transaction::Network::Regtest);

    let type_ = obj
        .get("type")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(transaction::Type::Unknown);

    let status = obj
        .get("status")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(transaction::Status::Pending);

    let processed_at = obj
        .get("processed_at")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|| chrono::Utc::now());

    // Extract optional fields
    let amount_sats = obj
        .get("amount_sats")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let token_amount = obj
        .get("token_amount")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let token_address = obj
        .get("token_address")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let from_identifier = obj
        .get("from_identifier")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let to_identifier = obj
        .get("to_identifier")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let bitcoin_txid = obj
        .get("bitcoin_txid")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let updated_at = obj
        .get("updated_at")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    let expired_time = obj
        .get("expired_time")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    // Create token_io_details containing all the original data for debugging/analysis
    let mut token_io_details = serde_json::Map::new();

    // Copy the original token_io_details if it exists
    if let Some(original_io_details) = obj.get("token_io_details") {
        token_io_details.insert("original".to_string(), original_io_details.clone());
    }

    // Store any unmapped fields as fallback data
    let mapped_fields = [
        "id",
        "network",
        "type",
        "status",
        "processed_at",
        "amount_sats",
        "token_amount",
        "token_address",
        "from_identifier",
        "to_identifier",
        "bitcoin_txid",
        "updated_at",
        "expired_time",
        "token_io_details",
    ];

    let mut unmapped = serde_json::Map::new();
    for (key, value) in obj {
        if !mapped_fields.contains(&key.as_str()) {
            unmapped.insert(key.clone(), value.clone());
        }
    }

    if !unmapped.is_empty() {
        token_io_details.insert(
            "unmapped_fields".to_string(),
            serde_json::Value::Object(unmapped),
        );
    }

    Ok(transaction::TransactionPayload {
        id,
        network,
        type_,
        status,
        processed_at,
        amount_sats,
        token_amount,
        token_address,
        from_identifier,
        to_identifier,
        bitcoin_txid,
        token_io_details: if token_io_details.is_empty() {
            None
        } else {
            Some(token_io_details)
        },
        updated_at,
        expired_time,
    })
}

/// Helper function to try parsing a message based on expected topic type.
pub fn parse_message_for_topic(
    topic: &Topic,
    data: &[u8],
) -> crate::error::Result<SparkScanMessage> {
    // Debug: Log the raw data structure to understand the WebSocket message format
    #[cfg(feature = "tracing")]
    {
        if let Ok(raw_str) = std::str::from_utf8(data) {
            tracing::debug!("Raw WebSocket data for topic {:?}: {}", topic, raw_str);
        }
    }

    // First, try to parse as a JSON value using tokio-centrifuge's decode_json
    let json_value: serde_json::Value = decode_json(data).map_err(|e| {
        crate::error::SparkScanWsError::InvalidMessageFormat(format!(
            "Failed to decode JSON: {:?}",
            e
        ))
    })?;

    // Handle nested JSON scenarios more robustly
    let payload_data = extract_payload_data(json_value)?;

    // Parse the message based on topic type, with transaction fallback
    match topic {
        Topic::Balances | Topic::BalanceNetwork(_) | Topic::BalanceAddress(_) => {
            let payload: balance::BalancePayload = serde_json::from_value(payload_data)?;
            Ok(SparkScanMessage::Balance(payload))
        }
        Topic::TokenBalances
        | Topic::TokenBalanceNetwork(_)
        | Topic::TokenBalanceIdentifier(_)
        | Topic::TokenBalanceAddress(_) => {
            let payload: token_balance::TokenBalancePayload = serde_json::from_value(payload_data)?;
            Ok(SparkScanMessage::TokenBalance(payload))
        }
        Topic::TokenPrices | Topic::TokenPriceNetwork(_) | Topic::TokenPriceIdentifier(_) => {
            let payload: token_price::TokenPricePayload = serde_json::from_value(payload_data)?;
            Ok(SparkScanMessage::TokenPrice(payload))
        }
        Topic::Tokens
        | Topic::TokenIdentifier(_)
        | Topic::TokenNetwork(_)
        | Topic::TokenIssuer(_) => {
            let payload: token::TokenPayload = serde_json::from_value(payload_data)?;
            Ok(SparkScanMessage::Token(payload))
        }
        Topic::Transactions
        | Topic::TransactionNetwork(_)
        | Topic::TransactionIn(_, _)
        | Topic::TransactionOut(_, _) => {
            // First try normal parsing, then fallback to field mapping
            match serde_json::from_value::<transaction::TransactionPayload>(payload_data.clone()) {
                Ok(payload) => Ok(SparkScanMessage::Transaction(payload)),
                Err(_) => {
                    // Create fallback transaction payload with unmappable fields in token_io_details
                    let fallback_payload = create_fallback_transaction_payload(payload_data)?;
                    Ok(SparkScanMessage::Transaction(fallback_payload))
                }
            }
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
