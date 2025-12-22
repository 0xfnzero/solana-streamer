use std::sync::Arc;

use futures::StreamExt;
use solana_sdk::pubkey::Pubkey;

use crate::common::AnyResult;
use crate::protos::shredstream::SubscribeEntriesRequest;
use crate::streaming::common::{process_shred_transaction, SubscriptionHandle};
use crate::streaming::event_parser::common::filter::EventTypeFilter;
use crate::streaming::event_parser::common::high_performance_clock::get_high_perf_clock;
use crate::streaming::event_parser::{Protocol, DexEvent};
use crate::streaming::grpc::MetricsManager;
use crate::streaming::shred::pool::factory;
use log::error;
use solana_entry::entry::Entry;

use super::ShredStreamGrpc;

impl ShredStreamGrpc {
    /// Subscribe to ShredStream events (supports batch and real-time processing)
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
        // If there's an active subscription, stop it first
        self.stop().await;

        let mut metrics_handle = None;
        // Start automatic performance monitoring (if enabled)
        if self.config.enable_metrics {
            metrics_handle = MetricsManager::global().start_auto_monitoring().await;
        }

        // Start stream processing
        let mut client = (*self.shredstream_client).clone();
        let request = tonic::Request::new(SubscribeEntriesRequest {});
        let mut stream = client.subscribe_entries(request).await?.into_inner();

        // Wrap callback once before the async block
        let callback = Arc::new(callback);

        let stream_task = tokio::spawn(async move {
            while let Some(message) = stream.next().await {
                match message {
                    Ok(msg) => {
                        if let Ok(entries) = bincode::deserialize::<Vec<Entry>>(&msg.entries) {
                            for entry in entries {
                                for transaction in entry.transactions {
                                    let transaction_with_slot =
                                        factory::create_transaction_with_slot_pooled(
                                            transaction.clone(),
                                            msg.slot,
                                            get_high_perf_clock(),
                                        );
                                    // Process transaction - clone Arc and Vec for each call
                                    if let Err(e) = process_shred_transaction(
                                        transaction_with_slot,
                                        &protocols,
                                        event_type_filter.as_ref(),
                                        callback.clone(),
                                        bot_wallet,
                                    )
                                    .await
                                    {
                                        error!("Error handling message: {e:?}");
                                    }
                                }
                            }
                        }
                        continue;
                    }
                    Err(error) => {
                        error!("Stream error: {error:?}");
                        break;
                    }
                }
            }
        });

        // Save subscription handle
        let subscription_handle = SubscriptionHandle::new(stream_task, None, metrics_handle);
        let mut handle_guard = self.subscription_handle.lock().await;
        *handle_guard = Some(subscription_handle);

        Ok(())
    }
}
