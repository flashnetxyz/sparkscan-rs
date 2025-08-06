//! Integration tests for SparkScan WebSocket SDK.

#[cfg(test)]
mod tests {
    use sparkscan_ws::{
        types::{
            balance::{Network as BalanceNetwork},
            parse_message_for_topic,
            token_balance::{Network as TokenBalanceNetwork},
        },
        SparkScanMessage, SparkScanWsClient, SparkScanWsConfig, Topic,
    };

    #[tokio::test]
    async fn test_client_creation() {
        let client = SparkScanWsClient::new("ws://sparkscan.io/");
        assert_eq!(client.config().url, "ws://sparkscan.io/");
        assert!(!client.config().use_protobuf);
    }

    #[test]
    fn test_config_builder() {
        let config = SparkScanWsConfig::new("ws://sparkscan.io/")
            .with_protobuf(true)
            .with_timeout(60)
            .with_auto_reconnect(false);

        assert_eq!(config.url, "ws://sparkscan.io/");
        assert!(config.use_protobuf);
        assert_eq!(config.connection_timeout, 60);
        assert!(!config.auto_reconnect);
    }

    #[test]
    fn test_topic_conversion() {
        // Test basic topics
        assert_eq!(Topic::Balances.as_str(), "balances");
        assert_eq!(Topic::TokenBalances.as_str(), "token_balances");
        assert_eq!(Topic::Transactions.as_str(), "transactions");

        // Test address-specific topics
        let address = "sp1abc123";
        let topic = Topic::BalanceAddress(address.to_string());
        assert_eq!(topic.as_str(), "/balance/address/sp1abc123");

        // Test token-specific topics
        let token = "btkn1def456";
        let topic = Topic::TokenPriceIdentifier(token.to_string());
        assert_eq!(topic.as_str(), "/token_price/identifier/btkn1def456");
    }

    #[test]
    fn test_topic_parsing() {
        // Test parsing basic topics
        assert_eq!(Topic::from_str("balances"), Topic::Balances);
        assert_eq!(Topic::from_str("token_balances"), Topic::TokenBalances);
        assert_eq!(Topic::from_str("transactions"), Topic::Transactions);

        // Test parsing address-specific topics
        let parsed = Topic::from_str("/balance/address/sp1abc123");
        match parsed {
            Topic::BalanceAddress(addr) => assert_eq!(addr, "sp1abc123"),
            _ => panic!("Expected BalanceAddress"),
        }

        // Test parsing token-specific topics
        let parsed = Topic::from_str("/token_price/identifier/btkn1def456");
        match parsed {
            Topic::TokenPriceIdentifier(token) => assert_eq!(token, "btkn1def456"),
            _ => panic!("Expected TokenPriceIdentifier"),
        }

        // Test that unknown topics panic (strictly typed)
        let result = std::panic::catch_unwind(|| Topic::from_str("unknown_topic"));
        assert!(result.is_err(), "Expected panic for unknown topic");
    }

    #[test]
    fn test_message_parsing_with_mock_data() {
        // Mock balance data based on the schema
        let balance_json = r#"{
            "address": "sp1pgssx6rwqjer2xsmhe5x6mg6ng0cfu77q58vtcz9f0emuuzftnl7zvv6qujs5s",
            "hard_balance": "301",
            "network": "MAINNET",
            "processed_at": "2025-08-02T20:02:54.035000Z",
            "soft_balance": "379"
        }"#;

        let topic = Topic::Balances;
        let result = parse_message_for_topic(&topic, balance_json.as_bytes());

        assert!(result.is_ok());
        let message = result.unwrap();

        match message {
            SparkScanMessage::Balance(balance) => {
                // Address is now a typed Address, not a string
                println!("Balance address: {:?}", balance.address);
                assert_eq!(balance.hard_balance, "301");
                assert_eq!(balance.soft_balance, "379");
                assert_eq!(balance.network, BalanceNetwork::Mainnet);
            }
            _ => panic!("Expected Balance message"),
        }
    }

    #[test]
    fn test_message_parsing_token_balance() {
        // Mock token balance data with valid addresses
        let token_balance_json = r#"{
            "network": "MAINNET",
            "address": "sp1pgssx6rwqjer2xsmhe5x6mg6ng0cfu77q58vtcz9f0emuuzftnl7zvv6qujs5s",
            "token_address": "btkn1daywtenlww42njymqzyegvcwuy3p9f26zknme0srxa7tagewvuys86h553",
            "balance": "1000",
            "processed_at": "2025-08-02T20:02:54.035000Z"
        }"#;

        let topic = Topic::TokenBalances;
        let result = parse_message_for_topic(&topic, token_balance_json.as_bytes());

        assert!(result.is_ok());
        let message = result.unwrap();

        match message {
            SparkScanMessage::TokenBalance(token_balance) => {
                assert_eq!(token_balance.network, TokenBalanceNetwork::Mainnet);
                // Address and token_address are now typed structs, not strings
                println!("Token balance address: {:?}", token_balance.address);
                println!("Token address: {:?}", token_balance.token_address);
                assert_eq!(token_balance.balance, "1000");
            }
            _ => panic!("Expected TokenBalance message"),
        }
    }

    #[test]
    fn test_message_parsing_invalid_data() {
        let invalid_json = r#"{"invalid": "data"}"#;
        let topic = Topic::Balances;
        let result = parse_message_for_topic(&topic, invalid_json.as_bytes());

        assert!(result.is_err());
    }

    #[test]
    fn test_message_type_extraction() {
        // Test message type extraction using parsed JSON data with valid address
        let balance_json = r#"{
            "address": "sp1pgssx6rwqjer2xsmhe5x6mg6ng0cfu77q58vtcz9f0emuuzftnl7zvv6qujs5s",
            "network": "MAINNET",
            "soft_balance": "100",
            "hard_balance": "90",
            "processed_at": "2025-08-02T20:02:54.035000Z"
        }"#;

        let result = parse_message_for_topic(&Topic::Balances, balance_json.as_bytes());
        assert!(result.is_ok());

        if let Ok(message) = result {
            assert_eq!(message.message_type(), "balance");
            assert!(message.network().is_some());
        }
    }

    // Note: Full integration tests with real WebSocket connections would
    // require a test server and are better suited for separate integration
    // test files or end-to-end testing infrastructure.

    #[tokio::test]
    async fn test_subscription_creation() {
        // Test that we can create subscriptions without panicking
        let client = SparkScanWsClient::new("ws://sparkscan.io/");

        // Note: This will fail to actually connect in tests, but we can
        // test that the subscription creation doesn't panic
        let result = client.subscribe(Topic::Balances).await;
        // In a real test environment with a server, this would succeed
        // For now, we just test that the method exists and is callable
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_handling_for_parsing_edge_cases() {
        use sparkscan_ws::types::parse_message_for_topic;

        // Test empty data
        let result = parse_message_for_topic(&Topic::Balances, &[]);
        assert!(result.is_err());

        // Test invalid UTF-8
        let invalid_utf8 = &[0xFF, 0xFE, 0xFD];
        let result = parse_message_for_topic(&Topic::Balances, invalid_utf8);
        assert!(result.is_err());

        // Test completely invalid JSON
        let invalid_json = b"this is not json at all";
        let result = parse_message_for_topic(&Topic::Balances, invalid_json);
        assert!(result.is_err());

        // Test JSON with wrong structure
        let wrong_structure = br#"["array", "instead", "of", "object"]"#;
        let result = parse_message_for_topic(&Topic::Balances, wrong_structure);
        assert!(result.is_err());
    }

    #[test]
    fn test_transaction_fallback_with_edge_cases() {
        use sparkscan_ws::types::parse_message_for_topic;

        // Test transaction with only required timestamp
        let minimal_transaction = br#"{
            "processed_at": "2025-08-06T16:28:42.955000Z"
        }"#;
        let result = parse_message_for_topic(&Topic::Transactions, minimal_transaction);
        assert!(result.is_ok());

        if let Ok(sparkscan_ws::SparkScanMessage::Transaction(tx)) = result {
            assert_eq!(tx.id, "unknown");
            assert!(tx.processed_at.to_string().contains("2025-08-06"));
        }

        // Test transaction with invalid datetime
        let invalid_datetime_transaction = br#"{
            "id": "test_invalid_datetime",
            "processed_at": "invalid-datetime-format"
        }"#;
        let result = parse_message_for_topic(&Topic::Transactions, invalid_datetime_transaction);
        // Should still parse due to fallback, but with default processed_at
        assert!(result.is_ok());

        // Test transaction with null values
        let null_values_transaction = br#"{
            "id": "null_test",
            "network": null,
            "type": null,
            "status": null,
            "processed_at": "2025-08-06T16:28:42.955000Z",
            "amount_sats": null,
            "token_amount": null
        }"#;
        let result = parse_message_for_topic(&Topic::Transactions, null_values_transaction);
        assert!(result.is_ok());

        if let Ok(sparkscan_ws::SparkScanMessage::Transaction(tx)) = result {
            assert_eq!(tx.id, "null_test");
            assert!(tx.amount_sats.is_none());
            assert!(tx.token_amount.is_none());
        }
    }

    #[test]
    fn test_deeply_nested_json_envelopes() {
        use sparkscan_ws::types::parse_message_for_topic;

        // Test deeply nested JSON structures
        let deeply_nested = serde_json::json!({
            "outer": {
                "middle": {
                    "data": serde_json::to_string(&serde_json::json!({
                        "id": "deeply_nested_test",
                        "network": "REGTEST",
                        "type": "spark_to_spark",
                        "status": "confirmed",
                        "processed_at": "2025-08-06T16:28:42.955000Z"
                    })).unwrap()
                }
            }
        });

        // This should fail gracefully since we don't handle triple-nested structures
        let result = parse_message_for_topic(&Topic::Transactions, serde_json::to_string(&deeply_nested).unwrap().as_bytes());
        // It might fail or succeed depending on fallback handling - either is acceptable
        // The important thing is it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_large_json_payload_handling() {
        use sparkscan_ws::types::parse_message_for_topic;

        // Test with a large JSON payload
        let mut large_token_io_details = serde_json::Map::new();
        
        // Create large arrays
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        for i in 0..1000 {
            inputs.push(serde_json::json!({
                "amount": format!("{}", i * 1000),
                "output_id": format!("output-{}", i),
                "vout": i
            }));
            outputs.push(serde_json::json!({
                "amount": format!("{}", i * 500),
                "output_id": format!("output-{}", i + 1000),
                "vout": i
            }));
        }
        
        large_token_io_details.insert("inputs".to_string(), serde_json::Value::Array(inputs));
        large_token_io_details.insert("outputs".to_string(), serde_json::Value::Array(outputs));

        let large_transaction = serde_json::json!({
            "id": "large_payload_test",
            "network": "REGTEST",
            "type": "token_multi_transfer",
            "status": "pending",
            "processed_at": "2025-08-06T16:28:42.955000Z",
            "token_io_details": large_token_io_details
        });

        let result = parse_message_for_topic(&Topic::Transactions, serde_json::to_string(&large_transaction).unwrap().as_bytes());
        assert!(result.is_ok());

        if let Ok(sparkscan_ws::SparkScanMessage::Transaction(tx)) = result {
            assert_eq!(tx.id, "large_payload_test");
            assert!(tx.token_io_details.is_some());
        }
    }

    #[test] 
    fn test_unicode_and_special_characters() {
        use sparkscan_ws::types::parse_message_for_topic;

        // Test with Unicode and special characters
        let unicode_transaction = serde_json::json!({
            "id": "unicode_test_🚀",
            "network": "REGTEST",
            "type": "spark_to_spark", 
            "status": "confirmed",
            "processed_at": "2025-08-06T16:28:42.955000Z",
            "from_identifier": "sp1_测试_address",
            "to_identifier": "sp1_тест_address",
            "custom_field": "Special chars: \"quotes\", 'apostrophes', \\backslashes\\, and émojis 🎉"
        });

        let result = parse_message_for_topic(&Topic::Transactions, serde_json::to_string(&unicode_transaction).unwrap().as_bytes());
        assert!(result.is_ok());

        if let Ok(sparkscan_ws::SparkScanMessage::Transaction(tx)) = result {
            assert_eq!(tx.id, "unicode_test_🚀");
            assert_eq!(tx.from_identifier, Some("sp1_测试_address".to_string()));
            assert_eq!(tx.to_identifier, Some("sp1_тест_address".to_string()));
        }
    }

    #[test]
    fn test_boundary_conditions() {
        use sparkscan_ws::types::parse_message_for_topic;

        // Test with very long string values
        let long_string = "a".repeat(10000);
        let long_string_transaction = serde_json::json!({
            "id": long_string.clone(),
            "network": "REGTEST",
            "type": "spark_to_spark",
            "status": "confirmed", 
            "processed_at": "2025-08-06T16:28:42.955000Z",
            "very_long_field": long_string.clone()
        });

        let result = parse_message_for_topic(&Topic::Transactions, serde_json::to_string(&long_string_transaction).unwrap().as_bytes());
        assert!(result.is_ok());

        if let Ok(sparkscan_ws::SparkScanMessage::Transaction(tx)) = result {
            assert_eq!(tx.id, long_string);
        }

        // Test with empty string values
        let empty_strings_transaction = serde_json::json!({
            "id": "",
            "network": "REGTEST",
            "type": "spark_to_spark",
            "status": "confirmed",
            "processed_at": "2025-08-06T16:28:42.955000Z",
            "from_identifier": "",
            "to_identifier": ""
        });

        let result = parse_message_for_topic(&Topic::Transactions, serde_json::to_string(&empty_strings_transaction).unwrap().as_bytes());
        assert!(result.is_ok());

        if let Ok(sparkscan_ws::SparkScanMessage::Transaction(tx)) = result {
            assert_eq!(tx.id, "");
            assert_eq!(tx.from_identifier, Some("".to_string()));
            assert_eq!(tx.to_identifier, Some("".to_string()));
        }
    }
}
