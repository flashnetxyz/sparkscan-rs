//! SparkScan WebSocket API Example
//!
//! This example demonstrates connecting to the SparkScan WebSocket API and
//! subscribing to real-time data streams including balance updates, transactions,
//! token prices, and token metadata.
//!
//! Run with: cargo run --example example
//! Run with debug logging: RUST_LOG=debug cargo run --example example

use sparkscan_ws::{SparkScanMessage, SparkScanWsClient, Topic};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("SparkScan WebSocket API Example");
    println!("===============================");
    println!("Connecting to SparkScan API to receive real-time data...\n");

    // Create WebSocket client
    let client = SparkScanWsClient::new("ws://updates.sparkscan.io/");

    // Connection state tracking
    let connected = Arc::new(AtomicBool::new(false));
    let message_count = Arc::new(AtomicU32::new(0));

    // Configure connection event handlers
    {
        let connected = connected.clone();
        client.on_connected(move || {
            println!("Connected to SparkScan WebSocket API");
            connected.store(true, Ordering::Relaxed);
        });
    }

    client.on_error(|error| {
        eprintln!("Connection error: {}", error);
    });

    // Establish connection
    println!("Connecting...");
    client.connect().await?;
    
    // Wait for connection establishment
    for i in 1..=10 {
        if connected.load(Ordering::Relaxed) {
            break;
        }
        println!("Waiting for connection... ({}/10)", i);
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    if !connected.load(Ordering::Relaxed) {
        return Err("Failed to establish connection".into());
    }

    println!("Setting up subscriptions...\n");

    // Subscribe to main data topics
    let topics = vec![
        (Topic::Balances, "Balance updates"),
        (Topic::TokenBalances, "Token balance updates"),
        (Topic::TokenPrices, "Token price updates"),
        (Topic::Tokens, "Token metadata updates"),
        (Topic::Transactions, "Transaction updates"),
    ];

    for (topic, description) in topics {
        let topic_name = topic.as_str();
        println!("Subscribing to: {} ({})", topic_name, description);
        
        let subscription = client.subscribe(topic).await?;
        
        subscription.on_subscribed(|| {
            println!("  Subscription active");
        });

        subscription.on_error(|err| {
            eprintln!("  Subscription error: {}", err);
        });

        // Message handler with proper type handling
        let message_count = message_count.clone();
        subscription.on_message(move |message| {
            let count = message_count.fetch_add(1, Ordering::Relaxed) + 1;
            println!("\nMessage #{} received:", count);
            
            match message {
                SparkScanMessage::Balance(balance) => {
                    println!("  Type: Balance Update");
                    println!("  Address: {:?}", balance.address);
                    println!("  Soft Balance: {} sats", balance.soft_balance);
                    println!("  Hard Balance: {} sats", balance.hard_balance);
                    println!("  Network: {:?}", balance.network);
                    println!("  Processed At: {}", balance.processed_at);
                }
                SparkScanMessage::TokenBalance(token_balance) => {
                    println!("  Type: Token Balance Update");
                    println!("  Address: {:?}", token_balance.address);
                    println!("  Token Address: {:?}", token_balance.token_address);
                    println!("  Balance: {}", token_balance.balance);
                    println!("  Network: {:?}", token_balance.network);
                    println!("  Processed At: {}", token_balance.processed_at);
                }
                SparkScanMessage::TokenPrice(price) => {
                    println!("  Type: Token Price Update");
                    println!("  Token Address: {:?}", price.address);
                    println!("  Price: {:?} sats", price.price_sats);
                    println!("  Protocol: {:?}", price.protocol);
                    println!("  Network: {:?}", price.network);
                    println!("  Processed At: {}", price.processed_at);
                }
                SparkScanMessage::Token(token) => {
                    println!("  Type: Token Update");
                    println!("  Address: {:?}", token.address);
                    println!("  Ticker: {}", token.ticker);
                    println!("  Name: {}", token.name);
                    println!("  Decimals: {}", token.decimals);
                    println!("  Network: {:?}", token.network);
                    if let Some(calculated_at) = token.calculated_at {
                        println!("  Calculated At: {}", calculated_at);
                    }
                }
                SparkScanMessage::Transaction(tx) => {
                    println!("  Type: Transaction Update");
                    println!("  ID: {}", tx.id);
                    println!("  Type: {:?}", tx.type_);
                    println!("  Status: {:?}", tx.status);
                    println!("  Network: {:?}", tx.network);
                    println!("  Processed At: {}", tx.processed_at);
                    
                    // Optional amount fields
                    if let Some(amount) = &tx.amount_sats {
                        println!("  Amount (sats): {}", amount);
                    }
                    if let Some(token_amount) = &tx.token_amount {
                        println!("  Token Amount: {}", token_amount);
                    }
                    
                    // Optional identifier fields
                    if let Some(from) = &tx.from_identifier {
                        println!("  From: {}", from);
                    }
                    if let Some(to) = &tx.to_identifier {
                        println!("  To: {}", to);
                    }
                    
                    // Optional token fields
                    if let Some(token_address) = &tx.token_address {
                        println!("  Token Address: {}", token_address);
                    }
                    if let Some(token_io_details) = &tx.token_io_details {
                        println!("  Token I/O Details: {:?}", token_io_details);
                    }
                    
                    // Optional Bitcoin transaction ID
                    if let Some(bitcoin_txid) = &tx.bitcoin_txid {
                        println!("  Bitcoin TXID: {}", bitcoin_txid);
                    }
                    
                    // Optional timestamp fields
                    if let Some(updated_at) = &tx.updated_at {
                        println!("  Updated At: {}", updated_at);
                    }
                    if let Some(expired_time) = &tx.expired_time {
                        println!("  Expired Time: {}", expired_time);
                    }
                }
            }
            println!("  ------------------------------------");
        });

        subscription.subscribe();
    }

    println!("\nAll subscriptions configured");
    println!("Waiting for real-time messages...");
    println!("Press Ctrl+C to exit\n");

    // Wait for messages with periodic status updates
    let mut seconds_elapsed = 0;
    let max_wait = 300;

    loop {
        // Check for shutdown signal with timeout
        match tokio::time::timeout(Duration::from_secs(1), tokio::signal::ctrl_c()).await {
            Ok(Ok(())) => {
                println!("\nShutdown signal received");
                break;
            }
            Ok(Err(_)) => break,
            Err(_) => {
                // Timeout occurred, continue monitoring
                seconds_elapsed += 1;
                
                if seconds_elapsed >= max_wait {
                    println!("\n60 seconds elapsed - ending example");
                    break;
                }
                
                // Status update every 15 seconds
                if seconds_elapsed % 15 == 0 {
                    let msg_count = message_count.load(Ordering::Relaxed);
                    println!("Status: {}s elapsed, {} messages received", seconds_elapsed, msg_count);
                }
            }
        }
    }

    let final_count = message_count.load(Ordering::Relaxed);
    println!("\nExample Summary:");
    println!("  Total messages received: {}", final_count);
    
    if final_count == 0 {
        println!("  No messages received - possible reasons:");
        println!("    - API may not have active data streams at this time");
        println!("    - Network connectivity issues");
        println!("    - API maintenance mode");
    } else {
        println!("  Success: SparkScan API is sending real-time data");
    }

    println!("\nExample complete");
    Ok(())
}