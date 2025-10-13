use crate::streaming::event_parser::{
    common::{
        read_i32_le, read_option_bool, read_u128_le, read_u64_le, read_u8_le, EventMetadata,
        EventType, ProtocolType,
    },
    core::GenericEventParseConfig,
    protocols::raydium_clmm::{
        discriminators, RaydiumClmmClosePositionEvent, RaydiumClmmCreatePoolEvent,
        RaydiumClmmDecreaseLiquidityV2Event, RaydiumClmmIncreaseLiquidityV2Event,
        RaydiumClmmOpenPositionV2Event, RaydiumClmmOpenPositionWithToken22NftEvent,
        RaydiumClmmSwapEvent, RaydiumClmmSwapV2Event,
    },
    DexEvent,
};
use solana_sdk::pubkey::Pubkey;

/// Raydium CLMM程序ID
pub const RAYDIUM_CLMM_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK");

// 配置所有事件类型
pub const CONFIGS: &[GenericEventParseConfig] = &[
    GenericEventParseConfig {
        program_id: RAYDIUM_CLMM_PROGRAM_ID,
        protocol_type: ProtocolType::RaydiumClmm,
        inner_instruction_discriminator: &[],
        instruction_discriminator: discriminators::SWAP,
        event_type: EventType::RaydiumClmmSwap,
        inner_instruction_parser: None,
        instruction_parser: Some(parse_swap_instruction),
        requires_inner_instruction: false,
    },
    GenericEventParseConfig {
        program_id: RAYDIUM_CLMM_PROGRAM_ID,
        protocol_type: ProtocolType::RaydiumClmm,
        inner_instruction_discriminator: &[],
        instruction_discriminator: discriminators::SWAP_V2,
        event_type: EventType::RaydiumClmmSwapV2,
        inner_instruction_parser: None,
        instruction_parser: Some(parse_swap_v2_instruction),
        requires_inner_instruction: false,
    },
    GenericEventParseConfig {
        program_id: RAYDIUM_CLMM_PROGRAM_ID,
        protocol_type: ProtocolType::RaydiumClmm,
        inner_instruction_discriminator: &[],
        instruction_discriminator: discriminators::CLOSE_POSITION,
        event_type: EventType::RaydiumClmmClosePosition,
        inner_instruction_parser: None,
        instruction_parser: Some(parse_close_position_instruction),
        requires_inner_instruction: false,
    },
    GenericEventParseConfig {
        program_id: RAYDIUM_CLMM_PROGRAM_ID,
        protocol_type: ProtocolType::RaydiumClmm,
        inner_instruction_discriminator: &[],
        instruction_discriminator: discriminators::DECREASE_LIQUIDITY_V2,
        event_type: EventType::RaydiumClmmDecreaseLiquidityV2,
        inner_instruction_parser: None,
        instruction_parser: Some(parse_decrease_liquidity_v2_instruction),
        requires_inner_instruction: false,
    },
    GenericEventParseConfig {
        program_id: RAYDIUM_CLMM_PROGRAM_ID,
        protocol_type: ProtocolType::RaydiumClmm,
        inner_instruction_discriminator: &[],
        instruction_discriminator: discriminators::CREATE_POOL,
        event_type: EventType::RaydiumClmmCreatePool,
        inner_instruction_parser: None,
        instruction_parser: Some(parse_create_pool_instruction),
        requires_inner_instruction: false,
    },
    GenericEventParseConfig {
        program_id: RAYDIUM_CLMM_PROGRAM_ID,
        protocol_type: ProtocolType::RaydiumClmm,
        inner_instruction_discriminator: &[],
        instruction_discriminator: discriminators::INCREASE_LIQUIDITY_V2,
        event_type: EventType::RaydiumClmmIncreaseLiquidityV2,
        inner_instruction_parser: None,
        instruction_parser: Some(parse_increase_liquidity_v2_instruction),
        requires_inner_instruction: false,
    },
    GenericEventParseConfig {
        program_id: RAYDIUM_CLMM_PROGRAM_ID,
        protocol_type: ProtocolType::RaydiumClmm,
        inner_instruction_discriminator: &[],
        instruction_discriminator: discriminators::OPEN_POSITION_WITH_TOKEN_22_NFT,
        event_type: EventType::RaydiumClmmOpenPositionWithToken22Nft,
        inner_instruction_parser: None,
        instruction_parser: Some(parse_open_position_with_token_22_nft_instruction),
        requires_inner_instruction: false,
    },
    GenericEventParseConfig {
        program_id: RAYDIUM_CLMM_PROGRAM_ID,
        protocol_type: ProtocolType::RaydiumClmm,
        inner_instruction_discriminator: &[],
        instruction_discriminator: discriminators::OPEN_POSITION_V2,
        event_type: EventType::RaydiumClmmOpenPositionV2,
        inner_instruction_parser: None,
        instruction_parser: Some(parse_open_position_v2_instruction),
        requires_inner_instruction: false,
    },
];

/// 解析打开仓位V2指令事件
fn parse_open_position_v2_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    if data.len() < 51 || accounts.len() < 22 {
        return None;
    }
    Some(DexEvent::RaydiumClmmOpenPositionV2Event(RaydiumClmmOpenPositionV2Event {
        metadata,
        tick_lower_index: read_i32_le(data, 0)?,
        tick_upper_index: read_i32_le(data, 4)?,
        tick_array_lower_start_index: read_i32_le(data, 8)?,
        tick_array_upper_start_index: read_i32_le(data, 12)?,
        liquidity: read_u128_le(data, 16)?,
        amount0_max: read_u64_le(data, 32)?,
        amount1_max: read_u64_le(data, 40)?,
        with_metadata: read_u8_le(data, 48)? == 1,
        base_flag: read_option_bool(data, &mut 49)?,
        payer: accounts[0],
        position_nft_owner: accounts[1],
        position_nft_mint: accounts[2],
        position_nft_account: accounts[3],
        metadata_account: accounts[4],
        pool_state: accounts[5],
        protocol_position: accounts[6],
        tick_array_lower: accounts[7],
        tick_array_upper: accounts[8],
        personal_position: accounts[9],
        token_account0: accounts[10],
        token_account1: accounts[11],
        token_vault0: accounts[12],
        token_vault1: accounts[13],
        rent: accounts[14],
        system_program: accounts[15],
        token_program: accounts[16],
        associated_token_program: accounts[17],
        metadata_program: accounts[18],
        token_program2022: accounts[19],
        vault0_mint: accounts[20],
        vault1_mint: accounts[21],
        remaining_accounts: accounts[22..].to_vec(),
    }))
}

/// 解析打开仓位v2指令事件
fn parse_open_position_with_token_22_nft_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    if data.len() < 51 || accounts.len() < 20 {
        return None;
    }
    Some(DexEvent::RaydiumClmmOpenPositionWithToken22NftEvent(
        RaydiumClmmOpenPositionWithToken22NftEvent {
            metadata,
            tick_lower_index: read_i32_le(data, 0)?,
            tick_upper_index: read_i32_le(data, 4)?,
            tick_array_lower_start_index: read_i32_le(data, 8)?,
            tick_array_upper_start_index: read_i32_le(data, 12)?,
            liquidity: read_u128_le(data, 16)?,
            amount0_max: read_u64_le(data, 32)?,
            amount1_max: read_u64_le(data, 40)?,
            with_metadata: read_u8_le(data, 48)? == 1,
            base_flag: read_option_bool(data, &mut 49)?,
            payer: accounts[0],
            position_nft_owner: accounts[1],
            position_nft_mint: accounts[2],
            position_nft_account: accounts[3],
            pool_state: accounts[4],
            protocol_position: accounts[5],
            tick_array_lower: accounts[6],
            tick_array_upper: accounts[7],
            personal_position: accounts[8],
            token_account0: accounts[9],
            token_account1: accounts[10],
            token_vault0: accounts[11],
            token_vault1: accounts[12],
            rent: accounts[13],
            system_program: accounts[14],
            token_program: accounts[15],
            associated_token_program: accounts[16],
            token_program2022: accounts[17],
            vault0_mint: accounts[18],
            vault1_mint: accounts[19],
        },
    ))
}

/// 解析增加流动性v2指令事件
fn parse_increase_liquidity_v2_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    if data.len() < 34 || accounts.len() < 15 {
        return None;
    }
    Some(DexEvent::RaydiumClmmIncreaseLiquidityV2Event(RaydiumClmmIncreaseLiquidityV2Event {
        metadata,
        liquidity: read_u128_le(data, 0)?,
        amount0_max: read_u64_le(data, 16)?,
        amount1_max: read_u64_le(data, 24)?,
        base_flag: read_option_bool(data, &mut 32)?,
        nft_owner: accounts[0],
        nft_account: accounts[1],
        pool_state: accounts[2],
        protocol_position: accounts[3],
        personal_position: accounts[4],
        tick_array_lower: accounts[5],
        tick_array_upper: accounts[6],
        token_account0: accounts[7],
        token_account1: accounts[8],
        token_vault0: accounts[9],
        token_vault1: accounts[10],
        token_program: accounts[11],
        token_program2022: accounts[12],
        vault0_mint: accounts[13],
        vault1_mint: accounts[14],
    }))
}

/// 解析创建池指令事件
fn parse_create_pool_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    if data.len() < 24 || accounts.len() < 13 {
        return None;
    }
    Some(DexEvent::RaydiumClmmCreatePoolEvent(RaydiumClmmCreatePoolEvent {
        metadata,
        sqrt_price_x64: read_u128_le(data, 0)?,
        open_time: read_u64_le(data, 16)?,
        pool_creator: accounts[0],
        amm_config: accounts[1],
        pool_state: accounts[2],
        token_mint0: accounts[3],
        token_mint1: accounts[4],
        token_vault0: accounts[5],
        token_vault1: accounts[6],
        observation_state: accounts[7],
        tick_array_bitmap: accounts[8],
        token_program0: accounts[9],
        token_program1: accounts[10],
        system_program: accounts[11],
        rent: accounts[12],
    }))
}

/// 解析减少流动性v2指令事件
fn parse_decrease_liquidity_v2_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    if data.len() < 32 || accounts.len() < 16 {
        return None;
    }
    Some(DexEvent::RaydiumClmmDecreaseLiquidityV2Event(RaydiumClmmDecreaseLiquidityV2Event {
        metadata,
        liquidity: read_u128_le(data, 0)?,
        amount0_min: read_u64_le(data, 16)?,
        amount1_min: read_u64_le(data, 24)?,
        nft_owner: accounts[0],
        nft_account: accounts[1],
        personal_position: accounts[2],
        pool_state: accounts[3],
        protocol_position: accounts[4],
        token_vault0: accounts[5],
        token_vault1: accounts[6],
        tick_array_lower: accounts[7],
        tick_array_upper: accounts[8],
        recipient_token_account0: accounts[9],
        recipient_token_account1: accounts[10],
        token_program: accounts[11],
        token_program2022: accounts[12],
        memo_program: accounts[13],
        vault0_mint: accounts[14],
        vault1_mint: accounts[15],
        remaining_accounts: accounts[16..].to_vec(),
    }))
}

/// 解析关闭仓位指令事件
fn parse_close_position_instruction(
    _data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    if accounts.len() < 6 {
        return None;
    }
    Some(DexEvent::RaydiumClmmClosePositionEvent(RaydiumClmmClosePositionEvent {
        metadata,
        nft_owner: accounts[0],
        position_nft_mint: accounts[1],
        position_nft_account: accounts[2],
        personal_position: accounts[3],
        system_program: accounts[4],
        token_program: accounts[5],
    }))
}

/// 解析交易指令事件
fn parse_swap_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    if data.len() < 33 || accounts.len() < 10 {
        return None;
    }

    let amount = read_u64_le(data, 0)?;
    let other_amount_threshold = read_u64_le(data, 8)?;
    let sqrt_price_limit_x64 = read_u128_le(data, 16)?;
    let is_base_input = read_u8_le(data, 32)?;

    Some(DexEvent::RaydiumClmmSwapEvent(RaydiumClmmSwapEvent {
        metadata,
        amount,
        other_amount_threshold,
        sqrt_price_limit_x64,
        is_base_input: is_base_input == 1,
        payer: accounts[0],
        amm_config: accounts[1],
        pool_state: accounts[2],
        input_token_account: accounts[3],
        output_token_account: accounts[4],
        input_vault: accounts[5],
        output_vault: accounts[6],
        observation_state: accounts[7],
        token_program: accounts[8],
        tick_array: accounts[9],
        remaining_accounts: accounts[10..].to_vec(),
    }))
}

fn parse_swap_v2_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    if data.len() < 33 || accounts.len() < 13 {
        return None;
    }

    let amount = read_u64_le(data, 0)?;
    let other_amount_threshold = read_u64_le(data, 8)?;
    let sqrt_price_limit_x64 = read_u128_le(data, 16)?;
    let is_base_input = read_u8_le(data, 32)?;

    Some(DexEvent::RaydiumClmmSwapV2Event(RaydiumClmmSwapV2Event {
        metadata,
        amount,
        other_amount_threshold,
        sqrt_price_limit_x64,
        is_base_input: is_base_input == 1,
        payer: accounts[0],
        amm_config: accounts[1],
        pool_state: accounts[2],
        input_token_account: accounts[3],
        output_token_account: accounts[4],
        input_vault: accounts[5],
        output_vault: accounts[6],
        observation_state: accounts[7],
        token_program: accounts[8],
        token_program2022: accounts[9],
        memo_program: accounts[10],
        input_vault_mint: accounts[11],
        output_vault_mint: accounts[12],
        remaining_accounts: accounts[13..].to_vec(),
    }))
}
