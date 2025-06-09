use sp_runtime::traits::Block as BlockT;
use crate::consensus::error::{ConsensusError, Result};
use crate::consensus::types::ImportResult;
use crate::consensus::validator_set::ValidatorSet;
use std::collections::HashMap;

pub struct ImportQueue<B: BlockT> {
    validator_set: ValidatorSet,
    pending_blocks: HashMap<B::Hash, B::Header>,
    min_block_time: u64,
    last_import_time: u64,
}

impl<B: BlockT> ImportQueue<B> {
    pub fn new(validator_set: ValidatorSet, min_block_time: u64) -> Self {
        Self {
            validator_set,
            pending_blocks: HashMap::new(),
            min_block_time,
            last_import_time: 0,
        }
    }

    pub fn import_block(&mut self, header: B::Header, current_time: u64) -> Result<ImportResult<B>> {
        // Check block time
        if current_time - self.last_import_time < self.min_block_time {
            return Err(ConsensusError::BlockValidation(
                "Block time too close to previous block".into(),
            ));
        }

        // Validate block author
        let author = header.extrinsics_root();
        if !self.validator_set.get_validator(author.as_ref()).is_some() {
            return Err(ConsensusError::InvalidAuthor(
                "Block author not in validator set".into(),
            ));
        }

        // Check if block already exists
        let block_hash = header.hash();
        if self.pending_blocks.contains_key(&block_hash) {
            return Ok(ImportResult::AlreadyInChain(block_hash));
        }

        // Add to pending blocks
        self.pending_blocks.insert(block_hash, header);
        self.last_import_time = current_time;

        Ok(ImportResult::Imported(block_hash))
    }

    pub fn finalize_block(&mut self, block_hash: &B::Hash) -> Result<()> {
        self.pending_blocks
            .remove(block_hash)
            .ok_or_else(|| ConsensusError::BlockValidation("Block not found".into()))?;
        Ok(())
    }

    pub fn get_pending_blocks(&self) -> &HashMap<B::Hash, B::Header> {
        &self.pending_blocks
    }

    pub fn update_validator_set(&mut self, validator_set: ValidatorSet) {
        self.validator_set = validator_set;
    }

    pub fn set_min_block_time(&mut self, min_block_time: u64) {
        self.min_block_time = min_block_time;
    }
}
