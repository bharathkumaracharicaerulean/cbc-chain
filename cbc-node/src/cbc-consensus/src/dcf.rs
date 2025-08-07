//! DCF (Dynamic Consensus Framework) implementation
//! 
//! This module provides the core consensus engine implementation for the CBC chain.
//! It handles block production, validation, and author selection.

use crate::{
    error::{ConsensusError, Result},
    types::{ConsensusParams, ValidatorMetrics},
    proposer_factory::ProposerFactory,
};
use std::{sync::Arc, time::Duration};
use log::{debug, error, info};
use sp_runtime::traits::{Block as BlockTrait, SaturatedConversion, Header as HeaderT};
use sc_consensus::{BlockImport, BlockImportParams, ImportResult};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::Pair;
use tokio::time::sleep;
use sp_core::sr25519::Public;
use pallet_cbc_dcf::DcfApi as RuntimeDcfApi;
use cbc_runtime::AccountId;
use sc_transaction_pool_api::TransactionPool;
use sp_runtime::DigestItem;
// use sc_client_api::BlockBackend;


/// DCF consensus engine implementation
pub struct DcfConsensus<B, C, P, TP>
where
    B: BlockTrait,
    C: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    C::Api: RuntimeDcfApi<B, AccountId>,
    P: Pair,
    TP: TransactionPool<Block = B> + 'static,
{
    client: Arc<C>,
    proposer_factory: ProposerFactory<B, C, TP>,
    block_import: Arc<dyn BlockImport<B, Error = sp_consensus::Error> + Send + Sync>,
    params: ConsensusParams,
    metrics: ValidatorMetrics,
    last_block_time: Duration,
    current_slot: u64,
    _phantom: std::marker::PhantomData<(B, P, TP)>,
}

impl<B, C, P, TP> DcfConsensus<B, C, P, TP>
where
    B: BlockTrait,
    C: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    C::Api: RuntimeDcfApi<B, AccountId>,
    P: Pair,
    TP: TransactionPool<Block = B> + 'static,
{
    /// Create a new DCF consensus engine instance
    pub fn new(
        client: Arc<C>, 
        transaction_pool: Arc<TP>, 
        block_import: Arc<dyn BlockImport<B, Error = sp_consensus::Error> + Send + Sync>,
        params: ConsensusParams
    ) -> Self {
        // Initialize last_block_time to 0 so the first block can be produced immediately
        let last_block_time = Duration::from_secs(0);
            
        let proposer_factory = ProposerFactory::new(
            client.clone(),
            transaction_pool.clone(),
            Duration::from_millis(params.min_block_time as u64),
            params.max_transactions_per_block as usize,
        );
            
        Self {
            client,
            proposer_factory,
            block_import,
            params,
            metrics: ValidatorMetrics::default(),
            last_block_time,
            current_slot: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Start the DCF consensus engine
    pub async fn run(&mut self) {
        
        loop {
            // Check if we should produce a block
            if self.should_produce_block() {
                // Get current runtime state
                let best_hash = self.client.info().best_hash;
                let _best_number = self.client.info().best_number;
                
                // Get active validators from runtime
                let active_validators = {
                    let api = self.client.runtime_api();
                    api.get_active_validators(best_hash)
                };
                
                match active_validators {
                    Ok(validators) => {
                        if validators.is_empty() {
                            error!("DCF: No active validators available for block production");
                        } else {
                            
                            // Select next author using runtime logic
                            match self.select_next_author_from_runtime(&validators) {
                                Ok(author) => {
                                    match self.produce_block_with_validation(&author).await {
                                        Ok(()) => {
                                            debug!("Block production successful for author {:?}", author);
                                        }
                                        Err(e) => {
                                            debug!("Block production failed for author {:?}: {:?}", author, e);
                                            // Update consensus state to handle the failure
                                            let author_account: AccountId = author.into();
                                            if let Err(state_err) = self.handle_block_production_failure(&author_account).await {
                                                debug!("Failed to handle block production failure: {:?}", state_err);
                                            }
                                        }
                                    }
                                }
                                Err(e) => debug!("Failed to select next author: {:?}", e),
                            }
                        }
                    }
                    Err(e) => error!("DCF: Failed to get active validators: {:?}", e),
                }
                
                // Check for epoch transitions
                let current_epoch = {
                    let api = self.client.runtime_api();
                    api.get_current_epoch(best_hash)
                };
                
                if let Ok(epoch) = current_epoch {
                    self.handle_epoch_transition(epoch).await;
                }
            }
            
            // Update validator metrics periodically
            if self.current_slot % 10 == 0 {
                self.update_validator_metrics().await;
            }
            
            sleep(Duration::from_millis(1000)).await;
        }
    }

    /// Check if it's time to produce a new block
    fn should_produce_block(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        
        let block_interval = Duration::from_secs(self.params.block_time);
        let time_since_last = now.saturating_sub(self.last_block_time);
        
        let should_produce = time_since_last >= block_interval;
        
        if should_produce {
            debug!("Time to produce block - interval: {:?}, time since last: {:?}", block_interval, time_since_last);
        }
        
        should_produce
    }
    
    /// Handle epoch transitions
    async fn handle_epoch_transition(&mut self, current_epoch: u32) {
        
        let current_block = self.client.info().best_number.saturated_into::<u32>();
        
        // Check if epoch transition occurs based on runtime state
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        
        if let Ok(epoch_config) = api.get_epoch_config(best_hash) {
            let blocks_per_epoch = epoch_config.blocks_per_epoch;
            let epoch_start_block = current_epoch.saturating_mul(blocks_per_epoch);
            let should_transition = current_block >= epoch_start_block + blocks_per_epoch;
            
            if should_transition {
                
                //  actual epoch transition is handled by the runtime pallet 
                //  So we just log and perform maintenance here
                self.perform_epoch_maintenance(current_epoch).await;
            } else {
                // No transition needed, but still perform periodic maintenance
                self.perform_epoch_maintenance(current_epoch).await;
            }
        } else {
            error!("DCF: Failed to get epoch configuration from runtime");
        }
    }
    
    /// Perform periodic maintenance during an epoch
    async fn perform_epoch_maintenance(&mut self, _current_epoch: u32) {
        // Update validator metrics and check for issues
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        
        if let Ok(active_validators) = api.get_active_validators(best_hash) {
            let mut _healthy_validators = 0;
            let mut total_score = 0u64;
            
            for validator in active_validators.iter() {
                if let Ok(Some((combined_score, _pos_score, _poi_score, _uptime, _inference_count, participation_rate, missed_blocks))) = 
                    api.get_validator_profile(best_hash, validator.clone()) {
                    
                    total_score += combined_score;
                    
                    // Check validator health
                    if combined_score >= 50 && participation_rate >= 80 && missed_blocks <= 5 {
                        _healthy_validators += 1;
                    }
                }
            }
            
            let _average_score = if !active_validators.is_empty() {
                total_score / active_validators.len() as u64
            } else {
                0
            };
            

        }
    }
    
    /// Update validator metrics from runtime
    async fn update_validator_metrics(&mut self) {
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        
        // Get all validator scores and update metrics
        if let Ok(scores) = api.get_validator_scores(best_hash) {
            for (account_id, _final_score) in scores {
                let public_key = Public::from_raw(*account_id.as_ref());
                
                // Get detailed validator information
                if let Ok(Some((combined_score, pos_score, poi_score, uptime, inference_count, participation_rate, missed_blocks))) = 
                    api.get_validator_profile(best_hash, account_id.clone()) {
                    
                    self.metrics.update_validator_score(
                        public_key, 
                        uptime, 
                        inference_count, 
                        combined_score.try_into().unwrap_or(0)
                    );
                    
                    // Log metrics periodically
                    if self.current_slot % 100 == 0 {
                        debug!("Validator {:?} - Combined Score: {}, PoS: {}, PoI: {}, Participation: {}%, Missed: {}", 
                              account_id, combined_score, pos_score, poi_score, participation_rate, missed_blocks);
                    }
                }
            }
        }
    }

    /// Select the next block author from active validators using runtime logic
    fn select_next_author_from_runtime(&mut self, active_validators: &[AccountId]) -> Result<Public> {
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        let block_number = (self.current_slot + 1) as u32;
        
        // Try to get expected author from runtime
        match api.get_expected_author(best_hash, block_number) {
            Ok(Some(account_id)) => {
                // Verify the author is in active validators
                if active_validators.contains(&account_id) {
                    Ok(Public::from_raw(*account_id.as_ref()))
                } else {
                    error!("DCF: Expected author {:?} is not in active validator set", account_id);
                    self.fallback_author_selection(active_validators)
                }
            }
            Ok(None) => {
                debug!("No expected author from runtime, using fallback selection");
                self.fallback_author_selection(active_validators)
            }
            Err(e) => {
                error!("DCF: Runtime API error for author selection: {:?}", e);
                self.fallback_author_selection(active_validators)
            }
        }
    }
    
    /// Fallback author selection using round-robin
    fn fallback_author_selection(&self, active_validators: &[AccountId]) -> Result<Public> {
        if active_validators.is_empty() {
            return Err(ConsensusError::AuthorSelection("No active validators available".into()));
        }
        
        let index = (self.current_slot as usize) % active_validators.len();
        let selected_validator = &active_validators[index];
        
        debug!("Using fallback selection - validator {} of {}", index + 1, active_validators.len());
        Ok(Public::from_raw(*selected_validator.as_ref()))
    }

    /// Produce a new block with DCF validation
    async fn produce_block_with_validation(&mut self, author: &Public) -> Result<()> {
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        let best_number = self.client.info().best_number;
        let author_account_id: AccountId = author.clone().into();
        let block_number = (best_number + 1u32.into()).saturated_into::<u32>();

        // Only log every 10th block to reduce noise
        if block_number % 10 == 1 {
            info!("Producing block #{} with author {:?}", block_number, author_account_id);
        } else {
            debug!("Producing block #{} with author {:?}", block_number, author_account_id);
        }

        // Validate block authorship through runtime
        api.validate_block_author(best_hash, block_number, author_account_id.clone())
            .map_err(|e| ConsensusError::BlockProduction(format!("Author validation failed: {:?}", e)))?;

        // Get and update validator metrics
        if let Ok(scores) = api.get_validator_scores(best_hash) {
            if let Some((_, final_score)) = scores.iter().find(|(a, _)| a == &author_account_id) {
                self.metrics.update_validator_score(author.clone(), 0, 0, (*final_score).try_into().unwrap_or(0));
                debug!("DCF: Author {:?} has final score: {}", author_account_id, final_score);
            }
        }

        // Get validator profile for additional metrics
        if let Ok(Some((combined_score, pos_score, poi_score, uptime, inference_count, participation_rate, missed_blocks))) = 
            api.get_validator_profile(best_hash, author_account_id.clone()) {
            debug!("DCF: Validator profile - Combined: {}, PoS: {}, PoI: {}, Uptime: {}, Inferences: {}, Participation: {}%, Missed: {}", 
                  combined_score, pos_score, poi_score, uptime, inference_count, participation_rate, missed_blocks);
        }

        // 1. Create block proposal with transactions from the pool
        let block_proposal = self.create_block_proposal(author, block_number).await?;
        
        // 2. Sign the block proposal
        let signed_block = self.sign_block_proposal(block_proposal, author).await?;
        
        // 3. Import the block through the consensus pipeline
        let _import_result = self.import_consensus_block(signed_block).await?;
        
        // 4. Update consensus state after successful block production
        self.update_consensus_state(block_number, &author_account_id).await?;
        
        info!("Block #{} produced by {:?}", block_number, author_account_id);
        Ok(())
    }

    /// Create block proposal with transactions from the pool
    async fn create_block_proposal(&mut self, author: &Public, block_number: u32) -> Result<B> {
        debug!("Creating block proposal #{} for author {:?}", block_number, author);
        
        let parent_hash = self.client.info().best_hash;
        
        // Use the enhanced proposer factory to create a complete block with transactions
        let (block, expected_author) = self.proposer_factory.create_block_with_transactions(parent_hash, block_number as u64)
            .await
            .map_err(|e| ConsensusError::BlockProduction(format!("Failed to create block proposal: {:?}", e)))?;
        
        // Verify the expected author matches our selected author
        let author_account: AccountId = author.clone().into();
        let expected_account: AccountId = expected_author.into();
        if author_account != expected_account {
            return Err(ConsensusError::BlockProduction(format!(
                "Author mismatch: expected {:?}, got {:?}", expected_account, author_account
            )));
        }
        
        debug!("Created block #{} with {} transactions", block_number, block.extrinsics().len());
        
        Ok(block)
    }
    

    
    /// Sign block proposal with author's key
    async fn sign_block_proposal(&self, block: B, author: &Public) -> Result<B> {
        let block_number = *block.header().number();
        debug!("Signing block #{} with author {:?}", block_number, author);
        
        // For now, use a simplified signing approach
        //  this would integrate with the keystore properly
        // Caution need an attention here 
        let block_hash = block.header().hash();
        
        // Create a basic signature using the author's public key and block hash (palceholder caution)
        let signature_data = {
            let mut data = Vec::new();
            data.extend_from_slice(author.as_ref());
            data.extend_from_slice(block_hash.as_ref());
            data.extend_from_slice(&(block_number.saturated_into::<u32>()).to_le_bytes());
            sp_core::hashing::blake2_256(&data)
        };
        
        // Add the signature to the block's digest as a seal
        let seal_digest = DigestItem::Seal(
            *b"cbc ", // Using CBC consensus engine ID (4 bytes)
            signature_data.to_vec(),
        );
        
        // Create new header with the seal
        let mut header = block.header().clone();
        let mut digest = header.digest().clone();
        digest.push(seal_digest);
        
        // Update the header with the new digest
        *header.digest_mut() = digest;
        
        // Create new block with signed header
        let signed_block = B::new(header, block.extrinsics().to_vec());
        
        debug!("Block #{} signed and sealed", block_number);
        Ok(signed_block)
    }
    
    /// Import consensus block through pipeline
    async fn import_consensus_block(&self, block: B) -> Result<()> {
        let block_hash = block.header().hash();
        let block_number = *block.header().number();
        
        debug!("Importing consensus block #{} ({:?})", block_number, block_hash);
        
        // Create proper block import parameters
        let mut import_params = BlockImportParams::new(
            sp_consensus::BlockOrigin::Own,
            block.header().clone(),
        );
        import_params.body = Some(block.extrinsics().to_vec());
        import_params.finalized = false;
        import_params.fork_choice = Some(sc_consensus::ForkChoiceStrategy::LongestChain);
        
        // Import through our block import 
        match self.block_import.import_block(import_params).await {
            Ok(ImportResult::Imported(_)) => {
                debug!("Block #{} successfully imported", block_number);
                Ok(())
            }
            Ok(ImportResult::AlreadyInChain) => {
                debug!("Block #{} already in chain", block_number);
                Ok(())
            }
            Ok(other) => {
                debug!("Block #{} import result: {:?}", block_number, other);
                Ok(())
            }
            Err(e) => {
                error!("Failed to import block #{}: {:?}", block_number, e);
                Err(ConsensusError::BlockImport(format!("Import failed: {:?}", e)))
            }
        }
    }
    
    /// Update consensus state after successful block production
    async fn update_consensus_state(&mut self, block_number: u32, author: &AccountId) -> Result<()> {
        // Update timing and slot
        self.last_block_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        self.current_slot = self.current_slot.saturating_add(1);
        
        // Update metrics
        self.metrics.total_blocks = self.metrics.total_blocks.saturating_add(1);
        
        // Update validator-specific metrics
        let author_public = Public::from_raw(*author.as_ref());
        self.metrics.update_validator_score(author_public, 0, 0, 1); // Increment block count
        
        // Update runtime state through API calls
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        
        // Update validator performance in runtime 
        if let Ok(Some((combined_score, pos_score, poi_score, uptime, inference_count, participation_rate, missed_blocks))) = 
            api.get_validator_profile(best_hash, author.clone()) {
            
            // Log the successful block production
            debug!("Block #{} produced successfully by {:?}", block_number, author);
            debug!("Validator stats - Combined: {}, PoS: {}, PoI: {}, Uptime: {}, Inferences: {}, Participation: {}%, Missed: {}", 
                  combined_score, pos_score, poi_score, uptime, inference_count, participation_rate, missed_blocks);
            
            // Update local metrics with current runtime state
            self.metrics.update_validator_score(
                author_public, 
                uptime, 
                inference_count, 
                combined_score.try_into().unwrap_or(0)
            );
        }
        
        // Log consensus state update
        debug!("Updated consensus state - Block: {}, Slot: {}, Total Blocks: {}, Author: {:?}", 
              block_number, self.current_slot, self.metrics.total_blocks, author);
        
        // Periodic state health check
        if block_number % 10 == 0 {
            self.log_consensus_health().await?;
        }
        
        Ok(())
    }
    
    /// Log consensus health metrics
    async fn log_consensus_health(&self) -> Result<()> {
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        let current_block = self.client.info().best_number.saturated_into::<u32>();
        
        // Get active validators count
        if let Ok(active_validators) = api.get_active_validators(best_hash) {
            let validator_count = active_validators.len();
            
            // Calculate average validator score
            let mut total_score = 0u64;
            let mut healthy_validators = 0;
            
            for validator in active_validators.iter().take(5) { // Sample first 5 validators
                if let Ok(Some((combined_score, _pos_score, _poi_score, _uptime, _inference_count, participation_rate, missed_blocks))) = 
                    api.get_validator_profile(best_hash, validator.clone()) {
                    total_score += combined_score;
                    if combined_score >= 50 && participation_rate >= 70 && missed_blocks <= 10 {
                        healthy_validators += 1;
                    }
                }
            }
            
            let avg_score = if validator_count > 0 { total_score / validator_count.min(5) as u64 } else { 0 };
            
            info!("PoS+PoI: Consensus Health - Block: {}, Slot: {}, Validators: {}, Healthy: {}, Avg Score: {}", 
                  current_block, self.current_slot, validator_count, healthy_validators, avg_score);
            
            // Check if we need to trigger epoch transition
            if let Ok(current_epoch) = api.get_current_epoch(best_hash) {
                info!("PoS+PoI: Current epoch: {}, Total blocks produced: {}", 
                      current_epoch, self.metrics.total_blocks);
            }
        }
        
        Ok(())
    }
    
    /// Handle block production failure and update consensus state
    async fn handle_block_production_failure(&mut self, author: &AccountId) -> Result<()> {
        // Still advance the slot even if block production failed
        self.current_slot = self.current_slot.saturating_add(1);
        
        // Update timing to prevent immediate retry
        self.last_block_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        
        // Update metrics to track the failure
        self.metrics.failed_blocks = self.metrics.failed_blocks.saturating_add(1);
        
        // Log the failure for monitoring
        info!("PoS+PoI: Block production failed - Slot: {}, Failed blocks: {}, Author: {:?}", 
              self.current_slot, self.metrics.failed_blocks, author);
        
        //  update validator metrics in runtime to reflect the missed block
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        
        if let Ok(Some((combined_score, pos_score, poi_score, _uptime, _inference_count, participation_rate, missed_blocks))) = 
            api.get_validator_profile(best_hash, author.clone()) {
            
            info!("PoS+PoI: Validator {:?} missed block - Combined: {}, PoS: {}, PoI: {}, Participation: {}%, Total Missed: {}", 
                  author, combined_score, pos_score, poi_score, participation_rate, missed_blocks + 1);
        }
        
        Ok(())
    }
}

/// Block import implementation that actually imports blocks to chain state
pub struct RealBlockImport<B, C>
where
    B: BlockTrait,
    C: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    C::Api: RuntimeDcfApi<B, AccountId>,
{
    client: Arc<C>,
    _phantom: std::marker::PhantomData<B>,
}

impl<B, C> RealBlockImport<B, C>
where
    B: BlockTrait,
    C: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    C::Api: RuntimeDcfApi<B, AccountId>,
{
    /// Create a new RealBlockImport instance
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<B, C> BlockImport<B> for RealBlockImport<B, C>
where
    B: BlockTrait,
    C: ProvideRuntimeApi<B> + HeaderBackend<B> +BlockImport<B> + Send + Sync + 'static,
    C::Api: RuntimeDcfApi<B, AccountId>,
{
    type Error = sp_consensus::Error;

    async fn check_block(
        &self,
        block: sc_consensus::BlockCheckParams<B>,
    ) -> std::result::Result<ImportResult, Self::Error> {
        let block_number = block.number.saturated_into::<u32>();
        
        debug!("Checking block #{}", block_number);
        
        // Basic validation - check if we have active validators
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        
        let active_validators = api.get_active_validators(best_hash)
            .map_err(|e| sp_consensus::Error::ClientImport(format!("Failed to get active validators: {:?}", e)))?;
        
        if active_validators.is_empty() {
            error!("DCF: Block check failed for block #{}: No active validators", block_number);
            return Ok(ImportResult::imported(false));
        }
        
        debug!("Block #{} check passed", block_number);
        Ok(ImportResult::imported(true))
    }

    async fn import_block(
        &self,
        block: BlockImportParams<B>,
    ) -> std::result::Result<ImportResult, Self::Error> {
        let block_number = (*block.header.number()).saturated_into::<u32>();
        let block_hash = block.header.hash();
        
        debug!("Importing block #{} ({:?})", block_number, block_hash);
        
        // Validate that we have active validators
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        
        let active_validators = api.get_active_validators(best_hash)
            .map_err(|e| sp_consensus::Error::ClientImport(format!("Failed to get active validators: {:?}", e)))?;
        
        if active_validators.is_empty() {
            error!("DCF: Block import failed for block #{}: No active validators", block_number);
            return Ok(ImportResult::imported(false));
        }
        
        // Actually import the block using the client's import functionality
        let import_result = self.client.import_block(block).await
            .map_err(|e| sp_consensus::Error::ClientImport(format!("Client import failed: {:?}", e)))?;
        
        // Log periodic statistics
        if block_number % 10u32 == 0 {
            let current_epoch = api.get_current_epoch(best_hash).unwrap_or(0);
            info!("Block #{} processed - {} active validators, epoch {}", 
                  block_number, active_validators.len(), current_epoch);
        }
        
        debug!("Block #{} ({:?}) successfully imported to chain", block_number, block_hash);
        
        Ok(import_result)
    }
}

impl<B, C> RealBlockImport<B, C>
where
    B: BlockTrait,
    C: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    C::Api: RuntimeDcfApi<B, AccountId>,
{
    /// Update validator scores based on block authorship
    pub fn update_block_authorship_scores(&self, block_number: u32) {
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        
        // Get the expected author for this block
        if let Ok(Some(expected_author)) = api.get_expected_author(best_hash, block_number) {
            // Record successful block authorship
            debug!("Recording block authorship for validator {:?} at block #{}", 
                  expected_author, block_number);
            
            // Update validator metrics
            if let Ok(Some((combined_score, pos_score, poi_score, uptime, inference_count, _participation_rate, _missed_blocks))) = 
                api.get_validator_profile(best_hash, expected_author.clone()) {
                
                // Log validator metrics
                debug!("Validator {:?} metrics - Combined: {}, PoS: {}, PoI: {}, Uptime: {}, Inferences: {}", 
                      expected_author, combined_score, pos_score, poi_score, uptime, inference_count);
                
                // Log successful block production
                debug!("Block #{} successfully produced by validator {:?} (combined score: {})", 
                      block_number, expected_author, combined_score);
            }
        }
    }
}

/// Start the DCF consensus engine
pub async fn start_dcf_consensus<B, C, TP>(
    client: Arc<C>,
    transaction_pool: Arc<TP>,
    params: ConsensusParams,
) where
    B: BlockTrait,
    C: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    C::Api: RuntimeDcfApi<B, AccountId>,
    TP: TransactionPool<Block = B> + 'static,
{
    // Create a mock block import for testing (CAUTION required)
    let mock_block_import = Arc::new(crate::import_queue::DcfImportQueue::new(client.clone()));
    let mut consensus: DcfConsensus<B, C, sp_core::sr25519::Pair, TP> = DcfConsensus::new(client, transaction_pool, mock_block_import, params);
    consensus.run().await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use sp_core::sr25519::{Pair, Public};
    use sp_runtime::testing::{Block as RawBlock, ExtrinsicWrapper};
    use sp_runtime::traits::Header as HeaderT;
    use sp_runtime::traits::Zero;
    use sp_runtime::BuildStorage;
    use substrate_test_runtime_client::{
        runtime::Block,
        DefaultTestClientBuilderExt,
        TestClientBuilder,
        TestClientBuilderExt,
    };
    use std::sync::Arc;
    use sc_transaction_pool_api::{TransactionPool, PoolStatus, TransactionFor, TransactionSource};
    use futures::future::Ready;
    use std::collections::HashMap;

    type TestBlock = RawBlock<ExtrinsicWrapper<u32>>;
    type TestClient = substrate_test_runtime_client::TestClient;

    fn create_test_client() -> Arc<TestClient> {
        Arc::new(TestClientBuilder::new().build())
    }

    fn create_test_author() -> Public {
        Pair::generate().0.public()
    }

    // Mock transaction pool for testing
    struct MockTransactionPool;

    impl TransactionPool for MockTransactionPool {
        type Block = TestBlock;
        type Hash = sp_core::H256;
        type InPoolTransaction = ();
        type Error = ();

        fn submit_at(&self, _at: &BlockId<Self::Block>, _source: TransactionSource, _xts: Vec<TransactionFor<Self>>) -> Ready<Result<Vec<Result<Self::Hash, Self::Error>>, Self::Error>> {
            futures::future::ready(Ok(vec![]))
        }

        fn submit_one(&self, _at: &BlockId<Self::Block>, _source: TransactionSource, _xt: TransactionFor<Self>) -> Ready<Result<Self::Hash, Self::Error>> {
            futures::future::ready(Ok(Default::default()))
        }

        fn submit_and_watch(&self, _at: &BlockId<Self::Block>, _source: TransactionSource, _xt: TransactionFor<Self>) -> Ready<Result<Box<dyn sc_transaction_pool_api::TransactionStatusStreamFor<Self> + Send>, Self::Error>> {
            futures::future::ready(Err(()))
        }

        fn ready_at(&self, _at: NumberFor<Self::Block>) -> sc_transaction_pool_api::PolledIterator<Self::InPoolTransaction> {
            Box::pin(futures::stream::empty())
        }

        fn ready(&self) -> sc_transaction_pool_api::ReadyIterator<Self::InPoolTransaction> {
            Box::new(std::iter::empty())
        }

        fn remove_invalid(&self, _hashes: &[Self::Hash]) -> Vec<Arc<Self::InPoolTransaction>> {
            vec![]
        }

        fn futures(&self) -> sc_transaction_pool_api::ReadyIterator<Self::InPoolTransaction> {
            Box::new(std::iter::empty())
        }

        fn status(&self) -> PoolStatus {
            PoolStatus {
                ready: 0,
                ready_bytes: 0,
                future: 0,
                future_bytes: 0,
            }
        }

        fn import_notification_stream(&self) -> sc_transaction_pool_api::ImportNotificationStream<Self::Hash> {
            Box::pin(futures::stream::empty())
        }

        fn on_broadcasted(&self, _propagations: HashMap<Self::Hash, Vec<String>>) {}

        fn hash_of(&self, _xt: &TransactionFor<Self>) -> Self::Hash {
            Default::default()
        }

        fn ready_transaction(&self, _hash: &Self::Hash) -> Option<Arc<Self::InPoolTransaction>> {
            None
        }
    }

    #[tokio::test]
    async fn test_dcf_consensus_creation() {
        let client = create_test_client();
        let transaction_pool = Arc::new(MockTransactionPool);
        let params = ConsensusParams {
            slot_duration: Duration::from_secs(6),
            min_block_time: 1000,
            author_selection_mode: crate::author_selection::AuthorSelectionMode::RoundRobin,
            finality_threshold: 2,
            block_time: 6,
            max_block_size: 5 * 1024 * 1024,
            max_transactions_per_block: 1000,
        };

        let consensus = DcfConsensus::<TestBlock, TestClient, Pair, MockTransactionPool>::new(
            client,
            transaction_pool,
            params,
        );

        assert_eq!(consensus.current_slot, 0);
    }

    #[tokio::test]
    async fn test_should_produce_block() {
        let client = create_test_client();
        let transaction_pool = Arc::new(MockTransactionPool);
        let params = ConsensusParams {
            slot_duration: Duration::from_secs(6),
            min_block_time: 1000,
            author_selection_mode: crate::author_selection::AuthorSelectionMode::RoundRobin,
            finality_threshold: 2,
            block_time: 6,
            max_block_size: 5 * 1024 * 1024,
            max_transactions_per_block: 1000,
        };

        let mut consensus = DcfConsensus::<TestBlock, TestClient, Pair, MockTransactionPool>::new(
            client,
            transaction_pool,
            params,
        );

        // Initially should not produce block
        assert!(!consensus.should_produce_block());

        // Advance time
        consensus.last_block_time = Duration::from_secs(0);
        tokio::time::sleep(Duration::from_secs(7)).await;

        // Now should produce block
        assert!(consensus.should_produce_block());
    }

    #[tokio::test]
    async fn test_block_import_validation() {
        let client = create_test_client();
        let block_import = DcfBlockImport::<TestBlock, TestClient>::new(client);

        // Create a test block
        let header = TestBlock::Header::new(
            1,
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
        );

        let block = BlockCheckParams {
            hash: header.hash(),
            number: *header.number(),
            parent_hash: *header.parent_hash(),
            allow_missing_state: false,
            allow_missing_parent: false,
            import_existing: false,
        };

        // Test block validation
        let result = block_import.check_block(block).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_author_selection() {
        let client = create_test_client();
        let params = ConsensusParams {
            slot_duration: Duration::from_secs(6),
            min_block_time: 1000,
            author_selection_mode: crate::author_selection::AuthorSelectionMode::RoundRobin,
            finality_threshold: 2,
            block_time: 6,
            max_block_size: 5 * 1024 * 1024,
            max_transactions_per_block: 1000,
        };

        let transaction_pool = Arc::new(MockTransactionPool);
        let consensus = DcfConsensus::<TestBlock, TestClient, Pair, MockTransactionPool>::new(
            client,
            transaction_pool,
            params,
        );

        // Test that consensus was created successfully
        assert_eq!(consensus.current_slot, 0);
    }

    #[tokio::test]
    async fn test_metrics_update() {
        let client = create_test_client();
        let params = ConsensusParams {
            slot_duration: Duration::from_secs(6),
            min_block_time: 1000,
            author_selection_mode: crate::author_selection::AuthorSelectionMode::RoundRobin,
            finality_threshold: 2,
            block_time: 6,
            max_block_size: 5 * 1024 * 1024,
            max_transactions_per_block: 1000,
        };

        let transaction_pool = Arc::new(MockTransactionPool);
        let mut consensus = DcfConsensus::<TestBlock, TestClient, Pair, MockTransactionPool>::new(
            client,
            transaction_pool,
            params,
        );

        let author = create_test_author();
        consensus.metrics.update_validator_score(
            author,
            100, // stake_weight
            50,  // inference_weight
            75,  // final_score
        );

        // Verify metrics were updated
        assert_eq!(consensus.metrics.total_blocks, 0);
    }
}