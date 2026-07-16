//! Minimal advanced interop example for the public sol-parser-sdk bridge.
//!
//! Most bots should use the normal streamer subscription callbacks. Use this bridge when you
//! already have raw `sol-parser-sdk::DexEvent` values and want to adapt them into streamer
//! `DexEvent` values, or when you need direct access to raw SDK APIs.

use solana_streamer_sdk::sdk_bridge;
use solana_streamer_sdk::streaming::event_parser::common::{filter::EventTypeFilter, EventType};

fn main() {
    let streamer_filter = EventTypeFilter::include_only(vec![
        EventType::PumpFunBuy,
        EventType::PumpFunSell,
        EventType::TokenAccount,
    ]);
    let sdk_filter = sdk_bridge::event_type_filter_to_sdk(Some(&streamer_filter));

    println!("SDK filter built: {}", sdk_filter.is_some());
    println!("Raw SDK event type: {}", std::any::type_name::<sdk_bridge::SdkDexEvent>());
    println!(
        "Raw SDK crate re-export: {}",
        std::any::type_name::<solana_streamer_sdk::parser_sdk::DexEvent>()
    );
}
