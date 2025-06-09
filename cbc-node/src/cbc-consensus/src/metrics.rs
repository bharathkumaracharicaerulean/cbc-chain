#![allow(unused_imports)]
//! Consensus metrics module
//!
//! This module provides telemetry and monitoring capabilities for the consensus system,
//! tracking block production, validator performance, and system health metrics.

use sp_runtime::traits::Block as BlockT;
use sp_core::ed25519::Public;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Block statistics for consensus metrics
#[derive(Debug, Clone)]
pub struct BlockStats {
    /// Total number of blocks
    pub total_blocks: u64,
    /// Average block time
    pub avg_block_time: u64,
    /// Total number of transactions
    pub total_transactions: u64,
    /// Average transactions per block
    pub avg_transactions_per_block: u64,
    /// Number of failed block imports
    pub failed_imports: u64,
}

impl Default for BlockStats {
    fn default() -> Self {
        Self {
            total_blocks: 0,
            avg_block_time: 0,
            total_transactions: 0,
            avg_transactions_per_block: 0,
            failed_imports: 0,
        }
    }
}

/// Validator performance metrics
#[derive(Debug, Clone)]
pub struct ValidatorMetrics {
    /// Number of blocks authored
    pub blocks_authored: u64,
    /// Average block time
    pub avg_block_time: u64,
    /// Number of missed slots
    pub missed_slots: u64,
    /// Time of last block authored
    pub last_block_time: Option<Instant>,
}

impl Default for ValidatorMetrics {
    fn default() -> Self {
        Self {
            blocks_authored: 0,
            avg_block_time: 0,
            missed_slots: 0,
            last_block_time: None,
        }
    }
}

/// Metrics collector for consensus system
#[derive(Debug, Clone)]
pub struct ConsensusMetricsCollector {
    /// Block production statistics
    pub block_stats: BlockStats,
    /// Validator metrics
    pub validator_metrics: HashMap<Public, ValidatorMetrics>,
    /// Start time of metrics collection
    pub start_time: Instant,
}

impl ConsensusMetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            block_stats: BlockStats::default(),
            validator_metrics: HashMap::new(),
            start_time: Instant::now(),
        }
    }

    /// Record block production
    pub fn record_block_production(&mut self, author: Public, block_time: u64, transaction_count: u64) {
        // Update block stats
        self.block_stats.total_blocks += 1;
        self.block_stats.total_transactions += transaction_count;
        
        // Calculate new average block time
        if self.block_stats.total_blocks > 1 {
            self.block_stats.avg_block_time = ((self.block_stats.avg_block_time * (self.block_stats.total_blocks - 1)) + block_time) / self.block_stats.total_blocks;
        } else {
            self.block_stats.avg_block_time = block_time;
        }
        
        // Calculate new average transactions per block
        self.block_stats.avg_transactions_per_block = self.block_stats.total_transactions / self.block_stats.total_blocks;

        // Update validator metrics
        let validator_metrics = self.validator_metrics.entry(author).or_insert_with(ValidatorMetrics::default);
        validator_metrics.blocks_authored += 1;
        
        // Calculate new average block time for validator
        if validator_metrics.blocks_authored > 1 {
            validator_metrics.avg_block_time = ((validator_metrics.avg_block_time * (validator_metrics.blocks_authored - 1)) + block_time) / validator_metrics.blocks_authored;
        } else {
            validator_metrics.avg_block_time = block_time;
        }
        
        validator_metrics.last_block_time = Some(Instant::now());
    }

    /// Record missed slot
    pub fn record_missed_slot(&mut self, author: Public) {
        let validator_metrics = self.validator_metrics.entry(author).or_insert_with(ValidatorMetrics::default);
        validator_metrics.missed_slots += 1;
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> (BlockStats, HashMap<Public, ValidatorMetrics>) {
        (self.block_stats.clone(), self.validator_metrics.clone())
    }

    /// Get validator metrics
    pub fn get_validator_metrics(&self, author: &Public) -> Option<&ValidatorMetrics> {
        self.validator_metrics.get(author)
    }

    /// Get block stats
    pub fn get_block_stats(&self) -> &BlockStats {
        &self.block_stats
    }

    /// Reset metrics
    pub fn reset(&mut self) {
        self.block_stats = BlockStats::default();
        self.validator_metrics.clear();
        self.start_time = Instant::now();
    }
}

impl Default for ConsensusMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
} 