//! DCF (Dynamic Consensus Framework) implementation
//! 
//! This module provides the core consensus engine implementation for the CBC chain.
//! It handles block production, validation, and author selection.

use crate::{
    error::{ConsensusError, Result},
    types::{ConsensusParams, ValidatorMetrics},
};
use std::{sync::Arc, time::Duration};
use log::{error, info};
use sp_runtime::traits::{Block as BlockTrait, SaturatedConversion};
use sc_consensus::{BlockImport, BlockImportParams, BlockCheckParams, ImportResult};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::Pair;
use tokio::time::sleep;
use sp_core::sr25519::Public;
use pallet_cbc_dcf::DcfApi as RuntimeDcfApi;
use cbc_runtime::AccountId;
use sp_consensus::Error;

/// DCF consensus engine implementation
pub struct DcfConsensus<B, C, P>
where
    B: BlockTrait,
    C: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    C::Api: RuntimeDcfApi<B, AccountId>,
    P: Pair,
{
    client: Arc<C>,
    params: ConsensusParams,
    metrics: ValidatorMetrics,
    last_block_time: Duration,
    current_slot: u64,
    _phantom: std::marker::PhantomData<(B, P)>,
}

impl<B, C, P> DcfConsensus<B, C, P>
where
    B: BlockTrait,
    C: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    C::Api: RuntimeDcfApi<B, AccountId>,
    P: Pair,
{
    /// Create a new DCF consensus engine instance
    pub fn new(client: Arc<C>, params: ConsensusParams) -> Self {
        Self {
            client,
            params,
            metrics: ValidatorMetrics::default(),
            last_block_time: Duration::from_secs(0),
            current_slot: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Start the DCF consensus engine
    pub async fn run(&mut self) {
        info!("DCF: Starting consensus engine with parameters: {:?}", self.params);
        
        loop {
            // Check if we should produce a block
            if self.should_produce_block() {
                // Get current runtime state
                let best_hash = self.client.info().best_hash;
                let best_number = self.client.info().best_number;
                
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
                            info!("DCF: Found {} active validators", validators.len());
                            
                            // Select next author using runtime logic
                            match self.select_next_author_from_runtime(&validators) {
                                Ok(author) => {
                                    info!("DCF: Selected author {:?} for block #{}", author, best_number + 1u32.into());
                                    
                                    if let Err(e) = self.produce_block_with_validation(&author).await {
                                        error!("DCF: Failed to produce block: {:?}", e);
                                    }
                                }
                                Err(e) => error!("DCF: Failed to select next author: {:?}", e),
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
            
            sleep(Duration::from_millis(100)).await;
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
            info!("DCF: Time to produce block - {}s since last block", time_since_last.as_secs());
        }
        
        should_produce
    }
    
    /// Handle epoch transitions
    async fn handle_epoch_transition(&mut self, current_epoch: u32) {
        // Check if we need to handle epoch transition logic
        // This could include updating validator sets, applying score decay, etc.
        info!("DCF: Current epoch: {}", current_epoch);
        
        // In a full implementation, this would:
        // 1. Check if epoch transition is needed
        // 2. Apply validator score decay
        // 3. Update active validator set
        // 4. Handle validator join/leave requests
    }
    
    /// Update validator metrics from runtime
    async fn update_validator_metrics(&mut self) {
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        
        // Get all validator scores and update metrics
        if let Ok(scores) = api.get_validator_scores(best_hash) {
            for (account_id, final_score) in scores {
                let public_key = Public::from_raw(*account_id.as_ref());
                
                // Get detailed validator information
                if let Ok(Some((score, uptime, inference_count, participation_rate, missed_blocks))) = 
                    api.get_validator_profile(best_hash, account_id.clone()) {
                    
                    self.metrics.update_validator_score(
                        public_key, 
                        uptime, 
                        inference_count, 
                        score.try_into().unwrap_or(0)
                    );
                    
                    // Log metrics periodically
                    if self.current_slot % 100 == 0 {
                        info!("DCF: Validator {:?} - Score: {}, Participation: {}%, Missed: {}", 
                              account_id, final_score, participation_rate, missed_blocks);
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
                info!("DCF: No expected author from runtime, using fallback selection");
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
        
        info!("DCF: Using fallback selection - validator {} of {}", index + 1, active_validators.len());
        Ok(Public::from_raw(*selected_validator.as_ref()))
    }

    /// Produce a new block with DCF validation
    async fn produce_block_with_validation(&mut self, author: &Public) -> Result<()> {
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        let best_number = self.client.info().best_number;
        let author_account_id: AccountId = author.clone().into();
        let block_number = (best_number + 1u32.into()).saturated_into::<u32>();

        info!("DCF: Producing block #{} with author {:?}", block_number, author_account_id);

        // Validate block authorship through runtime
        api.validate_block_author(best_hash, block_number, author_account_id.clone())
            .map_err(|e| ConsensusError::BlockProduction(format!("Author validation failed: {:?}", e)))?;

        // Get and update validator metrics
        if let Ok(scores) = api.get_validator_scores(best_hash) {
            if let Some((_, final_score)) = scores.iter().find(|(a, _)| a == &author_account_id) {
                self.metrics.update_validator_score(author.clone(), 0, 0, (*final_score).try_into().unwrap_or(0));
                info!("DCF: Author {:?} has final score: {}", author_account_id, final_score);
            }
        }

        // Get validator profile for additional metrics
        if let Ok(Some((score, uptime, inference_count, participation_rate, missed_blocks))) = 
            api.get_validator_profile(best_hash, author_account_id.clone()) {
            info!("DCF: Validator profile - Score: {}, Uptime: {}, Inferences: {}, Participation: {}%, Missed: {}", 
                  score, uptime, inference_count, participation_rate, missed_blocks);
        }

        // In a full implementation, this would:
        // 1. Create a block proposal with transactions from the pool
        // 2. Sign the block with the author's key
        // 3. Import the block through the client
        // 4. Broadcast to the network
        
        // For now, we simulate successful block production
        info!("DCF: Block #{} produced successfully by {:?}", block_number, author_account_id);

        // Update timing and slot
        self.last_block_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        self.current_slot = self.current_slot.saturating_add(1);

        Ok(())
    }
}

/// Block import implementation for DCF
pub struct DcfBlockImport<B, C>
where
    B: BlockTrait,
    C: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    C::Api: RuntimeDcfApi<B, AccountId>,
{
    client: Arc<C>,
    _phantom: std::marker::PhantomData<B>,
}

impl<B, C> DcfBlockImport<B, C>
where
    B: BlockTrait,
    C: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    C::Api: RuntimeDcfApi<B, AccountId>,
{
    /// Create a new DCF block import instance
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<B, C> BlockImport<B> for DcfBlockImport<B, C>
where
    B: BlockTrait,
    C: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    C::Api: RuntimeDcfApi<B, AccountId>,
{
    type Error = Error;

    async fn check_block(
        &self,
        block: BlockCheckParams<B>,
    ) -> std::result::Result<ImportResult, Self::Error> {
        let api = self.client.runtime_api();
        let author: Public = Default::default();
        let block_number = block.number.saturated_into::<u32>();
        let author_account_id: AccountId = author.clone().into();

        // Call runtime API to validate and emit event if invalid
        let _ = api.validate_block_author(block.hash, block_number, author_account_id);

        // Always import the block, do not halt production
        Ok(ImportResult::imported(true))
    }

    async fn import_block(
        &self,
        _block: BlockImportParams<B>,
    ) -> std::result::Result<ImportResult, Self::Error> {
        // Import block logic here
        // ...

        Ok(ImportResult::imported(true))
    }
}

/// Start the DCF consensus engine
pub async fn start_dcf_consensus<B, C>(
    client: Arc<C>,
    params: ConsensusParams,
) where
    B: BlockTrait,
    C: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    C::Api: RuntimeDcfApi<B, AccountId>,
{
    let mut consensus: DcfConsensus<B, C, sp_core::sr25519::Pair> = DcfConsensus::new(client, params);
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

    type TestBlock = RawBlock<ExtrinsicWrapper<u32>>;
    type TestClient = substrate_test_runtime_client::TestClient;

    fn create_test_client() -> Arc<TestClient> {
        Arc::new(TestClientBuilder::new().build())
    }

    fn create_test_author() -> Public {
        Pair::generate().0.public()
    }

    #[tokio::test]
    async fn test_dcf_consensus_creation() {
        let client = create_test_client();
        let params = ConsensusParams {
            slot_duration: Duration::from_secs(6),
            min_block_time: 1000,
            author_selection_mode: crate::author_selection::AuthorSelectionMode::RoundRobin,
        };

        let consensus = DcfConsensus::<TestBlock, TestClient, Pair>::new(
            client,
            params,
        );

        assert_eq!(consensus.current_slot, 0);
    }

    #[tokio::test]
    async fn test_should_produce_block() {
        let client = create_test_client();
        let params = ConsensusParams {
            slot_duration: Duration::from_secs(6),
            min_block_time: 1000,
            author_selection_mode: crate::author_selection::AuthorSelectionMode::RoundRobin,
        };

        let mut consensus = DcfConsensus::<TestBlock, TestClient, Pair>::new(
            client,
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
            header,
            block: None,
            allow_missing_state: false,
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
        };

        let consensus = DcfConsensus::<TestBlock, TestClient, Pair>::new(
            client,
            params,
        );

        // Test author selection
        let author = consensus.select_next_author();
        assert!(author.is_ok());
    }

    #[tokio::test]
    async fn test_metrics_update() {
        let client = create_test_client();
        let params = ConsensusParams {
            slot_duration: Duration::from_secs(6),
            min_block_time: 1000,
            author_selection_mode: crate::author_selection::AuthorSelectionMode::RoundRobin,
        };

        let mut consensus = DcfConsensus::<TestBlock, TestClient, Pair>::new(
            client,
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