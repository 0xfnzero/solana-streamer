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
use solana_sdk::signature::Signature;
use solana_sdk::transaction::VersionedTransaction;
use std::cell::RefCell;
use std::sync::Arc;
use yellowstone_grpc_proto::geyser::SubscribeUpdateTransaction;

thread_local! {
    static SHRED_SDK_EVENTS: RefCell<Vec<sol_parser_sdk::DexEvent>> =
        RefCell::new(Vec::with_capacity(4));
}

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
    let update = SubscribeUpdateTransaction { slot, transaction: Some(grpc_tx) };
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
                    callback(*event);
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
    parse_shred_transaction_events(
        &tx,
        signature,
        slot,
        tx_index,
        recv_us,
        protocols,
        event_type_filter,
        bot_wallet,
        |event| adapter_callback(event),
    );

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn parse_shred_transaction_events(
    tx: &VersionedTransaction,
    signature: Signature,
    slot: u64,
    tx_index: Option<u64>,
    recv_us: i64,
    protocols: &[Protocol],
    event_type_filter: Option<&EventTypeFilter>,
    bot_wallet: Option<Pubkey>,
    mut on_event: impl FnMut(DexEvent),
) {
    let sdk_parse_filter = build_sdk_shred_parse_event_filter(protocols, event_type_filter);

    SHRED_SDK_EVENTS.with(|slot_events| {
        let mut events = {
            let mut slot_events = slot_events.borrow_mut();
            std::mem::take(&mut *slot_events)
        };
        events.clear();
        sol_parser_sdk::shredstream::parse_transaction_dex_events_with_filter(
            tx,
            signature,
            slot,
            tx_index.unwrap_or(0),
            recv_us,
            sdk_parse_filter.as_ref(),
            &mut events,
        );

        for sdk_event in events.drain(..) {
            if let Some(mut event) =
                adapt_parser_event(sdk_event, None, recv_us, protocols, event_type_filter)
            {
                event.metadata_mut().tx_index = tx_index;
                event.metadata_mut().handle_us = elapsed_micros_since(recv_us);
                event = crate::streaming::event_parser::core::event_parser::helpers::process_event(
                    event, bot_wallet,
                );
                on_event(event);
            }
        }

        *slot_events.borrow_mut() = events;
    });
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

#[cfg(test)]
mod tests {
    use super::process_shred_transaction;
    use crate::streaming::event_parser::{DexEvent, Protocol};
    use crate::streaming::shred::TransactionWithSlot;
    use sol_parser_sdk::instr::program_ids::PUMPFUN_PROGRAM_ID;
    use solana_sdk::hash::Hash;
    use solana_sdk::message::{
        compiled_instruction::CompiledInstruction, v0, MessageHeader, VersionedMessage,
    };
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::signature::Signature;
    use solana_sdk::transaction::VersionedTransaction;
    use std::sync::{Arc, Mutex};

    fn push_string(data: &mut Vec<u8>, value: &str) {
        data.extend_from_slice(&(value.len() as u32).to_le_bytes());
        data.extend_from_slice(value.as_bytes());
    }

    fn pumpfun_create_data() -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&[24, 30, 200, 40, 5, 28, 7, 119]);
        push_string(&mut data, "Index Test");
        push_string(&mut data, "IDX");
        push_string(&mut data, "https://example.invalid/index.json");
        data.extend_from_slice(Pubkey::new_unique().as_ref());
        data
    }

    fn pumpfun_create_tx() -> VersionedTransaction {
        let mut account_keys = (0..10).map(|_| Pubkey::new_unique()).collect::<Vec<_>>();
        account_keys.push(PUMPFUN_PROGRAM_ID);

        VersionedTransaction {
            signatures: vec![Signature::default()],
            message: VersionedMessage::V0(v0::Message {
                header: MessageHeader {
                    num_required_signatures: 1,
                    num_readonly_signed_accounts: 0,
                    num_readonly_unsigned_accounts: 0,
                },
                account_keys,
                recent_blockhash: Hash::default(),
                instructions: vec![CompiledInstruction::new_from_raw_parts(
                    10,
                    pumpfun_create_data(),
                    (0..10).collect(),
                )],
                address_table_lookups: Vec::new(),
            }),
        }
    }

    async fn parse_single_shred_create(tx_index: Option<u64>) -> DexEvent {
        let events = Arc::new(Mutex::new(Vec::new()));
        let captured = events.clone();
        let callback = Arc::new(move |event| {
            captured.lock().unwrap().push(event);
        });

        process_shred_transaction(
            TransactionWithSlot::new(pumpfun_create_tx(), 42, 1_000_000, tx_index),
            &[Protocol::PumpFun],
            None,
            callback,
            None,
        )
        .await
        .expect("process shred transaction");

        let mut events = events.lock().unwrap();
        assert_eq!(events.len(), 1);
        events.pop().unwrap()
    }

    #[tokio::test]
    async fn shred_transaction_preserves_unknown_tx_index() {
        let event = parse_single_shred_create(None).await;

        assert!(matches!(event, DexEvent::PumpFunCreateTokenEvent(_)));
        assert_eq!(event.metadata().tx_index, None);
    }

    #[tokio::test]
    async fn shred_transaction_preserves_present_tx_index() {
        let event = parse_single_shred_create(Some(42)).await;

        assert!(matches!(event, DexEvent::PumpFunCreateTokenEvent(_)));
        assert_eq!(event.metadata().tx_index, Some(42));
    }
}
