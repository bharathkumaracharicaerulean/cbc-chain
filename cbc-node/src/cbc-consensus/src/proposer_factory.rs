//! Block proposer factory implementation
//!
//! This module creates real blocks with transactions using the DCF runtime API for author selection.

use crate::error::{ConsensusError, Result};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::NumberFor;
use sp_core::sr25519::Public;
use cbc_runtime::AccountId;
use pallet_cbc_dcf::DcfApi as RuntimeDcfApi;
use std::sync::Arc;
use sp_runtime::traits::{Block as BlockTrait, Header as HeaderTrait, Zero, SaturatedConversion, Hash};
use std::time::{Duration, Instant};
use std::marker::PhantomData;
use sc_transaction_pool_api::{TransactionPool, InPoolTransaction};
use sp_inherents::{InherentDataProvider, InherentData};
use sp_timestamp::InherentDataProvider as TimestampInherentDataProvider;
use sp_core::Encode;
use log::{debug, warn};

/// Factory for creating real blocks with transactions using DCF runtime API for author selection
pub struct ProposerFactory<B: BlockTrait, C, TP>
where
    B: sp_runtime::traits::Block,
    C: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    C::Api: RuntimeDcfApi<B, AccountId, u128, NumberFor<B>>,
    TP: TransactionPool<Block = B> + 'static,
{
    client: Arc<C>,
    transaction_pool: Arc<TP>,
    min_block_time: Duration,
    last_block_time: Option<Instant>,
    max_transactions_per_block: usize,
    _phantom: PhantomData<B>,
}

impl<B: BlockTrait, C, TP> ProposerFactory<B, C, TP>
where
    B: sp_runtime::traits::Block,
    C: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    C::Api: RuntimeDcfApi<B, AccountId, u128, NumberFor<B>>,
    TP: TransactionPool<Block = B> + 'static,
{
    /// Create a new proposer factory with the specified parameters
    pub fn new(client: Arc<C>, transaction_pool: Arc<TP>, min_block_time: Duration, max_transactions_per_block: usize) -> Self {
        Self {
            client,
            transaction_pool,
            min_block_time,
            last_block_time: None,
            max_transactions_per_block,
            _phantom: PhantomData,
        }
    }

    /// Create a new block with transactions from the pool using the expected author from DCF runtime API
    pub async fn create_block_with_transactions(&mut self, parent_hash: B::Hash, slot: u64) -> Result<(B, Public)> {
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

        debug!("Creating block #{} with author {:?}", block_number, author);

        // Get transactions from the pool
        let ready_transactions = self.collect_transactions_from_pool().await?;
        
        // Get parent header
        let parent_header = self.client.header(parent_hash)
            .map_err(|e| ConsensusError::Proposer(format!("Failed to get parent header: {:?}", e)))?
            .ok_or_else(|| ConsensusError::Proposer("Parent header not found".into()))?;
        
        let block_number = (*parent_header.number()).saturated_into::<u32>() + 1;
        let header_number = (block_number as u64).saturated_into::<<B::Header as HeaderTrait>::Number>();
        
        // For now, create a simple block with just the ready transactions
        // The runtime will handle inherents and proper root calculations during execution
        let all_extrinsics = ready_transactions;
        
        // Calculate extrinsics root using the correct method
        let extrinsics_root = <<B::Header as HeaderTrait>::Hashing as Hash>::ordered_trie_root(
            all_extrinsics.iter().map(|xt| xt.encode()).collect(),
            sp_runtime::StateVersion::V1,
        );
        
        // Create header with proper roots
        let header = B::Header::new(
            header_number,
            parent_hash,
            Default::default(), // state root will be calculated during execution
            extrinsics_root,
            Default::default(), // digest will be set during execution
        );

        // Create the complete block with transactions
        let block = B::new(header, all_extrinsics);
        
        debug!("Created block #{} with {} extrinsics from pool", 
              block_number, block.extrinsics().len());

        self.last_block_time = Some(Instant::now());
        Ok((block, author))
    }
    
    /// Collect transactions from the transaction pool
    async fn collect_transactions_from_pool(&self) -> Result<Vec<B::Extrinsic>> {
        let ready_transactions = self.transaction_pool.ready()
            .take(self.max_transactions_per_block)
            .map(|tx| (**tx.data()).clone())
            .collect::<Vec<_>>();
        
        debug!("Collected {} transactions from pool", ready_transactions.len());
        Ok(ready_transactions)
    }
    
    /// Create inherent data for the block
    async fn _create_inherent_data(&self) -> Result<InherentData> {
        let mut inherent_data = InherentData::new();
        
        // Add timestamp inherent
        let timestamp_provider = TimestampInherentDataProvider::from_system_time();
        timestamp_provider.provide_inherent_data(&mut inherent_data)
            .await
            .map_err(|e| ConsensusError::Proposer(format!("Failed to create timestamp inherent: {:?}", e)))?;
        
        Ok(inherent_data)
    }

    /// Create a new block with the expected author from the DCF runtime API (legacy method for compatibility)
    pub fn create_block(&mut self, parent_hash: B::Hash, slot: u64) -> Result<(B::Header, Public)> {
        warn!("ProposerFactory: Using legacy create_block method - consider using create_block_with_transactions");
        
        // Fetch expected author from runtime API
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        let block_number = slot as u32;
        let author = match api.get_expected_author(best_hash, block_number) {
            Ok(Some(account_id)) => Public::from_raw(*account_id.as_ref()),
            Ok(None) => return Err(ConsensusError::AuthorSelection("No expected author returned by runtime".into())),
            Err(e) => return Err(ConsensusError::AuthorSelection(format!("Runtime API error: {:?}", e))),
        };

        // Create basic block header
        let number = <<B as BlockTrait>::Header as HeaderTrait>::Number::zero();
        let header = B::Header::new(
            number,
            parent_hash,
            Default::default(),
            Default::default(),
            Default::default(),
        );

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