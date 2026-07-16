//! Finality tracking implementation
//! 
//! This module handles block finality and confirmation tracking.

use sp_runtime::traits::{Block as BlockTrait, NumberFor};
use std::sync::Arc;
use tokio::sync::mpsc;
use log::{debug, info, warn};

/// Finality notification sent when a block is finalized
#[derive(Debug, Clone)]
pub struct FinalityNotification<Block: BlockTrait> {
    /// The finalized block number
    pub block_number: NumberFor<Block>,
    /// The finalized block hash
    pub block_hash: Block::Hash,
    /// The round ID that finalized this block
    pub round_id: u32,
}

/// Finality Notifier
///
/// Broadcasts finality events and synchronizes finalized head across node components.
pub struct FinalityNotifier<Block: BlockTrait> {
    subscribers: Arc<tokio::sync::RwLock<Vec<mpsc::UnboundedSender<FinalityNotification<Block>>>>>,
}

impl<Block: BlockTrait> FinalityNotifier<Block> {
    /// Creates a new finality notifier
    pub fn new() -> Self {
        info!("DVF Finality Notifier: Initializing");
        Self {
            subscribers: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// Subscribes to finality notifications
    ///
    /// Returns a receiver that will receive finality notifications.
    pub async fn subscribe(&self) -> mpsc::UnboundedReceiver<FinalityNotification<Block>> {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut subscribers = self.subscribers.write().await;
        subscribers.push(tx);
        debug!("DVF Finality Notifier: New subscriber added (total: {})", subscribers.len());
        rx
    }

    /// Notifies all subscribers of finalization
    ///
    /// Broadcasts the finality notification to all registered subscribers.
    pub async fn notify(&self, notification: FinalityNotification<Block>) {
        let subscribers = self.subscribers.read().await;
        debug!(
            "DVF Finality Notifier: Broadcasting finality notification for block #{} to {} subscribers",
            notification.block_number, subscribers.len()
        );

        for subscriber in subscribers.iter() {
            if let Err(e) = subscriber.send(notification.clone()) {
                warn!("DVF Finality Notifier: Failed to send notification to subscriber: {:?}", e);
            }
        }
    }
}

impl<Block: BlockTrait> Default for FinalityNotifier<Block> {
    fn default() -> Self {
        Self::new()
    }
}
