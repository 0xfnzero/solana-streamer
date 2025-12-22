use crate::streaming::event_parser::{
    common::{read_u64_le, EventMetadata, EventType},
    protocols::pumpswap::{
        discriminators, pump_swap_buy_event_log_decode,
        pump_swap_buy_exact_quote_in_event_log_decode,
        pump_swap_create_pool_event_log_decode, pump_swap_deposit_event_log_decode,
        pump_swap_sell_event_log_decode, pump_swap_withdraw_event_log_decode, PumpSwapBuyEvent,
        PumpSwapBuyExactQuoteInEvent, PumpSwapCreatePoolEvent, PumpSwapDepositEvent,
        PumpSwapSellEvent, PumpSwapWithdrawEvent,
    },
    DexEvent,
};
use solana_sdk::pubkey::Pubkey;

/// PumpSwap program ID
pub const PUMPSWAP_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");

/// Parse PumpSwap instruction data
///
/// Routes to specific instruction parser based on discriminator
pub fn parse_pumpswap_instruction_data(
    discriminator: &[u8],
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    match discriminator {
        discriminators::BUY_IX => parse_buy_instruction(data, accounts, metadata),
        discriminators::BUY_EXACT_QUOTE_IN_IX => parse_buy_exact_quote_in_instruction(data, accounts, metadata),
        discriminators::SELL_IX => parse_sell_instruction(data, accounts, metadata),
        discriminators::CREATE_POOL_IX => {
            parse_create_pool_instruction(data, accounts, metadata)
        }
        discriminators::DEPOSIT_IX => parse_deposit_instruction(data, accounts, metadata),
        discriminators::WITHDRAW_IX => parse_withdraw_instruction(data, accounts, metadata),
        _ => None,
    }
}

/// Parse PumpSwap inner instruction data
///
/// Routes to specific inner instruction parser based on discriminator
pub fn parse_pumpswap_inner_instruction_data(
    discriminator: &[u8],
    data: &[u8],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    match discriminator {
        discriminators::BUY_EVENT => parse_buy_inner_instruction(data, metadata),
        discriminators::SELL_EVENT => parse_sell_inner_instruction(data, metadata),
        discriminators::CREATE_POOL_EVENT => {
            parse_create_pool_inner_instruction(data, metadata)
        }
        discriminators::DEPOSIT_EVENT => parse_deposit_inner_instruction(data, metadata),
        discriminators::WITHDRAW_EVENT => parse_withdraw_inner_instruction(data, metadata),
        _ => None,
    }
}


/// Parse PumpSwap account data
///
/// Routes to specific account parser based on discriminator
pub fn parse_pumpswap_account_data(
    discriminator: &[u8],
    account: &crate::streaming::grpc::AccountPretty,
    metadata: crate::streaming::event_parser::common::EventMetadata,
) -> Option<crate::streaming::event_parser::DexEvent> {
    match discriminator {
        discriminators::GLOBAL_CONFIG_ACCOUNT => {
            crate::streaming::event_parser::protocols::pumpswap::types::global_config_parser(account, metadata)
        }
        discriminators::POOL_ACCOUNT => {
            crate::streaming::event_parser::protocols::pumpswap::types::pool_parser(account, metadata)
        }
        _ => None,
    }
}

/// Parse buy event log
fn parse_buy_inner_instruction(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    // First try to decode as buy_exact_quote_in to read ix_name
    if let Some(event) = pump_swap_buy_exact_quote_in_event_log_decode(data) {
        if event.ix_name == "buy_exact_quote_in" {
            return Some(DexEvent::PumpSwapBuyExactQuoteInEvent(PumpSwapBuyExactQuoteInEvent {
                metadata,
                ..event
            }));
        }
    }
    
    // If not buy_exact_quote_in, decode as normal buy
    pump_swap_buy_event_log_decode(data).map(|event| {
        DexEvent::PumpSwapBuyEvent(PumpSwapBuyEvent { metadata, ..event })
    })
}

/// Parse sell event log
fn parse_sell_inner_instruction(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    // Note: event_type will be set by instruction parser
    if let Some(event) = pump_swap_sell_event_log_decode(data) {
        Some(DexEvent::PumpSwapSellEvent(PumpSwapSellEvent { metadata, ..event }))
    } else {
        None
    }
}

/// Parse create pool event log
fn parse_create_pool_inner_instruction(
    data: &[u8],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    // Note: event_type will be set by instruction parser
    if let Some(event) = pump_swap_create_pool_event_log_decode(data) {
        Some(DexEvent::PumpSwapCreatePoolEvent(PumpSwapCreatePoolEvent { metadata, ..event }))
    } else {
        None
    }
}

/// Parse deposit event log
fn parse_deposit_inner_instruction(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    // Note: event_type will be set by instruction parser
    if let Some(event) = pump_swap_deposit_event_log_decode(data) {
        Some(DexEvent::PumpSwapDepositEvent(PumpSwapDepositEvent { metadata, ..event }))
    } else {
        None
    }
}

/// Parse withdraw event log
fn parse_withdraw_inner_instruction(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    // Note: event_type will be set by instruction parser
    if let Some(event) = pump_swap_withdraw_event_log_decode(data) {
        Some(DexEvent::PumpSwapWithdrawEvent(PumpSwapWithdrawEvent { metadata, ..event }))
    } else {
        None
    }
}

/// Parse buy instruction
fn parse_buy_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::PumpSwapBuy;

    if data.len() < 16 || accounts.len() < 13 {
        return None;
    }

    let base_amount_out = read_u64_le(data, 0)?;
    let max_quote_amount_in = read_u64_le(data, 8)?;

    Some(DexEvent::PumpSwapBuyEvent(PumpSwapBuyEvent {
        metadata,
        base_amount_out,
        max_quote_amount_in,
        pool: accounts[0],
        user: accounts[1],
        base_mint: accounts[3],
        quote_mint: accounts[4],
        user_base_token_account: accounts[5],
        user_quote_token_account: accounts[6],
        pool_base_token_account: accounts[7],
        pool_quote_token_account: accounts[8],
        protocol_fee_recipient: accounts[9],
        protocol_fee_recipient_token_account: accounts[10],
        base_token_program: accounts[11],
        quote_token_program: accounts[12],
        coin_creator_vault_ata: accounts.get(17).copied().unwrap_or_default(),
        coin_creator_vault_authority: accounts.get(18).copied().unwrap_or_default(),
        ..Default::default()
    }))
}

/// Parse `buy_exact_quote_in` instruction
fn parse_buy_exact_quote_in_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::PumpSwapBuyExactQuoteIn;

    if data.len() < 16 || accounts.len() < 13 {
        return None;
    }

    let spendable_quote_in = read_u64_le(data, 0)?;
    let min_base_amount_out = read_u64_le(data, 8)?;

    Some(DexEvent::PumpSwapBuyExactQuoteInEvent(
        PumpSwapBuyExactQuoteInEvent {
            metadata,
            spendable_quote_in,
            min_base_amount_out,
            pool: accounts[0],
            user: accounts[1],
            base_mint: accounts[3],
            quote_mint: accounts[4],
            user_base_token_account: accounts[5],
            user_quote_token_account: accounts[6],
            pool_base_token_account: accounts[7],
            pool_quote_token_account: accounts[8],
            protocol_fee_recipient: accounts[9],
            protocol_fee_recipient_token_account: accounts[10],
            base_token_program: accounts[11],
            quote_token_program: accounts[12],
            coin_creator_vault_ata: accounts.get(17).copied().unwrap_or_default(),
            coin_creator_vault_authority: accounts.get(18).copied().unwrap_or_default(),
            ..Default::default()
        },
    ))
}

/// Parse sell instruction
fn parse_sell_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::PumpSwapSell;

    if data.len() < 16 || accounts.len() < 13 {
        return None;
    }

    let base_amount_in = read_u64_le(data, 0)?;
    let min_quote_amount_out = read_u64_le(data, 8)?;

    Some(DexEvent::PumpSwapSellEvent(PumpSwapSellEvent {
        metadata,
        base_amount_in,
        min_quote_amount_out,
        pool: accounts[0],
        user: accounts[1],
        base_mint: accounts[3],
        quote_mint: accounts[4],
        user_base_token_account: accounts[5],
        user_quote_token_account: accounts[6],
        pool_base_token_account: accounts[7],
        pool_quote_token_account: accounts[8],
        protocol_fee_recipient: accounts[9],
        protocol_fee_recipient_token_account: accounts[10],
        base_token_program: accounts[11],
        quote_token_program: accounts[12],
        coin_creator_vault_ata: accounts.get(17).copied().unwrap_or_default(),
        coin_creator_vault_authority: accounts.get(18).copied().unwrap_or_default(),
        ..Default::default()
    }))
}

/// Parse create pool instruction
fn parse_create_pool_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::PumpSwapCreatePool;

    if data.len() < 18 || accounts.len() < 11 {
        return None;
    }

    let index = u16::from_le_bytes(data[0..2].try_into().ok()?);
    let base_amount_in = u64::from_le_bytes(data[2..10].try_into().ok()?);
    let quote_amount_in = u64::from_le_bytes(data[10..18].try_into().ok()?);
    let coin_creator = if data.len() >= 50 {
        Pubkey::new_from_array(data[18..50].try_into().ok()?)
    } else {
        Pubkey::default()
    };

    Some(DexEvent::PumpSwapCreatePoolEvent(PumpSwapCreatePoolEvent {
        metadata,
        index,
        base_amount_in,
        quote_amount_in,
        pool: accounts[0],
        creator: accounts[2],
        base_mint: accounts[3],
        quote_mint: accounts[4],
        lp_mint: accounts[5],
        user_base_token_account: accounts[6],
        user_quote_token_account: accounts[7],
        user_pool_token_account: accounts[8],
        pool_base_token_account: accounts[9],
        pool_quote_token_account: accounts[10],
        coin_creator,
        ..Default::default()
    }))
}

/// Parse deposit instruction
fn parse_deposit_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::PumpSwapDeposit;

    if data.len() < 24 || accounts.len() < 11 {
        return None;
    }

    let lp_token_amount_out = u64::from_le_bytes(data[0..8].try_into().ok()?);
    let max_base_amount_in = u64::from_le_bytes(data[8..16].try_into().ok()?);
    let max_quote_amount_in = u64::from_le_bytes(data[16..24].try_into().ok()?);

    Some(DexEvent::PumpSwapDepositEvent(PumpSwapDepositEvent {
        metadata,
        lp_token_amount_out,
        max_base_amount_in,
        max_quote_amount_in,
        pool: accounts[0],
        user: accounts[2],
        base_mint: accounts[3],
        quote_mint: accounts[4],
        user_base_token_account: accounts[6],
        user_quote_token_account: accounts[7],
        user_pool_token_account: accounts[8],
        pool_base_token_account: accounts[9],
        pool_quote_token_account: accounts[10],
        ..Default::default()
    }))
}

/// Parse withdraw instruction
fn parse_withdraw_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::PumpSwapWithdraw;

    if data.len() < 24 || accounts.len() < 11 {
        return None;
    }

    let lp_token_amount_in = u64::from_le_bytes(data[0..8].try_into().ok()?);
    let min_base_amount_out = u64::from_le_bytes(data[8..16].try_into().ok()?);
    let min_quote_amount_out = u64::from_le_bytes(data[16..24].try_into().ok()?);

    Some(DexEvent::PumpSwapWithdrawEvent(PumpSwapWithdrawEvent {
        metadata,
        lp_token_amount_in,
        min_base_amount_out,
        min_quote_amount_out,
        pool: accounts[0],
        user: accounts[2],
        base_mint: accounts[3],
        quote_mint: accounts[4],
        user_base_token_account: accounts[6],
        user_quote_token_account: accounts[7],
        user_pool_token_account: accounts[8],
        pool_base_token_account: accounts[9],
        pool_quote_token_account: accounts[10],
        ..Default::default()
    }))
}
