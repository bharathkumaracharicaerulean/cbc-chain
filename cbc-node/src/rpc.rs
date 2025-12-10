#![allow(dead_code, missing_docs)]

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
use pallet_cbc_pos::PosApi;
use cbc_runtime::pallet_cbc_poi::PoiApi;

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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ValidatorStatus {
    Active,
    Inactive,
    Slashed,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InferenceResult {
    pub result: u32,
    pub confidence: u32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeWindow {
    pub start_block: u32,
    pub end_block: u32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InferenceStatus {
    Pending,
    Submitted,
    Challenged,
    Verified,
}

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

pub struct FullDeps<C, P> {
    pub client: Arc<C>,
    pub pool: Arc<P>,
    pub rpc_config: RpcSecurityConfig,
}

#[rpc(server)]
pub trait ChainApi {
    #[method(name = "chain_getChainName")]
    fn get_chain_name(&self) -> RpcResult<String>;
}

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
}

// PoS RPC API
#[rpc(server)]
pub trait PosRpcApi {
    #[method(name = "pos_getValidatorScore")]
    fn get_validator_score(&self, validator: AccountId) -> RpcResult<u32>;
    
    #[method(name = "pos_getValidatorStake")]
    fn get_validator_stake(&self, validator: AccountId) -> RpcResult<Balance>;
    
    #[method(name = "pos_getSlashingCount")]
    fn get_slashing_count(&self, validator: AccountId) -> RpcResult<u32>;
    
    #[method(name = "pos_getValidatorStatus")]
    fn get_validator_status(&self, validator: AccountId) -> RpcResult<ValidatorStatus>;
}

pub struct PosRpcApiImpl<C> {
    client: Arc<C>,
}

impl<C> PosRpcApiImpl<C> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }
}

impl<C> PosRpcApiServer for PosRpcApiImpl<C>
where
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    C::Api: pallet_cbc_pos::PosApi<Block, AccountId, Balance>,
{
    fn get_validator_score(&self, validator: AccountId) -> RpcResult<u32> {
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        
        api.get_validator_score(best_hash, validator)
            .map_err(|e| jsonrpsee::types::ErrorObjectOwned::owned(
                -32000,
                format!("Runtime API call failed: {:?}", e),
                None::<()>
            ))
    }
    
    fn get_validator_stake(&self, validator: AccountId) -> RpcResult<Balance> {
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        
        api.get_validator_stake(best_hash, validator)
            .map_err(|e| jsonrpsee::types::ErrorObjectOwned::owned(
                -32000,
                format!("Runtime API call failed: {:?}", e),
                None::<()>
            ))
    }
    
    fn get_slashing_count(&self, validator: AccountId) -> RpcResult<u32> {
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        
        api.get_slashing_count(best_hash, validator)
            .map_err(|e| jsonrpsee::types::ErrorObjectOwned::owned(
                -32000,
                format!("Runtime API call failed: {:?}", e),
                None::<()>
            ))
    }
    
    fn get_validator_status(&self, validator: AccountId) -> RpcResult<ValidatorStatus> {
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        
        // Get active validators list
        let active_validators = api.get_active_validators(best_hash)
            .map_err(|e| jsonrpsee::types::ErrorObjectOwned::owned(
                -32000,
                format!("Runtime API call failed: {:?}", e),
                None::<()>
            ))?;
        
        // Check if validator is in active list
        if !active_validators.contains(&validator) {
            return Ok(ValidatorStatus::Inactive);
        }
        
        // Check slashing count to determine if slashed
        let slashing_count = api.get_slashing_count(best_hash, validator.clone())
            .map_err(|e| jsonrpsee::types::ErrorObjectOwned::owned(
                -32000,
                format!("Runtime API call failed: {:?}", e),
                None::<()>
            ))?;
        
        if slashing_count > 0 {
            Ok(ValidatorStatus::Slashed)
        } else {
            Ok(ValidatorStatus::Active)
        }
    }
}

// PoI RPC API
#[rpc(server)]
pub trait PoiRpcApi {
    #[method(name = "poi_getInferenceResult")]
    fn get_inference_result(&self, validator: AccountId) -> RpcResult<Option<InferenceResult>>;
    
    #[method(name = "poi_getInferenceConfidence")]
    fn get_inference_confidence(&self, validator: AccountId) -> RpcResult<Option<u32>>;
    
    #[method(name = "poi_getChallengeWindow")]
    fn get_challenge_window(&self) -> RpcResult<ChallengeWindow>;
    
    #[method(name = "poi_getInferenceStatus")]
    fn get_inference_status(&self, validator: AccountId) -> RpcResult<InferenceStatus>;
}

pub struct PoiRpcApiImpl<C> {
    client: Arc<C>,
}

impl<C> PoiRpcApiImpl<C> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }
}

impl<C> PoiRpcApiServer for PoiRpcApiImpl<C>
where
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    C::Api: cbc_runtime::pallet_cbc_poi::PoiApi<Block, AccountId>,
{
    fn get_inference_result(&self, validator: AccountId) -> RpcResult<Option<InferenceResult>> {
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        
        match api.get_inference_result(best_hash, validator) {
            Ok(Some((result, confidence))) => Ok(Some(InferenceResult { result, confidence })),
            Ok(None) => Ok(None),
            Err(e) => Err(jsonrpsee::types::ErrorObjectOwned::owned(
                -32000,
                format!("Runtime API call failed: {:?}", e),
                None::<()>
            ))
        }
    }
    
    fn get_inference_confidence(&self, validator: AccountId) -> RpcResult<Option<u32>> {
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        
        match api.get_inference_result(best_hash, validator) {
            Ok(Some((_result, confidence))) => Ok(Some(confidence)),
            Ok(None) => Ok(None),
            Err(e) => Err(jsonrpsee::types::ErrorObjectOwned::owned(
                -32000,
                format!("Runtime API call failed: {:?}", e),
                None::<()>
            ))
        }
    }
    
    fn get_challenge_window(&self) -> RpcResult<ChallengeWindow> {
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        
        // Get current epoch from PoI API
        let current_epoch = api.get_current_epoch(best_hash)
            .map_err(|e| jsonrpsee::types::ErrorObjectOwned::owned(
                -32000,
                format!("Runtime API call failed: {:?}", e),
                None::<()>
            ))?;
        
        // Calculate challenge window based on current epoch
        // Assuming each epoch is 100 blocks and challenge window is 50 blocks
        let start_block = current_epoch * 100;
        let end_block = start_block + 50;
        
        Ok(ChallengeWindow { start_block, end_block })
    }
    
    fn get_inference_status(&self, validator: AccountId) -> RpcResult<InferenceStatus> {
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        
        // Check if validator has an inference result
        let has_inference = api.get_inference_result(best_hash, validator.clone())
            .map_err(|e| jsonrpsee::types::ErrorObjectOwned::owned(
                -32000,
                format!("Runtime API call failed: {:?}", e),
                None::<()>
            ))?;
        
        // Check if validator has a challenge
        let has_challenge = api.get_challenge(best_hash, validator)
            .map_err(|e| jsonrpsee::types::ErrorObjectOwned::owned(
                -32000,
                format!("Runtime API call failed: {:?}", e),
                None::<()>
            ))?;
        
        match (has_inference, has_challenge) {
            (None, None) => Ok(InferenceStatus::Pending),
            (Some(_), None) => Ok(InferenceStatus::Submitted),
            (Some(_), Some(_)) => Ok(InferenceStatus::Challenged),
            (None, Some(_)) => Ok(InferenceStatus::Verified), // Challenge resolved
        }
    }
}

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
    C::Api: pallet_cbc_pos::PosApi<Block, AccountId, Balance>,
    C::Api: cbc_runtime::pallet_cbc_poi::PoiApi<Block, AccountId>,
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

    module.merge(System::new(client.clone(), pool).into_rpc())?;
    module.merge(TransactionPayment::new(client.clone()).into_rpc())?;

    if rpc_config.enable_cbc_extensions {
        let chain_api = ChainApiImpl { client: client.clone() };
        module.merge(ChainApiServer::into_rpc(chain_api))?;
        
        // Register PoS RPC handler
        let pos_api = PosRpcApiImpl::new(client.clone());
        module.merge(PosRpcApiServer::into_rpc(pos_api))?;
        
        // Register PoI RPC handler
        let poi_api = PoiRpcApiImpl::new(client.clone());
        module.merge(PoiRpcApiServer::into_rpc(poi_api))?;
    }

    if rpc_config.expose_unsafe_methods {
    }

    Ok(module)
}