//! PumpSwap subscription with metrics enabled.
//!
//! Usage: cargo run --example pumpswap_with_metrics --release

use solana_streamer_sdk::streaming::event_parser::protocols::pumpswap::parser::PUMPSWAP_PROGRAM_ID;
use solana_streamer_sdk::streaming::event_parser::{
    common::filter::EventTypeFilter, common::EventType, DexEvent, Protocol,
};
use solana_streamer_sdk::streaming::grpc::ClientConfig;
use solana_streamer_sdk::streaming::yellowstone_grpc::{TransactionFilter, YellowstoneGrpc};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
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
    let event_count = Arc::new(AtomicU64::new(0));
    let callback_count = event_count.clone();
    let callback = move |_event: DexEvent| {
        callback_count.fetch_add(1, Ordering::Relaxed);
    };
    let event_filter =
        EventTypeFilter::include_only(vec![EventType::PumpSwapBuy, EventType::PumpSwapSell]);

    grpc.subscribe_events_immediate(
        vec![Protocol::PumpSwap],
        None,
        vec![transaction_filter],
        vec![],
        Some(event_filter),
        None,
        callback,
    )
    .await?;

    println!("Press Ctrl+C to stop...\n");
    tokio::signal::ctrl_c().await?;
    grpc.stop().await;
    println!("Processed {} PumpSwap trade events", event_count.load(Ordering::Relaxed));
    Ok(())
}
