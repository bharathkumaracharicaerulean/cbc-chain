<<<<<<< HEAD
//! A collection of node-specific RPC methods.
//!
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! runtime-specific capabilities for the CBC chain.

#![warn(missing_docs)] // Emit a warning if any public item is missing Rust doc comments.
=======
#![allow(dead_code, missing_docs)]
>>>>>>> c0e1c816d4065ea122aac1de1ee507dc1010eacc

use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

<<<<<<< HEAD
use jsonrpsee::RpcModule; // JSON-RPC server abstraction from jsonrpsee (used in Substrate v3+)
use sc_transaction_pool_api::TransactionPool; // Trait for interacting with the transaction pool
use cbc_runtime::{opaque::Block, AccountId, Balance, Nonce}; // Reuse CBC runtime types
use sp_api::ProvideRuntimeApi; // Trait that allows accessing runtime APIs from the client
use sp_block_builder::BlockBuilder; // Trait for building blocks
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata}; // Block metadata for blockchain access

/// Simple rate limiter implementation
#[derive(Clone)]
pub struct RateLimiter {
	/// Rate limiting window in seconds
	window: Duration,
	/// Maximum requests per window
	max_requests: u32,
	/// Request history for each IP
	requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
}

impl RateLimiter {
	/// Create a new rate limiter
	pub fn new(window_secs: u64, max_requests: u32) -> Self {
		Self {
			window: Duration::from_secs(window_secs),
			max_requests,
			requests: Arc::new(Mutex::new(HashMap::new())),
		}
	}

	/// Check if a request is allowed
	pub fn check_rate_limit(&self, ip: &str) -> bool {
		let now = Instant::now();
		let mut requests = self.requests.lock().unwrap();
		
		// Get or create request history for this IP
		let history = requests.entry(ip.to_string()).or_insert_with(Vec::new);
		
		// Remove old requests outside the window
		history.retain(|&time| now.duration_since(time) <= self.window);
		
		// Check if under limit
		if history.len() as u32 >= self.max_requests {
			return false;
		}
		
		// Add new request
		history.push(now);
		true
	}
}

/// Configuration for RPC security settings
#[derive(Clone, Debug)]
pub struct RpcSecurityConfig {
	/// Whether to enable CBC custom extensions
	pub enable_cbc_extensions: bool,
	/// Whether to expose unsafe RPC methods
	pub expose_unsafe_methods: bool,
	/// Rate limiting window in seconds
	pub rate_limit_window: u64,
	/// Maximum requests per window
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
///
/// This structure groups all dependencies needed to extend the JSON-RPC server
/// with runtime-specific APIs (like account nonces or transaction fees).
pub struct FullDeps<C, P> {
	/// Shared reference to the full Substrate client.
	pub client: Arc<C>,

	/// Shared reference to the transaction pool.
	pub pool: Arc<P>,

	/// RPC security configuration
	pub rpc_config: RpcSecurityConfig,
}
use jsonrpsee::core::{RpcResult};
use jsonrpsee::proc_macros::rpc;

/// Custom RPC trait for CBC node.
=======
use jsonrpsee::RpcModule;
use sc_transaction_pool_api::TransactionPool;
use cbc_runtime::{opaque::Block, AccountId, Balance, Nonce};
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;

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

>>>>>>> c0e1c816d4065ea122aac1de1ee507dc1010eacc
#[rpc(server)]
pub trait ChainApi {
    #[method(name = "chain_getChainName")]
    fn get_chain_name(&self) -> RpcResult<String>;
}

<<<<<<< HEAD

/// Implementation of the CustomApi trait.
pub struct ChainApiImpl;

impl ChainApiServer for ChainApiImpl {
=======
pub struct ChainApiImpl<C: ProvideRuntimeApi<Block> + Send + Sync + 'static> {
    pub client: Arc<C>,
}

impl<C> ChainApiServer for ChainApiImpl<C>
where
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
{
>>>>>>> c0e1c816d4065ea122aac1de1ee507dc1010eacc
    fn get_chain_name(&self) -> RpcResult<String> {
        Ok("CBC-Chain".to_string())
    }
}

<<<<<<< HEAD

/// Creates a complete RPC module with all CBC-specific runtime extensions.
/// This will be called when the full node is started to build the JSON-RPC interface.
///
/// # Type Parameters:
/// - `C`: The type of the client (must implement runtime API access and block metadata)
/// - `P`: The transaction pool type (must implement basic transaction pool operations)
pub fn create_full<C, P>(
	deps: FullDeps<C, P>, // Struct containing dependencies (client + transaction pool)
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>> // Returns a JSON-RPC module or an error
where
	C: ProvideRuntimeApi<Block>, // Client must provide access to runtime APIs
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static, // Client must allow block header access and metadata lookup
	C: Send + Sync + 'static, // Must be thread-safe and have a static lifetime
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>, // Runtime must support the AccountNonce API
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>, // Runtime must support TransactionPayment API
	C::Api: BlockBuilder<Block>, // Runtime must support block building (for dry-run/validation)
	P: TransactionPool + 'static, // Transaction pool must implement required trait and be thread-safe
{
	// Import traits required for JSON-RPC server creation.
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer}; // API to query fee information
	use substrate_frame_rpc_system::{System, SystemApiServer}; // System-level RPC (e.g. nonce, block hashes)

	let mut module = RpcModule::new(()); // Create a new empty JSON-RPC module
	let FullDeps { client, pool, rpc_config } = deps; // Destructure dependencies into local variables

	// Create rate limiter
	let _rate_limiter = RateLimiter::new(
		rpc_config.rate_limit_window,
		rpc_config.rate_limit_requests,
	);
	
	// Note: Rate limiting would typically be implemented at the transport layer
	// or using a reverse proxy like nginx for production deployments

	// Always enable safe methods
	module.merge(System::new(client.clone(), pool).into_rpc())?;
	module.merge(TransactionPayment::new(client.clone()).into_rpc())?;

	// Only enable CBC custom RPCs if configured
	if rpc_config.enable_cbc_extensions {
		let chain_api = ChainApiImpl;
		module.merge(ChainApiServer::into_rpc(chain_api))?;
	}

	// Only expose unsafe methods if configured
	if rpc_config.expose_unsafe_methods {
		// Add any unsafe methods here
		// For example: module.merge(UnsafeDebugApi::new(client.clone()).into_rpc())?;
	}

	//  curl -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1,"method":"chain_getChainName","params":[]}' http://localhost:9944
	
	Ok(module) // Return the composed module with all active RPCs
}
=======
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

    module.merge(System::new(client.clone(), pool).into_rpc())?;
    module.merge(TransactionPayment::new(client.clone()).into_rpc())?;

    if rpc_config.enable_cbc_extensions {
        let chain_api = ChainApiImpl { client: client.clone() };
        module.merge(ChainApiServer::into_rpc(chain_api))?;
    }

    if rpc_config.expose_unsafe_methods {
    }

    Ok(module)
}
>>>>>>> c0e1c816d4065ea122aac1de1ee507dc1010eacc
