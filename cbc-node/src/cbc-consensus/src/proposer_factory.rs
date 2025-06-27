//! Block proposer factory implementation
//!
//! This module now delegates author selection to the DCF runtime API.

use crate::error::{ConsensusError, Result};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::sr25519::Public;
use cbc_runtime::AccountId;
use pallet_cbc_dcf::DcfApi as RuntimeDcfApi;
use std::sync::Arc;
use sp_runtime::traits::{Block as BlockTrait, Header as HeaderTrait, Zero};
use std::time::{Duration, Instant};
use std::marker::PhantomData;

/// Factory for creating block proposers and headers using DCF runtime API for author selection
pub struct ProposerFactory<B: BlockTrait, C>
where
    B: sp_runtime::traits::Block,
    C: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    C::Api: RuntimeDcfApi<B, AccountId>,
{
    client: Arc<C>,
    min_block_time: Duration,
    last_block_time: Option<Instant>,
    _phantom: PhantomData<B>,
}

impl<B: BlockTrait, C> ProposerFactory<B, C>
where
    B: sp_runtime::traits::Block,
    C: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    C::Api: RuntimeDcfApi<B, AccountId>,
{
    /// Create a new proposer factory with the specified parameters
    pub fn new(client: Arc<C>, min_block_time: Duration) -> Self {
        Self {
            client,
            min_block_time,
            last_block_time: None,
            _phantom: PhantomData,
        }
    }

    /// Create a new block with the expected author from the DCF runtime API
    pub fn create_block(&mut self, parent_hash: B::Hash, slot: u64) -> Result<(B::Header, Public)> {
        // Check if enough time has passed since last block
        if let Some(last_time) = self.last_block_time {
            if last_time.elapsed() < self.min_block_time {
                return Err(ConsensusError::Proposer(
                    "Not enough time since last block".into(),
                ));
            }
        }

        // Fetch expected author from runtime API
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        let block_number = slot as u32;
        let author = match api.get_expected_author(best_hash, block_number) {
            Ok(Some(account_id)) => Public::from_raw(*account_id.as_ref()),
            Ok(None) => return Err(ConsensusError::AuthorSelection("No expected author returned by runtime".into())),
            Err(e) => return Err(ConsensusError::AuthorSelection(format!("Runtime API error: {:?}", e))),
        };

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
        Ok((header, author))
    }

    /// Set the minimum time between blocks
    pub fn set_min_block_time(&mut self, min_block_time: Duration) {
        self.min_block_time = min_block_time;
    }

    /// Create a new block proposer (header only, for compatibility)
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