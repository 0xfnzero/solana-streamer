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
    ///
    /// Uses the SDK direct-callback path for minimum latency. User callbacks should avoid
    /// blocking work; use the SDK queue API directly when heavy asynchronous processing is needed.
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

        let sdk_parse_filter =
            build_sdk_shred_parse_event_filter(&protocols, event_type_filter.as_ref());

        // Wrap callback once before the async block
        let callback = Arc::new(callback);
        let callback_for_sdk = callback.clone();
        let protocols_for_sdk = protocols;
        let filter_for_sdk = event_type_filter;

        self.sdk_shredstream_client
            .subscribe_with_filter_callback(sdk_parse_filter, move |sdk_event| {
                MetricsManager::global().add_tx_process_count();
                let recv_wall_us = sdk_event.metadata().grpc_recv_us;
                if let Some(mut event) = adapt_parser_event(
                    sdk_event,
                    None,
                    recv_wall_us,
                    &protocols_for_sdk,
                    filter_for_sdk.as_ref(),
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

                    callback_for_sdk(event);

                    MetricsManager::global().update_metrics_with_latency(
                        MetricsEventType::Transaction,
                        1,
                        processing_time_us,
                        recv_us,
                        block_time_ms,
                    );
                }
            })
            .await
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        let mut metrics_handle = None;
        if self.config.enable_metrics {
            metrics_handle = MetricsManager::global().start_auto_monitoring().await;
        }

        // 保存订阅句柄
        let subscription_handle = SubscriptionHandle::metrics_only(metrics_handle);
        let mut handle_guard = self.subscription_handle.lock().await;
        *handle_guard = Some(subscription_handle);

        Ok(())
    }
}
