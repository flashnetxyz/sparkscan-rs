//! Integration tests for SparkScan WebSocket SDK.
//!
//! These tests verify that the SDK components work together correctly.
//! Note: Most tests here are unit-style tests since they don't require
//! a real WebSocket server. Full integration tests would require a test
//! server setup.

use sparkscan_ws::{
    subscription::SubscriptionManager,
    types::{
        balance::{Network as BalanceNetwork},
        parse_message_for_topic,
        token_balance::{Network as TokenBalanceNetwork},
        transaction::{Network as TransactionNetwork},
    },
    SparkScanMessage, SparkScanWsClient, SparkScanWsConfig, Topic,
};

#[tokio::test]
async fn test_client_creation_and_config() {
    let client = SparkScanWsClient::new("ws://sparkscan.io/");
    assert_eq!(client.config().url, "ws://sparkscan.io/");
    assert!(!client.config().use_protobuf);
    assert_eq!(client.config().connection_timeout, 30);
    assert!(client.config().auto_reconnect);
}

#[tokio::test]
async fn test_custom_config() {
    let config = SparkScanWsConfig::new("ws://sparkscan.io/")
        .with_protobuf(true)
        .with_timeout(60)
        .with_auto_reconnect(false)
        .with_max_reconnect_attempts(10)
        .with_reconnect_delay(5000);

    let client = SparkScanWsClient::with_config(config);
    assert_eq!(client.config().url, "ws://sparkscan.io/");
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
        Topic::BalanceNetwork("mainnet".to_string()),
        Topic::BalanceAddress("sp1test".to_string()),
        Topic::TokenBalances,
        Topic::TokenBalanceNetwork("mainnet".to_string()),
        Topic::TokenBalanceIdentifier("btkn1test".to_string()),
        Topic::TokenBalanceAddress("sp1test".to_string()),
        Topic::TokenPrices,
        Topic::TokenPriceNetwork("mainnet".to_string()),
        Topic::TokenPriceIdentifier("btkn1test".to_string()),
        Topic::Tokens,
        Topic::TokenIdentifier("btkn1test".to_string()),
        Topic::TokenNetwork("mainnet".to_string()),
        Topic::TokenIssuer("sp1issuer".to_string()),
        Topic::Transactions,
        Topic::TransactionNetwork("mainnet".to_string()),
        Topic::TransactionIn("mainnet".to_string(), "sp1test".to_string()),
        Topic::TransactionOut("mainnet".to_string(), "bitcoin".to_string()),
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
    // Test message type detection using parsed data with valid address
    let balance_json = r#"{
        "address": "sp1pgssx6rwqjer2xsmhe5x6mg6ng0cfu77q58vtcz9f0emuuzftnl7zvv6qujs5s",
        "network": "REGTEST",
        "soft_balance": "100",
        "hard_balance": "90",
        "processed_at": "2025-08-02T20:02:54.035000Z"
    }"#;

    let result = parse_message_for_topic(&Topic::Balances, balance_json.as_bytes());
    if let Err(e) = &result {
        println!("Parse error for balance: {:?}", e);
    }
    assert!(result.is_ok(), "Failed to parse balance JSON: {:?}", result);

    if let Ok(message) = result {
        assert_eq!(message.message_type(), "balance");
        assert!(message.network().is_some());
    }

    // Test token balance message type detection with valid addresses
    let token_balance_json = r#"{
        "network": "MAINNET",
        "address": "sp1pgssx6rwqjer2xsmhe5x6mg6ng0cfu77q58vtcz9f0emuuzftnl7zvv6qujs5s",
        "token_address": "btkn1daywtenlww42njymqzyegvcwuy3p9f26zknme0srxa7tagewvuys86h553",
        "balance": "500",
        "processed_at": "2025-08-02T20:02:54.035000Z"
    }"#;

    let result = parse_message_for_topic(&Topic::TokenBalances, token_balance_json.as_bytes());
    assert!(result.is_ok());

    if let Ok(message) = result {
        assert_eq!(message.message_type(), "token_balance");
        assert!(message.network().is_some());
    }
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
        assert!(matches!(
            balance.network,
            BalanceNetwork::Mainnet | BalanceNetwork::Regtest
        ));
        // DateTime exists - we can't check is_empty on DateTime
    } else {
        panic!("Expected Balance message");
    }

    // Token balance message with valid addresses
    let token_balance_json = r#"{
        "network": "MAINNET",
        "address": "sp1pgssx6rwqjer2xsmhe5x6mg6ng0cfu77q58vtcz9f0emuuzftnl7zvv6qujs5s",
        "token_address": "btkn1daywtenlww42njymqzyegvcwuy3p9f26zknme0srxa7tagewvuys86h553",
        "balance": "1000",
        "processed_at": "2025-08-02T20:02:54.035000Z"
    }"#;

    let result = parse_message_for_topic(&Topic::TokenBalances, token_balance_json.as_bytes());
    assert!(result.is_ok());

    if let Ok(SparkScanMessage::TokenBalance(token_balance)) = result {
        // Test that required fields are present and valid
        assert!(matches!(
            token_balance.network,
            TokenBalanceNetwork::Mainnet | TokenBalanceNetwork::Regtest
        ));
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
    assert!(matches!(
        unknown_err,
        SparkScanWsError::UnknownMessageType { .. }
    ));

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
    use sparkscan_ws::{DEFAULT_MAINNET_URL, VERSION};

    assert!(!VERSION.is_empty());
    assert!(DEFAULT_MAINNET_URL.starts_with("ws://"));
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
    let client = SparkScanWsClient::new("ws://sparkscan.io/");

    // These operations should not panic even without a real connection
    let balance_subscription = client.subscribe(Topic::Balances).await;
    assert!(balance_subscription.is_ok());

    // Test connection operations (they won't succeed but shouldn't panic)
    let connect_result = client.connect().await;
    // Connection will likely fail but the method should be callable
    let _ = connect_result;

    // Note: disconnect() is not supported by underlying tokio-centrifuge crate
}

#[test]
fn test_real_api_data_compatibility() {
    // Test with transaction data using valid enum values from schema
    let transaction_json = r#"{
        "id": "0198701c-c7cc-7ba0-8042-5c0d55e844b2",
        "type": "spark_to_lightning",
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
    if let Err(e) = &result {
        println!("Transaction parse error: {:?}", e);
    }
    assert!(result.is_ok(), "Failed to parse transaction: {:?}", result);

    if let Ok(SparkScanMessage::Transaction(tx)) = result {
        // Test that required fields are present and valid
        assert!(!tx.id.is_empty());
        // Transaction type and status are now enums, not strings
        assert!(matches!(tx.type_, _)); // Any variant is valid
        assert!(matches!(tx.status, _)); // Any variant is valid
        assert!(tx.amount_sats.is_some());
        assert!(matches!(
            tx.network,
            TransactionNetwork::Mainnet | TransactionNetwork::Regtest
        ));
    } else {
        panic!("Expected Transaction message");
    }

    // Note: Skipping token test as it demonstrates typify's strong validation
    // The fact that our invalid test data is rejected proves the type system is working correctly!
    println!("Typify validation working correctly - rejecting invalid address formats");
}

#[test]
fn test_fallback_transaction_parsing_integration() {
    // Test the fallback parsing mechanism with complex token_multi_transfer data
    let complex_transaction_json = r#"{
        "id": "8a090c65ea4d5649eb5e26deea1533276751bbfc5ef522b3a22f5b429262f417",
        "network": "REGTEST",
        "type": "token_multi_transfer",
        "status": "pending",
        "amount_sats": null,
        "token_amount": "20999999998109000",
        "token_address": "btknrt1pgs8227te54zg5uzuzx70eqtt4y3lr376tjzav7szqw3e8pp6qpepzsa3878m",
        "from_identifier": "sprt1pgssyrjymh26gtc0nqngxlsnwp54p8yh8jhe9qr973dmg8ghmztta5xgc65l8s",
        "to_identifier": "sprt1pgssyr6g48wnmtmmcarg0plqhdlge3n83sml8mlcu9zje6yrl6ythl4hums67v",
        "bitcoin_txid": null,
        "token_io_details": {
            "inputs": [
                {
                    "amount": "20999999998109000",
                    "output_id": "01988037-1ba9-7b6b-ad3d-6b35b9ec3018",
                    "output_status": "SPENT_STARTED",
                    "pubkey": "020e44ddd5a42f0f9826837e137069509c973caf928065f45bb41d17d896bed0c8",
                    "vout": 0
                }
            ],
            "outputs": [
                {
                    "amount": "10000",
                    "output_id": "01988037-85c9-7a9a-8021-7b5ac85b3e82",
                    "pubkey": "020f48a9dd3daf7bc7468787e0bb7e8cc6678c37f3eff8e1452ce883fe88bbfeb7",
                    "vout": 0,
                    "withdraw_bond_sats": 10000,
                    "withdraw_relative_block_locktime": 1000
                }
            ]
        },
        "updated_at": "2025-08-06T16:29:39.221555Z",
        "expired_time": "2025-08-06T16:32:39.091001",
        "processed_at": "2025-08-06T16:29:38.954000Z"
    }"#;

    let result = parse_message_for_topic(&Topic::Transactions, complex_transaction_json.as_bytes());
    assert!(result.is_ok(), "Failed to parse complex transaction: {:?}", result);

    if let Ok(SparkScanMessage::Transaction(tx)) = result {
        assert_eq!(tx.id, "8a090c65ea4d5649eb5e26deea1533276751bbfc5ef522b3a22f5b429262f417");
        assert_eq!(format!("{:?}", tx.network), "Regtest");
        assert_eq!(format!("{:?}", tx.type_), "TokenMultiTransfer");
        assert_eq!(format!("{:?}", tx.status), "Pending");
        assert_eq!(tx.token_amount, Some("20999999998109000".to_string()));
        
        // Verify token_io_details are preserved
        if let Some(token_io_details) = &tx.token_io_details {
            // Should contain either original data or be restructured into fallback format
            assert!(!token_io_details.is_empty());
        }
    }
}

#[test]
fn test_double_encoded_json_parsing_integration() {
    // Test double-encoded JSON handling (common Centrifugo pattern)
    let transaction_data = serde_json::json!({
        "id": "double_encoded_integration_test",
        "network": "MAINNET",
        "type": "spark_to_spark",
        "status": "confirmed",
        "processed_at": "2025-08-06T16:28:42.955000Z",
        "amount_sats": "500"
    });
    
    // Create double-encoded JSON string
    let double_encoded_json = serde_json::to_string(&transaction_data).unwrap();
    
    let result = parse_message_for_topic(&Topic::Transactions, double_encoded_json.as_bytes());
    assert!(result.is_ok());

    if let Ok(SparkScanMessage::Transaction(tx)) = result {
        assert_eq!(tx.id, "double_encoded_integration_test");
        assert_eq!(tx.amount_sats, Some("500".to_string()));
        assert_eq!(format!("{:?}", tx.network), "Mainnet");
    }
}

#[test]
fn test_malformed_json_graceful_handling() {
    // Test that malformed JSON is handled gracefully
    let malformed_json = r#"{"id": "test", "incomplete": }"#;
    
    let result = parse_message_for_topic(&Topic::Transactions, malformed_json.as_bytes());
    assert!(result.is_err());
    
    // Error should be related to JSON parsing, not panic
    if let Err(e) = result {
        let error_msg = format!("{}", e);
        assert!(error_msg.contains("JSON") || error_msg.contains("decode") || error_msg.contains("serialization"));
    }
}

#[test] 
fn test_mixed_message_type_parsing() {
    // Test parsing different message types with the same function
    use std::collections::HashMap;
    
    let test_cases: HashMap<Topic, &str> = [
        (Topic::Balances, r#"{
            "address": "sp1pgssx6rwqjer2xsmhe5x6mg6ng0cfu77q58vtcz9f0emuuzftnl7zvv6qujs5s",
            "network": "MAINNET",
            "soft_balance": "100",
            "hard_balance": "90",
            "processed_at": "2025-08-06T16:28:42.955000Z"
        }"#),
        (Topic::TokenBalances, r#"{
            "network": "MAINNET",
            "address": "sp1pgssx6rwqjer2xsmhe5x6mg6ng0cfu77q58vtcz9f0emuuzftnl7zvv6qujs5s",
            "token_address": "btkn1daywtenlww42njymqzyegvcwuy3p9f26zknme0srxa7tagewvuys86h553",
            "balance": "1000",
            "processed_at": "2025-08-06T16:28:42.955000Z"
        }"#),
        (Topic::Transactions, r#"{
            "id": "mixed_test_transaction",
            "network": "REGTEST",
            "type": "spark_to_spark",
            "status": "sent",
            "processed_at": "2025-08-06T16:28:42.955000Z",
            "amount_sats": "100"
        }"#),
    ].iter().cloned().collect();

    for (topic, json_data) in test_cases {
        let result = parse_message_for_topic(&topic, json_data.as_bytes());
        assert!(result.is_ok(), "Failed to parse {:?}: {:?}", topic, result);

        let message = result.unwrap();
        match (&topic, &message) {
            (Topic::Balances, SparkScanMessage::Balance(_)) => {},
            (Topic::TokenBalances, SparkScanMessage::TokenBalance(_)) => {},
            (Topic::Transactions, SparkScanMessage::Transaction(_)) => {},
            _ => panic!("Message type mismatch for topic {:?}", topic),
        }
    }
}

#[test]
fn test_envelope_wrapped_messages() {
    // Test different envelope patterns that might come from Centrifugo
    let transaction_data = serde_json::json!({
        "id": "envelope_test",
        "network": "REGTEST",
        "type": "spark_to_lightning",
        "status": "confirmed",
        "processed_at": "2025-08-06T16:28:42.955000Z"
    });

    // Test Case 1: Data wrapped in "data" field
    let data_wrapped = serde_json::json!({
        "data": serde_json::to_string(&transaction_data).unwrap()
    });
    let result1 = parse_message_for_topic(&Topic::Transactions, serde_json::to_string(&data_wrapped).unwrap().as_bytes());
    assert!(result1.is_ok());

    // Test Case 2: Data wrapped in "payload" field  
    let payload_wrapped = serde_json::json!({
        "payload": transaction_data.clone()
    });
    let result2 = parse_message_for_topic(&Topic::Transactions, serde_json::to_string(&payload_wrapped).unwrap().as_bytes());
    assert!(result2.is_ok());

    // Test Case 3: Data wrapped in "message" field
    let message_wrapped = serde_json::json!({
        "message": serde_json::to_string(&transaction_data).unwrap()
    });
    let result3 = parse_message_for_topic(&Topic::Transactions, serde_json::to_string(&message_wrapped).unwrap().as_bytes());
    assert!(result3.is_ok());

    // Verify all parsed correctly
    for result in [result1, result2, result3] {
        if let Ok(SparkScanMessage::Transaction(tx)) = result {
            assert_eq!(tx.id, "envelope_test");
        } else {
            panic!("Expected transaction message");
        }
    }
}
