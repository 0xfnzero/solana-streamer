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
use sol_parser_sdk::{parse_rpc_transaction, parse_transaction_from_rpc};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::signature::Signature;
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding};

use crate::streaming::event_parser::common::filter::{
    build_sdk_parse_event_filter, EventTypeFilter,
};
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
    let sdk_filter = build_sdk_parse_event_filter(event_type_filter);
    let pb_events = parse_rpc_transaction(rpc_tx, sdk_filter.as_ref())?;
    let block_ts = rpc_tx.block_time.map(|sec| Timestamp { seconds: sec, nanos: 0 });
    Ok(adapt_parser_events_list(
        pb_events,
        block_ts.as_ref(),
        recv_wall_us,
        protocols,
        event_type_filter,
    ))
}

/// Blocking RPC fetch by signature, then adapt SDK events to streamer events.
pub fn fetch_rpc_transaction_as_streamer_events(
    rpc_client: &RpcClient,
    signature: &Signature,
    recv_wall_us: i64,
    protocols: &[Protocol],
    event_type_filter: Option<&EventTypeFilter>,
) -> Result<Vec<DexEvent>, ParseError> {
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
        max_supported_transaction_version: Some(0),
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
