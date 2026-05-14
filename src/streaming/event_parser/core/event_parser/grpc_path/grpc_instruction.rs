//! Single Yellowstone [`CompiledInstruction`] parsing: dispatch, inner merge, swap enrichment.
use crate::streaming::event_parser::{
    common::{
        filter::{passes_event_type_filter, EventTypeFilter},
        high_performance_clock::elapsed_micros_since,
        parse_swap_data_from_next_grpc_instructions, EventMetadata,
    },
    core::{dispatcher::EventDispatcher, merger_event::merge},
    protocols::{
        raydium_amm_v4::parser::RAYDIUM_AMM_V4_PROGRAM_ID,
        sol_parser_forward::METEORA_DLMM_PROGRAM_ID,
    },
    DexEvent, Protocol,
};
use prost_types::Timestamp;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::sync::Arc;

pub(super) fn parse_events_from_grpc_instruction(
    protocols: &[Protocol],
    event_type_filter: Option<&EventTypeFilter>,
    instruction: &yellowstone_grpc_proto::prelude::CompiledInstruction,
    accounts: &[Pubkey],
    signature: Signature,
    slot: u64,
    block_time: Option<Timestamp>,
    recv_us: i64,
    outer_index: i64,
    inner_index: Option<i64>,
    bot_wallet: Option<Pubkey>,
    tx_index: Option<u64>,
    recent_blockhash: Option<&str>,
    inner_instructions: Option<&yellowstone_grpc_proto::prelude::InnerInstructions>,
    callback: Arc<dyn Fn(DexEvent) + Send + Sync>,
) -> anyhow::Result<()> {
    // Bounds check before reading the program id index.
    let program_id_index = instruction.program_id_index as usize;
    if program_id_index >= accounts.len() {
        return Ok(());
    }
    let program_id = accounts[program_id_index];
    if !super::super::helpers::should_handle(protocols, event_type_filter, &program_id) {
        return Ok(());
    }

    let is_cu_program = EventDispatcher::is_compute_budget_program(&program_id);

    let disc_len = match program_id {
        RAYDIUM_AMM_V4_PROGRAM_ID | METEORA_DLMM_PROGRAM_ID => 1,
        _ => 8,
    };

    // Non-ComputeBudget instructions need at least a discriminator.
    if !is_cu_program && instruction.data.len() < disc_len {
        return Ok(());
    }
    // Build streamer metadata.
    let timestamp = block_time.unwrap_or(Timestamp { seconds: 0, nanos: 0 });
    let block_time_ms = timestamp.seconds * 1000 + (timestamp.nanos as i64) / 1_000_000;
    let metadata = EventMetadata::new(
        signature,
        slot,
        timestamp.seconds,
        block_time_ms,
        Default::default(), // protocol will be set by dispatcher
        Default::default(), // event_type will be set by dispatcher
        program_id,
        outer_index,
        inner_index,
        recv_us,
        tx_index,
        recent_blockhash.map(|s| s.to_string()),
    );

    if is_cu_program {
        if let Some(event) = EventDispatcher::dispatch_compute_budget_instruction(
            &instruction.data,
            metadata.clone(),
        ) {
            if passes_event_type_filter(event_type_filter, &event) {
                callback(event);
            }
        }
        return Ok(());
    }

    // Match the parser protocol.
    let protocol = match EventDispatcher::match_protocol_by_program_id(&program_id) {
        Some(p) => p,
        None => return Ok(()),
    };

    // Split discriminator and instruction payload.
    let instruction_discriminator = &instruction.data[..disc_len];
    let instruction_data = &instruction.data[disc_len..];

    // Build the account pubkey list for this instruction.
    let account_pubkeys: Vec<Pubkey> = instruction
        .accounts
        .iter()
        .filter_map(|&idx| accounts.get(idx as usize).copied())
        .collect();

    // Parse the instruction event.
    let mut event = match EventDispatcher::dispatch_instruction(
        protocol.clone(),
        instruction_discriminator,
        instruction_data,
        &account_pubkeys,
        metadata.clone(),
    ) {
        Some(e) => e,
        None => return Ok(()),
    };

    // Find the next CPI log for merge. The gRPC hot path stays sequential to avoid
    // thread::scope spawn/join overhead.
    let mut inner_instruction_event: Option<DexEvent> = None;
    if let Some(inner_instructions_ref) = inner_instructions {
        let raw = inner_index.unwrap_or(-1);
        let current_inner_idx = raw.clamp(i32::MIN as i64, i32::MAX as i64) as i32;

        for (idx, inner_instruction) in inner_instructions_ref.instructions.iter().enumerate() {
            if (idx as i32) <= current_inner_idx {
                continue;
            }
            let inner_data = &inner_instruction.data;
            if inner_data.len() < 16 {
                continue;
            }
            let inner_discriminator = &inner_data[..16];
            let inner_instruction_data = &inner_data[16..];
            if let Some(inner_event) = EventDispatcher::dispatch_inner_instruction(
                protocol.clone(),
                inner_discriminator,
                inner_instruction_data,
                metadata.clone(),
            ) {
                inner_instruction_event = Some(inner_event);
                break;
            }
        }

        if event.metadata().swap_data.is_none() {
            if let Some(swap_data) = parse_swap_data_from_next_grpc_instructions(
                &event,
                inner_instructions_ref,
                current_inner_idx,
                accounts,
            ) {
                event.metadata_mut().set_swap_data(swap_data);
            }
        }
    }

    // PumpFun MIGRATE emits instruction-only data when no CPI log exists.

    // Merge CPI details into the outer event.
    if let Some(inner_instruction_event) = inner_instruction_event {
        merge(&mut event, inner_instruction_event);
    }

    // Stamp handling latency using the high-performance clock.
    event.metadata_mut().handle_us = elapsed_micros_since(recv_us);
    event = super::super::helpers::process_event(event, bot_wallet);
    if passes_event_type_filter(event_type_filter, &event) {
        callback(event);
    }

    Ok(())
}
