use solana_sdk::pubkey::Pubkey;

use crate::streaming::event_parser::{
    common::{utils::*, EventMetadata, EventType},
    protocols::bonk::{
        bonk_pool_create_event_log_decode, bonk_trade_event_log_decode, discriminators, AmmFeeOn,
        BonkMigrateToAmmEvent, BonkMigrateToCpswapEvent, BonkPoolCreateEvent, BonkTradeEvent,
        ConstantCurve, CurveParams, FixedCurve, LinearCurve, MintParams, TradeDirection,
        VestingParams,
    },
    DexEvent,
};

/// Bonk Program ID
pub const BONK_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj");

/// Parse Bonk instruction data
///
/// Route to specific instruction parser based on discriminator
pub fn parse_bonk_instruction_data(
    discriminator: &[u8],
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    match discriminator {
        discriminators::BUY_EXACT_IN => {
            parse_buy_exact_in_instruction(data, accounts, metadata)
        }
        discriminators::BUY_EXACT_OUT => {
            parse_buy_exact_out_instruction(data, accounts, metadata)
        }
        discriminators::SELL_EXACT_IN => {
            parse_sell_exact_in_instruction(data, accounts, metadata)
        }
        discriminators::SELL_EXACT_OUT => {
            parse_sell_exact_out_instruction(data, accounts, metadata)
        }
        discriminators::INITIALIZE => {
            parse_initialize_instruction(data, accounts, metadata)
        }
        discriminators::INITIALIZE_V2 => {
            parse_initialize_v2_instruction(data, accounts, metadata)
        }
        discriminators::INITIALIZE_WITH_TOKEN_2022 => {
            parse_initialize_with_token_2022_instruction(data, accounts, metadata)
        }
        discriminators::MIGRATE_TO_AMM => {
            parse_migrate_to_amm_instruction(data, accounts, metadata)
        }
        discriminators::MIGRATE_TO_CP_SWAP => {
            parse_migrate_to_cpswap_instruction(data, accounts, metadata)
        }
        _ => None,
    }
}

/// Parse Bonk inner instruction data
///
/// Route to specific inner instruction parser based on discriminator
pub fn parse_bonk_inner_instruction_data(
    discriminator: &[u8],
    data: &[u8],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    match discriminator {
        discriminators::TRADE_EVENT => {
            parse_trade_inner_instruction(data, metadata)
        }
        discriminators::POOL_CREATE_EVENT => {
            parse_pool_create_inner_instruction(data, metadata)
        }
        _ => None,
    }
}

/// Parse Bonk account data
///
/// Route to specific account parser based on discriminator
pub fn parse_bonk_account_data(
    discriminator: &[u8],
    account: &crate::streaming::grpc::AccountPretty,
    metadata: crate::streaming::event_parser::common::EventMetadata,
) -> Option<crate::streaming::event_parser::DexEvent> {
    match discriminator {
        discriminators::POOL_STATE_ACCOUNT => {
            crate::streaming::event_parser::protocols::bonk::types::pool_state_parser(account, metadata)
        }
        discriminators::GLOBAL_CONFIG_ACCOUNT => {
            crate::streaming::event_parser::protocols::bonk::types::global_config_parser(account, metadata)
        }
        discriminators::PLATFORM_CONFIG_ACCOUNT => {
            crate::streaming::event_parser::protocols::bonk::types::platform_config_parser(account, metadata)
        }
        _ => None,
    }
}

/// Parse pool creation event
fn parse_pool_create_inner_instruction(
    data: &[u8],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    // Note: event_type will be set by the instruction parser, not here
    // Because different initialize instructions have different event types
    if let Some(event) = bonk_pool_create_event_log_decode(data) {
        Some(DexEvent::BonkPoolCreateEvent(BonkPoolCreateEvent { metadata, ..event }))
    } else {
        None
    }
}

/// Parse trade event
fn parse_trade_inner_instruction(data: &[u8], metadata: EventMetadata) -> Option<DexEvent> {
    if let Some(event) = bonk_trade_event_log_decode(data) {
        if metadata.event_type == EventType::BonkBuyExactIn
            || metadata.event_type == EventType::BonkBuyExactOut
        {
            if event.trade_direction != TradeDirection::Buy {
                return None;
            }
        } else if (metadata.event_type == EventType::BonkSellExactIn
            || metadata.event_type == EventType::BonkSellExactOut)
            && event.trade_direction != TradeDirection::Sell
        {
            return None;
        }
        Some(DexEvent::BonkTradeEvent(BonkTradeEvent { metadata, ..event }))
    } else {
        None
    }
}

/// Parse buy instruction event
fn parse_buy_exact_in_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::BonkBuyExactIn;

    if data.len() < 16 || accounts.len() < 18 {
        return None;
    }

    let amount_in = read_u64_le(data, 0)?;
    let minimum_amount_out = read_u64_le(data, 8)?;
    let share_fee_rate = read_u64_le(data, 16)?;

    Some(DexEvent::BonkTradeEvent(BonkTradeEvent {
        metadata,
        amount_in,
        minimum_amount_out,
        share_fee_rate,
        payer: accounts[0],
        global_config: accounts[2],
        platform_config: accounts[3],
        pool_state: accounts[4],
        user_base_token: accounts[5],
        user_quote_token: accounts[6],
        base_vault: accounts[7],
        quote_vault: accounts[8],
        base_token_mint: accounts[9],
        quote_token_mint: accounts[10],
        base_token_program: accounts[11],
        quote_token_program: accounts[12],
        system_program: accounts[15],
        platform_associated_account: accounts[16],
        creator_associated_account: accounts[17],
        trade_direction: TradeDirection::Buy,
        ..Default::default()
    }))
}

fn parse_buy_exact_out_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::BonkBuyExactOut;

    if data.len() < 16 || accounts.len() < 18 {
        return None;
    }

    let amount_out = read_u64_le(data, 0)?;
    let maximum_amount_in = read_u64_le(data, 8)?;
    let share_fee_rate = read_u64_le(data, 16)?;

    Some(DexEvent::BonkTradeEvent(BonkTradeEvent {
        metadata,
        amount_out,
        maximum_amount_in,
        share_fee_rate,
        payer: accounts[0],
        global_config: accounts[2],
        platform_config: accounts[3],
        pool_state: accounts[4],
        user_base_token: accounts[5],
        user_quote_token: accounts[6],
        base_vault: accounts[7],
        quote_vault: accounts[8],
        base_token_mint: accounts[9],
        quote_token_mint: accounts[10],
        base_token_program: accounts[11],
        quote_token_program: accounts[12],
        system_program: accounts[15],
        platform_associated_account: accounts[16],
        creator_associated_account: accounts[17],
        trade_direction: TradeDirection::Buy,
        ..Default::default()
    }))
}

fn parse_sell_exact_in_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::BonkSellExactIn;

    if data.len() < 16 || accounts.len() < 18 {
        return None;
    }

    let amount_in = read_u64_le(data, 0)?;
    let minimum_amount_out = read_u64_le(data, 8)?;
    let share_fee_rate = read_u64_le(data, 16)?;

    Some(DexEvent::BonkTradeEvent(BonkTradeEvent {
        metadata,
        amount_in,
        minimum_amount_out,
        share_fee_rate,
        payer: accounts[0],
        global_config: accounts[2],
        platform_config: accounts[3],
        pool_state: accounts[4],
        user_base_token: accounts[5],
        user_quote_token: accounts[6],
        base_vault: accounts[7],
        quote_vault: accounts[8],
        base_token_mint: accounts[9],
        quote_token_mint: accounts[10],
        base_token_program: accounts[11],
        quote_token_program: accounts[12],
        system_program: accounts[15],
        platform_associated_account: accounts[16],
        creator_associated_account: accounts[17],
        trade_direction: TradeDirection::Sell,
        ..Default::default()
    }))
}

fn parse_sell_exact_out_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::BonkSellExactOut;

    if data.len() < 16 || accounts.len() < 18 {
        return None;
    }

    let amount_out = read_u64_le(data, 0)?;
    let maximum_amount_in = read_u64_le(data, 8)?;
    let share_fee_rate = read_u64_le(data, 16)?;

    Some(DexEvent::BonkTradeEvent(BonkTradeEvent {
        metadata,
        amount_out,
        maximum_amount_in,
        share_fee_rate,
        payer: accounts[0],
        global_config: accounts[2],
        platform_config: accounts[3],
        pool_state: accounts[4],
        user_base_token: accounts[5],
        user_quote_token: accounts[6],
        base_vault: accounts[7],
        quote_vault: accounts[8],
        base_token_mint: accounts[9],
        quote_token_mint: accounts[10],
        base_token_program: accounts[11],
        quote_token_program: accounts[12],
        system_program: accounts[15],
        platform_associated_account: accounts[16],
        creator_associated_account: accounts[17],
        trade_direction: TradeDirection::Sell,
        ..Default::default()
    }))
}

/// Parse initialize event
fn parse_initialize_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::BonkInitialize;

    if data.len() < 24 {
        return None;
    }

    let mut offset = 0;
    let base_mint_param = parse_mint_params(data, &mut offset)?;
    let curve_param = parse_curve_params(data, &mut offset)?;
    let vesting_param = parse_vesting_params(data, &mut offset)?;

    Some(DexEvent::BonkPoolCreateEvent(BonkPoolCreateEvent {
        metadata,
        payer: accounts[0],
        creator: accounts[1],
        global_config: accounts[2],
        platform_config: accounts[3],
        pool_state: accounts[5],
        base_mint: accounts[6],
        quote_mint: accounts[7],
        base_vault: accounts[8],
        quote_vault: accounts[9],
        base_mint_param,
        curve_param,
        vesting_param,
        ..Default::default()
    }))
}

/// Parse initialize event
fn parse_initialize_v2_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::BonkInitializeV2;

    if data.len() < 24 {
        return None;
    }

    let mut offset = 0;
    let base_mint_param = parse_mint_params(data, &mut offset)?;
    let curve_param = parse_curve_params(data, &mut offset)?;
    let vesting_param = parse_vesting_params(data, &mut offset)?;
    let amm_fee_on = data[offset];

    Some(DexEvent::BonkPoolCreateEvent(BonkPoolCreateEvent {
        metadata,
        payer: accounts[0],
        creator: accounts[1],
        global_config: accounts[2],
        platform_config: accounts[3],
        pool_state: accounts[5],
        base_mint: accounts[6],
        quote_mint: accounts[7],
        base_vault: accounts[8],
        quote_vault: accounts[9],
        base_mint_param,
        curve_param,
        vesting_param,
        amm_fee_on: if amm_fee_on == 0 {
            Some(AmmFeeOn::QuoteToken)
        } else {
            Some(AmmFeeOn::BothToken)
        },
        ..Default::default()
    }))
}

/// Parse initialize event
fn parse_initialize_with_token_2022_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::BonkInitializeWithToken2022;

    if data.len() < 24 {
        return None;
    }

    let mut offset = 0;
    let base_mint_param = parse_mint_params(data, &mut offset)?;
    let curve_param = parse_curve_params(data, &mut offset)?;
    let vesting_param = parse_vesting_params(data, &mut offset)?;
    let amm_fee_on = data[offset];

    Some(DexEvent::BonkPoolCreateEvent(BonkPoolCreateEvent {
        metadata,
        payer: accounts[0],
        creator: accounts[1],
        global_config: accounts[2],
        platform_config: accounts[3],
        pool_state: accounts[5],
        base_mint: accounts[6],
        quote_mint: accounts[7],
        base_vault: accounts[8],
        quote_vault: accounts[9],
        base_mint_param,
        curve_param,
        vesting_param,
        amm_fee_on: if amm_fee_on == 0 {
            Some(AmmFeeOn::QuoteToken)
        } else {
            Some(AmmFeeOn::BothToken)
        },
        ..Default::default()
    }))
}

/// Parse MintParams structure
fn parse_mint_params(data: &[u8], offset: &mut usize) -> Option<MintParams> {
    // Read decimals (1 byte)
    let decimals = read_u8(data, *offset)?;
    *offset += 1;

    // Read name string length and content
    let name_len = read_u32_le(data, *offset)? as usize;
    *offset += 4;
    if data.len() < *offset + name_len {
        return None;
    }
    let name = String::from_utf8(data[*offset..*offset + name_len].to_vec()).ok()?;
    *offset += name_len;

    // Read symbol string length and content
    let symbol_len = read_u32_le(data, *offset)? as usize;
    *offset += 4;
    if data.len() < *offset + symbol_len {
        return None;
    }
    let symbol = String::from_utf8(data[*offset..*offset + symbol_len].to_vec()).ok()?;
    *offset += symbol_len;

    // Read uri string length and content
    let uri_len = read_u32_le(data, *offset)? as usize;
    *offset += 4;
    if data.len() < *offset + uri_len {
        return None;
    }
    let uri = String::from_utf8(data[*offset..*offset + uri_len].to_vec()).ok()?;
    *offset += uri_len;

    Some(MintParams { decimals, name, symbol, uri })
}

/// Parse CurveParams structure
fn parse_curve_params(data: &[u8], offset: &mut usize) -> Option<CurveParams> {
    // Read curve type identifier (1 byte)
    let curve_type = read_u8(data, *offset)?;
    *offset += 1;

    match curve_type {
        0 => {
            // Constant curve
            let supply = read_u64_le(data, *offset)?;
            *offset += 8;
            let total_base_sell = read_u64_le(data, *offset)?;
            *offset += 8;
            let total_quote_fund_raising = read_u64_le(data, *offset)?;
            *offset += 8;
            let migrate_type = read_u8(data, *offset)?;
            *offset += 1;

            Some(CurveParams::Constant {
                data: ConstantCurve {
                    supply,
                    total_base_sell,
                    total_quote_fund_raising,
                    migrate_type,
                },
            })
        }
        1 => {
            // Fixed curve
            let supply = read_u64_le(data, *offset)?;
            *offset += 8;
            let total_quote_fund_raising = read_u64_le(data, *offset)?;
            *offset += 8;
            let migrate_type = read_u8(data, *offset)?;
            *offset += 1;

            Some(CurveParams::Fixed {
                data: FixedCurve { supply, total_quote_fund_raising, migrate_type },
            })
        }
        2 => {
            // Linear curve
            let supply = read_u64_le(data, *offset)?;
            *offset += 8;
            let total_quote_fund_raising = read_u64_le(data, *offset)?;
            *offset += 8;
            let migrate_type = read_u8(data, *offset)?;
            *offset += 1;

            Some(CurveParams::Linear {
                data: LinearCurve { supply, total_quote_fund_raising, migrate_type },
            })
        }
        _ => None,
    }
}

/// Parse VestingParams structure
fn parse_vesting_params(data: &[u8], offset: &mut usize) -> Option<VestingParams> {
    let total_locked_amount = read_u64_le(data, *offset)?;
    *offset += 8;
    let cliff_period = read_u64_le(data, *offset)?;
    *offset += 8;
    let unlock_period = read_u64_le(data, *offset)?;
    *offset += 8;

    Some(VestingParams { total_locked_amount, cliff_period, unlock_period })
}

/// Parse migrate to AMM event
fn parse_migrate_to_amm_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::BonkMigrateToAmm;

    if data.len() < 16 {
        return None;
    }

    let base_lot_size = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let quote_lot_size = u64::from_le_bytes(data[8..16].try_into().unwrap());
    let market_vault_signer_nonce = data[16];

    Some(DexEvent::BonkMigrateToAmmEvent(BonkMigrateToAmmEvent {
        metadata,
        base_lot_size,
        quote_lot_size,
        market_vault_signer_nonce,
        payer: accounts[0],
        base_mint: accounts[1],
        quote_mint: accounts[2],
        openbook_program: accounts[3],
        market: accounts[4],
        request_queue: accounts[5],
        event_queue: accounts[6],
        bids: accounts[7],
        asks: accounts[8],
        market_vault_signer: accounts[9],
        market_base_vault: accounts[10],
        market_quote_vault: accounts[11],
        amm_program: accounts[12],
        amm_pool: accounts[13],
        amm_authority: accounts[14],
        amm_open_orders: accounts[15],
        amm_lp_mint: accounts[16],
        amm_base_vault: accounts[17],
        amm_quote_vault: accounts[18],
        amm_target_orders: accounts[19],
        amm_config: accounts[20],
        amm_create_fee_destination: accounts[21],
        authority: accounts[22],
        pool_state: accounts[23],
        global_config: accounts[24],
        base_vault: accounts[25],
        quote_vault: accounts[26],
        pool_lp_token: accounts[27],
        spl_token_program: accounts[28],
        associated_token_program: accounts[29],
        system_program: accounts[30],
        rent_program: accounts[31],
        ..Default::default()
    }))
}

/// Parse migrate to CP Swap event
fn parse_migrate_to_cpswap_instruction(
    _data: &[u8],
    accounts: &[Pubkey],
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::BonkMigrateToCpswap;

    Some(DexEvent::BonkMigrateToCpswapEvent(BonkMigrateToCpswapEvent {
        metadata,
        payer: accounts[0],
        base_mint: accounts[1],
        quote_mint: accounts[2],
        platform_config: accounts[3],
        cpswap_program: accounts[4],
        cpswap_pool: accounts[5],
        cpswap_authority: accounts[6],
        cpswap_lp_mint: accounts[7],
        cpswap_base_vault: accounts[8],
        cpswap_quote_vault: accounts[9],
        cpswap_config: accounts[10],
        cpswap_create_pool_fee: accounts[11],
        cpswap_observation: accounts[12],
        lock_program: accounts[13],
        lock_authority: accounts[14],
        lock_lp_vault: accounts[15],
        authority: accounts[16],
        pool_state: accounts[17],
        global_config: accounts[18],
        base_vault: accounts[19],
        quote_vault: accounts[20],
        pool_lp_token: accounts[21],
        base_token_program: accounts[22],
        quote_token_program: accounts[23],
        associated_token_program: accounts[24],
        system_program: accounts[25],
        rent_program: accounts[26],
        metadata_program: accounts[27],
        remaining_accounts: accounts[28..].to_vec(),
        ..Default::default()
    }))
}
