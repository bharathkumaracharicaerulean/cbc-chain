//! Consensus metrics implementation
//! 
//! This module handles tracking and reporting of consensus-related metrics.

use sp_runtime::traits::Block as BlockTrait;
use std::collections::HashMap;
use std::time::Instant;

/// Metrics for tracking consensus performance
pub struct ConsensusMetrics<B: BlockTrait> {
    /// Total number of blocks produced
    pub total_blocks: u32,
    /// Total number of transactions processed
    pub total_transactions: u32,
    /// Average time between blocks
    pub average_block_time: u64,
    /// Per-validator metrics
    pub validator_metrics: HashMap<B::Hash, u32>,
    /// Start time of the current epoch
    pub epoch_start_time: Instant,
}

impl<B: BlockTrait> ConsensusMetrics<B> {
    /// Create a new metrics tracker
    pub fn new() -> Self {
        Self {
            total_blocks: 0,
            total_transactions: 0,
            average_block_time: 0,
            validator_metrics: HashMap::new(),
            epoch_start_time: Instant::now(),
        }
    }

    /// Update block metrics with new block time and transaction count
    pub fn update_block_metrics(&mut self, block_time: u64, tx_count: u32) {
        self.total_blocks = self.total_blocks.saturating_add(1);
        self.total_transactions = self.total_transactions.saturating_add(tx_count);
        self.average_block_time = (self.average_block_time + block_time) / 2;
    }

    /// Update validator metrics with new score
    pub fn update_validator_metrics(&mut self, validator: B::Hash, score: u32) {
        self.validator_metrics.insert(validator, score);
    }

    /// Get the duration of the current epoch
    pub fn get_epoch_duration(&self) -> u64 {
        self.epoch_start_time.elapsed().as_secs()
    }
}