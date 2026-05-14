//! Block time and `recv_us` alignment with streamer [`EventMetadata`].
use crate::streaming::event_parser::common::types::{EventType, ProtocolType};
use crate::streaming::event_parser::common::EventMetadata;
use crate::streaming::event_parser::DexEvent;
use prost_types::Timestamp;
use solana_sdk::pubkey::Pubkey;

/// Build a prost `Timestamp` from streamer `EventMetadata`.
pub(crate) fn block_timestamp_from_stream_meta(meta: &EventMetadata) -> Timestamp {
    let sec = meta.block_time;
    let rem_ms = meta.block_time_ms.saturating_sub(sec.saturating_mul(1000));
    let nanos = rem_ms.saturating_mul(1_000_000).min(999_999_999) as i32;
    Timestamp { seconds: sec, nanos }
}

/// Preserve outer parser instruction indexes and optional blockhash when the SDK metadata has no
/// equivalent context.
pub(crate) fn fuse_streamer_ix_ctx(mut ev: DexEvent, sm: &EventMetadata) -> DexEvent {
    let m = ev.metadata_mut();
    m.outer_index = sm.outer_index;
    m.inner_index = sm.inner_index;
    if sm.recent_blockhash.is_some() {
        m.recent_blockhash = sm.recent_blockhash.clone();
    }
    ev
}

pub(crate) fn adapt_pm(
    pm: sol_parser_sdk::core::events::EventMetadata,
    bt: Option<&Timestamp>,
    recv_wall_us: i64,
    proto: ProtocolType,
    et: EventType,
    program_id: Pubkey,
) -> EventMetadata {
    let block_time_sec = bt.map(|t| t.seconds).unwrap_or_else(|| pm.block_time_us / 1_000_000);
    let block_time_ms = bt
        .map(|t| t.seconds * 1000 + t.nanos as i64 / 1_000_000)
        .unwrap_or(pm.block_time_us / 1000);
    EventMetadata::new(
        pm.signature,
        pm.slot,
        block_time_sec,
        block_time_ms,
        proto,
        et,
        program_id,
        0,
        None,
        recv_wall_us,
        Some(pm.tx_index),
        pm.recent_blockhash.clone(),
    )
}
