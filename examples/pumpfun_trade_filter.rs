//! PumpFun trade event filter: subscribe to Buy/Sell/Create and print details with latency.
//!
//! Usage: cargo run --example pumpfun_trade_filter --release

use solana_streamer_sdk::streaming::event_parser::common::filter::EventTypeFilter;
use solana_streamer_sdk::streaming::event_parser::common::types::EventType;
use solana_streamer_sdk::streaming::event_parser::protocols::pumpfun::events::{
    PumpFunCreateTokenEvent, PumpFunCreateV2TokenEvent, PumpFunTradeEvent,
};
use solana_streamer_sdk::streaming::event_parser::protocols::pumpfun::parser::PUMPFUN_PROGRAM_ID;
use solana_streamer_sdk::streaming::event_parser::DexEvent;
use solana_streamer_sdk::streaming::event_parser::Protocol;
use solana_streamer_sdk::streaming::grpc::ClientConfig;
use solana_streamer_sdk::streaming::yellowstone_grpc::{
    AccountFilter, TransactionFilter, YellowstoneGrpc,
};

fn now_micros() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros() as i64
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = rustls::crypto::ring::default_provider().install_default();

    println!("🚀 PumpFun Trade Event Filter (solana-streamer)\n");

    let mut config = ClientConfig::default();
    config.enable_metrics = true;

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
    let event_filter = Some(EventTypeFilter {
        include: vec![
            EventType::PumpFunBuy,
            EventType::PumpFunSell,
            EventType::PumpFunCreateToken,
            EventType::PumpFunCreateV2Token,
        ],
        ..Default::default()
    });

    let callback = |event: DexEvent| {
        let now_us = now_micros();
        match &event {
            DexEvent::PumpFunTradeEvent(e) => print_trade(e, now_us),
            DexEvent::PumpFunCreateTokenEvent(e) => print_create_legacy(e, now_us),
            DexEvent::PumpFunCreateV2TokenEvent(e) => print_create_v2(e, now_us),
            _ => {}
        }
    };

    grpc.subscribe_events_immediate(
        vec![Protocol::PumpFun],
        None,
        vec![transaction_filter],
        vec![account_filter],
        event_filter,
        None,
        callback,
    )
    .await?;

    println!("🛑 Press Ctrl+C to stop...\n");
    tokio::signal::ctrl_c().await?;
    grpc.stop().await;
    Ok(())
}

fn print_trade(e: &PumpFunTradeEvent, now_us: i64) {
    let latency_us = now_us - e.metadata.recv_us;
    let kind = if e.is_buy { "BUY" } else { "SELL" };
    println!(
        "│ {} | sig={:.8}.. | mint={:.8}.. | sol={} tok={} | user={:.8}.. | latency={} μs",
        kind, e.metadata.signature, e.mint, e.sol_amount, e.token_amount, e.user, latency_us
    );
}

fn print_create_legacy(e: &PumpFunCreateTokenEvent, now_us: i64) {
    let latency_us = now_us - e.metadata.recv_us;
    println!(
        "│ CREATE | sig={:.8}.. | name={} symbol={} | mint={:.8}.. | latency={} μs",
        e.metadata.signature, e.name, e.symbol, e.mint, latency_us
    );
}

fn print_create_v2(e: &PumpFunCreateV2TokenEvent, now_us: i64) {
    let latency_us = now_us - e.metadata.recv_us;
    println!(
        "│ CREATE_V2 | sig={:.8}.. | name={} symbol={} | mint={:.8}.. | latency={} μs",
        e.metadata.signature, e.name, e.symbol, e.mint, latency_us
    );
}
