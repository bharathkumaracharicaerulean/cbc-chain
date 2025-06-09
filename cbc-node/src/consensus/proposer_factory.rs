use sp_runtime::traits::Block as BlockT;
use crate::consensus::error::{ConsensusError, Result};
use crate::consensus::types::ValidatorInfo;
use crate::consensus::author_selection::AuthorSelection;
use std::time::{Duration, Instant};

pub struct ProposerFactory<B: BlockT> {
    author_selection: AuthorSelection,
    min_block_time: Duration,
    last_block_time: Option<Instant>,
}

impl<B: BlockT> ProposerFactory<B> {
    pub fn new(author_selection: AuthorSelection, min_block_time: Duration) -> Self {
        Self {
            author_selection,
            min_block_time,
            last_block_time: None,
        }
    }

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
        let header = B::Header::new(
            0, // number will be set by runtime
            parent_hash,
            Default::default(), // state root will be set by runtime
            Default::default(), // extrinsics root will be set by runtime
            Default::default(), // digest will be set by runtime
        );

        self.last_block_time = Some(Instant::now());
        Ok((header, author.clone()))
    }

    pub fn update_author_selection(&mut self, validators: Vec<ValidatorInfo>) {
        self.author_selection.update_validators(validators);
    }

    pub fn set_min_block_time(&mut self, min_block_time: Duration) {
        self.min_block_time = min_block_time;
    }
}
