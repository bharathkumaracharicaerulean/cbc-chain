//! A collection of node-specific RPC methods.
//!
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! runtime-specific capabilities for the CBC chain.

#![warn(missing_docs)] // Emit a warning if any public item is missing Rust doc comments.

use std::sync::Arc;

use jsonrpsee::RpcModule; // JSON-RPC server abstraction from jsonrpsee (used in Substrate v3+)
use sc_transaction_pool_api::TransactionPool; // Trait for interacting with the transaction pool
use cbc_runtime::{opaque::Block, AccountId, Balance, Nonce}; // Reuse CBC runtime types
use sp_api::ProvideRuntimeApi; // Trait that allows accessing runtime APIs from the client
use sp_block_builder::BlockBuilder; // Trait for building blocks
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata}; // Block metadata for blockchain access

/// Full client dependencies for setting up RPC extensions.
///
/// This structure groups all dependencies needed to extend the JSON-RPC server
/// with runtime-specific APIs (like account nonces or transaction fees).
pub struct FullDeps<C, P> {
	/// Shared reference to the full Substrate client.
	pub client: Arc<C>,

	/// Shared reference to the transaction pool.
	pub pool: Arc<P>,
}

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
	let FullDeps { client, pool } = deps; // Destructure dependencies into local variables

	// Merge system-level runtime APIs into the module (account nonce, chain head, etc.)
	module.merge(System::new(client.clone(), pool).into_rpc())?;

	// Merge transaction payment APIs (used to estimate fees for extrinsics)
	module.merge(TransactionPayment::new(client).into_rpc())?;

	// === You can define and merge custom RPCs here ===
	// For example, if you want to expose a custom storage query or chain state logic:
	//
	// module.merge(YourCustomApi::new(client.clone()).into_rpc())?;
	//
	// Example:
	// `YourRpcStruct` should have access to a runtime client.
	// `YourRpcTrait` is your trait (defined with `#[jsonrpsee::rpc]`) that generates the server interface.

	// === Optional: Extend RPCs with chainSpec info ===
	// If needed, you can add a `/chain_spec` RPC endpoint using the commented template below:
	//
	// let chain_name = chain_spec.name().to_string(); // Get the human-readable chain name
	// let genesis_hash = client.block_hash(0).ok().flatten().expect("Genesis block exists; qed"); // Fetch the genesis hash
	// let properties = chain_spec.properties(); // Chain-specific metadata (token symbol, decimals, etc.)
	// module.merge(ChainSpec::new(chain_name, genesis_hash, properties).into_rpc())?;

	Ok(module) // Return the composed module with all active RPCs
}
