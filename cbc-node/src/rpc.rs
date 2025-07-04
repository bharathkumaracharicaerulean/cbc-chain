//! A collection of node-specific RPC methods for CBC-Chain with DCF consensus only.
//!
//! This file extends the Substrate RPC layer with runtime-specific capabilities for the CBC chain.
//! It is fully compatible with the DCF-only node and runtime.

#![warn(missing_docs)]

use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use jsonrpsee::RpcModule;
use sc_transaction_pool_api::TransactionPool;
use cbc_runtime::{opaque::Block, AccountId, Balance, Nonce};
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use pallet_cbc_dcf::DcfApi;
use jsonrpsee::core::error as RpcError;


/// Simple rate limiter implementation
#[derive(Clone)]
pub struct RateLimiter {
    window: Duration,
    max_requests: u32,
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
}

impl RateLimiter {
    pub fn new(window_secs: u64, max_requests: u32) -> Self {
        Self {
            window: Duration::from_secs(window_secs),
            max_requests,
            requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn check_rate_limit(&self, ip: &str) -> bool {
        let now = Instant::now();
        let mut requests = self.requests.lock().unwrap();
        let history = requests.entry(ip.to_string()).or_insert_with(Vec::new);
        history.retain(|&time| now.duration_since(time) <= self.window);
        if history.len() as u32 >= self.max_requests {
            return false;
        }
        history.push(now);
        true
    }
}

/// Configuration for RPC security settings
#[derive(Clone, Debug)]
pub struct RpcSecurityConfig {
    pub enable_cbc_extensions: bool,
    pub expose_unsafe_methods: bool,
    pub rate_limit_window: u64,
    pub rate_limit_requests: u32,
}

impl Default for RpcSecurityConfig {
    fn default() -> Self {
        Self {
            enable_cbc_extensions: false,
            expose_unsafe_methods: false,
            rate_limit_window: 60,
            rate_limit_requests: 100,
        }
    }
}

/// Full client dependencies for setting up RPC extensions.
pub struct FullDeps<C, P> {
    pub client: Arc<C>,
    pub pool: Arc<P>,
    pub rpc_config: RpcSecurityConfig,
}

/// Custom RPC trait for CBC node (example: chain name).
#[rpc(server)]
pub trait ChainApi {
    #[method(name = "chain_getChainName")]
    fn get_chain_name(&self) -> RpcResult<String>;

    // Example DCF-specific RPC: get validator scores
    #[method(name = "dcf_getValidatorScores")]
    fn get_validator_scores(&self) -> RpcResult<Vec<(AccountId, u64)>>;
}

/// Implementation of the CustomApi trait.
pub struct ChainApiImpl<C: ProvideRuntimeApi<Block> + Send + Sync + 'static> {
    pub client: Arc<C>,
}

impl<C> ChainApiServer for ChainApiImpl<C>
where
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
{
    fn get_chain_name(&self) -> RpcResult<String> {
        Ok("CBC-Chain".to_string())
    }

    fn get_validator_scores(&self) -> RpcResult<Vec<(AccountId, u64)>> {
        // Use the DCF runtime API to fetch validator scores
        let best_hash = self.client.info().best_hash;
        let api = self.client.runtime_api();
        match api.get_validator_scores(best_hash) {
            Ok(scores) => Ok(scores),
            Err(e) => Err(Custom(format!("Runtime API error: {:?}", e))),
        }
    }
}

/// Creates a complete RPC module with all CBC-specific runtime extensions.
pub fn create_full<C, P>(
    deps: FullDeps<C, P>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + Send + Sync + 'static,
    C: Send + Sync + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: BlockBuilder<Block>,
    P: TransactionPool + 'static,
{
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
    use substrate_frame_rpc_system::{System, SystemApiServer};

    let mut module = RpcModule::new(());
    let FullDeps { client, pool, rpc_config } = deps;

    let _rate_limiter = RateLimiter::new(
        rpc_config.rate_limit_window,
        rpc_config.rate_limit_requests,
    );

    // Always enable safe methods
    module.merge(System::new(client.clone(), pool).into_rpc())?;
    module.merge(TransactionPayment::new(client.clone()).into_rpc())?;

    // Enable CBC custom RPCs if configured
    if rpc_config.enable_cbc_extensions {
        let chain_api = ChainApiImpl { client: client.clone() };
        module.merge(ChainApiServer::into_rpc(chain_api))?;
    }

    // Only expose unsafe methods if configured
    if rpc_config.expose_unsafe_methods {
        // Add any unsafe methods here
    }

    Ok(module)
}
