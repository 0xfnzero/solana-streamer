use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

use crate::streaming::event_parser::common::utils::{read_i64_le, read_u32_le, read_u64_le};
use crate::streaming::event_parser::common::EventMetadata;
use crate::streaming::event_parser::protocols::pumpswap::types::{GlobalConfig, Pool};

/// 买入事件
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PumpSwapBuyEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub timestamp: i64,
    pub base_amount_out: u64,
    pub max_quote_amount_in: u64,
    pub user_base_token_reserves: u64,
    pub user_quote_token_reserves: u64,
    pub pool_base_token_reserves: u64,
    pub pool_quote_token_reserves: u64,
    pub quote_amount_in: u64,
    pub lp_fee_basis_points: u64,
    pub lp_fee: u64,
    pub protocol_fee_basis_points: u64,
    pub protocol_fee: u64,
    pub quote_amount_in_with_lp_fee: u64,
    pub user_quote_amount_in: u64,
    pub pool: Pubkey,
    pub user: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub protocol_fee_recipient: Pubkey,
    pub protocol_fee_recipient_token_account: Pubkey,
    pub coin_creator: Pubkey,
    pub coin_creator_fee_basis_points: u64,
    pub coin_creator_fee: u64,
    pub track_volume: bool,
    pub total_unclaimed_tokens: u64,
    pub total_claimed_tokens: u64,
    pub current_sol_volume: u64,
    pub last_update_timestamp: i64,
    /// Minimum base out (IDL extension; also set on `buy_exact_quote_in` from ix args).
    pub min_base_amount_out: u64,
    /// Instruction name from event (`buy`, `buy_exact_quote_in`, …).
    pub ix_name: String,
    pub cashback_fee_basis_points: u64,
    pub cashback: u64,
    #[borsh(skip)]
    pub is_pump_pool: bool,
    #[borsh(skip)]
    pub base_mint: Pubkey,
    #[borsh(skip)]
    pub quote_mint: Pubkey,
    #[borsh(skip)]
    pub pool_base_token_account: Pubkey,
    #[borsh(skip)]
    pub pool_quote_token_account: Pubkey,
    #[borsh(skip)]
    pub coin_creator_vault_ata: Pubkey,
    #[borsh(skip)]
    pub coin_creator_vault_authority: Pubkey,
    #[borsh(skip)]
    pub base_token_program: Pubkey,
    #[borsh(skip)]
    pub quote_token_program: Pubkey,
}

/// Minimum bytes through `last_update_timestamp` (Anchor/Borsh layout, bool = 1 byte).
pub const PUMP_SWAP_BUY_EVENT_LOG_MIN: usize = 385;
/// Backwards-compatible name for the minimum BuyEvent payload size (legacy `borsh` slice length).
pub const PUMP_SWAP_BUY_EVENT_LOG_SIZE: usize = PUMP_SWAP_BUY_EVENT_LOG_MIN;

pub fn pump_swap_buy_event_log_decode(data: &[u8]) -> Option<PumpSwapBuyEvent> {
    if data.len() < PUMP_SWAP_BUY_EVENT_LOG_MIN {
        return None;
    }
    let timestamp = read_i64_le(data, 0)?;
    let base_amount_out = read_u64_le(data, 8)?;
    let max_quote_amount_in = read_u64_le(data, 16)?;
    let user_base_token_reserves = read_u64_le(data, 24)?;
    let user_quote_token_reserves = read_u64_le(data, 32)?;
    let pool_base_token_reserves = read_u64_le(data, 40)?;
    let pool_quote_token_reserves = read_u64_le(data, 48)?;
    let quote_amount_in = read_u64_le(data, 56)?;
    let lp_fee_basis_points = read_u64_le(data, 64)?;
    let lp_fee = read_u64_le(data, 72)?;
    let protocol_fee_basis_points = read_u64_le(data, 80)?;
    let protocol_fee = read_u64_le(data, 88)?;
    let quote_amount_in_with_lp_fee = read_u64_le(data, 96)?;
    let user_quote_amount_in = read_u64_le(data, 104)?;
    let pool = Pubkey::new_from_array(data.get(112..144)?.try_into().ok()?);
    let user = Pubkey::new_from_array(data.get(144..176)?.try_into().ok()?);
    let user_base_token_account = Pubkey::new_from_array(data.get(176..208)?.try_into().ok()?);
    let user_quote_token_account = Pubkey::new_from_array(data.get(208..240)?.try_into().ok()?);
    let protocol_fee_recipient = Pubkey::new_from_array(data.get(240..272)?.try_into().ok()?);
    let protocol_fee_recipient_token_account =
        Pubkey::new_from_array(data.get(272..304)?.try_into().ok()?);
    let coin_creator = Pubkey::new_from_array(data.get(304..336)?.try_into().ok()?);
    let coin_creator_fee_basis_points = read_u64_le(data, 336)?;
    let coin_creator_fee = read_u64_le(data, 344)?;
    let track_volume = *data.get(352)? != 0;
    let total_unclaimed_tokens = read_u64_le(data, 353)?;
    let total_claimed_tokens = read_u64_le(data, 361)?;
    let current_sol_volume = read_u64_le(data, 369)?;
    let last_update_timestamp = read_i64_le(data, 377)?;

    let mut offset = 385usize;
    let min_base_amount_out = if data.len() >= offset + 8 {
        let v = read_u64_le(data, offset)?;
        offset += 8;
        v
    } else {
        0
    };

    let ix_name = if data.len() >= offset + 4 {
        let slen = read_u32_le(data, offset)? as usize;
        let str_start = offset + 4;
        if data.len() < str_start + slen {
            return None;
        }
        offset = str_start + slen;
        String::from_utf8_lossy(&data[str_start..offset]).into_owned()
    } else {
        String::new()
    };

    let cashback_fee_basis_points = read_u64_le(data, offset).unwrap_or(0);
    let cashback = read_u64_le(data, offset + 8).unwrap_or(0);

    Some(PumpSwapBuyEvent {
        metadata: EventMetadata::default(),
        timestamp,
        base_amount_out,
        max_quote_amount_in,
        user_base_token_reserves,
        user_quote_token_reserves,
        pool_base_token_reserves,
        pool_quote_token_reserves,
        quote_amount_in,
        lp_fee_basis_points,
        lp_fee,
        protocol_fee_basis_points,
        protocol_fee,
        quote_amount_in_with_lp_fee,
        user_quote_amount_in,
        pool,
        user,
        user_base_token_account,
        user_quote_token_account,
        protocol_fee_recipient,
        protocol_fee_recipient_token_account,
        coin_creator,
        coin_creator_fee_basis_points,
        coin_creator_fee,
        track_volume,
        total_unclaimed_tokens,
        total_claimed_tokens,
        current_sol_volume,
        last_update_timestamp,
        min_base_amount_out,
        ix_name,
        cashback_fee_basis_points,
        cashback,
        ..Default::default()
    })
}

/// 卖出事件
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PumpSwapSellEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub timestamp: i64,
    pub base_amount_in: u64,
    pub min_quote_amount_out: u64,
    pub user_base_token_reserves: u64,
    pub user_quote_token_reserves: u64,
    pub pool_base_token_reserves: u64,
    pub pool_quote_token_reserves: u64,
    pub quote_amount_out: u64,
    pub lp_fee_basis_points: u64,
    pub lp_fee: u64,
    pub protocol_fee_basis_points: u64,
    pub protocol_fee: u64,
    pub quote_amount_out_without_lp_fee: u64,
    pub user_quote_amount_out: u64,
    pub pool: Pubkey,
    pub user: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub protocol_fee_recipient: Pubkey,
    pub protocol_fee_recipient_token_account: Pubkey,
    pub coin_creator: Pubkey,
    pub coin_creator_fee_basis_points: u64,
    pub coin_creator_fee: u64,
    pub cashback_fee_basis_points: u64,
    pub cashback: u64,
    #[borsh(skip)]
    pub is_pump_pool: bool,
    #[borsh(skip)]
    pub base_mint: Pubkey,
    #[borsh(skip)]
    pub quote_mint: Pubkey,
    #[borsh(skip)]
    pub pool_base_token_account: Pubkey,
    #[borsh(skip)]
    pub pool_quote_token_account: Pubkey,
    #[borsh(skip)]
    pub coin_creator_vault_ata: Pubkey,
    #[borsh(skip)]
    pub coin_creator_vault_authority: Pubkey,
    #[borsh(skip)]
    pub base_token_program: Pubkey,
    #[borsh(skip)]
    pub quote_token_program: Pubkey,
}

pub const PUMP_SWAP_SELL_EVENT_LOG_MIN: usize = 352;
pub const PUMP_SWAP_SELL_EVENT_WITH_CASHBACK: usize = 368;
/// Backwards-compatible name for the pre-cashback SellEvent payload size.
pub const PUMP_SWAP_SELL_EVENT_LOG_SIZE: usize = PUMP_SWAP_SELL_EVENT_LOG_MIN;

pub fn pump_swap_sell_event_log_decode(data: &[u8]) -> Option<PumpSwapSellEvent> {
    if data.len() < PUMP_SWAP_SELL_EVENT_LOG_MIN {
        return None;
    }
    let timestamp = read_i64_le(data, 0)?;
    let base_amount_in = read_u64_le(data, 8)?;
    let min_quote_amount_out = read_u64_le(data, 16)?;
    let user_base_token_reserves = read_u64_le(data, 24)?;
    let user_quote_token_reserves = read_u64_le(data, 32)?;
    let pool_base_token_reserves = read_u64_le(data, 40)?;
    let pool_quote_token_reserves = read_u64_le(data, 48)?;
    let quote_amount_out = read_u64_le(data, 56)?;
    let lp_fee_basis_points = read_u64_le(data, 64)?;
    let lp_fee = read_u64_le(data, 72)?;
    let protocol_fee_basis_points = read_u64_le(data, 80)?;
    let protocol_fee = read_u64_le(data, 88)?;
    let quote_amount_out_without_lp_fee = read_u64_le(data, 96)?;
    let user_quote_amount_out = read_u64_le(data, 104)?;
    let pool = Pubkey::new_from_array(data.get(112..144)?.try_into().ok()?);
    let user = Pubkey::new_from_array(data.get(144..176)?.try_into().ok()?);
    let user_base_token_account = Pubkey::new_from_array(data.get(176..208)?.try_into().ok()?);
    let user_quote_token_account = Pubkey::new_from_array(data.get(208..240)?.try_into().ok()?);
    let protocol_fee_recipient = Pubkey::new_from_array(data.get(240..272)?.try_into().ok()?);
    let protocol_fee_recipient_token_account =
        Pubkey::new_from_array(data.get(272..304)?.try_into().ok()?);
    let coin_creator = Pubkey::new_from_array(data.get(304..336)?.try_into().ok()?);
    let coin_creator_fee_basis_points = read_u64_le(data, 336)?;
    let coin_creator_fee = read_u64_le(data, 344)?;
    let (cashback_fee_basis_points, cashback) = if data.len() >= PUMP_SWAP_SELL_EVENT_WITH_CASHBACK
    {
        (read_u64_le(data, 352)?, read_u64_le(data, 360)?)
    } else {
        (0, 0)
    };

    Some(PumpSwapSellEvent {
        metadata: EventMetadata::default(),
        timestamp,
        base_amount_in,
        min_quote_amount_out,
        user_base_token_reserves,
        user_quote_token_reserves,
        pool_base_token_reserves,
        pool_quote_token_reserves,
        quote_amount_out,
        lp_fee_basis_points,
        lp_fee,
        protocol_fee_basis_points,
        protocol_fee,
        quote_amount_out_without_lp_fee,
        user_quote_amount_out,
        pool,
        user,
        user_base_token_account,
        user_quote_token_account,
        protocol_fee_recipient,
        protocol_fee_recipient_token_account,
        coin_creator,
        coin_creator_fee_basis_points,
        coin_creator_fee,
        cashback_fee_basis_points,
        cashback,
        ..Default::default()
    })
}

/// 创建池子事件
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PumpSwapCreatePoolEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub timestamp: i64,
    pub index: u16,
    pub creator: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_mint_decimals: u8,
    pub quote_mint_decimals: u8,
    pub base_amount_in: u64,
    pub quote_amount_in: u64,
    pub pool_base_amount: u64,
    pub pool_quote_amount: u64,
    pub minimum_liquidity: u64,
    pub initial_liquidity: u64,
    pub lp_token_amount_out: u64,
    pub pool_bump: u8,
    pub pool: Pubkey,
    pub lp_mint: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub coin_creator: Pubkey,
    #[borsh(skip)]
    pub user_pool_token_account: Pubkey,
    #[borsh(skip)]
    pub pool_base_token_account: Pubkey,
    #[borsh(skip)]
    pub pool_quote_token_account: Pubkey,
}

pub const PUMP_SWAP_CREATE_POOL_EVENT_LOG_SIZE: usize = 325;

pub fn pump_swap_create_pool_event_log_decode(data: &[u8]) -> Option<PumpSwapCreatePoolEvent> {
    if data.len() < PUMP_SWAP_CREATE_POOL_EVENT_LOG_SIZE {
        return None;
    }
    borsh::from_slice::<PumpSwapCreatePoolEvent>(&data[..PUMP_SWAP_CREATE_POOL_EVENT_LOG_SIZE]).ok()
}

/// 存款事件
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PumpSwapDepositEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub timestamp: i64,
    pub lp_token_amount_out: u64,
    pub max_base_amount_in: u64,
    pub max_quote_amount_in: u64,
    pub user_base_token_reserves: u64,
    pub user_quote_token_reserves: u64,
    pub pool_base_token_reserves: u64,
    pub pool_quote_token_reserves: u64,
    pub base_amount_in: u64,
    pub quote_amount_in: u64,
    pub lp_mint_supply: u64,
    pub pool: Pubkey,
    pub user: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub user_pool_token_account: Pubkey,
    #[borsh(skip)]
    pub base_mint: Pubkey,
    #[borsh(skip)]
    pub quote_mint: Pubkey,
    #[borsh(skip)]
    pub pool_base_token_account: Pubkey,
    #[borsh(skip)]
    pub pool_quote_token_account: Pubkey,
}

pub const PUMP_SWAP_DEPOSIT_EVENT_LOG_SIZE: usize = 248;

pub fn pump_swap_deposit_event_log_decode(data: &[u8]) -> Option<PumpSwapDepositEvent> {
    if data.len() < PUMP_SWAP_DEPOSIT_EVENT_LOG_SIZE {
        return None;
    }
    borsh::from_slice::<PumpSwapDepositEvent>(&data[..PUMP_SWAP_DEPOSIT_EVENT_LOG_SIZE]).ok()
}

/// 提款事件
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PumpSwapWithdrawEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub timestamp: i64,
    pub lp_token_amount_in: u64,
    pub min_base_amount_out: u64,
    pub min_quote_amount_out: u64,
    pub user_base_token_reserves: u64,
    pub user_quote_token_reserves: u64,
    pub pool_base_token_reserves: u64,
    pub pool_quote_token_reserves: u64,
    pub base_amount_out: u64,
    pub quote_amount_out: u64,
    pub lp_mint_supply: u64,
    pub pool: Pubkey,
    pub user: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub user_pool_token_account: Pubkey,
    #[borsh(skip)]
    pub base_mint: Pubkey,
    #[borsh(skip)]
    pub quote_mint: Pubkey,
    #[borsh(skip)]
    pub pool_base_token_account: Pubkey,
    #[borsh(skip)]
    pub pool_quote_token_account: Pubkey,
}

pub const PUMP_SWAP_WITHDRAW_EVENT_LOG_SIZE: usize = 248;

pub fn pump_swap_withdraw_event_log_decode(data: &[u8]) -> Option<PumpSwapWithdrawEvent> {
    if data.len() < PUMP_SWAP_WITHDRAW_EVENT_LOG_SIZE {
        return None;
    }
    borsh::from_slice::<PumpSwapWithdrawEvent>(&data[..PUMP_SWAP_WITHDRAW_EVENT_LOG_SIZE]).ok()
}

/// 全局配置
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PumpSwapGlobalConfigAccountEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub global_config: GlobalConfig,
}

/// 池
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PumpSwapPoolAccountEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub pool: Pool,
}

/// 事件鉴别器常量
pub mod discriminators {
    // 事件鉴别器
    // pub const BUY_EVENT: &str = "0xe445a52e51cb9a1d67f4521f2cf57777";
    pub const BUY_EVENT: &[u8] =
        &[228, 69, 165, 46, 81, 203, 154, 29, 103, 244, 82, 31, 44, 245, 119, 119];
    // pub const SELL_EVENT: &str = "0xe445a52e51cb9a1d3e2f370aa503dc2a";
    pub const SELL_EVENT: &[u8] =
        &[228, 69, 165, 46, 81, 203, 154, 29, 62, 47, 55, 10, 165, 3, 220, 42];
    // pub const CREATE_POOL_EVENT: &str = "0xe445a52e51cb9a1db1310cd2a076a774";
    pub const CREATE_POOL_EVENT: &[u8] =
        &[228, 69, 165, 46, 81, 203, 154, 29, 177, 49, 12, 210, 160, 118, 167, 116];
    // pub const DEPOSIT_EVENT: &str = "0xe445a52e51cb9a1d78f83d531f8e6b90";
    pub const DEPOSIT_EVENT: &[u8] =
        &[228, 69, 165, 46, 81, 203, 154, 29, 120, 248, 61, 83, 31, 142, 107, 144];
    // pub const WITHDRAW_EVENT: &str = "0xe445a52e51cb9a1d1609851aa02c47c0";
    pub const WITHDRAW_EVENT: &[u8] =
        &[228, 69, 165, 46, 81, 203, 154, 29, 22, 9, 133, 26, 160, 44, 71, 192];

    // 指令鉴别器
    pub const BUY_IX: &[u8] = &[102, 6, 61, 18, 1, 218, 235, 234];
    pub const BUY_EXACT_QUOTE_IN_IX: &[u8] = &[198, 46, 21, 82, 180, 217, 232, 112];
    pub const SELL_IX: &[u8] = &[51, 230, 133, 164, 1, 127, 131, 173];
    pub const CREATE_POOL_IX: &[u8] = &[233, 146, 209, 142, 207, 104, 64, 188];
    pub const DEPOSIT_IX: &[u8] = &[242, 35, 198, 137, 82, 225, 242, 182];
    pub const WITHDRAW_IX: &[u8] = &[183, 18, 70, 156, 148, 109, 161, 34];

    // 账户鉴别器
    pub const GLOBAL_CONFIG_ACCOUNT: &[u8] = &[149, 8, 156, 202, 160, 252, 176, 217];
    pub const POOL_ACCOUNT: &[u8] = &[241, 154, 109, 4, 17, 177, 109, 188];
}
