#![allow(unused_imports)]
//! Block proposer factory module
//!
//! This module provides the block proposer factory that creates and submits new blocks
//! to the chain.

use crate::types::{ValidatorInfo, BlockStats, ProposerConfig};
use crate::error::{ConsensusError, BlockValidationError};
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};
use sp_core::ed25519::Public;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Block proposer factory
pub struct ProposerFactory<Block: BlockT> {
    /// Proposer configuration
    config: ProposerConfig,
    /// Block statistics
    stats: BlockStats,
    /// Last block time
    last_block_time: Instant,
    /// Phantom data for Block
    _marker: std::marker::PhantomData<Block>,
}

impl<Block: BlockT> ProposerFactory<Block> {
    /// Create a new proposer factory
    pub fn new(config: ProposerConfig) -> Self {
        Self {
            config,
            stats: BlockStats::default(),
            last_block_time: Instant::now(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a new block
    pub fn create_block(
        &mut self,
        parent_hash: <Block as BlockT>::Hash,
        number: u32,
        author: Public,
    ) -> Result<Block, ConsensusError> {
        // Validate author
        if author == Public::default() {
            return Err(ConsensusError::BlockValidation(BlockValidationError::InvalidAuthor));
        }

        // Create block header
        let header = Block::Header::new(
            number.into(),
            parent_hash,
            Default::default(),
            Default::default(),
            Default::default(),
        );

        // Create block
        let block = Block::new(header, Vec::new());

        // Update statistics
        self.update_stats(0);

        Ok(block)
    }

    /// Update block statistics
    pub fn update_stats(&mut self, transaction_count: u32) {
        let now = Instant::now();
        let block_time = now.duration_since(self.last_block_time).as_secs_f64();
        self.stats.avg_block_time = (self.stats.avg_block_time * self.stats.total_blocks as f64
            + block_time)
            / (self.stats.total_blocks + 1) as f64;
        self.stats.total_blocks += 1;
        self.stats.total_transactions += transaction_count as u64;
        self.stats.avg_transactions_per_block = self.stats.total_transactions as f64
            / self.stats.total_blocks as f64;
        self.last_block_time = now;
    }

    /// Get block statistics
    pub fn get_stats(&self) -> &BlockStats {
        &self.stats
    }

    /// Get proposer configuration
    pub fn get_config(&self) -> &ProposerConfig {
        &self.config
    }

    /// Update proposer configuration
    pub fn update_config(&mut self, config: ProposerConfig) {
        self.config = config;
    }
}

impl<Block: BlockT> Default for ProposerFactory<Block> {
    fn default() -> Self {
        Self::new(ProposerConfig::default())
    }
} 