use anyhow::Result;
use solana_sdk::signature::{Keypair, Signer};
use solana_streamer_sdk::streaming::event_parser::common::filter::EventTypeFilter;
use solana_streamer_sdk::streaming::event_parser::common::types::EventType;
use solana_streamer_sdk::streaming::event_parser::Protocol;
use solana_streamer_sdk::streaming::yellowstone_grpc::{
    AccountFilter, TransactionFilter, YellowstoneGrpc,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;

const PUMPFUN_PROGRAM_ID: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
const RAYDIUM_CPMM_PROGRAM_ID: &str = "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C";

const GRPC_ENDPOINT: &str = "https://solana-yellowstone-grpc.publicnode.com:443";
const API_KEY: Option<&str> = None;
const MONITORING_DURATION_SECS: u64 = 10;

/// Demonstrates dynamic subscription updates and filter changes in real-time
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    println!("Connecting to Yellowstone gRPC at {}", GRPC_ENDPOINT);
    let client =
        Arc::new(YellowstoneGrpc::new(GRPC_ENDPOINT.to_string(), API_KEY.map(|s| s.to_string()))?);

    let event_counter = Arc::new(AtomicU64::new(0));
    let counter = event_counter.clone();

    let callback =
        move |event: solana_streamer_sdk::streaming::event_parser::DexEvent| {
            let count = counter.fetch_add(1, Ordering::Relaxed);

            let protocol = match event.metadata().event_type {
                EventType::PumpFunBuy | EventType::PumpFunSell => "PumpFun",
                EventType::RaydiumCpmmSwapBaseInput | EventType::RaydiumCpmmSwapBaseOutput => {
                    "RaydiumCpmm"
                }
                _ => "Unknown",
            };

            println!("Event #{}: {:11} - {:.8}...", count + 1, protocol, event.metadata().signature);
        };

    println!("\n=== Phase 1: PumpFun only ===");
    let pumpfun_filter = TransactionFilter {
        account_include: vec![PUMPFUN_PROGRAM_ID.to_string()],
        account_exclude: vec![],
        account_required: vec![],
    };

    let account_filter = AccountFilter { account: vec![], owner: vec![], filters: vec![] };
    let trade_event_filter = EventTypeFilter {
        include: vec![
            EventType::PumpFunBuy,
            EventType::PumpFunSell,
            EventType::RaydiumCpmmSwapBaseInput,
            EventType::RaydiumCpmmSwapBaseOutput,
        ],
    };

    if let Err(e) = client
        .subscribe_events_immediate(
            vec![Protocol::PumpFun, Protocol::RaydiumCpmm],
            None,
            vec![pumpfun_filter],
            vec![account_filter],
            Some(trade_event_filter),
            None,
            callback,
        )
        .await
    {
        println!("Failed to create subscription: {}", e);
        return Ok(());
    }

    println!(
        "Subscribed to PumpFun transactions with trade event filters, monitoring for {}s...",
        MONITORING_DURATION_SECS
    );
    sleep(Duration::from_secs(MONITORING_DURATION_SECS)).await;
    let phase1_count = event_counter.load(Ordering::Relaxed);
    println!("Phase 1: {} events", phase1_count);

    println!("\n=== Phase 2: PumpFun + RaydiumCpmm ===");
    let multi_protocol_filter = TransactionFilter {
        account_include: vec![PUMPFUN_PROGRAM_ID.to_string(), RAYDIUM_CPMM_PROGRAM_ID.to_string()],
        account_exclude: vec![],
        account_required: vec![],
    };

    if let Err(e) = client
        .update_subscription(
            vec![multi_protocol_filter],
            vec![AccountFilter { account: vec![], owner: vec![], filters: vec![] }],
        )
        .await
    {
        println!("Failed to update subscription: {}", e);
        return Ok(());
    }

    println!(
        "Updated to PumpFun + RaydiumCpmm transactions, monitoring for {}s...",
        MONITORING_DURATION_SECS
    );
    sleep(Duration::from_secs(MONITORING_DURATION_SECS)).await;
    let phase2_count = event_counter.load(Ordering::Relaxed);
    println!("Phase 2: {} events", phase2_count - phase1_count);

    println!("\n=== Phase 3: RaydiumCpmm only ===");
    let raydium_cpmm_filter = TransactionFilter {
        account_include: vec![RAYDIUM_CPMM_PROGRAM_ID.to_string()],
        account_exclude: vec![],
        account_required: vec![],
    };

    if let Err(e) = client
        .update_subscription(
            vec![raydium_cpmm_filter],
            vec![AccountFilter { account: vec![], owner: vec![], filters: vec![] }],
        )
        .await
    {
        println!("Failed to update subscription: {}", e);
        return Ok(());
    }

    sleep(Duration::from_secs(MONITORING_DURATION_SECS)).await;
    println!(
        "Updated to RaydiumCpmm transactions only, monitoring for {}s...",
        MONITORING_DURATION_SECS
    );
    let phase3_count = event_counter.load(Ordering::Relaxed);
    println!("Phase 3: {} events", phase3_count - phase2_count);

    println!("\n=== Phase 4: Back to PumpFun only ===");
    let pumpfun_only_filter = TransactionFilter {
        account_include: vec![PUMPFUN_PROGRAM_ID.to_string()],
        account_exclude: vec![],
        account_required: vec![],
    };

    if let Err(e) = client
        .update_subscription(
            vec![pumpfun_only_filter],
            vec![AccountFilter { account: vec![], owner: vec![], filters: vec![] }],
        )
        .await
    {
        println!("Failed to update subscription: {}", e);
        return Ok(());
    }

    sleep(Duration::from_secs(MONITORING_DURATION_SECS)).await;
    println!(
        "Updated to PumpFun transactions only, monitoring for {}s...",
        MONITORING_DURATION_SECS
    );
    let phase4_count = event_counter.load(Ordering::Relaxed);
    println!("Phase 4: {} events", phase4_count - phase3_count);

    println!("\n=== Phase 5: All events ===");
    let empty_filter = TransactionFilter {
        account_include: vec![],
        account_exclude: vec![],
        account_required: vec![],
    };

    if let Err(e) = client
        .update_subscription(
            vec![empty_filter],
            vec![AccountFilter { account: vec![], owner: vec![], filters: vec![] }],
        )
        .await
    {
        println!("Failed to update subscription: {}", e);
        return Ok(());
    }

    sleep(Duration::from_secs(MONITORING_DURATION_SECS)).await;
    println!(
        "Updated to all transactions (no filters), monitoring for {}s...",
        MONITORING_DURATION_SECS
    );
    let phase5_count = event_counter.load(Ordering::Relaxed);
    println!("Phase 5: {} events", phase5_count - phase4_count);

    println!("\n=== Phase 6: Silence ===");

    let random_keypair_1 = Keypair::new();
    let random_keypair_2 = Keypair::new();
    let random_pubkey_1 = random_keypair_1.pubkey();
    let random_pubkey_2 = random_keypair_2.pubkey();

    let silence_filter = TransactionFilter {
        account_include: vec![],
        account_exclude: vec![],
        account_required: vec![random_pubkey_1.to_string(), random_pubkey_2.to_string()],
    };

    if let Err(e) = client
        .update_subscription(
            vec![silence_filter],
            vec![AccountFilter { account: vec![], owner: vec![], filters: vec![] }],
        )
        .await
    {
        println!("Failed to update subscription: {}", e);
        return Ok(());
    }

    println!("Updated to random addresses (expecting silence), monitoring for 3s...");
    let before_silence = event_counter.load(Ordering::Relaxed);
    let start_time = Instant::now();
    let last_event_time = Arc::new(Mutex::new(start_time));
    let last_event_time_clone = last_event_time.clone();

    let mut last_count = before_silence;
    for _ in 0..6 {
        sleep(Duration::from_millis(500)).await;
        let current_count = event_counter.load(Ordering::Relaxed);
        if current_count > last_count {
            if let Ok(mut time) = last_event_time_clone.lock() {
                *time = Instant::now();
            }
            last_count = current_count;
        }
    }

    let final_count = event_counter.load(Ordering::Relaxed);
    let events_during_silence = final_count - before_silence;

    if events_during_silence == 0 {
        println!("Phase 6: 0 events (immediate filter application)");
    } else if let Ok(last_time) = last_event_time.lock() {
        let propagation_time = last_time.duration_since(start_time);
        println!(
            "Phase 6: {} events during propagation, filter took {}ms",
            events_during_silence,
            propagation_time.as_millis()
        );
    }

    println!("\n=== Phase 7: Shutdown ===");

    let shutdown_client =
        Arc::new(YellowstoneGrpc::new(GRPC_ENDPOINT.to_string(), API_KEY.map(|s| s.to_string()))?);

    let shutdown_event_counter = Arc::new(AtomicU64::new(0));
    let shutdown_counter = shutdown_event_counter.clone();
    let shutdown_callback =
        move |_event: solana_streamer_sdk::streaming::event_parser::DexEvent| {
            shutdown_counter.fetch_add(1, Ordering::Relaxed);
        };

    if let Err(e) = shutdown_client
        .subscribe_events_immediate(
            vec![Protocol::PumpFun, Protocol::RaydiumCpmm],
            None,
            vec![TransactionFilter {
                account_include: vec![],
                account_exclude: vec![],
                account_required: vec![],
            }],
            vec![AccountFilter { account: vec![], owner: vec![], filters: vec![] }],
            None,
            None,
            shutdown_callback,
        )
        .await
    {
        println!("Failed to subscribe shutdown client: {}", e);
        return Ok(());
    }

    sleep(Duration::from_millis(1000)).await;
    let pre_stop_count = shutdown_event_counter.load(Ordering::Relaxed);
    println!("Received {} events before stop", pre_stop_count);

    let stop_time = Instant::now();
    shutdown_client.stop().await;
    let shutdown_duration = stop_time.elapsed();
    println!("stop() completed in {:.1}ms", shutdown_duration.as_millis());

    let post_stop_count = shutdown_event_counter.load(Ordering::Relaxed);
    let during_stop = post_stop_count - pre_stop_count;
    if during_stop > 0 {
        println!("  {} events received during stop()", during_stop);
    }

    let last_event_time = Arc::new(Mutex::new(stop_time));
    let last_event_time_clone = last_event_time.clone();
    let mut last_count = post_stop_count;

    for _ in 0..20 {
        sleep(Duration::from_millis(100)).await;
        let current_count = shutdown_event_counter.load(Ordering::Relaxed);
        if current_count > last_count {
            if let Ok(mut time) = last_event_time_clone.lock() {
                *time = Instant::now();
            }
            last_count = current_count;
        }
    }

    let final_count = shutdown_event_counter.load(Ordering::Relaxed);
    let after_stop = final_count - post_stop_count;

    if after_stop == 0 {
        println!("Phase 7: Clean shutdown - no events after stop()");
    } else if let Ok(last_time) = last_event_time.lock() {
        let post_stop_duration = last_time.duration_since(stop_time);
        let silence_duration = Instant::now().duration_since(*last_time);
        println!(
            "Phase 7: {} events arrived up to {}ms after stop(), then silent for {}ms",
            after_stop,
            post_stop_duration.as_millis(),
            silence_duration.as_millis()
        );
    }

    println!("\n=== Subscription enforcement ===");

    let test_callback =
        |_event: solana_streamer_sdk::streaming::event_parser::DexEvent| {};

    match client
        .subscribe_events_immediate(
            vec![Protocol::RaydiumCpmm],
            None,
            vec![TransactionFilter {
                account_include: vec![RAYDIUM_CPMM_PROGRAM_ID.to_string()],
                account_exclude: vec![],
                account_required: vec![],
            }],
            vec![AccountFilter { account: vec![], owner: vec![], filters: vec![] }],
            None,
            None,
            test_callback,
        )
        .await
    {
        Ok(_) => println!("ERROR: Same client created second subscription"),
        Err(e) if e.to_string().contains("Already subscribed") => {
            println!("✓ Single subscription enforcement working");
        }
        Err(e) => println!("Unexpected error: {}", e),
    }

    let client2 =
        Arc::new(YellowstoneGrpc::new(GRPC_ENDPOINT.to_string(), API_KEY.map(|s| s.to_string()))?);

    let client2_counter = Arc::new(AtomicU64::new(0));
    let counter2 = client2_counter.clone();
    let client2_callback =
        move |_event: solana_streamer_sdk::streaming::event_parser::DexEvent| {
            counter2.fetch_add(1, Ordering::Relaxed);
        };

    match client2
        .subscribe_events_immediate(
            vec![Protocol::RaydiumCpmm],
            None,
            vec![TransactionFilter {
                account_include: vec![RAYDIUM_CPMM_PROGRAM_ID.to_string()],
                account_exclude: vec![],
                account_required: vec![],
            }],
            vec![AccountFilter { account: vec![], owner: vec![], filters: vec![] }],
            None,
            None,
            client2_callback,
        )
        .await
    {
        Ok(_) => {
            sleep(Duration::from_millis(500)).await;
            let count = client2_counter.load(Ordering::Relaxed);
            println!("✓ Second client: {} events", count);
            client2.stop().await;
        }
        Err(e) => println!("ERROR: Second client failed: {}", e),
    }

    println!("\n=== Advanced subscription enforcement ===");

    let test_callback_advanced =
        |_event: solana_streamer_sdk::streaming::event_parser::DexEvent| {};

    let client3 =
        Arc::new(YellowstoneGrpc::new(GRPC_ENDPOINT.to_string(), API_KEY.map(|s| s.to_string()))?);

    // First subscription should succeed
    match client3
        .subscribe_events_immediate(
            vec![Protocol::RaydiumCpmm],
            None,
            vec![TransactionFilter {
                account_include: vec![RAYDIUM_CPMM_PROGRAM_ID.to_string()],
                account_exclude: vec![],
                account_required: vec![],
            }],
            vec![AccountFilter { account: vec![], owner: vec![], filters: vec![] }],
            None,
            None,
            test_callback_advanced,
        )
        .await
    {
        Ok(_) => {
            // Second subscription attempt on same client should fail
            match client3
                .subscribe_events_immediate(
                    vec![Protocol::RaydiumCpmm],
                    None,
                    vec![TransactionFilter {
                        account_include: vec![RAYDIUM_CPMM_PROGRAM_ID.to_string()],
                        account_exclude: vec![],
                        account_required: vec![],
                    }],
                    vec![AccountFilter { account: vec![], owner: vec![], filters: vec![] }],
                    None,
                    None,
                    |_| {},
                )
                .await
            {
                Ok(_) => println!("ERROR: Same client created second advanced subscription"),
                Err(e) if e.to_string().contains("Already subscribed") => {
                    println!("✓ Advanced single subscription enforcement working");
                }
                Err(e) => println!("Unexpected error: {}", e),
            }
        }
        Err(e) => println!("ERROR: First advanced subscription failed: {}", e),
    }

    // Test that a second client can subscribe using advanced method
    let client4 =
        Arc::new(YellowstoneGrpc::new(GRPC_ENDPOINT.to_string(), API_KEY.map(|s| s.to_string()))?);

    let client4_counter = Arc::new(AtomicU64::new(0));
    let counter4 = client4_counter.clone();
    let client4_callback =
        move |_event: solana_streamer_sdk::streaming::event_parser::DexEvent| {
            counter4.fetch_add(1, Ordering::Relaxed);
        };

    match client4
        .subscribe_events_immediate(
            vec![Protocol::RaydiumCpmm],
            None,
            vec![TransactionFilter {
                account_include: vec![RAYDIUM_CPMM_PROGRAM_ID.to_string()],
                account_exclude: vec![],
                account_required: vec![],
            }],
            vec![AccountFilter { account: vec![], owner: vec![], filters: vec![] }],
            None,
            None,
            client4_callback,
        )
        .await
    {
        Ok(_) => {
            sleep(Duration::from_millis(500)).await;
            let count = client4_counter.load(Ordering::Relaxed);
            println!("✓ Second client (advanced): {} events", count);
            client4.stop().await;
        }
        Err(e) => println!("ERROR: Second client (advanced) failed: {}", e),
    }

    client3.stop().await;

    client.stop().await;

    Ok(())
}
