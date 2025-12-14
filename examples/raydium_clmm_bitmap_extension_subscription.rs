use solana_streamer_sdk::streaming::{
    event_parser::{
        protocols::raydium_clmm::parser::RAYDIUM_CLMM_PROGRAM_ID, DexEvent, Protocol,
    },
    grpc::ClientConfig,
    yellowstone_grpc::{AccountFilter, TransactionFilter},
    YellowstoneGrpc,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志系统（可选）
    // env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    println!("开始 Raydium CLMM TickArrayBitmapExtension 账户数据订阅示例...");
    subscribe_raydium_clmm_bitmap_extension().await?;
    Ok(())
}

async fn subscribe_raydium_clmm_bitmap_extension() -> Result<(), Box<dyn std::error::Error>> {
    println!("订阅 Raydium CLMM TickArrayBitmapExtension 账户数据...");

    // 创建客户端配置
    let mut config: ClientConfig = ClientConfig::default();
    // 启用性能监控（可选，有性能开销）
    config.enable_metrics = true;
    let grpc = YellowstoneGrpc::new_with_config(
        "https://solana-yellowstone-grpc.publicnode.com:443".to_string(),
        None,
        config,
    )?;

    println!("gRPC 客户端创建成功");

    let callback = create_event_callback();

    // 只订阅 Raydium CLMM 协议
    let protocols = vec![Protocol::RaydiumClmm];

    println!("监控协议: {:?}", protocols);

    // 账户过滤器 - 订阅 Raydium CLMM 程序拥有的账户
    let account_filter = AccountFilter {
        account: vec![],
        owner: vec![RAYDIUM_CLMM_PROGRAM_ID.to_string()],
        filters: vec![],
    };

    // 交易过滤器（可选，如果只想订阅账户数据，可以留空）
    let transaction_filter = TransactionFilter {
        account_include: vec![],
        account_exclude: vec![],
        account_required: vec![],
    };

    // 事件类型过滤器 - 只订阅 TickArrayBitmapExtension 账户事件
    use solana_streamer_sdk::streaming::event_parser::common::EventType;
    use solana_streamer_sdk::streaming::event_parser::common::filter::EventTypeFilter;
    let event_type_filter = Some(EventTypeFilter {
        include: vec![EventType::AccountRaydiumClmmTickArrayBitmapExtension],
    });

    println!("开始监听事件，按 Ctrl+C 停止...");
    println!("监控程序: {}", RAYDIUM_CLMM_PROGRAM_ID);

    println!("开始订阅...");

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

    // 支持 stop 方法，测试代码 - 异步1000秒之后停止
    let grpc_clone = grpc.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(1000)).await;
        grpc_clone.stop().await;
    });

    println!("等待 Ctrl+C 停止...");
    tokio::signal::ctrl_c().await?;

    Ok(())
}

fn create_event_callback() -> impl Fn(DexEvent) {
    |event: DexEvent| {
        match event {
            DexEvent::RaydiumClmmTickArrayBitmapExtensionAccountEvent(e) => {
                println!("=== Raydium CLMM TickArrayBitmapExtension 账户更新 ===");
                println!("账户地址: {}", e.pubkey);
                println!("Pool ID: {}", e.tick_array_bitmap_extension.pool_id);
                println!("Executable: {}", e.executable);
                println!("Lamports: {}", e.lamports);
                println!("Owner: {}", e.owner);
                println!("Rent Epoch: {}", e.rent_epoch);
                println!("Slot: {}", e.metadata.slot);
                println!("Signature: {}", e.metadata.signature);
                
                // 由于使用了 #[repr(C, packed)]，需要先复制数据到本地变量
                let positive_bitmap = e.tick_array_bitmap_extension.positive_tick_array_bitmap;
                let negative_bitmap = e.tick_array_bitmap_extension.negative_tick_array_bitmap;
                
                // 打印 positive_tick_array_bitmap 的统计信息
                let positive_non_zero_count = positive_bitmap
                    .iter()
                    .flatten()
                    .filter(|&&x| x != 0)
                    .count();
                println!("Positive Tick Array Bitmap: {} 个非零值", positive_non_zero_count);
                
                // 打印 negative_tick_array_bitmap 的统计信息
                let negative_non_zero_count = negative_bitmap
                    .iter()
                    .flatten()
                    .filter(|&&x| x != 0)
                    .count();
                println!("Negative Tick Array Bitmap: {} 个非零值", negative_non_zero_count);
                
                // 可选：打印前几个非零值作为示例
                println!("\nPositive Bitmap 前 5 个非零值:");
                let mut count = 0;
                for (i, row) in positive_bitmap.iter().enumerate() {
                    for (j, &value) in row.iter().enumerate() {
                        if value != 0 && count < 5 {
                            println!("  [{}][{}] = {}", i, j, value);
                            count += 1;
                        }
                    }
                }
                
                println!("\nNegative Bitmap 前 5 个非零值:");
                let mut count = 0;
                for (i, row) in negative_bitmap.iter().enumerate() {
                    for (j, &value) in row.iter().enumerate() {
                        if value != 0 && count < 5 {
                            println!("  [{}][{}] = {}", i, j, value);
                            count += 1;
                        }
                    }
                }
                
                println!("=====================================\n");
            }
            _ => {
                // 其他事件类型，可以忽略或记录
                // println!("其他事件类型: {:?}", event.metadata().event_type);
            }
        }
    }
}

