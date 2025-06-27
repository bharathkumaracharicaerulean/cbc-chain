//! A collection of node-specific RPC methods.
//!
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! runtime-specific capabilities for the CBC chain.

#![warn(missing_docs)] // Emit a warning if any public item is missing Rust doc comments.

use jsonrpsee_core::server::{RpcModule};
use jsonrpsee_http_server::{HttpServerBuilder};

use sc_rpc_api::DenyUnsafe;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};
use std::sync::Arc;
use cbc_runtime::opaque::Block;
use pallet_cbc_dcf::DcfApi as RuntimeDcfApi;

/// Full client dependencies for setting up RPC extensions.
///
/// This structure groups all dependencies needed to extend the JSON-RPC server
/// with runtime-specific APIs (like account nonces or transaction fees).
pub struct FullDeps<C> {
	/// Shared reference to the full Substrate client.
	pub client: Arc<C>,
	pub deny_unsafe: DenyUnsafe,
}

/// Creates a complete RPC module with all CBC-specific runtime extensions.
/// This will be called when the full node is started to build the JSON-RPC interface.
///
/// # Type Parameters:
/// - `C`: The type of the client (must implement runtime API access and block metadata)
pub fn create_full<C>(
	deps: FullDeps<C>,
) -> RpcModule<()>
where
	C: sp_api::ProvideRuntimeApi<Block> + sp_blockchain::HeaderBackend<Block> + Send + Sync + 'static,
	C::Api: RuntimeDcfApi<Block, cbc_runtime::AccountId>,
{
	let FullDeps {
		client,
		deny_unsafe,
	} = deps;
	let mut module = RpcModule::new(());

	// Example: Add system RPCs (you may need to adapt this for jsonrpsee)
	// module.merge(SystemApi::to_delegate(System::new(client.clone(), deny_unsafe))).unwrap();

	module
}

// Example async HTTP server starter for jsonrpsee
pub async fn start_http(
	addr: std::net::SocketAddr,
	_cors: Option<Vec<String>>,
	module: RpcModule<()>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
	let builder = HttpServerBuilder::default();
	let server = builder.build(addr).await?;
	server.start(module)?.await;
	Ok(())
}
