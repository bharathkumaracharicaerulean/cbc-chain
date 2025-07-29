//! Finality tracking implementation
//! 
//! This module handles block finality and confirmation tracking.

use sp_runtime::traits::Block as BlockTrait;
use std::collections::HashMap;
use crate::error::{ConsensusError, Result};

/// Tracks block finality and confirmations
pub struct FinalityEngine<B: BlockTrait> {
    confirmations: HashMap<B::Hash, u32>,
    finality_threshold: u32,
}

impl<B: BlockTrait> FinalityEngine<B> {
    /// Create a new finality engine with the specified threshold
    pub fn new(finality_threshold: u32) -> Self {
        Self {
            confirmations: HashMap::new(),
            finality_threshold,
        }
    }

    /// Add a new block to track
    pub fn add_block(&mut self, block_hash: B::Hash) {
        self.confirmations.insert(block_hash, 0);
    }

    /// Update block confirmations and check for finality
    pub fn update_confirmations(&mut self, block_hash: B::Hash) -> Result<bool> {
        if let Some(confirmations) = self.confirmations.get_mut(&block_hash) {
            *confirmations += 1;
            Ok(*confirmations >= self.finality_threshold)
        } else {
            Err(ConsensusError::Finality("Block not found".into()))
        }
    }

    /// Check if a block is finalized
    pub fn is_finalized(&self, block_hash: &B::Hash) -> bool {
        self.confirmations
            .get(block_hash)
            .map_or(false, |&c| c >= self.finality_threshold)
    }

    /// Get the number of confirmations for a block
    pub fn get_confirmations(&self, block_hash: &B::Hash) -> Option<u32> {
        self.confirmations.get(block_hash).copied()
    }

    /// Remove old blocks that are no longer needed
    pub fn prune_old_blocks(&mut self, finalized_blocks: &[B::Hash]) {
        for block_hash in finalized_blocks {
            self.confirmations.remove(block_hash);
        }
    }

    /// Set the finality threshold
    pub fn set_finality_threshold(&mut self, threshold: u32) {
        self.finality_threshold = threshold;
    }
}
