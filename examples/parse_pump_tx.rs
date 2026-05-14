//! Parse a PumpFun transaction from RPC using the SDK-backed streamer RPC helper.
//!
//! Signature: env `TX_SIGNATURE` or first CLI arg. RPC: env `SOLANA_RPC_URL`.
//!
//!   cargo run --example parse_pump_tx --release
//!   TX_SIGNATURE=your_sig cargo run --example parse_pump_tx --release
//!   cargo run --example parse_pump_tx --release -- your_sig
//!   SOLANA_RPC_URL=https://... cargo run --example parse_pump_tx --release

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
        "64srGF8CnTz9zPbdayWYmzs5aVRFBcfjDcidFVvBgAD25VMh52wr88vma7ytSbAZT3C5Giu5BPyGfNfLexLSrKhP";
    let tx_sig = std::env::var("TX_SIGNATURE")
        .ok()
        .or_else(|| std::env::args().nth(1))
        .unwrap_or_else(|| default_sig.to_string());
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());

    println!("=== PumpFun RPC Parser (SDK-backed solana-streamer) ===\n");
    println!("Transaction: {}\nRPC: {}\n", tx_sig, rpc_url);

    let signature = Signature::from_str(&tx_sig)?;
    let rpc_client = solana_client::nonblocking::rpc_client::RpcClient::new(rpc_url);
    let protocols = [Protocol::PumpFun];

    // No event filter: surface every PumpFun event the SDK can parse for this transaction.
    let events = fetch_rpc_transaction_as_streamer_events_async(
        &rpc_client,
        &signature,
        now_micros(),
        &protocols,
        None,
    )
    .await
    .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    println!("Parsed {} PumpFun event(s):\n", events.len());
    for event in events {
        println!("{:?}\n", event);
    }

    Ok(())
}
