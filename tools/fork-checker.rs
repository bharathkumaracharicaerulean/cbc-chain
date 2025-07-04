use std::time::Duration;
use clap::Parser;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::rpc_params;
use jsonrpsee::ws_client::WsClientBuilder;
use anyhow::{Result, Context};

#[derive(Parser, Debug)]
#[command(author, version, about = "CBC Fork Detection Utility", long_about = None)]
struct Args {
    /// WebSocket URL of the local node
    #[arg(short, long, default_value = "ws://127.0.0.1:9944")]
    local: String,

    /// WebSocket URL of the peer node
    #[arg(long)]
    peer: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Connect to local node
    let local_client = WsClientBuilder::default()
        .connection_timeout(Duration::from_secs(10))
        .build(&args.local)
        .await
        .context("Failed to connect to local node")?;

    // Connect to peer node
    let peer_client = WsClientBuilder::default()
        .connection_timeout(Duration::from_secs(10))
        .build(&args.peer)
        .await
        .context("Failed to connect to peer node")?;

    // Fetch finalized and current block numbers
    let (local_finalized, local_current) = get_block_numbers(&local_client).await?;
    let (peer_finalized, peer_current) = get_block_numbers(&peer_client).await?;

    println!("Local:     finalized #{}, current #{}", local_finalized, local_current);
    println!("Peer:      finalized #{}, current #{}", peer_finalized, peer_current);

    let diff = (local_finalized as i64 - peer_finalized as i64).abs();
    if diff > 2 {
        println!("\x1b[31mWARNING: Finalized block difference is {} (> 2)! Possible fork or divergence detected.\x1b[0m", diff);
    } else {
        println!("\x1b[32mFinalized block difference is {} (OK)\x1b[0m", diff);
    }

    Ok(())
}

async fn get_block_numbers(client: &jsonrpsee::ws_client::WsClient) -> Result<(u32, u32)> {
    // Get finalized head
    let finalized_hash: String = client.request("chain_getFinalizedHead", rpc_params![]).await?;
    let finalized_header: serde_json::Value = client.request("chain_getHeader", rpc_params![finalized_hash.clone()]).await?;
    let finalized_number = finalized_header["number"].as_str().unwrap_or("0");
    let finalized_number = u32::from_str_radix(finalized_number.trim_start_matches("0x"), 16).unwrap_or(0);

    // Get best (current) head
    let best_header: serde_json::Value = client.request("chain_getHeader", rpc_params![]).await?;
    let best_number = best_header["number"].as_str().unwrap_or("0");
    let best_number = u32::from_str_radix(best_number.trim_start_matches("0x"), 16).unwrap_or(0);

    Ok((finalized_number, best_number))
} 