//! WebSocket subscription management for SparkScan.

use crate::{
    error::Result,
    types::{parse_message_for_topic, SparkScanMessage, Topic},
};
use std::sync::Arc;
use tokio_centrifuge::subscription::Subscription;

/// Typed WebSocket subscription handler.
///
/// Wraps tokio-centrifuge subscription with type-safe message deserialization
/// based on topic-specific message types.
pub struct SparkScanSubscription {
    /// The underlying centrifuge subscription
    inner: Subscription,
    /// The topic this subscription is for
    topic: Topic,
}

impl SparkScanSubscription {
    /// Create new typed subscription.
    ///
    /// Typically called internally by client.
    pub fn new(inner: Subscription, topic: Topic) -> Self {
        Self { inner, topic }
    }

    /// Get the topic for this subscription.
    pub fn topic(&self) -> &Topic {
        &self.topic
    }

    /// Register callback for subscription establishment.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use sparkscan_ws::*;
    /// # async fn example() -> Result<()> {
    /// # let client = SparkScanWsClient::new("ws://updates.sparkscan.io/");
    /// let subscription = client.subscribe(Topic::Balances).await?;
    ///
    /// subscription.on_subscribed(|| {
    ///     println!("Subscription active");
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

    /// Register callback for subscription termination.
    pub fn on_unsubscribed<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.inner.on_unsubscribed(callback);
    }

    /// Register callback for subscription initiation.
    pub fn on_subscribing<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.inner.on_subscribing(callback);
    }

    /// Register callback for typed message handling.
    ///
    /// Primary method for processing incoming messages. Callback receives
    /// parsed SparkScanMessage enum with topic-appropriate payload.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use sparkscan_ws::*;
    /// # async fn example() -> Result<()> {
    /// # let client = SparkScanWsClient::new("ws://updates.sparkscan.io/");
    /// let subscription = client.subscribe(Topic::Balances).await?;
    ///
    /// subscription.on_message(|message| {
    ///     match message {
    ///         SparkScanMessage::Balance(balance) => {
    ///             println!("Balance update: {} sats", balance.soft_balance);
    ///         }
    ///         _ => {
    ///             println!("Unexpected message type");
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

    /// Register callback for raw message data.
    ///
    /// Provides access to raw bytes for manual deserialization or debugging.
    pub fn on_raw_publication<F>(&self, callback: F)
    where
        F: Fn(&[u8]) + Send + Sync + 'static,
    {
        self.inner.on_publication(move |data| {
            callback(&data.data);
        });
    }

    /// Register callback for subscription errors.
    pub fn on_error<F>(&self, callback: F)
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        self.inner.on_error(move |err| {
            callback(format!("{:?}", err));
        });
    }

    /// Activate subscription to begin receiving messages.
    ///
    /// Must be called to start message delivery.
    pub fn subscribe(&self) {
        self.inner.subscribe();
    }

    /// Deactivate subscription.
    pub fn unsubscribe(&self) {
        self.inner.unsubscribe();
    }

    /// Publish message to subscription topic.
    ///
    /// Note: Requires server support for client publishing.
    pub fn publish(&self, message: &SparkScanMessage) -> Result<()> {
        let data = serde_json::to_vec(message)?;
        self.inner.publish(data);
        Ok(())
    }

    /// Publish raw data to subscription topic.
    pub fn publish_raw(&self, data: Vec<u8>) {
        self.inner.publish(data);
    }

    /// Check subscription activation status.
    ///
    /// # Note
    ///
    /// This function is not currently supported by the underlying tokio-centrifuge crate
    /// as it does not expose subscription state information.
    pub fn is_subscribed(&self) -> bool {
        todo!("Subscription state tracking not supported by tokio-centrifuge")
    }
}

/// Subscription collection manager.
///
/// Manages multiple subscriptions with bulk operation support.
#[derive(Default)]
pub struct SubscriptionManager {
    subscriptions: std::collections::HashMap<String, SparkScanSubscription>,
}

impl SubscriptionManager {
    /// Create new subscription manager.
    pub fn new() -> Self {
        Self {
            subscriptions: std::collections::HashMap::new(),
        }
    }

    /// Add subscription to manager.
    pub fn add(&mut self, subscription: SparkScanSubscription) {
        let topic_str = subscription.topic().as_str();
        self.subscriptions.insert(topic_str, subscription);
    }

    /// Get subscription by topic string.
    pub fn get(&self, topic: &str) -> Option<&SparkScanSubscription> {
        self.subscriptions.get(topic)
    }

    /// Remove subscription by topic string.
    pub fn remove(&mut self, topic: &str) -> Option<SparkScanSubscription> {
        self.subscriptions.remove(topic)
    }

    /// Get all managed subscriptions.
    pub fn subscriptions(&self) -> &std::collections::HashMap<String, SparkScanSubscription> {
        &self.subscriptions
    }

    /// Activate all managed subscriptions.
    pub fn subscribe_all(&self) {
        for subscription in self.subscriptions.values() {
            subscription.subscribe();
        }
    }

    /// Deactivate all managed subscriptions.
    pub fn unsubscribe_all(&self) {
        for subscription in self.subscriptions.values() {
            subscription.unsubscribe();
        }
    }

    /// Get count of managed subscriptions.
    pub fn len(&self) -> usize {
        self.subscriptions.len()
    }

    /// Check if manager contains no subscriptions.
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
