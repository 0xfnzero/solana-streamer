use crate::streaming::event_parser::{
    common::EventMetadata, core::EventDispatcher, DexEvent, Protocol,
};
use solana_sdk::pubkey::Pubkey;

pub use sol_parser_sdk::instr::program_ids::METEORA_DAMM_V2_PROGRAM_ID;

pub fn parse_meteora_damm_v2_instruction_data(
    discriminator: &[u8],
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    EventDispatcher::dispatch_instruction(
        Protocol::MeteoraDammV2,
        discriminator,
        data,
        accounts,
        metadata,
    )
}

pub fn parse_meteora_damm_v2_inner_instruction_data(
    discriminator: &[u8],
    data: &[u8],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    EventDispatcher::dispatch_inner_instruction(
        Protocol::MeteoraDammV2,
        discriminator,
        data,
        metadata,
    )
}

pub fn parse_meteora_damm_v2_account_data(
    discriminator: &[u8],
    account: &crate::streaming::grpc::AccountPretty,
    metadata: EventMetadata,
) -> Option<DexEvent> {
    EventDispatcher::dispatch_account(Protocol::MeteoraDammV2, discriminator, account, metadata)
}
