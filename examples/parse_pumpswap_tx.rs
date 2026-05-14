//! Parse a PumpSwap transaction from RPC using the SDK-backed streamer RPC helper.
//!
//! Usage:
//!   cargo run --example parse_pumpswap_tx --release
//!   TX_SIGNATURE=<sig> SOLANA_RPC_URL=<url> cargo run --example parse_pumpswap_tx --release

use anyhow::Result;
use solana_sdk::signature::Signature;
use solana_streamer_sdk::fetch_rpc_transaction_as_streamer_events_async;
use solana_streamer_sdk::streaming::event_parser::Protocol;
use std::str::FromStr;

fn now_micros() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros() as i64
}

#[tokio::main]
async fn main() -> Result<()> {
    let default_sig =
        "3zsihbygW7hoKGtduAyDDFzp4E1eis8gaBzEzzNKr8ma39baffpFcphok9wHFgR3EauDe9vYYsVf4Puh5pZ6UJiS";
    let tx_sig = std::env::var("TX_SIGNATURE").unwrap_or_else(|_| default_sig.to_string());
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());

    println!("=== PumpSwap RPC Parser (SDK-backed solana-streamer) ===\n");
    println!("Transaction: {}\nRPC: {}\n", tx_sig, rpc_url);

    let signature = Signature::from_str(&tx_sig)?;
    let rpc_client = solana_client::nonblocking::rpc_client::RpcClient::new(rpc_url);
    let protocols = [Protocol::PumpSwap];

    // No event filter: surface every PumpSwap event the SDK can parse for this transaction.
    let events = fetch_rpc_transaction_as_streamer_events_async(
        &rpc_client,
        &signature,
        now_micros(),
        &protocols,
        None,
    )
    .await
    .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    println!("Parsed {} PumpSwap event(s):\n", events.len());
    for event in events {
        println!("{:?}\n", event);
    }

    Ok(())
}
