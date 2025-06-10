//! Block proposer factory implementation
//! 
//! This module handles the creation of block proposers and block headers.

use crate::error::{ConsensusError, Result};
use crate::types::ValidatorInfo;
use crate::author_selection::AuthorSelection;
use sp_runtime::traits::{Block as BlockTrait, Header as HeaderTrait};
use sp_runtime::traits::Zero;
use std::time::{Duration, Instant};
use std::marker::PhantomData;

/// Factory for creating block proposers and headers
pub struct ProposerFactory<B: BlockTrait> {
    author_selection: AuthorSelection,
    min_block_time: Duration,
    last_block_time: Option<Instant>,
    _phantom: PhantomData<B>,
}

impl<B: BlockTrait> ProposerFactory<B> {
    /// Create a new proposer factory with the specified parameters
    pub fn new(author_selection: AuthorSelection, min_block_time: Duration) -> Self {
        Self {
            author_selection,
            min_block_time,
            last_block_time: None,
            _phantom: PhantomData,
        }
    }

    /// Create a new block with the specified parent hash and slot
    pub fn create_block(&mut self, parent_hash: B::Hash, slot: u64) -> Result<(B::Header, ValidatorInfo)> {
        // Check if enough time has passed since last block
        if let Some(last_time) = self.last_block_time {
            if last_time.elapsed() < self.min_block_time {
                return Err(ConsensusError::Proposer(
                    "Not enough time since last block".into(),
                ));
            }
        }

        // Select block author
        let author = self.author_selection.select_author(slot)?;

        // Create block header
        let number = <<B as BlockTrait>::Header as HeaderTrait>::Number::zero();
        let header = B::Header::new(
            number,
            parent_hash,
            Default::default(), // state root will be set by runtime
            Default::default(), // extrinsics root will be set by runtime
            Default::default(), // digest will be set by runtime
        );

        self.last_block_time = Some(Instant::now());
        Ok((header, author.clone()))
    }

    /// Update the author selection with new validators
    pub fn update_author_selection(&mut self, validators: Vec<ValidatorInfo>) {
        self.author_selection.update_validators(validators);
    }

    /// Set the minimum time between blocks
    pub fn set_min_block_time(&mut self, min_block_time: Duration) {
        self.min_block_time = min_block_time;
    }

    /// Create a new block proposer
    pub fn create_proposer(&self) -> Result<B::Header> {
        let number = <<B as BlockTrait>::Header as HeaderTrait>::Number::zero();
        let header = B::Header::new(
            number,
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
        );
        Ok(header)
    }
}