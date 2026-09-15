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

// These transactions were captured from current mainnet traffic in August/September 2026.
// Run with: RUN_MAINNET_TESTS=1 SOLANA_RPC_URL=<optional archive RPC> cargo test --test current_mainnet_transactions

#[test]
fn current_pumpfun_holder_reward_trade_preserves_reward_tail() {
    if !run_mainnet_tests() {
        return;
    }
    const SIGNATURE: &str =
        "4xesV2yysHa1S2t6wkW8gJKTJ23LbtnKqchbtuRAmeSGqVWXTuZ755nQRzg4xZNYUmyeZr8Hpq8UW4iVe24YLrf2";
    let signature = Signature::from_str(SIGNATURE).expect("valid fixture signature");
    let filter = EventTypeFilter::include_only([EventType::PumpFunBuy]);
    let events = fetch_rpc_transaction_as_streamer_events(
        &rpc_client(),
        &signature,
        0,
        &[Protocol::PumpFun],
        Some(&filter),
    )
    .expect("parse current PumpFun holder-reward trade");

    assert_eq!(events.len(), 1);
    let DexEvent::PumpFunTradeEvent(trade) = &events[0] else {
        panic!("expected PumpFun trade");
    };
    assert_eq!(trade.metadata.signature, signature);
    assert_eq!(trade.metadata.slot, 446_704_267);
    assert_eq!(trade.token_amount, 33_183_665_172);
    assert_eq!(trade.quote_amount, 111_485_301);
    assert_eq!(trade.creator_fee_basis_points, 300);
    assert_eq!(trade.creator_fee, 3_344_560);
    assert_eq!(trade.holder_rewards_bps, 300);
    assert_eq!(trade.holder_rewards, 3_344_560);
}

#[test]
fn current_pumpswap_buy_preserves_complete_upgrade_tail() {
    if !run_mainnet_tests() {
        return;
    }
    const SIGNATURE: &str =
        "4tCWJ41mFmJNU6KPk1dQpbVH9noysGKcuTTMcb7NQsE2U4Kr2x9rNNipCKLRuxA38j7TJPPYzC9eNA7piNyTPat1";
    let signature = Signature::from_str(SIGNATURE).expect("valid fixture signature");
    let filter = EventTypeFilter::include_only([EventType::PumpSwapBuy]);
    let events = fetch_rpc_transaction_as_streamer_events(
        &rpc_client(),
        &signature,
        0,
        &[Protocol::PumpSwap],
        Some(&filter),
    )
    .expect("parse current PumpSwap buy");

    assert_eq!(events.len(), 1);
    let DexEvent::PumpSwapBuyEvent(buy) = &events[0] else {
        panic!("expected PumpSwap buy");
    };
    assert_eq!(buy.metadata.signature, signature);
    assert_eq!(buy.metadata.slot, 446_704_571);
    assert_eq!(buy.base_amount_out, 88_002_570_945);
    assert_eq!(buy.quote_amount_in, 99_000_000);
    assert_eq!(buy.buyback_fee_basis_points, 5_000);
    assert_eq!(buy.buyback_fee, 24_457);
    assert_eq!(buy.virtual_quote_reserves, 17_584_505_355);
    assert!(buy.can_boost);
    assert_eq!(buy.base_supply, 976_275_975_409_128);
    assert_eq!(buy.holder_rewards_bps, 0);
    assert_eq!(buy.holder_rewards, 0);
}

#[test]
fn current_meteora_damm_v2_swap_passes_exact_filter() {
    if !run_mainnet_tests() {
        return;
    }
    const SIGNATURE: &str =
        "5WUC7ZMio6F1D5Dhcteb8gChkReQ1YVg3zaB2bBQpfccN1knU6F3gHBYdTv1dypX3VJyM4rTASp5YDoyXGqtmpCU";
    let signature = Signature::from_str(SIGNATURE).expect("valid fixture signature");
    let filter = EventTypeFilter::include_only([EventType::MeteoraDammV2Swap]);
    let events = fetch_rpc_transaction_as_streamer_events(
        &rpc_client(),
        &signature,
        0,
        &[Protocol::MeteoraDammV2],
        Some(&filter),
    )
    .expect("parse current Meteora DAMM v2 swap");

    assert_eq!(events.len(), 1);
    let DexEvent::MeteoraDammV2SwapEvent(swap) = &events[0] else {
        panic!("expected Meteora DAMM v2 swap");
    };
    assert_eq!(swap.metadata.signature, signature);
    assert_eq!(swap.metadata.slot, 443_486_348);
    assert_eq!((swap.amount_0, swap.amount_1, swap.swap_mode), (48_633_499_685, 55_554_409, 0));
    assert_eq!(swap.included_fee_input_amount, 48_633_499_685);
    assert_eq!(swap.output_amount, 56_115_565);
    assert_eq!((swap.trading_fee, swap.claiming_fee, swap.compounding_fee), (44_938, 44_938, 0));
    assert_eq!((swap.protocol_fee, swap.partner_fee, swap.referral_fee), (11_234, 0, 0));
    assert_eq!((swap.reserve_a_amount, swap.reserve_b_amount), (9_746_117_860_573, 11_200_603_532));
}

#[test]
fn issue_82_damm_v2_swap_accounts_reach_streamer() {
    if !run_mainnet_tests() {
        return;
    }
    for (signature, pool) in [
        ("4vPzV2JPbDZghNgE6pYQhcjTFrNQt1V2A23bRXCpmYRQdjmKTGYgrfiLWXdXrdZpBu1CrWWLQoJxbL7GnoQJLRhv", "GSKh9Q5BmhwWNvr9phx7ei1gNkpVLBTqqx9GbMTfwcxE"),
        ("4CBrBEWnoo3TKMMbBx8zscYqVcLrrCDwWykKjGUWfASABGQz7RKLJD7mCKm7pWyGaxRaBaapMqnrSwCy3ZHFCS9W", "52388vAjCySRMBjuQJD36iQ2941hKEzTpU46M7MjZeoM"),
    ] {
        let signature = Signature::from_str(signature).unwrap();
        let events = fetch_rpc_transaction_as_streamer_events(
            &rpc_client(), &signature, 0, &[Protocol::MeteoraDammV2],
            Some(&EventTypeFilter::include_only([EventType::MeteoraDammV2Swap])),
        ).expect("parse issue #82 swap");
        assert_eq!(events.len(), 1);
        let DexEvent::MeteoraDammV2SwapEvent(swap) = &events[0] else { panic!("expected DAMM swap") };
        assert_eq!(swap.pool.to_string(), pool);
        assert_ne!(swap.pool_authority, solana_sdk::pubkey::Pubkey::default());
        assert_ne!(swap.input_token_account, solana_sdk::pubkey::Pubkey::default());
        assert_ne!(swap.output_token_account, solana_sdk::pubkey::Pubkey::default());
        assert_ne!(swap.token_a_vault, solana_sdk::pubkey::Pubkey::default());
        assert_ne!(swap.token_b_vault, solana_sdk::pubkey::Pubkey::default());
        assert_ne!(swap.token_a_mint, solana_sdk::pubkey::Pubkey::default());
        assert_ne!(swap.token_b_mint, solana_sdk::pubkey::Pubkey::default());
        assert_ne!(swap.payer, solana_sdk::pubkey::Pubkey::default());
        assert_eq!(swap.program.to_string(), "cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG");
        assert_eq!(swap.referral_token_account, None);
    }
}

#[test]
fn current_meteora_damm_v2_add_liquidity_passes_exact_filter() {
    if !run_mainnet_tests() {
        return;
    }
    const SIGNATURE: &str =
        "67SA1qv4f6ZY948qt7C22dTReS8EcGG8PkVJYdoqSXUxf3h2QPUjdnbu6hqdR79WR1CYxweCePycpcuTFR8WYWbr";
    let signature = Signature::from_str(SIGNATURE).expect("valid fixture signature");
    let filter = EventTypeFilter::include_only([EventType::MeteoraDammV2AddLiquidity]);
    let events = fetch_rpc_transaction_as_streamer_events(
        &rpc_client(),
        &signature,
        0,
        &[Protocol::MeteoraDammV2],
        Some(&filter),
    )
    .expect("parse current Meteora DAMM v2 add liquidity");

    assert_eq!(events.len(), 1);
    let DexEvent::MeteoraDammV2AddLiquidityEvent(add) = &events[0] else {
        panic!("expected Meteora DAMM v2 add liquidity");
    };
    assert_eq!(add.metadata.signature, signature);
    assert_eq!(add.metadata.slot, 443_564_414);
    assert_eq!((add.token_a_amount, add.token_b_amount), (1_223_939_852, 4_178_320));
    assert_eq!(add.liquidity_delta, 1_319_169_404_971_592_647_400_000_000);
    assert_eq!(
        (add.token_a_amount_threshold, add.token_b_amount_threshold),
        (1_225_165_017, 4_182_502)
    );
    assert_eq!((add.total_amount_a, add.total_amount_b), (1_223_939_852, 4_178_320));
    assert_eq!((add.reserve_a_amount, add.reserve_b_amount), (8_471_243_526, 28_919_366));
}

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
