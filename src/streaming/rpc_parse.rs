//! Single RPC transaction parsing backed by `sol-parser-sdk`, adapted to streamer
//! [`DexEvent`](crate::streaming::event_parser::DexEvent).
//!
//! - Filter mapping uses [`crate::streaming::event_parser::common::filter::build_sdk_parse_event_filter`].
//! - Works with an existing [`EncodedConfirmedTransactionWithStatusMeta`], async fetch, or blocking
//!   [`RpcClient`] fetch.
//!
//! Async callers can also fetch with their own client and call
//! [`parse_encoded_rpc_transaction_as_streamer_events`].

use prost_types::Timestamp;
use sol_parser_sdk::{
    parse_rpc_transaction, parse_rpc_transaction_cost_with_signature,
    parse_rpc_transaction_with_cost, parse_transaction_from_rpc,
};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::signature::Signature;
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding};

use crate::streaming::event_parser::common::filter::{
    build_sdk_parse_event_filter, transaction_cost_selection, EventTypeFilter,
};
use crate::streaming::event_parser::core::transaction_cost_event::TransactionCostEvent;
use crate::streaming::event_parser::{DexEvent, Protocol};
use crate::streaming::parser_sdk_bridge::adapt_parser_events_list;
pub use sol_parser_sdk::ParseError;

/// Parse a transaction payload already returned by RPC.
///
/// `recv_wall_us` should be the caller's UNIX microsecond receive timestamp.
pub fn parse_encoded_rpc_transaction_as_streamer_events(
    rpc_tx: &EncodedConfirmedTransactionWithStatusMeta,
    recv_wall_us: i64,
    protocols: &[Protocol],
    event_type_filter: Option<&EventTypeFilter>,
) -> Result<Vec<DexEvent>, ParseError> {
    let block_ts = rpc_tx.block_time.map(|sec| Timestamp { seconds: sec, nanos: 0 });
    let cost_selection = transaction_cost_selection(event_type_filter);
    let (mut events, transaction_cost) = if cost_selection.only {
        (Vec::with_capacity(1), Some(parse_rpc_transaction_cost_with_signature(rpc_tx)?))
    } else {
        let sdk_filter = build_sdk_parse_event_filter(event_type_filter);
        let (pb_events, cost) = if cost_selection.requested {
            let parsed = parse_rpc_transaction_with_cost(rpc_tx, sdk_filter.as_ref())?;
            (parsed.events, Some((parsed.cost, parsed.signature)))
        } else {
            (parse_rpc_transaction(rpc_tx, sdk_filter.as_ref())?, None)
        };
        (
            adapt_parser_events_list(
                pb_events,
                block_ts.as_ref(),
                recv_wall_us,
                protocols,
                event_type_filter,
            ),
            cost,
        )
    };
    if let Some((cost, signature)) = transaction_cost {
        let block_time_us = rpc_tx.block_time.map(|seconds| seconds * 1_000_000);
        events.push(DexEvent::TransactionCostEvent(TransactionCostEvent::from_parser(
            cost,
            signature,
            rpc_tx.slot,
            None,
            block_time_us,
            recv_wall_us,
            None,
        )));
    }
    Ok(events)
}

/// Blocking RPC fetch by signature, then adapt SDK events to streamer events.
pub fn fetch_rpc_transaction_as_streamer_events(
    rpc_client: &RpcClient,
    signature: &Signature,
    recv_wall_us: i64,
    protocols: &[Protocol],
    event_type_filter: Option<&EventTypeFilter>,
) -> Result<Vec<DexEvent>, ParseError> {
    if transaction_cost_selection(event_type_filter).requested {
        let config = RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::Base64),
            commitment: None,
            max_supported_transaction_version: Some(1),
        };
        let rpc_tx = rpc_client
            .get_transaction_with_config(signature, config)
            .map_err(|error| map_async_rpc_err(error.to_string()))?;
        return parse_encoded_rpc_transaction_as_streamer_events(
            &rpc_tx,
            recv_wall_us,
            protocols,
            event_type_filter,
        );
    }
    let sdk_filter = build_sdk_parse_event_filter(event_type_filter);
    let pb_events = parse_transaction_from_rpc(rpc_client, signature, sdk_filter.as_ref())?;
    // The SDK already writes block_time_us into each event; adapter falls back to it when
    // no prost Timestamp is available.
    Ok(adapt_parser_events_list(pb_events, None, recv_wall_us, protocols, event_type_filter))
}

/// Async RPC fetch using the same request config as the SDK blocking helper.
pub async fn fetch_rpc_transaction_as_streamer_events_async(
    rpc_client: &solana_client::nonblocking::rpc_client::RpcClient,
    signature: &Signature,
    recv_wall_us: i64,
    protocols: &[Protocol],
    event_type_filter: Option<&EventTypeFilter>,
) -> Result<Vec<DexEvent>, ParseError> {
    let config = RpcTransactionConfig {
        encoding: Some(UiTransactionEncoding::Base64),
        commitment: None,
        max_supported_transaction_version: Some(1),
    };
    let rpc_tx = rpc_client
        .get_transaction_with_config(signature, config)
        .await
        .map_err(|e| map_async_rpc_err(e.to_string()))?;
    parse_encoded_rpc_transaction_as_streamer_events(
        &rpc_tx,
        recv_wall_us,
        protocols,
        event_type_filter,
    )
}

#[inline]
fn map_async_rpc_err(msg: String) -> ParseError {
    if msg.contains("invalid type: null")
        && msg.contains("EncodedConfirmedTransactionWithStatusMeta")
    {
        ParseError::RpcError(format!(
            "Transaction not found (RPC returned null). Common causes: 1) Transaction is too old and pruned (use an archive RPC). 2) Wrong network or invalid signature. Try an archive endpoint or a more recent tx. Original: {}",
            msg
        ))
    } else {
        ParseError::RpcError(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming::event_parser::common::filter::EventTypeFilter;
    use crate::streaming::event_parser::common::EventType;
    use sol_parser_sdk::SwqosProvider;
    use solana_client::rpc_client::RpcClient;
    use solana_client::rpc_response::UiTransactionStatusMeta;
    use solana_sdk::{
        hash::Hash,
        message::{v1, MessageHeader, VersionedMessage},
        pubkey::Pubkey,
        transaction::VersionedTransaction,
    };
    use solana_transaction_status::{
        option_serializer::OptionSerializer, Encodable, EncodedTransactionWithStatusMeta,
    };
    use std::str::FromStr;

    fn local_rpc_fixture() -> EncodedConfirmedTransactionWithStatusMeta {
        let transaction = VersionedTransaction {
            signatures: vec![Signature::from([7; 64])],
            message: VersionedMessage::V1(v1::Message {
                header: MessageHeader { num_required_signatures: 1, ..Default::default() },
                config: v1::TransactionConfig::empty()
                    .with_compute_unit_limit(200_000)
                    .with_priority_fee(1_000),
                lifetime_specifier: Hash::new_unique(),
                account_keys: vec![Pubkey::new_unique()],
                instructions: Vec::new(),
            }),
        };
        EncodedConfirmedTransactionWithStatusMeta {
            slot: 42,
            transaction: EncodedTransactionWithStatusMeta {
                transaction: transaction.encode(UiTransactionEncoding::Base64),
                meta: Some(UiTransactionStatusMeta {
                    err: None,
                    status: Ok(()),
                    fee: 6_000,
                    pre_balances: vec![1_000_000],
                    post_balances: vec![994_000],
                    inner_instructions: OptionSerializer::None,
                    log_messages: OptionSerializer::None,
                    pre_token_balances: OptionSerializer::None,
                    post_token_balances: OptionSerializer::None,
                    rewards: OptionSerializer::None,
                    loaded_addresses: OptionSerializer::None,
                    return_data: OptionSerializer::None,
                    compute_units_consumed: OptionSerializer::Some(10_000),
                    cost_units: OptionSerializer::None,
                }),
                version: None,
            },
            block_time: Some(1_700_000_000),
            transaction_index: Some(0),
        }
    }

    #[test]
    fn local_rpc_cost_only_and_combined_filters_preserve_cost_output() {
        let rpc_tx = local_rpc_fixture();
        for filter in [
            EventTypeFilter::include_only([EventType::TransactionCost]),
            EventTypeFilter::include_only([EventType::PumpFunBuy, EventType::TransactionCost]),
        ] {
            let events =
                parse_encoded_rpc_transaction_as_streamer_events(&rpc_tx, 123, &[], Some(&filter))
                    .expect("parse local RPC fixture");
            assert_eq!(events.len(), 1);
            let DexEvent::TransactionCostEvent(cost) = &events[0] else {
                panic!("expected transaction cost event");
            };
            assert_eq!(cost.metadata.signature, Signature::from([7; 64]));
            assert_eq!(cost.compute_unit_limit, Some(200_000));
            assert_eq!(cost.priority_fee_lamports, Some(1_000));
        }
    }

    #[test]
    fn current_mainnet_transaction_cost_is_reusable() {
        if std::env::var_os("RUN_MAINNET_TESTS").is_none() {
            return;
        }
        const SIGNATURE: &str =
            "4yaaD6ywu8epxVTvZEDAGPhdKK2V73XqvLqQWm1KbSFQ1uTk2nnC4uW7xTrpSuQYpTivmDQQawu7x3dFbYC1KuZ6";
        let rpc_url = std::env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
        let signature = Signature::from_str(SIGNATURE).expect("valid fixture signature");
        let filter = EventTypeFilter::include_only([EventType::TransactionCost]);
        let events = fetch_rpc_transaction_as_streamer_events(
            &RpcClient::new(rpc_url),
            &signature,
            0,
            &[],
            Some(&filter),
        )
        .expect("parse current transaction cost");

        assert_eq!(events.len(), 1);
        let DexEvent::TransactionCostEvent(cost) = &events[0] else {
            panic!("expected transaction cost event");
        };
        assert_eq!(cost.metadata.slot, 438_900_232);
        assert_eq!(cost.metadata.signature, signature);
        assert_eq!(cost.transaction_fee_lamports, Some(29_242));
        assert_eq!(cost.compute_units_consumed, Some(135_026));
        assert_eq!(cost.compute_unit_limit, Some(300_000));
        assert_eq!(cost.compute_unit_price_micro_lamports, Some(80_805));
        assert_eq!(cost.priority_fee_lamports, Some(24_242));
        assert_eq!(cost.tip_lamports, 137_273);
        assert_eq!(cost.total_fee_and_tip_lamports, Some(166_515));
        assert_eq!(cost.tip_lamports_for(SwqosProvider::Jito), 137_273);
    }
}
