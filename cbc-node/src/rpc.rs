//! A collection of node-specific RPC methods.
//!
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! runtime-specific capabilities for the CBC chain.

#![warn(missing_docs)] // Emit a warning if any public item is missing Rust doc comments.

use jsonrpsee_core::server::{RpcModule, Middleware, NoopMiddleware};
use jsonrpsee_http_server::server::{HttpServerBuilder, AccessControlAllowOrigin};

use sc_rpc_api::DenyUnsafe;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};
use std::sync::Arc;
use cbc_runtime::{opaque::Block, apis::RuntimeApi};
use sc_transaction_pool_api::TransactionPool;
use substrate_frame_rpc_system::{System, SystemApi};

/// Full client dependencies for setting up RPC extensions.
///
/// This structure groups all dependencies needed to extend the JSON-RPC server
/// with runtime-specific APIs (like account nonces or transaction fees).
pub struct FullDeps<C, P> {
	/// Shared reference to the full Substrate client.
	pub client: Arc<C>,
	pub deny_unsafe: DenyUnsafe,
}

/// Creates a complete RPC module with all CBC-specific runtime extensions.
/// This will be called when the full node is started to build the JSON-RPC interface.
///
/// # Type Parameters:
/// - `C`: The type of the client (must implement runtime API access and block metadata)
/// - `P`: The transaction pool type (must implement basic transaction pool operations)
pub fn create_full<C, P>(
	deps: FullDeps<C, P>, // Struct containing dependencies (client + transaction pool)
) -> jsonrpsee_core::IoHandler<sc_rpc::Metadata> where
	C: sp_api::ProvideRuntimeApi<Block> + sp_blockchain::HeaderBackend<Block> + Send + Sync + 'static,
	C::Api: RuntimeApi<Block>,
	P: TransactionPool + 'static,
{
	let mut io = IoHandler::default();
	let FullDeps {
		client,
		deny_unsafe,
	} = deps;

	io.extend_with(
		SystemApi::to_delegate(System::new(client.clone(), deny_unsafe))
	);

	io
}

pub fn start_http(
	addr: std::net::SocketAddr,
	cors: Option<Vec<String>>,
	io: IoHandler<Metadata>,
) -> std::io::Result<HttpServerBuilder> {
	let middleware = NoopMiddleware;
	let cors = cors.map(|cors| {
		let mut cors = cors.into_iter()
			.map(|origin| AccessControlAllowOrigin::Value(origin.parse().unwrap()))
			.collect::<Vec<_>>();
		cors.push(AccessControlAllowOrigin::Null);
		cors
	});

	HttpServerBuilder::new(io, middleware, cors)
		.threads(4)
		.start_http(&addr)
}
