//! Block import queue implementation
//! 
//! This module handles the queuing and processing of blocks for import.

use crate::error::{ConsensusError, Result};
use crate::validator_set::ValidatorSet;
use sp_runtime::traits::Block as BlockTrait;
use sp_core::sr25519::Public;
use std::collections::HashMap;

/// Manages the queue of blocks waiting to be imported
pub struct ImportQueue<B: BlockTrait> {
    pending_blocks: HashMap<B::Hash, B::Header>,
    validator_set: ValidatorSet,
    min_block_time: u64,
}

impl<B: BlockTrait> ImportQueue<B> {
    /// Create a new import queue
    pub fn new(validator_set: ValidatorSet) -> Self {
        Self {
            pending_blocks: HashMap::new(),
            validator_set,
            min_block_time: 1000, // Default 1 second
        }
    }

    /// Add a block to the import queue
    pub fn add_block(&mut self, block_hash: B::Hash, header: B::Header) -> Result<()> {
        if self.pending_blocks.contains_key(&block_hash) {
            return Err(ConsensusError::BlockImport("Block already in queue".into()));
        }

        self.pending_blocks.insert(block_hash, header);
        Ok(())
    }

    /// Process the next block in the queue
    pub fn process_next(&mut self) -> Result<Option<(B::Hash, B::Header)>> {
        if let Some((hash, header)) = self.pending_blocks.iter().next() {
            let hash = *hash;
            let header = header.clone();
            self.pending_blocks.remove(&hash);
            Ok(Some((hash, header)))
        } else {
            Ok(None)
        }
    }

    /// Finalize a block and remove it from the queue
    pub fn finalize_block(&mut self, block_hash: &B::Hash) -> Result<()> {
        self.pending_blocks.remove(block_hash);
        Ok(())
    }

    /// Get all pending blocks
    pub fn get_pending_blocks(&self) -> &HashMap<B::Hash, B::Header> {
        &self.pending_blocks
    }

    /// Update the validator set
    pub fn update_validator_set(&mut self, validator_set: ValidatorSet) {
        self.validator_set = validator_set;
    }

    /// Set the minimum time between blocks
    pub fn set_min_block_time(&mut self, min_block_time: u64) {
        self.min_block_time = min_block_time;
    }

    /// Validate a block's author
    pub fn validate_block(&self, author: &Public) -> Result<()> {
        if !self.validator_set.get_validator(author).is_some() {
            return Err(ConsensusError::BlockValidation("Invalid block author".into()));
        }
        Ok(())
    }
}