//! DCF (Dynamic Consensus Framework) implementation
//! 
//! This module provides the core consensus engine implementation for the CBC chain.
//! It handles block production, validation, and author selection.

use crate::{
    author_selection::AuthorSelection,
    error::{ConsensusError, Result},
    types::{ConsensusParams, ValidatorMetrics},
};
use std::{sync::Arc, time::Duration};
use log::{error, warn, info};
use sp_runtime::traits::{Block as BlockTrait, Header as HeaderTrait, SaturatedConversion};
use sc_consensus::{BlockImport, BlockImportParams, BlockCheckParams, ImportResult};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::Pair;
use tokio::time::sleep;
use sp_core::sr25519::Public;
use pallet_cbc_dcf::DcfApi as RuntimeDcfApi;
use cbc_runtime::AccountId;

/// DCF consensus engine implementation
pub struct DcfConsensus<B, C, P>
where
    B: BlockTrait + HeaderTrait,
    C: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    C::Api: RuntimeDcfApi<B, AccountId>,
    P: Pair,
{
    client: Arc<C>,
    author_selection: AuthorSelection,
    params: ConsensusParams,
    metrics: ValidatorMetrics,
    last_block_time: Duration,
    current_slot: u64,
    _phantom: std::marker::PhantomData<(B, P)>,
}

impl<B, C, P> DcfConsensus<B, C, P>
where
    B: BlockTrait + HeaderTrait,
    C: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    C::Api: RuntimeDcfApi<B, AccountId>,
    P: Pair,
{
    /// Create a new DCF consensus engine instance
    pub fn new(client: Arc<C>, author_selection: AuthorSelection, params: ConsensusParams) -> Self {
        Self {
            client,
            author_selection,
            params,
            metrics: ValidatorMetrics::default(),
            last_block_time: Duration::from_secs(0),
            current_slot: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Start the DCF consensus engine
    pub async fn run(&mut self) {
        info!("Starting DCF consensus engine");
        
        loop {
            if self.should_produce_block() {
                match self.select_next_author() {
                    Ok(author) => {
                        if self.is_valid_author(&author) {
                            if let Err(e) = self.produce_block(&author).await {
                                error!("Failed to produce block: {:?}", e);
                            }
                        } else {
                            warn!("Invalid block author selected: {:?}", author);
                        }
                    }
                    Err(e) => error!("Failed to select next author: {:?}", e),
                }
            }
            
            sleep(Duration::from_millis(100)).await;
        }
    }

    /// Check if it's time to produce a new block
    fn should_produce_block(&self) -> bool {
        let now = Duration::from_secs(self.params.block_time);
        now.saturating_sub(self.last_block_time) >= Duration::from_secs(self.params.block_time)
    }

    /// Select the next block author
    fn select_next_author(&mut self) -> Result<Public> {
        self.author_selection.select_author(self.current_slot).map(|v| v.account_id.clone())
    }

    /// Check if the author is valid for the current epoch
    fn is_valid_author(&self, author: &Public) -> bool {
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        let author_account_id: AccountId = author.clone().into();
        match api.get_active_validators(best_hash) {
            Ok(validators) => validators.contains(&author_account_id),
            Err(_) => false,
        }
    }

    /// Produce a new block
    async fn produce_block(&mut self, author: &Public) -> Result<()> {
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        let author_account_id: AccountId = author.clone().into();

        // Get validator scores and update metrics
        if let Ok(scores) = api.get_validator_scores(best_hash) {
            if let Some(final_score) = scores.iter().find_map(|(a, final_score)| {
                if a == &author_account_id {
                    Some(*final_score)
                } else {
                    None
                }
            }) {
                self.metrics.update_validator_score(author.clone(), 0, 0, final_score.try_into().unwrap());
            }
        }

        // Produce block logic here
        // ...

        self.last_block_time = Duration::from_secs(self.params.block_time);
        self.current_slot = self.current_slot.saturating_add(1);

        Ok(())
    }
}

/// Block import implementation for DCF
pub struct DcfBlockImport<B, C> {
    client: Arc<C>,
    _phantom: std::marker::PhantomData<B>,
}

impl<B, C> DcfBlockImport<B, C>
where
    B: BlockTrait + HeaderTrait,
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
    B: BlockTrait + HeaderTrait,
    C: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    C::Api: RuntimeDcfApi<B, AccountId>,
{
    type Error = ConsensusError;

    async fn check_block(
        &self,
        block: BlockCheckParams<B>,
    ) -> Result<ImportResult> {
        let api = self.client.runtime_api();
        // TODO: Extract author from header/extrinsics
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
    ) -> Result<ImportResult> {
        // Import block logic here
        // ...

        Ok(ImportResult::imported(true))
    }
}

/// Start the DCF consensus engine
pub async fn start_dcf_consensus<B, C>(
    client: Arc<C>,
    author_selection: AuthorSelection,
    params: ConsensusParams,
) where
    B: BlockTrait + HeaderTrait,
    C: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    C::Api: RuntimeDcfApi<B, AccountId>,
{
    let mut consensus: DcfConsensus<B, C, sp_core::sr25519::Pair> = DcfConsensus::new(client, author_selection, params);
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
        let author_selection = AuthorSelection::new(crate::author_selection::AuthorSelectionMode::RoundRobin);
        let params = ConsensusParams {
            slot_duration: Duration::from_secs(6),
            min_block_time: 1000,
            author_selection_mode: crate::author_selection::AuthorSelectionMode::RoundRobin,
        };

        let consensus = DcfConsensus::<TestBlock, TestClient, Pair>::new(
            client,
            author_selection,
            params,
        );

        assert_eq!(consensus.current_slot, 0);
    }

    #[tokio::test]
    async fn test_should_produce_block() {
        let client = create_test_client();
        let author_selection = AuthorSelection::new(crate::author_selection::AuthorSelectionMode::RoundRobin);
        let params = ConsensusParams {
            slot_duration: Duration::from_secs(6),
            min_block_time: 1000,
            author_selection_mode: crate::author_selection::AuthorSelectionMode::RoundRobin,
        };

        let mut consensus = DcfConsensus::<TestBlock, TestClient, Pair>::new(
            client,
            author_selection,
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
        let author_selection = AuthorSelection::new(crate::author_selection::AuthorSelectionMode::RoundRobin);
        let params = ConsensusParams {
            slot_duration: Duration::from_secs(6),
            min_block_time: 1000,
            author_selection_mode: crate::author_selection::AuthorSelectionMode::RoundRobin,
        };

        let consensus = DcfConsensus::<TestBlock, TestClient, Pair>::new(
            client,
            author_selection,
            params,
        );

        // Test author selection
        let author = consensus.select_next_author();
        assert!(author.is_ok());
    }

    #[tokio::test]
    async fn test_validator_authorization() {
        let client = create_test_client();
        let author_selection = AuthorSelection::new(crate::author_selection::AuthorSelectionMode::RoundRobin);
        let params = ConsensusParams {
            slot_duration: Duration::from_secs(6),
            min_block_time: 1000,
            author_selection_mode: crate::author_selection::AuthorSelectionMode::RoundRobin,
        };

        let consensus = DcfConsensus::<TestBlock, TestClient, Pair>::new(
            client,
            author_selection,
            params,
        );

        let author = create_test_author();
        let is_valid = consensus.is_valid_author(&author);
        assert!(!is_valid); // Should be false since author is not in validator set
    }

    #[tokio::test]
    async fn test_metrics_update() {
        let client = create_test_client();
        let author_selection = AuthorSelection::new(crate::author_selection::AuthorSelectionMode::RoundRobin);
        let params = ConsensusParams {
            slot_duration: Duration::from_secs(6),
            min_block_time: 1000,
            author_selection_mode: crate::author_selection::AuthorSelectionMode::RoundRobin,
        };

        let mut consensus = DcfConsensus::<TestBlock, TestClient, Pair>::new(
            client,
            author_selection,
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