//! ShredStream 订阅入口：底层订阅与热路径解析直接复用 `sol-parser-sdk::shredstream`，
//! 本模块只负责把 SDK `DexEvent` 适配回 streamer 原有 callback API。
use std::sync::Arc;

use solana_sdk::pubkey::Pubkey;

use crate::common::AnyResult;
use crate::streaming::common::{MetricsEventType, MetricsManager, SubscriptionHandle};
use crate::streaming::event_parser::common::filter::{
    build_sdk_shred_parse_event_filter, EventTypeFilter,
};
use crate::streaming::event_parser::common::high_performance_clock::elapsed_micros_since;
use crate::streaming::event_parser::{DexEvent, Protocol};
use crate::streaming::parser_sdk_bridge::adapt_parser_event;

use super::ShredStreamGrpc;

impl ShredStreamGrpc {
    /// 订阅ShredStream事件（支持批处理和即时处理）
    pub async fn shredstream_subscribe<F>(
        &self,
        protocols: Vec<Protocol>,
        bot_wallet: Option<Pubkey>,
        event_type_filter: Option<EventTypeFilter>,
        callback: F,
    ) -> AnyResult<()>
    where
        F: Fn(DexEvent) + Send + Sync + 'static,
    {
        // 如果已有活跃订阅，先停止它
        self.stop().await;

        let mut metrics_handle = None;
        // 启动自动性能监控（如果启用）
        if self.config.enable_metrics {
            metrics_handle = MetricsManager::global().start_auto_monitoring().await;
        }

        let sdk_parse_filter =
            build_sdk_shred_parse_event_filter(&protocols, event_type_filter.as_ref());
        let queue = self
            .sdk_shredstream_client
            .subscribe_with_filter(sdk_parse_filter)
            .await
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        // Wrap callback once before the async block
        let callback = Arc::new(callback);

        let stream_task = tokio::spawn(async move {
            loop {
                let Some(sdk_event) = queue.pop() else {
                    tokio::task::yield_now().await;
                    continue;
                };

                MetricsManager::global().add_tx_process_count();
                let recv_wall_us = sdk_event.metadata().grpc_recv_us;
                if let Some(mut event) = adapt_parser_event(
                    sdk_event,
                    None,
                    recv_wall_us,
                    &protocols,
                    event_type_filter.as_ref(),
                ) {
                    event.metadata_mut().handle_us = elapsed_micros_since(event.metadata().recv_us);
                    event =
                        crate::streaming::event_parser::core::event_parser::helpers::process_event(
                            event, bot_wallet,
                        );
                    let metadata = event.metadata();
                    let processing_time_us = metadata.handle_us as f64;
                    let recv_us = metadata.recv_us;
                    let block_time_ms = metadata.block_time_ms;

                    callback(event);

                    MetricsManager::global().update_metrics_with_latency(
                        MetricsEventType::Transaction,
                        1,
                        processing_time_us,
                        recv_us,
                        block_time_ms,
                    );
                }
            }
        });

        // 保存订阅句柄
        let subscription_handle = SubscriptionHandle::new(stream_task, None, metrics_handle);
        let mut handle_guard = self.subscription_handle.lock().await;
        *handle_guard = Some(subscription_handle);

        Ok(())
    }
}
