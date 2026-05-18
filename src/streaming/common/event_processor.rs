use crate::common::AnyResult;
use crate::streaming::common::MetricsEventType;
use crate::streaming::event_parser::common::filter::{
    build_sdk_parse_event_filter, build_sdk_shred_parse_event_filter, passes_event_type_filter,
    EventTypeFilter,
};
use crate::streaming::event_parser::common::high_performance_clock::elapsed_micros_since;
use crate::streaming::event_parser::core::common_event_parser::CommonEventParser;
use crate::streaming::event_parser::{core::traits::DexEvent, Protocol};
use crate::streaming::grpc::{EventPretty, MetricsManager};
use crate::streaming::parser_sdk_bridge::{
    adapt_parser_event, adapt_parser_events_list, parse_account_event_for_streamer,
    AccountParseResult,
};
use crate::streaming::shred::TransactionWithSlot;
use sol_parser_sdk::grpc::parse_subscribe_update_transaction_low_latency;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use yellowstone_grpc_proto::geyser::SubscribeUpdateTransaction;

/// Wrap the user callback and update transaction metrics after delivery.
#[inline]
fn create_metrics_callback(
    callback: Arc<dyn Fn(DexEvent) + Send + Sync>,
) -> Arc<dyn Fn(DexEvent) + Send + Sync> {
    Arc::new(move |event: DexEvent| {
        let metadata = event.metadata();
        let processing_time_us = metadata.handle_us as f64;
        let recv_us = metadata.recv_us;
        let block_time_ms = metadata.block_time_ms;

        callback(event);

        update_metrics_with_latency(
            MetricsEventType::Transaction,
            1,
            processing_time_us,
            recv_us,
            block_time_ms,
        );
    })
}

#[inline]
pub fn transaction_metrics_callback(
    callback: Arc<dyn Fn(DexEvent) + Send + Sync>,
) -> Arc<dyn Fn(DexEvent) + Send + Sync> {
    create_metrics_callback(callback)
}

pub fn parse_grpc_transaction_events(
    transaction_pretty: crate::streaming::grpc::TransactionPretty,
    protocols: &[Protocol],
    event_type_filter: Option<&EventTypeFilter>,
    bot_wallet: Option<Pubkey>,
) -> Vec<DexEvent> {
    MetricsManager::global().add_tx_process_count();

    let slot = transaction_pretty.slot;
    let block_time = transaction_pretty.block_time;
    let recv_us = transaction_pretty.recv_us;
    let grpc_tx = transaction_pretty.grpc_tx;
    let block_time_us = block_time.map(|t| t.seconds * 1_000_000 + t.nanos as i64 / 1_000);
    let update =
        SubscribeUpdateTransaction { slot, transaction: Some(grpc_tx), ..Default::default() };
    let sdk_parse_filter = build_sdk_parse_event_filter(event_type_filter);
    let sdk_events = parse_subscribe_update_transaction_low_latency(
        &update,
        recv_us,
        block_time_us,
        sdk_parse_filter.as_ref(),
    );
    let mut events = adapt_parser_events_list(
        sdk_events,
        block_time.as_ref(),
        recv_us,
        protocols,
        event_type_filter,
    );

    for event in events.iter_mut() {
        event.metadata_mut().handle_us = elapsed_micros_since(recv_us);
    }

    events
        .into_iter()
        .map(|event| {
            crate::streaming::event_parser::core::event_parser::helpers::process_event(
                event, bot_wallet,
            )
        })
        .collect()
}

/// Process GRPC transaction events
pub async fn process_grpc_transaction(
    event_pretty: EventPretty,
    protocols: &[Protocol],
    event_type_filter: Option<&EventTypeFilter>,
    callback: Arc<dyn Fn(DexEvent) + Send + Sync>,
    bot_wallet: Option<Pubkey>,
) -> AnyResult<()> {
    match event_pretty {
        EventPretty::Account(account_pretty) => {
            MetricsManager::global().add_account_process_count();

            match parse_account_event_for_streamer(&account_pretty, protocols, event_type_filter) {
                AccountParseResult::Event(mut event) => {
                    event.metadata_mut().handle_us = elapsed_micros_since(account_pretty.recv_us);
                    let processing_time_us = event.metadata().handle_us as f64;
                    callback(event);
                    update_metrics(MetricsEventType::Account, 1, processing_time_us);
                    return Ok(());
                }
                AccountParseResult::Filtered => return Ok(()),
                AccountParseResult::Unsupported => return Ok(()),
            }
        }
        EventPretty::Transaction(transaction_pretty) => {
            let callback = transaction_metrics_callback(callback);
            for event in parse_grpc_transaction_events(
                transaction_pretty,
                protocols,
                event_type_filter,
                bot_wallet,
            ) {
                callback(event);
            }
        }
        EventPretty::BlockMeta(block_meta_pretty) => {
            MetricsManager::global().add_block_meta_process_count();

            let block_time_ms = block_meta_pretty
                .block_time
                .map(|ts| ts.seconds * 1000 + ts.nanos as i64 / 1_000_000)
                .unwrap_or_else(|| {
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as i64
                });

            let block_meta_event = CommonEventParser::generate_block_meta_event(
                block_meta_pretty.slot,
                block_meta_pretty.block_hash,
                block_time_ms,
                block_meta_pretty.recv_us,
            );

            if !passes_event_type_filter(event_type_filter, &block_meta_event) {
                return Ok(());
            }

            let processing_time_us = block_meta_event.metadata().handle_us as f64;
            callback(block_meta_event);
            update_metrics(MetricsEventType::BlockMeta, 1, processing_time_us);
        }
    }

    Ok(())
}

/// Process Shred transaction events
pub async fn process_shred_transaction(
    transaction_with_slot: TransactionWithSlot,
    protocols: &[Protocol],
    event_type_filter: Option<&EventTypeFilter>,
    callback: Arc<dyn Fn(DexEvent) + Send + Sync>,
    bot_wallet: Option<Pubkey>,
) -> AnyResult<()> {
    MetricsManager::global().add_tx_process_count();

    let tx = transaction_with_slot.transaction;
    let slot = transaction_with_slot.slot;
    let tx_index = transaction_with_slot.tx_index;

    if tx.signatures.is_empty() {
        return Ok(());
    }

    let signature = tx.signatures[0];
    let recv_us = transaction_with_slot.recv_us;

    let adapter_callback = create_metrics_callback(callback);
    let sdk_parse_filter = build_sdk_shred_parse_event_filter(protocols, event_type_filter);
    let mut sdk_events = Vec::with_capacity(4);
    sol_parser_sdk::shredstream::parse_transaction_dex_events_with_filter(
        &tx,
        signature,
        slot,
        tx_index.unwrap_or(0),
        recv_us,
        sdk_parse_filter.as_ref(),
        &mut sdk_events,
    );

    for sdk_event in sdk_events {
        if let Some(mut event) =
            adapt_parser_event(sdk_event, None, recv_us, protocols, event_type_filter)
        {
            event.metadata_mut().handle_us = elapsed_micros_since(recv_us);
            event = crate::streaming::event_parser::core::event_parser::helpers::process_event(
                event, bot_wallet,
            );
            adapter_callback(event);
        }
    }

    Ok(())
}

/// Update metrics for event processing (with optional latency check)
#[inline]
fn update_metrics(ty: MetricsEventType, count: u64, time_us: f64) {
    MetricsManager::global().update_metrics(ty, count, time_us);
}

/// Update metrics with latency check
#[inline]
fn update_metrics_with_latency(
    ty: MetricsEventType,
    count: u64,
    time_us: f64,
    recv_us: i64,
    block_time_ms: i64,
) {
    MetricsManager::global().update_metrics_with_latency(
        ty,
        count,
        time_us,
        recv_us,
        block_time_ms,
    );
}
