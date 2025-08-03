//! WebSocket subscription management for SparkScan.

use std::sync::Arc;
use tokio_centrifuge::subscription::Subscription;
use crate::{
    error::Result,
    types::{SparkScanMessage, Topic, parse_message_for_topic},
};

/// A typed WebSocket subscription for SparkScan messages.
/// 
/// This wrapper provides type-safe message handling on top of tokio-centrifuge
/// subscriptions, automatically deserializing messages based on the topic type.
pub struct SparkScanSubscription {
    /// The underlying centrifuge subscription
    inner: Subscription,
    /// The topic this subscription is for
    topic: Topic,
}

impl SparkScanSubscription {
    /// Create a new SparkScan subscription.
    /// 
    /// This is typically called internally by the SparkScan client.
    pub fn new(inner: Subscription, topic: Topic) -> Self {
        Self { inner, topic }
    }

    /// Get the topic for this subscription.
    pub fn topic(&self) -> &Topic {
        &self.topic
    }

    /// Set a callback for when the subscription becomes subscribed.
    /// 
    /// # Example
    /// ```rust,no_run
    /// # use sparkscan_ws::*;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = SparkScanWsClient::new("ws://localhost:8000/connection/websocket");
    /// let subscription = client.subscribe(Topic::Balances).await?;
    /// 
    /// subscription.on_subscribed(|| {
    ///     println!("Successfully subscribed to balances!");
    /// });
    /// # Ok(())
    /// # }
    /// ```
    pub fn on_subscribed<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.inner.on_subscribed(callback);
    }

    /// Set a callback for when the subscription becomes unsubscribed.
    pub fn on_unsubscribed<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.inner.on_unsubscribed(callback);
    }

    /// Set a callback for when the subscription is in subscribing state.
    pub fn on_subscribing<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.inner.on_subscribing(callback);
    }

    /// Set a callback for receiving typed messages.
    /// 
    /// This is the main method for handling incoming SparkScan messages.
    /// The callback receives a parsed `SparkScanMessage` enum that contains
    /// the appropriate typed payload based on the subscription topic.
    /// 
    /// # Example
    /// ```rust,no_run
    /// # use sparkscan_ws::*;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = SparkScanWsClient::new("ws://localhost:8000/connection/websocket");
    /// let subscription = client.subscribe(Topic::Balances).await?;
    /// 
    /// subscription.on_message(|message| {
    ///     match message {
    ///         SparkScanMessage::Balance(balance) => {
    ///             println!("Received balance update: {} sats", balance.soft_balance);
    ///         }
    ///         _ => {
    ///             println!("Received unexpected message type");
    ///         }
    ///     }
    /// });
    /// # Ok(())
    /// # }
    /// ```
    pub fn on_message<F>(&self, callback: F)
    where
        F: Fn(SparkScanMessage) + Send + Sync + 'static,
    {
        let topic = self.topic.clone();
        let callback = Arc::new(callback);
        
        self.inner.on_publication(move |data| {
            let topic = topic.clone();
            let callback = callback.clone();
            
            match parse_message_for_topic(&topic, &data.data) {
                Ok(message) => {
                    callback(message);
                }
                Err(e) => {
                    #[cfg(feature = "tracing")]
                    tracing::error!("Failed to parse message for topic {:?}: {}", topic, e);
                    
                    #[cfg(not(feature = "tracing"))]
                    log::error!("Failed to parse message for topic {:?}: {}", topic, e);
                }
            }
        });
    }

    /// Set a callback for receiving raw publication data.
    /// 
    /// This provides access to the raw bytes if you need to handle
    /// deserialization manually or for debugging purposes.
    pub fn on_raw_publication<F>(&self, callback: F)
    where
        F: Fn(&[u8]) + Send + Sync + 'static,
    {
        self.inner.on_publication(move |data| {
            callback(&data.data);
        });
    }

    /// Set a callback for subscription errors.
    pub fn on_error<F>(&self, callback: F)
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        self.inner.on_error(move |err| {
            callback(format!("{:?}", err));
        });
    }

    /// Start the subscription.
    /// 
    /// This must be called to begin receiving messages.
    pub fn subscribe(&self) {
        self.inner.subscribe();
    }

    /// Stop the subscription.
    pub fn unsubscribe(&self) {
        self.inner.unsubscribe();
    }

    /// Publish a message to this subscription's topic.
    /// 
    /// Note: This requires the WebSocket server to support publishing
    /// from clients, which may not be available in all configurations.
    pub fn publish(&self, message: &SparkScanMessage) -> Result<()> {
        let data = serde_json::to_vec(message)?;
        self.inner.publish(data);
        Ok(())
    }

    /// Publish raw data to this subscription's topic.
    pub fn publish_raw(&self, data: Vec<u8>) {
        self.inner.publish(data);
    }

    /// Check if the subscription is currently subscribed.
    pub fn is_subscribed(&self) -> bool {
        // Note: tokio-centrifuge doesn't expose subscription state directly
        // This would need to be tracked internally if needed
        true // Placeholder implementation
    }
}

/// A collection of subscriptions for easier management.
/// 
/// This allows you to manage multiple subscriptions together and provides
/// convenience methods for bulk operations.
#[derive(Default)]
pub struct SubscriptionManager {
    subscriptions: std::collections::HashMap<String, SparkScanSubscription>,
}

impl SubscriptionManager {
    /// Create a new subscription manager.
    pub fn new() -> Self {
        Self {
            subscriptions: std::collections::HashMap::new(),
        }
    }

    /// Add a subscription to the manager.
    pub fn add(&mut self, subscription: SparkScanSubscription) {
        let topic_str = subscription.topic().as_str();
        self.subscriptions.insert(topic_str, subscription);
    }

    /// Get a subscription by topic string.
    pub fn get(&self, topic: &str) -> Option<&SparkScanSubscription> {
        self.subscriptions.get(topic)
    }

    /// Remove a subscription by topic string.
    pub fn remove(&mut self, topic: &str) -> Option<SparkScanSubscription> {
        self.subscriptions.remove(topic)
    }

    /// Get all active subscriptions.
    pub fn subscriptions(&self) -> &std::collections::HashMap<String, SparkScanSubscription> {
        &self.subscriptions
    }

    /// Subscribe to all managed subscriptions.
    pub fn subscribe_all(&self) {
        for subscription in self.subscriptions.values() {
            subscription.subscribe();
        }
    }

    /// Unsubscribe from all managed subscriptions.
    pub fn unsubscribe_all(&self) {
        for subscription in self.subscriptions.values() {
            subscription.unsubscribe();
        }
    }

    /// Get the count of managed subscriptions.
    pub fn len(&self) -> usize {
        self.subscriptions.len()
    }

    /// Check if the manager has no subscriptions.
    pub fn is_empty(&self) -> bool {
        self.subscriptions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_manager() {
        let manager = SubscriptionManager::new();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
        
        // Note: We can't easily test with real subscriptions in unit tests
        // since they require a WebSocket connection. Integration tests would
        // be better for testing the full subscription functionality.
    }

    #[test]
    fn test_topic_conversion() {
        let topic = Topic::Balances;
        assert_eq!(topic.as_str(), "balances");
        
        let parsed = Topic::from_str("balances");
        assert_eq!(parsed, Topic::Balances);
    }
}