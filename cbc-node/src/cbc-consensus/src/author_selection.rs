//! Author selection implementation for the consensus engine

use crate::error::{ConsensusError, Result};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::sr25519::Public;
use cbc_runtime::AccountId;
use pallet_cbc_dcf::DcfApi as RuntimeDcfApi;
use std::sync::Arc;

/// Fetch the expected block author for a given block number/slot using the DCF runtime API.
pub fn get_expected_author<B, C>(client: Arc<C>, block_number: u32) -> Result<Public>
where
    B: sp_runtime::traits::Block,
    C: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    C::Api: RuntimeDcfApi<B, AccountId>,
{
    let api = client.runtime_api();
    let best_hash = client.info().best_hash;
    match api.get_expected_author(best_hash, block_number) {
        Ok(Some(account_id)) => Ok(Public::from_raw(*account_id.as_ref())),
        Ok(None) => Err(ConsensusError::AuthorSelection("No expected author returned by runtime".into())),
        Err(e) => Err(ConsensusError::AuthorSelection(format!("Runtime API error: {:?}", e))),
    }
}