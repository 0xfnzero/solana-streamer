use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

use crate::streaming::event_parser::common::EventMetadata;
use crate::streaming::event_parser::protocols::pumpfun::types::{BondingCurve, Global};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PumpFunCreateTokenEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub mint: Pubkey,
    pub bonding_curve: Pubkey,
    pub user: Pubkey,
    pub creator: Pubkey,
    pub timestamp: i64,
    pub virtual_token_reserves: u64,
    pub virtual_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub token_total_supply: u64,
    #[borsh(skip)]
    pub token_program: Pubkey,
    #[borsh(skip)]
    pub is_mayhem_mode: bool,
    #[borsh(skip)]
    pub mint_authority: Pubkey,
    #[borsh(skip)]
    pub associated_bonding_curve: Pubkey,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PumpFunCreateV2TokenEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub mint: Pubkey,
    pub bonding_curve: Pubkey,
    pub user: Pubkey,
    pub creator: Pubkey,
    pub timestamp: i64,
    pub virtual_token_reserves: u64,
    pub virtual_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub token_total_supply: u64,
    pub token_program: Pubkey,
    pub is_mayhem_mode: bool,
    #[borsh(skip)]
    pub mint_authority: Pubkey,
    #[borsh(skip)]
    pub associated_bonding_curve: Pubkey,
}

pub fn pumpfun_create_v2_token_event_log_decode(data: &[u8]) -> Option<PumpFunCreateV2TokenEvent> {
    let mut offset = 0;

    // 解析 name 字符串: [长度(4字节 u32)][字符串内容]
    if data.len() < offset + 4 {
        return None;
    }
    let name_len = u32::from_le_bytes(data[offset..offset + 4].try_into().ok()?) as usize;
    offset += 4;
    if data.len() < offset + name_len {
        return None;
    }
    let name = String::from_utf8(data[offset..offset + name_len].to_vec()).ok()?;
    offset += name_len;

    // 解析 symbol 字符串
    if data.len() < offset + 4 {
        return None;
    }
    let symbol_len = u32::from_le_bytes(data[offset..offset + 4].try_into().ok()?) as usize;
    offset += 4;
    if data.len() < offset + symbol_len {
        return None;
    }
    let symbol = String::from_utf8(data[offset..offset + symbol_len].to_vec()).ok()?;
    offset += symbol_len;

    // 解析 uri 字符串
    if data.len() < offset + 4 {
        return None;
    }
    let uri_len = u32::from_le_bytes(data[offset..offset + 4].try_into().ok()?) as usize;
    offset += 4;
    if data.len() < offset + uri_len {
        return None;
    }
    let uri = String::from_utf8(data[offset..offset + uri_len].to_vec()).ok()?;
    offset += uri_len;

    // 解析 Pubkey 字段 (每个32字节)
    if data.len() < offset + 32 {
        return None;
    }
    let mint = Pubkey::new_from_array(data[offset..offset + 32].try_into().ok()?);
    offset += 32;

    if data.len() < offset + 32 {
        return None;
    }
    let bonding_curve = Pubkey::new_from_array(data[offset..offset + 32].try_into().ok()?);
    offset += 32;

    if data.len() < offset + 32 {
        return None;
    }
    let user = Pubkey::new_from_array(data[offset..offset + 32].try_into().ok()?);
    offset += 32;

    if data.len() < offset + 32 {
        return None;
    }
    let creator = Pubkey::new_from_array(data[offset..offset + 32].try_into().ok()?);
    offset += 32;

    // 解析数值字段
    if data.len() < offset + 8 {
        return None;
    }
    let timestamp = i64::from_le_bytes(data[offset..offset + 8].try_into().ok()?);
    offset += 8;

    if data.len() < offset + 8 {
        return None;
    }
    let virtual_token_reserves = u64::from_le_bytes(data[offset..offset + 8].try_into().ok()?);
    offset += 8;

    if data.len() < offset + 8 {
        return None;
    }
    let virtual_sol_reserves = u64::from_le_bytes(data[offset..offset + 8].try_into().ok()?);
    offset += 8;

    if data.len() < offset + 8 {
        return None;
    }
    let real_token_reserves = u64::from_le_bytes(data[offset..offset + 8].try_into().ok()?);
    offset += 8;

    if data.len() < offset + 8 {
        return None;
    }
    let token_total_supply = u64::from_le_bytes(data[offset..offset + 8].try_into().ok()?);
    offset += 8;

    // 如果数据长度足够,解析 V2 版本的额外字段: token_program (32字节) + is_mayhem_mode (1字节)
    let (token_program, is_mayhem_mode) = if data.len() >= offset + 33 {
        let token_program = Pubkey::new_from_array(data[offset..offset + 32].try_into().ok()?);
        let is_mayhem_mode = data[offset + 32] == 1;
        (token_program, is_mayhem_mode)
    } else {
        (Pubkey::default(), false)
    };

    Some(PumpFunCreateV2TokenEvent {
        name,
        symbol,
        uri,
        mint,
        bonding_curve,
        user,
        creator,
        timestamp,
        virtual_token_reserves,
        virtual_sol_reserves,
        real_token_reserves,
        token_total_supply,
        token_program,
        is_mayhem_mode,
        ..Default::default()
    })
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PumpFunTradeEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub mint: Pubkey,
    pub sol_amount: u64,
    pub token_amount: u64,
    pub is_buy: bool,
    pub user: Pubkey,
    pub timestamp: i64,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub fee_recipient: Pubkey,
    pub fee_basis_points: u64,
    pub fee: u64,
    pub creator: Pubkey,
    pub creator_fee_basis_points: u64,
    pub creator_fee: u64,
    pub track_volume: bool,
    pub total_unclaimed_tokens: u64,
    pub total_claimed_tokens: u64,
    pub current_sol_volume: u64,
    pub last_update_timestamp: i64,

    #[borsh(skip)]
    pub max_sol_cost: u64,
    #[borsh(skip)]
    pub min_sol_output: u64,
    #[borsh(skip)]
    pub amount: u64,
    #[borsh(skip)]
    pub is_bot: bool,
    #[borsh(skip)]
    pub is_dev_create_token_trade: bool, // 是否是dev创建token的交易

    #[borsh(skip)]
    pub global: Pubkey,
    // #[borsh(skip)]
    // pub fee_recipient: Pubkey,
    // #[borsh(skip)]
    // pub mint: Pubkey,
    #[borsh(skip)]
    pub bonding_curve: Pubkey,
    #[borsh(skip)]
    pub associated_bonding_curve: Pubkey,
    #[borsh(skip)]
    pub associated_user: Pubkey,
    // #[borsh(skip)]
    // pub user: Pubkey,
    #[borsh(skip)]
    pub system_program: Pubkey,
    #[borsh(skip)]
    pub token_program: Pubkey,
    #[borsh(skip)]
    pub creator_vault: Pubkey,
    #[borsh(skip)]
    pub event_authority: Pubkey,
    #[borsh(skip)]
    pub program: Pubkey,
    #[borsh(skip)]
    pub global_volume_accumulator: Pubkey,
    #[borsh(skip)]
    pub user_volume_accumulator: Pubkey,
    #[borsh(skip)]
    pub fee_config: Pubkey,
    #[borsh(skip)]
    pub fee_program: Pubkey,
}

pub const PUMPFUN_TRADE_EVENT_LOG_SIZE: usize = 250;

pub fn pumpfun_trade_event_log_decode(data: &[u8]) -> Option<PumpFunTradeEvent> {
    if data.len() < PUMPFUN_TRADE_EVENT_LOG_SIZE {
        return None;
    }
    borsh::from_slice::<PumpFunTradeEvent>(&data[..PUMPFUN_TRADE_EVENT_LOG_SIZE]).ok()
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PumpFunMigrateEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,

    pub user: Pubkey,
    pub mint: Pubkey,
    pub mint_amount: u64,
    pub sol_amount: u64,
    pub pool_migration_fee: u64,
    pub bonding_curve: Pubkey,
    pub timestamp: i64,
    pub pool: Pubkey,

    #[borsh(skip)]
    pub global: Pubkey,
    #[borsh(skip)]
    pub withdraw_authority: Pubkey,
    #[borsh(skip)]
    pub associated_bonding_curve: Pubkey,
    #[borsh(skip)]
    pub system_program: Pubkey,
    #[borsh(skip)]
    pub token_program: Pubkey,
    #[borsh(skip)]
    pub pump_amm: Pubkey,
    #[borsh(skip)]
    pub pool_authority: Pubkey,
    #[borsh(skip)]
    pub pool_authority_mint_account: Pubkey,
    #[borsh(skip)]
    pub pool_authority_wsol_account: Pubkey,
    #[borsh(skip)]
    pub amm_global_config: Pubkey,
    #[borsh(skip)]
    pub wsol_mint: Pubkey,
    #[borsh(skip)]
    pub lp_mint: Pubkey,
    #[borsh(skip)]
    pub user_pool_token_account: Pubkey,
    #[borsh(skip)]
    pub pool_base_token_account: Pubkey,
    #[borsh(skip)]
    pub pool_quote_token_account: Pubkey,
    #[borsh(skip)]
    pub token_2022_program: Pubkey,
    #[borsh(skip)]
    pub associated_token_program: Pubkey,
    #[borsh(skip)]
    pub pump_amm_event_authority: Pubkey,
    #[borsh(skip)]
    pub event_authority: Pubkey,
    #[borsh(skip)]
    pub program: Pubkey,
}

pub const PUMPFUN_MIGRATE_EVENT_LOG_SIZE: usize = 160;

pub fn pumpfun_migrate_event_log_decode(data: &[u8]) -> Option<PumpFunMigrateEvent> {
    if data.len() < PUMPFUN_MIGRATE_EVENT_LOG_SIZE {
        return None;
    }
    borsh::from_slice::<PumpFunMigrateEvent>(&data[..PUMPFUN_MIGRATE_EVENT_LOG_SIZE]).ok()
}

/// 铸币曲线
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PumpFunBondingCurveAccountEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub bonding_curve: BondingCurve,
}

/// 全局配置
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PumpFunGlobalAccountEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub global: Global,
}

/// 事件鉴别器常量
pub mod discriminators {
    // 事件鉴别器
    // pub const CREATE_TOKEN_EVENT: &str = "0xe445a52e51cb9a1d1b72a94ddeeb6376";
    pub const CREATE_TOKEN_EVENT: &[u8] =
        &[228, 69, 165, 46, 81, 203, 154, 29, 27, 114, 169, 77, 222, 235, 99, 118];
    // pub const TRADE_EVENT: &str = "0xe445a52e51cb9a1dbddb7fd34ee661ee";
    pub const TRADE_EVENT: &[u8] =
        &[228, 69, 165, 46, 81, 203, 154, 29, 189, 219, 127, 211, 78, 230, 97, 238];
    // pub const COMPLETE_PUMP_AMM_MIGRATION_EVENT: &str = "0xe445a52e51cb9a1dbde95db95c94ea94";
    pub const COMPLETE_PUMP_AMM_MIGRATION_EVENT: &[u8] =
        &[228, 69, 165, 46, 81, 203, 154, 29, 189, 233, 93, 185, 92, 148, 234, 148];

    // 指令鉴别器
    pub const CREATE_TOKEN_IX: &[u8] = &[24, 30, 200, 40, 5, 28, 7, 119];
    pub const CREATE_V2_TOKEN_IX: &[u8] = &[214, 144, 76, 236, 95, 139, 49, 180];
    pub const BUY_IX: &[u8] = &[102, 6, 61, 18, 1, 218, 235, 234];
    pub const BUY_EXACT_SOL_IN_IX: &[u8] = &[56, 252, 116, 8, 158, 223, 205, 95];
    pub const SELL_IX: &[u8] = &[51, 230, 133, 164, 1, 127, 131, 173];
    pub const MIGRATE_IX: &[u8] = &[155, 234, 231, 146, 236, 158, 162, 30];

    // 账户鉴别器
    pub const BONDING_CURVE_ACCOUNT: &[u8] = &[23, 183, 248, 55, 96, 216, 172, 96];
    pub const GLOBAL_ACCOUNT: &[u8] = &[167, 232, 232, 177, 200, 108, 114, 127];
}
