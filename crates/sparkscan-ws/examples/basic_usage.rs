//! WebSocket client implementation example demonstrating subscription patterns.
//!
//! Shows client initialization, connection management, subscription creation,
//! and message handling for different topic types.
//!
//! Run with: cargo run --example basic_usage

use sparkscan_ws::{
    SparkScanWsClient, SparkScanWsConfig, Topic, SparkScanMessage,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging based on available features
    #[cfg(feature = "tracing")]
    {
        use tracing_subscriber::filter::{LevelFilter, Targets};
        use tracing_subscriber::prelude::*;

        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            .with(
                Targets::new()
                    .with_default(LevelFilter::INFO)
                    .with_target("sparkscan_ws", LevelFilter::TRACE)
                    .with_target("tokio_centrifuge", LevelFilter::DEBUG)
            )
            .init();

        println!("Tracing initialized (tracing feature enabled)");
    }

    #[cfg(not(feature = "tracing"))]
    {
        env_logger::init();
        println!("Basic logging initialized (using env_logger)");
    }

    println!("SparkScan WebSocket SDK Basic Usage Example");
    println!("============================================");

    // Create a client configuration
    let config = SparkScanWsConfig::new("ws://updates.sparkscan.io")
        .with_auto_reconnect(true)
        .with_max_reconnect_attempts(5)
        .with_reconnect_delay(2000);

    let client = SparkScanWsClient::with_config(config);

    // Set up connection event handlers
    client.on_connecting(|| {
        println!("Connecting to SparkScan WebSocket...");
    });

    client.on_connected(|| {
        println!("Connected to SparkScan WebSocket!");
    });

    client.on_disconnected(|| {
        println!("Disconnected from SparkScan WebSocket");
    });

    client.on_error(|error| {
        eprintln!("WebSocket error: {}", error);
    });

    // Connect to the WebSocket server
    println!("Initiating connection...");
    client.connect().await?;

    // Wait a moment for connection to establish
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Example 1: Subscribe to all balance updates
    println!("\nSubscribing to balance updates...");
    let balance_subscription = client.subscribe(Topic::Balances).await?;
    
    balance_subscription.on_subscribed(|| {
        println!("Subscribed to balance updates");
    });

    balance_subscription.on_message(|message| {
        match message {
            SparkScanMessage::Balance(balance) => {
                println!("Balance Update:");
                println!("   Address: {}", balance.address);
                println!("   Soft Balance: {} sats", balance.soft_balance);
                println!("   Hard Balance: {} sats", balance.hard_balance);
                println!("   Network: {}", balance.network);
                println!("   Processed At: {}", balance.processed_at);
            }
            _ => {
                println!("Received unexpected message type for balance subscription");
            }
        }
    });

    balance_subscription.subscribe();

    // Example 2: Subscribe to token price updates
    println!("\nSubscribing to token price updates...");
    let token_price_subscription = client.subscribe(Topic::TokenPrices).await?;
    
    token_price_subscription.on_subscribed(|| {
        println!("Subscribed to token price updates");
    });

    token_price_subscription.on_message(|message| {
        match message {
            SparkScanMessage::TokenPrice(price) => {
                println!("Token Price Update:");
                println!("   Token: {}", price.address);
                println!("   Price: {:?} sats", price.price_sats);
                println!("   Protocol: {:?}", price.protocol);
                println!("   Network: {:?}", price.network);
                println!("   Processed At: {}", price.processed_at);
            }
            _ => {
                println!("Received unexpected message type for token price subscription");
            }
        }
    });

    token_price_subscription.subscribe();

    // Example 3: Subscribe to transaction updates
    println!("\nSubscribing to transaction updates...");
    let transaction_subscription = client.subscribe(Topic::Transactions).await?;
    
    transaction_subscription.on_subscribed(|| {
        println!("Subscribed to transaction updates");
    });

    transaction_subscription.on_message(|message| {
        match message {
            SparkScanMessage::Transaction(tx) => {
                println!("Transaction Update:");
                println!("   ID: {}", tx.id);
                println!("   Type: {}", tx.type_);
                println!("   Status: {}", tx.status);
                if let Some(amount) = &tx.amount_sats {
                    println!("   Amount: {} sats", amount);
                }
                if let Some(from) = &tx.from_identifier {
                    println!("   From: {}", from);
                }
                if let Some(to) = &tx.to_identifier {
                    println!("   To: {}", to);
                }
                println!("   Network: {:?}", tx.network);
                println!("   Processed At: {}", tx.processed_at);
            }
            _ => {
                println!("Received unexpected message type for transaction subscription");
            }
        }
    });

    transaction_subscription.subscribe();

    // Example 4: Subscribe to a specific address balance
    let specific_address = "sp1pgssx6rwqjer2xsmhe5x6mg6ng0cfu77q58vtcz9f0emuuzftnl7zvv6qujs5s";
    println!("\nSubscribing to balance updates for specific address: {}", specific_address);
    
    let address_balance_subscription = client.subscribe(
        Topic::BalanceAddress(specific_address.to_string())
    ).await?;
    
    address_balance_subscription.on_subscribed(|| {
        println!("Subscribed to address-specific balance updates");
    });

    address_balance_subscription.on_message(|message| {
        match message {
            SparkScanMessage::Balance(balance) => {
                println!("Address Balance Update:");
                println!("   Address: {}", balance.address);
                println!("   Soft Balance: {} sats", balance.soft_balance);
                println!("   Hard Balance: {} sats", balance.hard_balance);
            }
            _ => {
                println!("Received unexpected message type for address balance subscription");
            }
        }
    });

    address_balance_subscription.subscribe();

    // Example 5: Subscribe to mainnet balance updates only
    println!("\nSubscribing to mainnet balance updates...");
    let mainnet_balance_subscription = client.subscribe(
        Topic::BalanceNetwork("mainnet".to_string())
    ).await?;
    
    mainnet_balance_subscription.on_subscribed(|| {
        println!("Subscribed to mainnet balance updates");
    });

    mainnet_balance_subscription.on_message(|message| {
        match message {
            SparkScanMessage::Balance(balance) => {
                println!("Mainnet Balance Update:");
                println!("   Address: {}", balance.address);
                println!("   Balance: {} sats", balance.soft_balance);
            }
            _ => {
                println!("Received unexpected message type for mainnet balance subscription");
            }
        }
    });

    mainnet_balance_subscription.subscribe();

    // Example 6: Subscribe to incoming transactions for a specific network and address
    println!("\nSubscribing to incoming transactions for mainnet address...");
    let tx_in_subscription = client.subscribe(
        Topic::TransactionIn("mainnet".to_string(), specific_address.to_string())
    ).await?;
    
    tx_in_subscription.on_subscribed(|| {
        println!("Subscribed to incoming transaction updates");
    });

    tx_in_subscription.on_message(|message| {
        match message {
            SparkScanMessage::Transaction(tx) => {
                println!("Incoming Transaction:");
                println!("   ID: {}", tx.id);
                println!("   From: {:?}", tx.from_identifier);
                if let Some(amount) = &tx.amount_sats {
                    println!("   Amount: {} sats", amount);
                }
            }
            _ => {
                println!("Received unexpected message type for incoming transaction subscription");
            }
        }
    });

    tx_in_subscription.subscribe();

    // Example 7: Subscribe to outgoing transactions to Lightning Network
    println!("\nSubscribing to Lightning Network outgoing transactions...");
    let tx_out_lightning_subscription = client.subscribe(
        Topic::TransactionOut("mainnet".to_string(), "lightning".to_string())
    ).await?;
    
    tx_out_lightning_subscription.on_subscribed(|| {
        println!("Subscribed to Lightning Network outgoing transactions");
    });

    tx_out_lightning_subscription.on_message(|message| {
        match message {
            SparkScanMessage::Transaction(tx) => {
                println!("Lightning Outgoing Transaction:");
                println!("   ID: {}", tx.id);
                println!("   To: {:?}", tx.to_identifier);
                if let Some(amount) = &tx.amount_sats {
                    println!("   Amount: {} sats", amount);
                }
            }
            _ => {
                println!("Received unexpected message type for Lightning outgoing transaction subscription");
            }
        }
    });

    tx_out_lightning_subscription.subscribe();

    // Set up error handlers for subscriptions
    balance_subscription.on_error(|err| {
        eprintln!("Balance subscription error: {}", err);
    });

    token_price_subscription.on_error(|err| {
        eprintln!("Token price subscription error: {}", err);
    });

    transaction_subscription.on_error(|err| {
        eprintln!("Transaction subscription error: {}", err);
    });

    println!("\nAll subscriptions set up! Listening for messages...");
    println!("Press Ctrl+C to exit.\n");

    // Keep the application running
    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            println!("\nShutting down gracefully...");
        }
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        }
    }

    // Clean shutdown
    println!("Cleaning up subscriptions...");
    balance_subscription.unsubscribe();
    token_price_subscription.unsubscribe();
    transaction_subscription.unsubscribe();
    address_balance_subscription.unsubscribe();
    mainnet_balance_subscription.unsubscribe();
    tx_in_subscription.unsubscribe();
    tx_out_lightning_subscription.unsubscribe();

    println!("Shutdown complete!");
    Ok(())
}

/// Helper function to demonstrate publishing messages (if supported by server)
#[allow(dead_code)]
async fn demonstrate_publishing(client: &SparkScanWsClient) -> Result<(), Box<dyn std::error::Error>> {
    use sparkscan_ws::{BalancePayload, SparkScanMessage};
    
    println!("Demonstrating message publishing...");
    
    let subscription = client.subscribe(Topic::Balances).await?;
    
    // Create a sample balance message
    use sparkscan_ws::types::Network;
    
    let balance = BalancePayload {
        address: "sp1example123".to_string(),
        network: Network::Regtest,
        soft_balance: "1000".to_string(),
        hard_balance: "950".to_string(),
        processed_at: chrono::Utc::now(),
    };
    
    let message = SparkScanMessage::Balance(balance);
    
    // Publish the message (note: this requires server support for client publishing)
    match subscription.publish(&message) {
        Ok(()) => println!("Message published successfully"),
        Err(e) => eprintln!("Failed to publish message: {}", e),
    }
    
    Ok(())
}