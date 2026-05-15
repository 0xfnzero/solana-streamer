//! Backward-compatible SDK parser facade.

use crate::streaming::event_parser::common::EventMetadata;
use crate::streaming::event_parser::core::dispatcher::EventDispatcher;
use crate::streaming::event_parser::{DexEvent, Protocol};
use solana_sdk::pubkey::Pubkey;

pub fn dispatch_instruction(
    protocol: Protocol,
    instruction_discriminator: &[u8],
    instruction_data: &[u8],
    accounts: &[Pubkey],
    stream_meta: &EventMetadata,
) -> Option<DexEvent> {
    EventDispatcher::dispatch_instruction(
        protocol,
        instruction_discriminator,
        instruction_data,
        accounts,
        stream_meta.clone(),
    )
}

pub fn dispatch_inner_instruction(
    protocol: Protocol,
    inner_instruction_discriminator: &[u8],
    inner_instruction_data: &[u8],
    stream_meta: &EventMetadata,
) -> Option<DexEvent> {
    EventDispatcher::dispatch_inner_instruction(
        protocol,
        inner_instruction_discriminator,
        inner_instruction_data,
        stream_meta.clone(),
    )
}
