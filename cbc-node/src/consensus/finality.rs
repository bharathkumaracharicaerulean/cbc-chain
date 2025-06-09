use sp_runtime::traits::Block as BlockT;
use crate::consensus::error::{ConsensusError, Result};
use std::collections::HashMap;

pub struct FinalityEngine<B: BlockT> {
    finality_threshold: u32,
    block_confirmations: HashMap<B::Hash, u32>,
}

impl<B: BlockT> FinalityEngine<B> {
    pub fn new(finality_threshold: u32) -> Self {
        Self {
            finality_threshold,
            block_confirmations: HashMap::new(),
        }
    }

    pub fn add_block(&mut self, block_hash: B::Hash) {
        self.block_confirmations.insert(block_hash, 1);
    }

    pub fn update_confirmations(&mut self, block_hash: B::Hash) -> Result<bool> {
        let confirmations = self.block_confirmations
            .get_mut(&block_hash)
            .ok_or_else(|| ConsensusError::FinalityCheck("Block not found".into()))?;

        *confirmations += 1;
        Ok(*confirmations >= self.finality_threshold)
    }

    pub fn is_finalized(&self, block_hash: &B::Hash) -> bool {
        self.block_confirmations
            .get(block_hash)
            .map_or(false, |&confirmations| confirmations >= self.finality_threshold)
    }

    pub fn get_confirmations(&self, block_hash: &B::Hash) -> Option<u32> {
        self.block_confirmations.get(block_hash).copied()
    }

    pub fn prune_old_blocks(&mut self, finalized_blocks: &[B::Hash]) {
        for block_hash in finalized_blocks {
            self.block_confirmations.remove(block_hash);
        }
    }

    pub fn set_finality_threshold(&mut self, threshold: u32) {
        self.finality_threshold = threshold;
    }
}
