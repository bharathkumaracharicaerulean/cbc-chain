use std::time::Duration;
use serde::{Serialize, Deserialize};
use clap::Parser;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::rpc_params;
use jsonrpsee::ws_client::WsClientBuilder;
use anyhow::{Result, Context};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// WebSocket URL of the node
    #[arg(short, long, default_value = "ws://127.0.0.1:9944")]
    url: String,

    /// Output format (table, json, csv)
    #[arg(short, long, default_value = "table")]
    format: String,

    /// Export file path (optional)
    #[arg(short, long)]
    export: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ValidatorScore {
    pos_score: u64,
    poi_score: Option<u64>,
    combined_score: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ValidatorReport {
    validator_id: String,
    pos_score: u64,
    poi_score: Option<u64>,
    combined_score: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Connect to the node
    let client = WsClientBuilder::default()
        .connection_timeout(Duration::from_secs(10))
        .build(&args.url)
        .await
        .context("Failed to connect to node")?;

    // Get all validators
    let validators = get_validators(&client).await?;
    
    // Collect scores for each validator
    let mut report = Vec::new();
    for validator in validators {
        let pos_score = get_pos_score(&client, &validator).await?;
        let poi_score = get_poi_score(&client, &validator).await.ok();
        let combined_score = calculate_combined_score(pos_score, poi_score);
        
        report.push(ValidatorReport {
            validator_id: validator,
            pos_score,
            poi_score,
            combined_score,
        });
    }

    // Sort by combined score
    report.sort_by(|a, b| b.combined_score.cmp(&a.combined_score));

    // Print or export report
    match args.format.as_str() {
        "json" => print_json(&report, args.export)?,
        "csv" => print_csv(&report, args.export)?,
        _ => print_table(&report),
    }

    Ok(())
}

async fn get_validators(client: &jsonrpsee::ws_client::WsClient) -> Result<Vec<String>> {
    // Get validators from DCF pallet storage
    let params = rpc_params!["Dcf", "ActiveValidators", None::<()>];
    let response: Vec<String> = client.request("state_getStorage", params).await?;
    Ok(response)
}

async fn get_pos_score(client: &jsonrpsee::ws_client::WsClient, validator: &str) -> Result<u64> {
    // Get PoS score from storage
    let params = rpc_params!["Dcf", "ValidatorStakeScores", validator];
    let response: Option<u64> = client.request("state_getStorage", params).await?;
    Ok(response.unwrap_or(0))
}

async fn get_poi_score(client: &jsonrpsee::ws_client::WsClient, validator: &str) -> Result<u64> {
    // Get PoI score from storage
    let params = rpc_params!["Dcf", "ValidatorInferenceScores", validator];
    let response: Option<u64> = client.request("state_getStorage", params).await?;
    Ok(response.unwrap_or(0))
}

fn calculate_combined_score(pos_score: u64, poi_score: Option<u64>) -> u64 {
    // Simple additive scoring for now
    pos_score + poi_score.unwrap_or(0)
}

fn print_table(report: &[ValidatorReport]) {
    println!("Validator Scores Report");
    println!("======================");
    println!("{:<50} {:<10} {:<10} {:<10}", "Validator ID", "PoS Score", "PoI Score", "Combined");
    println!("{:-<50} {:-<10} {:-<10} {:-<10}", "", "", "", "");
    
    for entry in report {
        println!(
            "{:<50} {:<10} {:<10} {:<10}",
            entry.validator_id,
            entry.pos_score,
            entry.poi_score.unwrap_or(0),
            entry.combined_score
        );
    }
}

fn print_json(report: &[ValidatorReport], export_path: Option<String>) -> Result<()> {
    let json = serde_json::to_string_pretty(report)?;
    if let Some(path) = export_path {
        std::fs::write(path, json)?;
    } else {
        println!("{}", json);
    }
    Ok(())
}

fn print_csv(report: &[ValidatorReport], export_path: Option<String>) -> Result<()> {
    let mut csv = String::from("Validator ID,PoS Score,PoI Score,Combined Score\n");
    for entry in report {
        csv.push_str(&format!(
            "{},{},{},{}\n",
            entry.validator_id,
            entry.pos_score,
            entry.poi_score.unwrap_or(0),
            entry.combined_score
        ));
    }
    if let Some(path) = export_path {
        std::fs::write(path, csv)?;
    } else {
        print!("{}", csv);
    }
    Ok(())
} 