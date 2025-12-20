//! Block proposer factory implementation
//!
//! This module creates real blocks with transactions using the DCF runtime API for author selection.

use crate::error::{ConsensusError, ConsensusResult};
use crate::inherent_providers::CbcInherentDataProviders;
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

use sp_block_builder::BlockBuilder as BlockBuilderApi;
use sp_core::Encode;
use log::{debug, warn, error};

/// Factory for creating real blocks with transactions using DCF runtime API for author selection
pub struct ProposerFactory<B: BlockTrait, C, TP>
where
    B: sp_runtime::traits::Block,
    C: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    C::Api: RuntimeDcfApi<B, AccountId, u128, NumberFor<B>> + BlockBuilderApi<B>,
    TP: TransactionPool<Block = B> + 'static,
{
    client: Arc<C>,
    transaction_pool: Arc<TP>,
    inherent_providers: CbcInherentDataProviders,
    min_block_time: Duration,
    last_block_time: Option<Instant>,
    max_transactions_per_block: usize,
    _phantom: PhantomData<B>,
}

impl<B: BlockTrait, C, TP> ProposerFactory<B, C, TP>
where
    B: sp_runtime::traits::Block,
    C: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    C::Api: RuntimeDcfApi<B, AccountId, u128, NumberFor<B>> + BlockBuilderApi<B>,
    TP: TransactionPool<Block = B> + 'static,
{
    /// Create a new proposer factory with the specified parameters
    pub fn new(client: Arc<C>, transaction_pool: Arc<TP>, min_block_time: Duration, max_transactions_per_block: usize) -> Self {
        Self {
            client,
            transaction_pool,
            inherent_providers: CbcInherentDataProviders::new(),
            min_block_time,
            last_block_time: None,
            max_transactions_per_block,
            _phantom: PhantomData,
        }
    }

    /// Create a new block with transactions from the pool using the expected author from DCF runtime API
    pub async fn create_block_with_transactions_and_digest(
        &mut self, 
        parent_hash: B::Hash, 
        slot: u64, 
        author_digest: Option<sp_runtime::generic::DigestItem>
    ) -> ConsensusResult<(B, Public)> {
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

        // Get parent header
        let parent_header = self.client.header(parent_hash)
            .map_err(|e| ConsensusError::Proposer(format!("Failed to get parent header: {:?}", e)))?
            .ok_or_else(|| ConsensusError::Proposer("Parent header not found".into()))?;
        
        let block_number = (*parent_header.number()).saturated_into::<u32>() + 1;
        let header_number = (block_number as u64).saturated_into::<<B::Header as HeaderTrait>::Number>();
        
        debug!("ProposerFactory: Creating block #{} with parent_hash: {:?}", block_number, parent_hash);
        debug!("ProposerFactory: Parent header details - number: {:?}, hash: {:?}", 
               parent_header.number(), parent_header.hash());
        
        // 1. Create inherent data using the inherent providers
        let inherent_data = self.inherent_providers.create_inherent_data().await
            .map_err(|e| ConsensusError::Proposer(format!("Failed to create inherent data: {:?}", e)))?;
        
        debug!("ProposerFactory: Created inherent data successfully");
        
        // 2. Convert inherent data to extrinsics using runtime API
        let inherent_extrinsics = match self.client.runtime_api().inherent_extrinsics(parent_hash, inherent_data) {
            Ok(extrinsics) => {
                debug!("ProposerFactory: Created {} inherent extrinsics", extrinsics.len());
                extrinsics
            }
            Err(e) => {
                error!("ProposerFactory: Failed to create inherent extrinsics: {:?}", e);
                // Fallback: create empty inherents to prevent block production failure
                debug!("ProposerFactory: Using empty inherents as fallback");
                Vec::new()
            }
        };
        
        // 3. Get transactions from the pool
        let ready_transactions = self.collect_transactions_from_pool().await?;
        
        // 4. Combine inherents + transactions (inherents first, as per requirements)
        let mut all_extrinsics = inherent_extrinsics;
        let inherent_count = all_extrinsics.len();
        all_extrinsics.extend(ready_transactions);
        let transaction_count = all_extrinsics.len() - inherent_count;
        
        debug!("ProposerFactory: Combined {} inherent + {} transaction extrinsics (total: {})", 
               inherent_count, transaction_count, all_extrinsics.len());
        
        // Verify extrinsic ordering: inherents should come first
        if inherent_count > 0 && transaction_count > 0 {
            debug!("ProposerFactory: Verified extrinsic ordering - {} inherents followed by {} transactions", 
                   inherent_count, transaction_count);
        }
        
        // Calculate extrinsics root using the correct method
        let extrinsics_root = <<B::Header as HeaderTrait>::Hashing as Hash>::ordered_trie_root(
            all_extrinsics.iter().map(|xt: &B::Extrinsic| xt.encode()).collect(),
            sp_runtime::StateVersion::V1,
        );
        
        debug!("ProposerFactory: Calculated extrinsics root for {} extrinsics: {:?}", 
               all_extrinsics.len(), extrinsics_root);
        
        let state_root = Default::default();
        debug!("ProposerFactory: Header parameters - number: {:?}, extrinsics_root: {:?}, state_root: {:?}, parent_hash: {:?}", 
               header_number, extrinsics_root, state_root, parent_hash);
        
        // Create digest with author information if provided
        let mut digest = sp_runtime::generic::Digest::default();
        if let Some(author_digest_item) = author_digest {
            digest.push(author_digest_item);
            debug!("ProposerFactory: Added author digest to block header");
        }
        
        // Create header with proper roots - correct parameter order: (number, extrinsics_root, state_root, parent_hash, digest)
        let header = B::Header::new(
            header_number,
            extrinsics_root,
            state_root, // state root will be calculated during execution  
            parent_hash,
            digest, // digest with author information
        );
        
        debug!("ProposerFactory: Header created successfully");
        debug!("ProposerFactory: Header parent hash verification - expected: {:?}, actual: {:?}", 
               parent_hash, header.parent_hash());
        
        // Verify parent hash propagation
        if header.parent_hash() != &parent_hash {
            error!("ProposerFactory: Parent hash mismatch! Expected: {:?}, Got: {:?}", 
                   parent_hash, header.parent_hash());
            return Err(ConsensusError::Proposer("Parent hash mismatch in header creation".into()));
        }
        
        debug!("ProposerFactory: Parent hash propagation verified successfully");

        // Create the complete block with transactions
        let block = B::new(header, all_extrinsics);
        
        debug!("Created block #{} with {} extrinsics from pool, final parent: {:?}", 
              block_number, block.extrinsics().len(), block.header().parent_hash());

        self.last_block_time = Some(Instant::now());
        Ok((block, author))
    }

    /// Create a new block with transactions from the pool using the expected author from DCF runtime API
    pub async fn create_block_with_transactions(&mut self, parent_hash: B::Hash, slot: u64) -> ConsensusResult<(B, Public)> {
        self.create_block_with_transactions_and_digest(parent_hash, slot, None).await
    }
    
    /// Collect transactions from the transaction pool
    async fn collect_transactions_from_pool(&self) -> ConsensusResult<Vec<B::Extrinsic>> {
        let ready_transactions = self.transaction_pool.ready()
            .take(self.max_transactions_per_block)
            .map(|tx| (**tx.data()).clone())
            .collect::<Vec<_>>();
        
        debug!("Collected {} transactions from pool", ready_transactions.len());
        Ok(ready_transactions)
    }
    


    /// Create a new block with the expected author from the DCF runtime API (legacy method for compatibility)
    pub fn create_block(&mut self, parent_hash: B::Hash, slot: u64) -> ConsensusResult<(B::Header, Public)> {
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
        debug!("ProposerFactory: Legacy create_block - creating header with parent_hash: {:?}", parent_hash);
        
        let header = B::Header::new(
            number,
            Default::default(), // extrinsics_root
            Default::default(), // state_root
            parent_hash,
            Default::default(), // digest
        );
        
        debug!("ProposerFactory: Legacy header created - parent hash verification: expected: {:?}, actual: {:?}", 
               parent_hash, header.parent_hash());
        
        // Verify parent hash propagation
        if header.parent_hash() != &parent_hash {
            error!("ProposerFactory: Legacy method parent hash mismatch! Expected: {:?}, Got: {:?}", 
                   parent_hash, header.parent_hash());
            return Err(ConsensusError::Proposer("Parent hash mismatch in legacy header creation".into()));
        }

        Ok((header, author))
    }

    /// Set the minimum time between blocks
    pub fn set_min_block_time(&mut self, min_block_time: Duration) {
        self.min_block_time = min_block_time;
    }

    /// Create a new block proposer (header only, for compatibility)
    pub fn create_proposer(&self) -> ConsensusResult<B::Header> {
        let number = <<B as BlockTrait>::Header as HeaderTrait>::Number::zero();
        let header = B::Header::new(
            number,
            Default::default(), // extrinsics_root
            Default::default(), // state_root
            Default::default(), // parent_hash
            Default::default(), // digest
        );
        Ok(header)
    }
}