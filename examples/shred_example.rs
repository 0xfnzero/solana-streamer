use solana_streamer_sdk::streaming::{
    event_parser::common::{filter::EventTypeFilter, EventType},
    event_parser::{DexEvent, Protocol},
    shred::StreamClientConfig,
    ShredStreamGrpc,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting ShredStream Streamer...");
    test_shreds().await?;
    Ok(())
}

async fn test_shreds() -> Result<(), Box<dyn std::error::Error>> {
    println!("Subscribing to ShredStream events...");

    // Create low-latency configuration.
    // Metrics add overhead; enable explicitly with STREAMER_ENABLE_METRICS=1.
    let config = StreamClientConfig {
        enable_metrics: std::env::var("STREAMER_ENABLE_METRICS").as_deref() == Ok("1"),
        ..Default::default()
    };
    let shred_stream =
        ShredStreamGrpc::new_with_config("http://127.0.0.1:10800".to_string(), config).await?;

    let callback = create_event_callback();
    let protocols = vec![Protocol::PumpFun];

    // ShredStream uses the sol-parser-sdk ShredStream hot path. Keep filters narrow for
    // latency-sensitive bots; use None to receive every event the SDK ShredStream path emits.
    let event_type_filter = Some(EventTypeFilter::include_only(vec![
        EventType::PumpFunBuy,
        EventType::PumpFunSell,
        EventType::PumpFunCreateToken,
        EventType::PumpFunCreateV2Token,
    ]));

    println!("Listening for events, press Ctrl+C to stop...");
    shred_stream.shredstream_subscribe(protocols, None, event_type_filter, callback).await?;

    // Demo safety stop: stop automatically after 1000 seconds.
    let shred_clone = shred_stream.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(1000)).await;
        shred_clone.stop().await;
    });

    println!("Waiting for Ctrl+C to stop...");
    tokio::signal::ctrl_c().await?;

    Ok(())
}

fn create_event_callback() -> impl Fn(DexEvent) {
    |event: DexEvent| {
        println!(
            "🎉 Event received! Type: {:?}, tx_index: {:?}",
            event.metadata().event_type,
            event.metadata().tx_index
        );
        if let DexEvent::BlockMetaEvent(e) = event {
            println!("BlockMetaEvent: {:?}", e.metadata.handle_us);
        }
    }
}
