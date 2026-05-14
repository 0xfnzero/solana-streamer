//! Yellowstone transaction parsing: SDK-first DEX parsing plus optional local ix fallback.
//!
//! When `transaction.meta` is missing, the SDK low-latency parser cannot see logs / complete inner
//! instruction context and may return no events. In that case streamer uses the local full ix path.
//!
//! When meta exists, DEX events come from `sol-parser-sdk`; the local second pass is limited to
//! ComputeBudget events when the user asked for them.
use crate::streaming::event_parser::{
    common::{
        filter::{
            build_sdk_parse_event_filter, filter_includes_compute_budget_types, EventTypeFilter,
        },
        high_performance_clock::elapsed_micros_since,
    },
    core::dispatcher::EventDispatcher,
    DexEvent, Protocol,
};
use prost_types::Timestamp;
use sol_parser_sdk::grpc::parse_subscribe_update_transaction_low_latency;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::sync::Arc;
use yellowstone_grpc_proto::geyser::{SubscribeUpdateTransaction, SubscribeUpdateTransactionInfo};

use super::grpc_ix_mode::GrpcIxParseMode;

pub(crate) async fn parse_grpc_transaction(
    protocols: &[Protocol],
    event_type_filter: Option<&EventTypeFilter>,
    mut grpc_tx: SubscribeUpdateTransactionInfo,
    signature: Signature,
    slot: Option<u64>,
    block_time: Option<Timestamp>,
    recv_us: i64,
    bot_wallet: Option<Pubkey>,
    tx_index: Option<u64>,
    callback: Arc<dyn Fn(DexEvent) + Send + Sync>,
) -> anyhow::Result<()> {
    let slot_u = slot.unwrap_or(0);
    let block_us_micro = block_time.map(|t| t.seconds * 1_000_000 + t.nanos as i64 / 1_000);

    if grpc_tx.transaction.as_ref().and_then(|tx| tx.message.as_ref()).is_none() {
        return Ok(());
    }

    let use_sol_parser_sdk = grpc_tx.meta.is_some();
    let skip_ix_pass =
        use_sol_parser_sdk && !filter_includes_compute_budget_types(event_type_filter);

    if use_sol_parser_sdk {
        let mut update = SubscribeUpdateTransaction {
            slot: slot_u,
            transaction: Some(grpc_tx),
            ..Default::default()
        };
        let sdk_parse_filter = build_sdk_parse_event_filter(event_type_filter);
        let pb_events = parse_subscribe_update_transaction_low_latency(
            &update,
            recv_us,
            block_us_micro,
            sdk_parse_filter.as_ref(),
        );
        let adapted = crate::streaming::parser_sdk_bridge::adapt_parser_events_list(
            pb_events,
            block_time.as_ref(),
            recv_us,
            protocols,
            event_type_filter,
        );
        for mut ev in adapted {
            ev.metadata_mut().handle_us = elapsed_micros_since(recv_us);
            ev = super::super::helpers::process_event(ev, bot_wallet);
            callback(ev);
        }

        if skip_ix_pass {
            return Ok(());
        }

        let Some(tx) = update.transaction.take() else {
            return Ok(());
        };
        grpc_tx = tx;
    }

    let Some(transition) = grpc_tx.transaction.as_ref() else {
        return Ok(());
    };
    let Some(message) = transition.message.as_ref() else {
        return Ok(());
    };

    let ix_mode =
        if use_sol_parser_sdk { GrpcIxParseMode::ComputeBudgetOnly } else { GrpcIxParseMode::Full };

    let accounts = build_account_keys(message, grpc_tx.meta.as_ref());
    let inner_instructions =
        grpc_tx.meta.as_ref().map(|meta| meta.inner_instructions.as_slice()).unwrap_or_default();
    let recent_blockhash = if message.recent_blockhash.len() == 32 {
        Some(solana_sdk::bs58::encode(&message.recent_blockhash).into_string())
    } else {
        None
    };

    parse_instruction_events_from_grpc_transaction(
        protocols,
        event_type_filter,
        ix_mode,
        &message.instructions,
        signature,
        slot,
        block_time,
        recv_us,
        &accounts,
        inner_instructions,
        bot_wallet,
        tx_index,
        recent_blockhash,
        callback,
    )
    .await?;

    Ok(())
}

fn build_account_keys(
    message: &yellowstone_grpc_proto::prelude::Message,
    meta: Option<&yellowstone_grpc_proto::prelude::TransactionStatusMeta>,
) -> Vec<Pubkey> {
    let loaded_len = meta
        .map(|m| m.loaded_writable_addresses.len() + m.loaded_readonly_addresses.len())
        .unwrap_or(0);
    let mut accounts = Vec::with_capacity(message.account_keys.len() + loaded_len);

    for account in &message.account_keys {
        push_account_key(&mut accounts, account);
    }

    if let Some(meta) = meta {
        for account in
            meta.loaded_writable_addresses.iter().chain(meta.loaded_readonly_addresses.iter())
        {
            push_account_key(&mut accounts, account);
        }
    }

    accounts
}

#[inline]
fn push_account_key(accounts: &mut Vec<Pubkey>, account: &[u8]) {
    let pubkey = if account.len() == 32 {
        Pubkey::try_from(account).unwrap_or_default()
    } else {
        Pubkey::default()
    };
    accounts.push(pubkey);
}

pub(super) async fn parse_instruction_events_from_grpc_transaction(
    protocols: &[Protocol],
    event_type_filter: Option<&EventTypeFilter>,
    ix_mode: GrpcIxParseMode,
    compiled_instructions: &[yellowstone_grpc_proto::prelude::CompiledInstruction],
    signature: Signature,
    slot: Option<u64>,
    block_time: Option<Timestamp>,
    recv_us: i64,
    accounts: &[Pubkey],
    inner_instructions: &[yellowstone_grpc_proto::solana::storage::confirmed_block::InnerInstructions],
    bot_wallet: Option<Pubkey>,
    tx_index: Option<u64>,
    recent_blockhash: Option<String>,
    callback: Arc<dyn Fn(DexEvent) + Send + Sync>,
) -> anyhow::Result<()> {
    let mut accounts = accounts.to_vec();
    let has_program = match ix_mode {
        GrpcIxParseMode::Full => accounts.iter().any(|account| {
            super::super::helpers::should_handle(protocols, event_type_filter, account)
        }),
        GrpcIxParseMode::ComputeBudgetOnly => compiled_instructions.iter().any(|ix| {
            accounts
                .get(ix.program_id_index as usize)
                .map(EventDispatcher::is_compute_budget_program)
                .unwrap_or(false)
        }),
    };
    if has_program {
        // Parse each instruction in order.
        for (index, instruction) in compiled_instructions.iter().enumerate() {
            if let Some(program_id) = accounts.get(instruction.program_id_index as usize) {
                let program_id = *program_id;
                let inner_instructions_ref = inner_instructions
                    .iter()
                    .find(|inner_instruction| inner_instruction.index == index as u32);
                let max_idx = instruction.accounts.iter().max().unwrap_or(&0);
                if *max_idx as usize >= accounts.len() {
                    accounts.resize(*max_idx as usize + 1, Pubkey::default());
                }
                let handle_outer = match ix_mode {
                    GrpcIxParseMode::Full => super::super::helpers::should_handle(
                        protocols,
                        event_type_filter,
                        &program_id,
                    ),
                    GrpcIxParseMode::ComputeBudgetOnly => {
                        EventDispatcher::is_compute_budget_program(&program_id)
                    }
                };
                if handle_outer {
                    super::grpc_instruction::parse_events_from_grpc_instruction(
                        protocols,
                        event_type_filter,
                        instruction,
                        &accounts,
                        signature,
                        slot.unwrap_or(0),
                        block_time,
                        recv_us,
                        index as i64,
                        None,
                        bot_wallet,
                        tx_index,
                        recent_blockhash.as_deref(),
                        inner_instructions_ref,
                        callback.clone(),
                    )?;
                }
                if ix_mode == GrpcIxParseMode::Full {
                    if let Some(inner_instructions) = inner_instructions_ref {
                        for (inner_index, inner_instruction) in
                            inner_instructions.instructions.iter().enumerate()
                        {
                            let inner_accounts = &inner_instruction.accounts;
                            let data = &inner_instruction.data;
                            let instruction =
                                yellowstone_grpc_proto::prelude::CompiledInstruction {
                                    program_id_index: inner_instruction.program_id_index,
                                    accounts: inner_accounts.to_vec(),
                                    data: data.to_vec(),
                                };
                            super::grpc_instruction::parse_events_from_grpc_instruction(
                                protocols,
                                event_type_filter,
                                &instruction,
                                &accounts,
                                signature,
                                slot.unwrap_or(0),
                                block_time,
                                recv_us,
                                inner_instructions.index as i64,
                                Some(inner_index as i64),
                                bot_wallet,
                                tx_index,
                                recent_blockhash.as_deref(),
                                Some(inner_instructions),
                                callback.clone(),
                            )?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
