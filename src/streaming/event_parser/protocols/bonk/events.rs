use crate::streaming::event_parser::common::EventMetadata;
use crate::streaming::event_parser::protocols::bonk::types::{
    CurveParams, MintParams, PoolStatus, TradeDirection, VestingParams,
};
use crate::streaming::event_parser::protocols::bonk::{
    AmmFeeOn, GlobalConfig, PlatformConfig, PoolState,
};
use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// Trade event
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct BonkTradeEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub pool_state: Pubkey,
    pub total_base_sell: u64,
    pub virtual_base: u64,
    pub virtual_quote: u64,
    pub real_base_before: u64,
    pub real_quote_before: u64,
    pub real_base_after: u64,
    pub real_quote_after: u64,
    pub amount_in: u64,
    pub amount_out: u64,
    pub protocol_fee: u64,
    pub platform_fee: u64,
    pub creator_fee: u64,
    pub share_fee: u64,
    pub trade_direction: TradeDirection,
    pub pool_status: PoolStatus,
    pub exact_in: bool,
    #[borsh(skip)]
    pub minimum_amount_out: u64,
    #[borsh(skip)]
    pub maximum_amount_in: u64,
    #[borsh(skip)]
    pub share_fee_rate: u64,
    #[borsh(skip)]
    pub payer: Pubkey,
    #[borsh(skip)]
    pub global_config: Pubkey,
    #[borsh(skip)]
    pub platform_config: Pubkey,
    #[borsh(skip)]
    pub user_base_token: Pubkey,
    #[borsh(skip)]
    pub user_quote_token: Pubkey,
    #[borsh(skip)]
    pub base_vault: Pubkey,
    #[borsh(skip)]
    pub quote_vault: Pubkey,
    #[borsh(skip)]
    pub base_token_mint: Pubkey,
    #[borsh(skip)]
    pub quote_token_mint: Pubkey,
    #[borsh(skip)]
    pub base_token_program: Pubkey,
    #[borsh(skip)]
    pub quote_token_program: Pubkey,
    #[borsh(skip)]
    pub is_dev_create_token_trade: bool,
    #[borsh(skip)]
    pub is_bot: bool,
    #[borsh(skip)]
    pub system_program: Pubkey,
    #[borsh(skip)]
    pub platform_associated_account: Pubkey,
    #[borsh(skip)]
    pub creator_associated_account: Pubkey,
}

pub const BONK_TRADE_EVENT_LOG_SIZE: usize = 32 + 8 * 13 + 1 + 1 + 1;

pub fn bonk_trade_event_log_decode(data: &[u8]) -> Option<BonkTradeEvent> {
    if data.len() < BONK_TRADE_EVENT_LOG_SIZE {
        return None;
    }
    borsh::from_slice::<BonkTradeEvent>(&data[..BONK_TRADE_EVENT_LOG_SIZE]).ok()
}

/// Create pool event
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct BonkPoolCreateEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub pool_state: Pubkey,
    pub creator: Pubkey,
    pub config: Pubkey,
    pub base_mint_param: MintParams,
    pub curve_param: CurveParams,
    pub vesting_param: VestingParams,
    pub amm_fee_on: Option<AmmFeeOn>,
    #[borsh(skip)]
    pub payer: Pubkey,
    #[borsh(skip)]
    pub base_mint: Pubkey,
    #[borsh(skip)]
    pub quote_mint: Pubkey,
    #[borsh(skip)]
    pub base_vault: Pubkey,
    #[borsh(skip)]
    pub quote_vault: Pubkey,
    #[borsh(skip)]
    pub global_config: Pubkey,
    #[borsh(skip)]
    pub platform_config: Pubkey,
}

pub const BONK_POOL_CREATE_EVENT_LOG_SIZE: usize = 256;

pub fn bonk_pool_create_event_log_decode(data: &[u8]) -> Option<BonkPoolCreateEvent> {
    if data.len() < BONK_POOL_CREATE_EVENT_LOG_SIZE {
        return None;
    }
    borsh::from_slice::<BonkPoolCreateEvent>(&data[..BONK_POOL_CREATE_EVENT_LOG_SIZE]).ok()
}

/// Create pool event
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct BonkMigrateToAmmEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub base_lot_size: u64,
    pub quote_lot_size: u64,
    pub market_vault_signer_nonce: u8,
    #[borsh(skip)]
    pub payer: Pubkey,
    #[borsh(skip)]
    pub base_mint: Pubkey,
    #[borsh(skip)]
    pub quote_mint: Pubkey,
    #[borsh(skip)]
    pub openbook_program: Pubkey,
    #[borsh(skip)]
    pub market: Pubkey,
    #[borsh(skip)]
    pub request_queue: Pubkey,
    #[borsh(skip)]
    pub event_queue: Pubkey,
    #[borsh(skip)]
    pub bids: Pubkey,
    #[borsh(skip)]
    pub asks: Pubkey,
    #[borsh(skip)]
    pub market_vault_signer: Pubkey,
    #[borsh(skip)]
    pub market_base_vault: Pubkey,
    #[borsh(skip)]
    pub market_quote_vault: Pubkey,
    #[borsh(skip)]
    pub amm_program: Pubkey,
    #[borsh(skip)]
    pub amm_pool: Pubkey,
    #[borsh(skip)]
    pub amm_authority: Pubkey,
    #[borsh(skip)]
    pub amm_open_orders: Pubkey,
    #[borsh(skip)]
    pub amm_lp_mint: Pubkey,
    #[borsh(skip)]
    pub amm_base_vault: Pubkey,
    #[borsh(skip)]
    pub amm_quote_vault: Pubkey,
    #[borsh(skip)]
    pub amm_target_orders: Pubkey,
    #[borsh(skip)]
    pub amm_config: Pubkey,
    #[borsh(skip)]
    pub amm_create_fee_destination: Pubkey,
    #[borsh(skip)]
    pub authority: Pubkey,
    #[borsh(skip)]
    pub pool_state: Pubkey,
    #[borsh(skip)]
    pub global_config: Pubkey,
    #[borsh(skip)]
    pub base_vault: Pubkey,
    #[borsh(skip)]
    pub quote_vault: Pubkey,
    #[borsh(skip)]
    pub pool_lp_token: Pubkey,
    #[borsh(skip)]
    pub spl_token_program: Pubkey,
    #[borsh(skip)]
    pub associated_token_program: Pubkey,
    #[borsh(skip)]
    pub system_program: Pubkey,
    #[borsh(skip)]
    pub rent_program: Pubkey,
}

// Migrate to CP Swap event
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct BonkMigrateToCpswapEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub payer: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub platform_config: Pubkey,
    pub cpswap_program: Pubkey,
    pub cpswap_pool: Pubkey,
    pub cpswap_authority: Pubkey,
    pub cpswap_lp_mint: Pubkey,
    pub cpswap_base_vault: Pubkey,
    pub cpswap_quote_vault: Pubkey,
    pub cpswap_config: Pubkey,
    pub cpswap_create_pool_fee: Pubkey,
    pub cpswap_observation: Pubkey,
    pub lock_program: Pubkey,
    pub lock_authority: Pubkey,
    pub lock_lp_vault: Pubkey,
    pub authority: Pubkey,
    pub pool_state: Pubkey,
    pub global_config: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub pool_lp_token: Pubkey,
    pub base_token_program: Pubkey,
    pub quote_token_program: Pubkey,
    pub associated_token_program: Pubkey,
    pub system_program: Pubkey,
    pub rent_program: Pubkey,
    pub metadata_program: Pubkey,
    pub remaining_accounts: Vec<Pubkey>,
}

/// Pool state account event
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BonkPoolStateAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub pool_state: PoolState,
}

/// Global config account event
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BonkGlobalConfigAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub global_config: GlobalConfig,
}

/// Platform config account event
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BonkPlatformConfigAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub platform_config: PlatformConfig,
}

/// Event discriminator constants
pub mod discriminators {
    // Event discriminators
    // pub const TRADE_EVENT: &str = "0xe445a52e51cb9a1dbddb7fd34ee661ee";
    pub const TRADE_EVENT: &[u8] =
        &[228, 69, 165, 46, 81, 203, 154, 29, 189, 219, 127, 211, 78, 230, 97, 238];
    // pub const POOL_CREATE_EVENT: &str = "0xe445a52e51cb9a1d97d7e20976a173ae";
    pub const POOL_CREATE_EVENT: &[u8] =
        &[228, 69, 165, 46, 81, 203, 154, 29, 151, 215, 226, 9, 118, 161, 115, 174];

    // Instruction discriminators
    pub const BUY_EXACT_IN: &[u8] = &[250, 234, 13, 123, 213, 156, 19, 236];
    pub const BUY_EXACT_OUT: &[u8] = &[24, 211, 116, 40, 105, 3, 153, 56];
    pub const SELL_EXACT_IN: &[u8] = &[149, 39, 222, 155, 211, 124, 152, 26];
    pub const SELL_EXACT_OUT: &[u8] = &[95, 200, 71, 34, 8, 9, 11, 166];
    pub const INITIALIZE: &[u8] = &[175, 175, 109, 31, 13, 152, 155, 237];
    pub const INITIALIZE_V2: &[u8] = &[67, 153, 175, 39, 218, 16, 38, 32];
    pub const INITIALIZE_WITH_TOKEN_2022: &[u8] = &[37, 190, 126, 222, 44, 154, 171, 17];
    pub const MIGRATE_TO_AMM: &[u8] = &[207, 82, 192, 145, 254, 207, 145, 223];
    pub const MIGRATE_TO_CP_SWAP: &[u8] = &[136, 92, 200, 103, 28, 218, 144, 140];

    // 账户鉴别器
    pub const POOL_STATE_ACCOUNT: &[u8] = &[247, 237, 227, 245, 215, 195, 222, 70];
    pub const GLOBAL_CONFIG_ACCOUNT: &[u8] = &[149, 8, 156, 202, 160, 252, 176, 217];
    pub const PLATFORM_CONFIG_ACCOUNT: &[u8] = &[160, 78, 128, 0, 248, 83, 230, 160];
}
