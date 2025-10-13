use solana_streamer_sdk::streaming::{
    event_parser::{
        common::{filter::EventTypeFilter, EventType},
        DexEvent,
    },
    grpc::ClientConfig,
    yellowstone_grpc::{AccountFilter, TransactionFilter},
    YellowstoneGrpc,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Yellowstone gRPC Streamer...");
    test_grpc().await?;
    Ok(())
}

async fn test_grpc() -> Result<(), Box<dyn std::error::Error>> {
    println!("Subscribing to Yellowstone gRPC events...");
    // Create low-latency configuration
    let mut config: ClientConfig = ClientConfig::default();
    // Enable performance monitoring, has performance overhead, disabled by default
    config.enable_metrics = true;
    let grpc = YellowstoneGrpc::new_with_config(
        "https://solana-yellowstone-grpc.publicnode.com:443".to_string(),
        None,
        config,
    )?;
    println!("GRPC client created successfully");
    let callback = create_event_callback();
    // Will try to parse corresponding protocol events from transactions
    let protocols = vec![];
    println!("Protocols to monitor: {:?}", protocols);
    // Filter accounts
    let account_include = vec![];
    let account_exclude = vec![];
    let account_required = vec![];

    // Listen to transaction data
    let transaction_filter =
        TransactionFilter { account_include, account_exclude, account_required };

    let nonce_account = "use_your_nonce_account_here".to_string();
    // Listen to account data belonging to owner programs -> account event monitoring
    let account_filter =
        AccountFilter { account: vec![nonce_account], owner: vec![], filters: vec![] };

    // Event filtering
    let event_type_filter = Some(EventTypeFilter { include: vec![EventType::NonceAccount] });

    println!("Starting to listen for events, press Ctrl+C to stop...");
    println!("Starting subscription...");

    grpc.subscribe_events_immediate(
        protocols,
        None,
        vec![transaction_filter],
        vec![account_filter],
        event_type_filter,
        None,
        callback,
    )
    .await?;

    // 支持 stop 方法，测试代码 -  异步1000秒之后停止
    let grpc_clone = grpc.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(1000)).await;
        grpc_clone.stop().await;
    });

    println!("Waiting for Ctrl+C to stop...");
    tokio::signal::ctrl_c().await?;

    Ok(())
}

fn create_event_callback() -> impl Fn(DexEvent) {
    |event: DexEvent| {
        println!("🎉 Event received! {:?}", event);
    }
}
