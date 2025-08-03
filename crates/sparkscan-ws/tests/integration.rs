//! Integration tests for SparkScan WebSocket SDK.
//!
//! These tests verify that the SDK components work together correctly.
//! Note: Most tests here are unit-style tests since they don't require
//! a real WebSocket server. Full integration tests would require a test
//! server setup.

use sparkscan_ws::{
    SparkScanWsClient, SparkScanWsConfig, Topic, SparkScanMessage,
    types::{BalancePayload, TokenBalancePayload, parse_message_for_topic, Network},
    subscription::SubscriptionManager,
};
use chrono::{DateTime, Utc};

#[tokio::test]
async fn test_client_creation_and_config() {
    let client = SparkScanWsClient::new("ws://example.com/websocket");
    assert_eq!(client.config().url, "ws://example.com/websocket");
    assert!(!client.config().use_protobuf);
    assert_eq!(client.config().connection_timeout, 30);
    assert!(client.config().auto_reconnect);
}

#[tokio::test]
async fn test_custom_config() {
    let config = SparkScanWsConfig::new("ws://example.com/websocket")
        .with_protobuf(true)
        .with_timeout(60)
        .with_auto_reconnect(false)
        .with_max_reconnect_attempts(10)
        .with_reconnect_delay(5000);

    let client = SparkScanWsClient::with_config(config);
    assert_eq!(client.config().url, "ws://example.com/websocket");
    assert!(client.config().use_protobuf);
    assert_eq!(client.config().connection_timeout, 60);
    assert!(!client.config().auto_reconnect);
    assert_eq!(client.config().max_reconnect_attempts, 10);
    assert_eq!(client.config().reconnect_delay, 5000);
}

#[test]
fn test_topic_enum_completeness() {
    // Test all topic variants
    let topics = vec![
        Topic::Balances,
        Topic::AddressBalance("sp1test".to_string()),
        Topic::TokenBalances,
        Topic::AddressTokenBalance("sp1test".to_string()),
        Topic::TokenBalance("btkn1test".to_string()),
        Topic::TokenPrices,
        Topic::TokenPrice("btkn1test".to_string()),
        Topic::Tokens,
        Topic::Token("btkn1test".to_string()),
        Topic::Transactions,
        Topic::AddressTransactions("sp1test".to_string()),
    ];

    for topic in topics {
        let topic_str = topic.as_str();
        assert!(!topic_str.is_empty());
        
        // Test round-trip conversion
        let parsed = Topic::from_str(&topic_str);
        assert_eq!(parsed.as_str(), topic_str);
    }
}

#[test]
fn test_message_type_detection() {
    // Test balance message
    let balance = BalancePayload {
        address: "sp1test".to_string(),
        network: Network::Regtest,
        soft_balance: "100".to_string(),
        hard_balance: "90".to_string(),
        processed_at: "2025-08-02T20:02:54.035000Z".parse::<DateTime<Utc>>().unwrap(),
    };
    let message = SparkScanMessage::Balance(balance);
    assert_eq!(message.message_type(), "balance");
    assert!(message.network().is_some());

    // Test token balance message
    let token_balance = TokenBalancePayload {
        network: Network::Mainnet,
        address: "sp1test".to_string(),
        token_address: "btkn1test".to_string(),
        balance: "500".to_string(),
        processed_at: "2025-08-02T20:02:54.035000Z".parse::<DateTime<Utc>>().unwrap(),
    };
    let message = SparkScanMessage::TokenBalance(token_balance);
    assert_eq!(message.message_type(), "token_balance");
    assert!(message.network().is_some());
}

#[test]
fn test_message_parsing_with_real_schema_data() {
    // Test with data that matches the actual JSON schemas
    
    // Balance message using real API response format
    let balance_json = r#"{
        "address": "sp1pgssx6rwqjer2xsmhe5x6mg6ng0cfu77q58vtcz9f0emuuzftnl7zvv6qujs5s",
        "hard_balance": "561", 
        "network": "MAINNET",
        "processed_at": "2025-08-03T13:26:31.271938Z",
        "soft_balance": "561"
    }"#;
    
    let result = parse_message_for_topic(&Topic::Balances, balance_json.as_bytes());
    assert!(result.is_ok());
    
    if let Ok(SparkScanMessage::Balance(balance)) = result {
        // Just verify the structure is correct and fields are populated
        assert!(!balance.address.is_empty());
        assert!(!balance.hard_balance.is_empty());
        assert!(!balance.soft_balance.is_empty());
        // Network is an enum, check for valid SparkScan networks (mainnet and regtest)
        assert!(matches!(balance.network, Network::Mainnet | Network::Regtest));
        // DateTime exists - we can't check is_empty on DateTime
    } else {
        panic!("Expected Balance message");
    }

    // Token balance message
    let token_balance_json = r#"{
        "network": "MAINNET",
        "address": "sp1example",
        "token_address": "btkn1example",
        "balance": "1000",
        "processed_at": "2025-08-02T20:02:54.035000Z"
    }"#;
    
    let result = parse_message_for_topic(&Topic::TokenBalances, token_balance_json.as_bytes());
    assert!(result.is_ok());
    
    if let Ok(SparkScanMessage::TokenBalance(token_balance)) = result {
        // Test that required fields are present and valid
        assert!(matches!(token_balance.network, Network::Mainnet | Network::Regtest));
        assert!(!token_balance.address.is_empty());
        assert!(!token_balance.token_address.is_empty());
        assert!(!token_balance.balance.is_empty());
        // DateTime exists - we can't check is_empty on DateTime
    } else {
        panic!("Expected TokenBalance message");
    }
}

#[test]
fn test_subscription_manager() {
    let mut manager = SubscriptionManager::new();
    assert!(manager.is_empty());
    assert_eq!(manager.len(), 0);

    // Note: We can't create real subscriptions without a WebSocket connection
    // This test focuses on the manager's data structure functionality
    
    assert!(manager.get("nonexistent").is_none());
    assert!(manager.remove("nonexistent").is_none());
}

#[test]
fn test_error_types() {
    use sparkscan_ws::SparkScanWsError;
    
    // Test error creation methods
    let conn_err = SparkScanWsError::connection("Connection failed");
    assert!(matches!(conn_err, SparkScanWsError::ConnectionError(_)));
    
    let sub_err = SparkScanWsError::subscription("Subscription failed");
    assert!(matches!(sub_err, SparkScanWsError::SubscriptionError(_)));
    
    let unknown_err = SparkScanWsError::unknown_message_type("unknown");
    assert!(matches!(unknown_err, SparkScanWsError::UnknownMessageType { .. }));
    
    let config_err = SparkScanWsError::config("Invalid config");
    assert!(matches!(config_err, SparkScanWsError::ConfigError(_)));
    
    // Test that errors implement Display
    assert!(!format!("{}", conn_err).is_empty());
    assert!(!format!("{}", sub_err).is_empty());
    assert!(!format!("{}", unknown_err).is_empty());
    assert!(!format!("{}", config_err).is_empty());
}

#[test]
fn test_constants() {
    use sparkscan_ws::{VERSION, DEFAULT_MAINNET_URL, DEFAULT_REGTEST_URL};
    
    assert!(!VERSION.is_empty());
    assert!(DEFAULT_MAINNET_URL.starts_with("wss://"));
    assert!(DEFAULT_REGTEST_URL.starts_with("wss://"));
    assert_ne!(DEFAULT_MAINNET_URL, DEFAULT_REGTEST_URL);
}

#[test]
fn test_prelude_exports() {
    // Test that all important types are available in prelude
    use sparkscan_ws::prelude::*;
    
    // This should compile without errors, proving all types are exported
    let _client: Option<SparkScanWsClient> = None;
    let _config: Option<SparkScanWsConfig> = None;
    let _subscription: Option<SparkScanSubscription> = None;
    let _message: Option<SparkScanMessage> = None;
    let _topic: Option<Topic> = None;
    let _result: Option<Result<()>> = None;
    let _error: Option<SparkScanWsError> = None;
    let _balance: Option<BalancePayload> = None;
    let _token_balance: Option<TokenBalancePayload> = None;
    let _token_price: Option<TokenPricePayload> = None;
    let _token: Option<TokenPayload> = None;
    let _transaction: Option<TransactionPayload> = None;
}

#[tokio::test]
async fn test_async_operations() {
    // Test async creation of subscriptions
    let client = SparkScanWsClient::new("ws://localhost:8000/websocket");
    
    // These operations should not panic even without a real connection
    let balance_subscription = client.subscribe(Topic::Balances).await;
    assert!(balance_subscription.is_ok());
    
    // Test connection operations (they won't succeed but shouldn't panic)
    let connect_result = client.connect().await;
    // Connection will likely fail but the method should be callable
    let _ = connect_result;
    
    let disconnect_result = client.disconnect().await;
    assert!(disconnect_result.is_ok());
}

#[test]
fn test_real_api_data_compatibility() {
    // Test with real transaction data from Sparkscan API (raw payload format)
    let transaction_json = r#"{
        "id": "0198701c-c7cc-7ba0-8042-5c0d55e844b2",
        "type": "lightning_payment",
        "status": "confirmed",
        "amount_sats": "14",
        "from_identifier": "sp1pgssxsj2dzegse2mdzgmxdr2rd0vrw5pusf4mh4esf6xe5lkycrv5npdpqqzst",
        "to_identifier": "Lightning Network",
        "processed_at": "2025-08-03T13:26:31.271938Z",
        "updated_at": "2025-08-03T13:26:36.180147Z",
        "network": "MAINNET"
    }"#;

    // Test parsing a transaction payload
    let result = parse_message_for_topic(&Topic::Transactions, transaction_json.as_bytes());
    assert!(result.is_ok());

    if let Ok(SparkScanMessage::Transaction(tx)) = result {
        // Test that required fields are present and valid
        assert!(!tx.id.is_empty());
        assert!(!tx.type_.is_empty());
        assert!(!tx.status.is_empty());
        assert!(tx.amount_sats.is_some());
        assert!(matches!(tx.network, Network::Mainnet | Network::Regtest));
    } else {
        panic!("Expected Transaction message");
    }

    // Test with real token data structure (raw payload format)
    let token_json = r#"{
        "address": "btkn1hnvhjkd88nq7gmhxa0t7ac0s3x93xnsnvu3h0u9e6vyedlm6ksqukemd",
        "name": "SatsSpark",
        "ticker": "SATS", 
        "decimals": 8,
        "holders": 5272,
        "is_freezable": false,
        "issuer": "026a0b6af2f447a722fb5c3313872a2910187ac070e8a2bc1cf3dc2227e0f87ef2",
        "max_mcap": null,
        "max_supply": "500000000000000000",
        "network": "MAINNET",
        "price_sats": null,
        "pricing_source": null,
        "calculated_at": null,
        "circulating_mcap": null,
        "circulating_supply": null
    }"#;

    let result = parse_message_for_topic(&Topic::Tokens, token_json.as_bytes());
    assert!(result.is_ok());

    if let Ok(SparkScanMessage::Token(token)) = result {
        // Test that required fields are present and valid
        assert!(!token.name.is_empty());
        assert!(!token.ticker.is_empty());
        assert!(token.decimals > 0);
        assert!(token.holders >= 0);
        assert!(matches!(token.network, Network::Mainnet | Network::Regtest));
        assert!(!token.address.is_empty());
        assert!(!token.issuer.is_empty());
    } else {
        panic!("Expected Token message");
    }
}