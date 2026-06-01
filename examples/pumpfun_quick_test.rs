//! Quick test: subscribe to PumpFun events and print the first 10 (or run 60s).
//!
//! Usage: cargo run --example pumpfun_quick_test --release

use solana_streamer_sdk::streaming::event_parser::common::types::EventType;
use solana_streamer_sdk::streaming::event_parser::protocols::pumpfun::parser::PUMPFUN_PROGRAM_ID;
use solana_streamer_sdk::streaming::event_parser::DexEvent;
use solana_streamer_sdk::streaming::event_parser::Protocol;
use solana_streamer_sdk::streaming::grpc::ClientConfig;
use solana_streamer_sdk::streaming::yellowstone_grpc::{
    AccountFilter, TransactionFilter, YellowstoneGrpc,
};
use std::sync::atomic::{AtomicU64, Ordering};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = rustls::crypto::ring::default_provider().install_default();

    println!("🚀 Quick Test - Subscribing to PumpFun events...");

    // Metrics add overhead; enable explicitly with STREAMER_ENABLE_METRICS=1.
    let config = ClientConfig {
        enable_metrics: std::env::var("STREAMER_ENABLE_METRICS").as_deref() == Ok("1"),
        ..Default::default()
    };

    let grpc = YellowstoneGrpc::new_with_config(
        std::env::var("GRPC_ENDPOINT")
            .unwrap_or_else(|_| "https://solana-yellowstone-grpc.publicnode.com:443".to_string()),
        std::env::var("GRPC_AUTH_TOKEN").ok(),
        config,
    )?;

    let transaction_filter = TransactionFilter {
        account_include: vec![PUMPFUN_PROGRAM_ID.to_string()],
        account_exclude: vec![],
        account_required: vec![],
    };
    let account_filter = AccountFilter {
        account: vec![],
        owner: vec![PUMPFUN_PROGRAM_ID.to_string()],
        filters: vec![],
    };

    let event_count = std::sync::Arc::new(AtomicU64::new(0));
    let count = event_count.clone();
    let callback = move |event: DexEvent| {
        let n = count.fetch_add(1, Ordering::Relaxed);
        let event_type = match event.metadata().event_type {
            EventType::PumpFunCreateToken | EventType::PumpFunCreateV2Token => "PumpFunCreate",
            EventType::PumpFunBuy => "PumpFunBuy",
            EventType::PumpFunSell => "PumpFunSell",
            EventType::PumpFunMigrate => "PumpFunMigrate",
            _ => "Other",
        };
        println!("✅ Event #{}: {}", n + 1, event_type);
    };

    grpc.subscribe_events_immediate(
        vec![Protocol::PumpFun],
        None,
        vec![transaction_filter],
        vec![account_filter],
        None,
        None,
        callback,
    )
    .await?;

    println!("🎧 Listening (up to 60s or Ctrl+C)...\n");
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    grpc.stop().await;
    println!("✓ Stopped.");
    Ok(())
}
