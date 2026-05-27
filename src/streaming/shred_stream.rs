//! ShredStream 订阅入口：底层订阅与热路径解析直接复用 `sol-parser-sdk::shredstream`，
//! 本模块负责把 SDK `DexEvent` 适配回 streamer 原有 callback API。
#![allow(deprecated)]

use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use solana_entry::entry::Entry as SolanaEntry;
use solana_sdk::pubkey::Pubkey;
use tokio::time::timeout;

use crate::common::AnyResult;
use crate::streaming::common::parse_shred_transaction_events;
use crate::streaming::common::{MetricsEventType, MetricsManager, SubscriptionHandle};
use crate::streaming::event_parser::common::filter::EventTypeFilter;
use crate::streaming::event_parser::common::high_performance_clock::get_high_perf_clock;
use crate::streaming::event_parser::{DexEvent, Protocol};

use super::ShredStreamGrpc;

impl ShredStreamGrpc {
    /// Subscribe to ShredStream events on the direct low-latency path.
    ///
    /// The callback runs on the stream read task; keep it non-blocking. `tx_index` is an
    /// entry-local best-effort index because ShredStream entries do not carry the slot-level
    /// Yellowstone transaction index.
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

        let request_timeout = Duration::from_secs(self.config.connection.request_timeout);
        let client = self.shredstream_client.as_ref().clone();
        let callback = Arc::new(callback);
        let stream_task = tokio::spawn(async move {
            let mut delay_ms = 100u64;

            loop {
                let mut client = client.clone();
                let request =
                    tonic::Request::new(crate::protos::shredstream::SubscribeEntriesRequest {});
                let response = if request_timeout.is_zero() {
                    client.subscribe_entries(request).await
                } else {
                    match timeout(request_timeout, client.subscribe_entries(request)).await {
                        Ok(response) => response,
                        Err(_) => {
                            log::error!(
                                "ShredStream subscribe request timed out after {}s - retry in {}ms",
                                request_timeout.as_secs(),
                                delay_ms
                            );
                            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                            delay_ms = (delay_ms * 2).min(60_000);
                            continue;
                        }
                    }
                };

                let mut stream = match response {
                    Ok(response) => response.into_inner(),
                    Err(error) => {
                        log::error!(
                            "ShredStream subscribe failed: {error} - retry in {delay_ms}ms"
                        );
                        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                        delay_ms = (delay_ms * 2).min(60_000);
                        continue;
                    }
                };

                while let Some(message) = stream.next().await {
                    let entry = match message {
                        Ok(entry) => entry,
                        Err(error) => {
                            log::error!("ShredStream stream error: {error} - reconnecting");
                            break;
                        }
                    };
                    delay_ms = 100;
                    process_shred_entry_direct(
                        entry,
                        &protocols,
                        event_type_filter.as_ref(),
                        bot_wallet,
                        callback.as_ref(),
                    );
                }

                log::warn!("ShredStream stream ended - reconnecting in {delay_ms}ms");
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                delay_ms = (delay_ms * 2).min(60_000);
            }
        });

        let mut metrics_handle = None;
        if self.config.enable_metrics {
            metrics_handle = MetricsManager::global().start_auto_monitoring().await;
        }

        // 保存订阅句柄
        let subscription_handle = SubscriptionHandle::new(stream_task, None, metrics_handle);
        let mut handle_guard = self.subscription_handle.lock().await;
        *handle_guard = Some(subscription_handle);

        Ok(())
    }
}

#[inline]
fn process_shred_entry_direct(
    entry: crate::protos::shredstream::Entry,
    protocols: &[Protocol],
    event_type_filter: Option<&EventTypeFilter>,
    bot_wallet: Option<Pubkey>,
    callback: &(dyn Fn(DexEvent) + Send + Sync),
) {
    let slot = entry.slot;
    let recv_us = get_high_perf_clock();
    let entries = match bincode::deserialize::<Vec<SolanaEntry>>(&entry.entries) {
        Ok(entries) => entries,
        Err(error) => {
            log::debug!("Failed to deserialize ShredStream entries: {error}");
            return;
        }
    };

    let mut tx_index = 0u64;
    for entry in entries {
        for transaction in &entry.transactions {
            if transaction.signatures.is_empty() {
                tx_index += 1;
                continue;
            }

            MetricsManager::global().add_tx_process_count();
            let signature = transaction.signatures[0];
            parse_shred_transaction_events(
                transaction,
                signature,
                slot,
                Some(tx_index),
                recv_us,
                protocols,
                event_type_filter,
                bot_wallet,
                |event| {
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
                },
            );
            tx_index += 1;
        }
    }
}
