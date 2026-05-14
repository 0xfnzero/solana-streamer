use crate::common::AnyResult;
use crate::streaming::common::MetricsEventType;
use crate::streaming::event_parser::common::filter::{passes_event_type_filter, EventTypeFilter};
use crate::streaming::event_parser::common::high_performance_clock::elapsed_micros_since;
use crate::streaming::event_parser::core::account_event_parser::AccountEventParser;
use crate::streaming::event_parser::core::common_event_parser::CommonEventParser;
use crate::streaming::event_parser::core::event_parser::EventParser;
use crate::streaming::event_parser::{core::traits::DexEvent, Protocol};
use crate::streaming::grpc::{EventPretty, MetricsManager};
use crate::streaming::parser_sdk_bridge::{parse_account_event_for_streamer, AccountParseResult};
use crate::streaming::shred::TransactionWithSlot;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;

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
                AccountParseResult::Unsupported => {}
            }

            let account_event = AccountEventParser::parse_account_event(
                protocols,
                account_pretty,
                event_type_filter,
            );

            if let Some(event) = account_event {
                let processing_time_us = event.metadata().handle_us as f64;
                callback(event);
                update_metrics(MetricsEventType::Account, 1, processing_time_us);
            }
        }
        EventPretty::Transaction(transaction_pretty) => {
            MetricsManager::global().add_tx_process_count();

            let slot = transaction_pretty.slot;
            let signature = transaction_pretty.signature;
            let block_time = transaction_pretty.block_time;
            let recv_us = transaction_pretty.recv_us;
            let tx_index = transaction_pretty.tx_index;
            let grpc_tx = transaction_pretty.grpc_tx;

            let adapter_callback = create_metrics_callback(callback.clone());

            EventParser::parse_grpc_transaction(
                protocols,
                event_type_filter,
                grpc_tx,
                signature,
                Some(slot),
                block_time,
                recv_us,
                bot_wallet,
                tx_index,
                adapter_callback,
            )
            .await?;
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
    // Shred only exposes static account keys and no inner instructions; see
    // docs/SHREDSTREAM_LIMITATIONS.md for the expected parser limits.
    let accounts = tx.message.static_account_keys();

    EventParser::parse_instruction_events_from_versioned_transaction(
        protocols,
        event_type_filter,
        &tx,
        signature,
        Some(slot),
        None, // shred has no block_time
        recv_us,
        accounts,
        &[],
        bot_wallet,
        tx_index,
        adapter_callback,
    )
    .await?;

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
