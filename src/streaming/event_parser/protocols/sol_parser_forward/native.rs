//! 将 `sol-parser-sdk` 的 Orca Whirlpool / Meteora Pools / DLMM 顶层与 inner 指令解析接到 streamer `DexEvent`。

use crate::streaming::event_parser::common::EventMetadata;
use crate::streaming::event_parser::{DexEvent, Protocol};
use crate::streaming::parser_sdk_bridge::{
    block_timestamp_from_stream_meta, convert_parser_event, fuse_streamer_ix_ctx,
};
use sol_parser_sdk::core::events::EventMetadata as PbEventMetadata;
use sol_parser_sdk::instr::all_inner::{
    meteora_amm as pools_inner, meteora_dlmm as dlmm_inner, orca as orca_inner,
};
use sol_parser_sdk::instr::{meteora_amm, meteora_dlmm, orca_whirlpool};
use solana_sdk::pubkey::Pubkey;

#[inline]
fn block_time_us_for_sdk(sm: &EventMetadata) -> Option<i64> {
    Some(sm.block_time_ms.saturating_mul(1000))
}

#[inline]
fn pb_meta_from_streamer(sm: &EventMetadata) -> PbEventMetadata {
    PbEventMetadata {
        signature: sm.signature,
        slot: sm.slot,
        tx_index: sm.tx_index.unwrap_or(0),
        block_time_us: sm.block_time_ms.saturating_mul(1000),
        grpc_recv_us: sm.recv_us,
        recent_blockhash: sm.recent_blockhash.clone(),
    }
}

pub fn dispatch_instruction(
    protocol: Protocol,
    instruction_discriminator: &[u8],
    instruction_data: &[u8],
    accounts: &[Pubkey],
    stream_meta: &EventMetadata,
) -> Option<DexEvent> {
    let mut full = Vec::with_capacity(instruction_discriminator.len() + instruction_data.len());
    full.extend_from_slice(instruction_discriminator);
    full.extend_from_slice(instruction_data);

    let tx_index = stream_meta.tx_index.unwrap_or(0);
    let bt_us = block_time_us_for_sdk(stream_meta);

    let pb = match protocol {
        Protocol::OrcaWhirlpool => orca_whirlpool::parse_instruction(
            &full,
            accounts,
            stream_meta.signature,
            stream_meta.slot,
            tx_index,
            bt_us,
        )?,
        Protocol::MeteoraPools => meteora_amm::parse_instruction(
            &full,
            accounts,
            stream_meta.signature,
            stream_meta.slot,
            tx_index,
            bt_us,
        )?,
        Protocol::MeteoraDlmm => meteora_dlmm::parse_instruction(
            &full,
            accounts,
            stream_meta.signature,
            stream_meta.slot,
            tx_index,
            bt_us,
        )?,
        _ => return None,
    };

    let ts = block_timestamp_from_stream_meta(stream_meta);
    let ev = convert_parser_event(pb, Some(&ts), stream_meta.recv_us)?;
    Some(fuse_streamer_ix_ctx(ev, stream_meta))
}

pub fn dispatch_inner_instruction(
    protocol: Protocol,
    inner_instruction_discriminator: &[u8],
    inner_instruction_data: &[u8],
    stream_meta: &EventMetadata,
) -> Option<DexEvent> {
    let disc: [u8; 16] = inner_instruction_discriminator.try_into().ok()?;
    let pm = pb_meta_from_streamer(stream_meta);

    let pb = match protocol {
        Protocol::OrcaWhirlpool => orca_inner::parse(&disc, inner_instruction_data, pm)?,
        Protocol::MeteoraPools => pools_inner::parse(&disc, inner_instruction_data, pm)?,
        Protocol::MeteoraDlmm => dlmm_inner::parse(&disc, inner_instruction_data, pm)?,
        _ => return None,
    };

    let ts = block_timestamp_from_stream_meta(stream_meta);
    let ev = convert_parser_event(pb, Some(&ts), stream_meta.recv_us)?;
    Some(fuse_streamer_ix_ctx(ev, stream_meta))
}
