//! Integration tests for SparkScan WebSocket SDK.

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use sparkscan_ws::{
        types::{
            parse_message_for_topic,
            balance::{BalancePayload, Network as BalanceNetwork},
            token::Network as TokenNetwork,
            token_balance::{TokenBalancePayload, Network as TokenBalanceNetwork},
        },
        SparkScanMessage, SparkScanWsClient, SparkScanWsConfig, Topic,
    };

    #[tokio::test]
    async fn test_client_creation() {
        let client = SparkScanWsClient::new("ws://sparkscan.io/");
        assert_eq!(
            client.config().url,
            "ws://sparkscan.io/"
        );
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
}
