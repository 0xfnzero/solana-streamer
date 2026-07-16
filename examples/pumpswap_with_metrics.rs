//! PumpSwap subscription with metrics enabled.
//!
//! Usage: cargo run --example pumpswap_with_metrics --release

use solana_streamer_sdk::streaming::event_parser::protocols::pumpswap::parser::PUMPSWAP_PROGRAM_ID;
use solana_streamer_sdk::streaming::event_parser::{DexEvent, Protocol};
use solana_streamer_sdk::streaming::grpc::ClientConfig;
use solana_streamer_sdk::streaming::yellowstone_grpc::{
    AccountFilter, TransactionFilter, YellowstoneGrpc,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = rustls::crypto::ring::default_provider().install_default();

    println!("PumpSwap with metrics (solana-streamer)\n");

    let config = ClientConfig { enable_metrics: true, ..Default::default() };

    let grpc = YellowstoneGrpc::new_with_config(
        std::env::var("GRPC_ENDPOINT")
            .unwrap_or_else(|_| "https://solana-yellowstone-grpc.publicnode.com:443".to_string()),
        std::env::var("GRPC_AUTH_TOKEN").ok(),
        config,
    )?;

    let transaction_filter = TransactionFilter {
        account_include: vec![PUMPSWAP_PROGRAM_ID.to_string()],
        account_exclude: vec![],
        account_required: vec![],
    };
    let account_filter = AccountFilter {
        account: vec![],
        owner: vec![PUMPSWAP_PROGRAM_ID.to_string()],
        filters: vec![],
    };

    let callback = |event: DexEvent| {
        println!("Event: {:?}", event.metadata().event_type);
    };

    grpc.subscribe_events_immediate(
        vec![Protocol::PumpSwap],
        None,
        vec![transaction_filter],
        vec![account_filter],
        None,
        None,
        callback,
    )
    .await?;

    println!("Press Ctrl+C to stop...\n");
    tokio::signal::ctrl_c().await?;
    grpc.stop().await;
    Ok(())
}
