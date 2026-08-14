use sol_parser_sdk::instr::program_ids::METEORA_DLMM_PROGRAM_ID;
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::Signature;
use solana_streamer_sdk::fetch_rpc_transaction_as_streamer_events;
use solana_streamer_sdk::streaming::event_parser::common::filter::EventTypeFilter;
use solana_streamer_sdk::streaming::event_parser::common::EventType;
use solana_streamer_sdk::streaming::event_parser::{DexEvent, Protocol};
use std::str::FromStr;

fn run_mainnet_tests() -> bool {
    std::env::var("RUN_MAINNET_TESTS").as_deref() == Ok("1")
}

fn rpc_client() -> RpcClient {
    RpcClient::new(
        std::env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
    )
}

// These transactions were captured from current mainnet traffic on 2026-08-13.
// Run with: RUN_MAINNET_TESTS=1 SOLANA_RPC_URL=<optional archive RPC> cargo test --test current_mainnet_transactions

#[test]
fn current_meteora_dlmm_swap_passes_exact_filter() {
    if !run_mainnet_tests() {
        return;
    }
    const SIGNATURE: &str =
        "eEWaGsbRPoiD36Xf3epzSMmdtXX36va76b13YfsDV3ncsxQHBTjC68zZ8mbzFXTNWy3n3qKUAHjgHBconX4Gu1i";
    const TOKEN_X_MINT: &str = "4sWNB8zGWHkh6UnmwiEtzNxL4XrN7uK9tosbESbJFfVs";
    const TOKEN_Y_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    let signature = Signature::from_str(SIGNATURE).expect("valid fixture signature");
    let filter = EventTypeFilter::include_only([EventType::MeteoraDlmmSwap]);
    let events = fetch_rpc_transaction_as_streamer_events(
        &rpc_client(),
        &signature,
        0,
        &[Protocol::MeteoraDlmm],
        Some(&filter),
    )
    .expect("parse current Meteora DLMM swap");

    assert_eq!(METEORA_DLMM_PROGRAM_ID.to_string(), "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");
    assert_eq!(events.len(), 1);
    let DexEvent::MeteoraDlmmSwapEvent(swap) = &events[0] else {
        panic!("expected Meteora DLMM swap");
    };
    assert_eq!(swap.metadata.signature, signature);
    assert_eq!(swap.metadata.slot, 438_873_646);
    assert_eq!(swap.token_x_mint.to_string(), TOKEN_X_MINT);
    assert_eq!(swap.token_y_mint.to_string(), TOKEN_Y_MINT);
    assert_eq!(swap.amount_in, 2_738_183_783);
    assert_eq!(swap.amount_out, 81_555_062);
    assert_eq!(swap.fee, 18_486_656);
    assert_eq!(swap.protocol_fee, 2_054_072);
}

#[test]
fn current_raydium_launchlab_usd1_trade_exposes_quote_context() {
    if !run_mainnet_tests() {
        return;
    }
    const SIGNATURE: &str =
        "zuaKyxjpM7G5et2XqZofjjGNczNduGs6g8ipCEeZKKV7h6FFgRNJbXnzfufSZWD3bEacmf8sVktXpZaadQhmVuJ";
    const USD1_MINT: &str = "USD1ttGY1N17NEEHLmELoaybftRBUSErhqYiQzvEmuB";
    const USD1_GLOBAL_CONFIG: &str = "EPiZbnrThjyLnoQ6QQzkxeFqyL5uyg9RzNHHAudUPxBz";
    let signature = Signature::from_str(SIGNATURE).expect("valid fixture signature");
    let events = fetch_rpc_transaction_as_streamer_events(
        &rpc_client(),
        &signature,
        0,
        &[Protocol::RaydiumLaunchpad],
        None,
    )
    .expect("parse current Raydium LaunchLab USD1 trade");
    let trades: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            DexEvent::BonkTradeEvent(trade) => Some(trade),
            _ => None,
        })
        .collect();

    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].metadata.signature, signature);
    assert_eq!(trades[0].metadata.slot, 438_894_516);
    assert_eq!(trades[0].quote_token_mint.to_string(), USD1_MINT);
    assert_eq!(trades[0].global_config.to_string(), USD1_GLOBAL_CONFIG);
    assert_ne!(trades[0].quote_token_program, Default::default());
}
