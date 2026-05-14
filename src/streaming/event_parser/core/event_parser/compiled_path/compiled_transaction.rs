//! Sequential top-level and inner ix traversal for [`VersionedTransaction`].
use crate::streaming::event_parser::{common::filter::EventTypeFilter, DexEvent, Protocol};
use prost_types::Timestamp;
use solana_sdk::{pubkey::Pubkey, signature::Signature, transaction::VersionedTransaction};
use solana_transaction_status::InnerInstructions;
use std::sync::Arc;

pub(crate) async fn parse_instruction_events_from_versioned_transaction(
    protocols: &[Protocol],
    event_type_filter: Option<&EventTypeFilter>,
    transaction: &VersionedTransaction,
    signature: Signature,
    slot: Option<u64>,
    block_time: Option<Timestamp>,
    recv_us: i64,
    accounts: &[Pubkey],
    inner_instructions: &[InnerInstructions],
    bot_wallet: Option<Pubkey>,
    tx_index: Option<u64>,
    callback: Arc<dyn Fn(DexEvent) + Send + Sync>,
) -> anyhow::Result<()> {
    let compiled_instructions = transaction.message.instructions();
    let recent_blockhash = Some(transaction.message.recent_blockhash().to_string());
    let mut accounts: Vec<Pubkey> = accounts.to_vec();
    let has_program = accounts
        .iter()
        .any(|account| super::super::helpers::should_handle(protocols, event_type_filter, account));
    if has_program {
        // Parse each instruction in order.
        for (index, instruction) in compiled_instructions.iter().enumerate() {
            if let Some(program_id) = accounts.get(instruction.program_id_index as usize) {
                let program_id = *program_id;
                let inner_instructions = inner_instructions
                    .iter()
                    .find(|inner_instruction| inner_instruction.index == index as u8);
                if super::super::helpers::should_handle(protocols, event_type_filter, &program_id) {
                    let max_idx = instruction.accounts.iter().max().unwrap_or(&0);
                    if *max_idx as usize >= accounts.len() {
                        accounts.resize(*max_idx as usize + 1, Pubkey::default());
                    }
                    super::compiled_instruction::parse_events_from_instruction(
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
                        inner_instructions,
                        callback.clone(),
                    )?;
                }
                // Immediately process inner instructions for correct ordering
                if let Some(inner_instructions) = inner_instructions {
                    for (inner_index, inner_instruction) in
                        inner_instructions.instructions.iter().enumerate()
                    {
                        super::compiled_instruction::parse_events_from_instruction(
                            protocols,
                            event_type_filter,
                            &inner_instruction.instruction,
                            &accounts,
                            signature,
                            slot.unwrap_or(0),
                            block_time,
                            recv_us,
                            index as i64,
                            Some(inner_index as i64),
                            bot_wallet,
                            tx_index,
                            recent_blockhash.as_deref(),
                            Some(&inner_instructions),
                            callback.clone(),
                        )?;
                    }
                }
            }
        }
    }
    Ok(())
}
