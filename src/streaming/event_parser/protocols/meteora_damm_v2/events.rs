use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

use crate::streaming::event_parser::common::EventMetadata;

/// Base fee parameters
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct BaseFeeParameters {
    pub cliff_fee_numerator: u64,
    pub first_factor: u16,
    pub second_factor: [u8; 8],
    pub third_factor: u64,
    pub base_fee_mode: u8,
}

/// Dynamic fee parameters
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct DynamicFeeParameters {
    pub bin_step: u16,
    pub bin_step_u128: u128,
    pub filter_period: u16,
    pub decay_period: u16,
    pub reduction_factor: u16,
    pub max_volatility_accumulator: u32,
    pub variable_fee_control: u32,
}

/// Pool fee parameters
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PoolFeeParameters {
    pub base_fee: BaseFeeParameters,
    pub padding: [u8; 3],
    pub dynamic_fee: Option<DynamicFeeParameters>,
}

/// Meteora DAMM v2 Swap Event (对应 swap 指令)
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct MeteoraDammV2SwapEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,

    // 来自 CPI Log Event 的数据
    pub pool: Pubkey,
    pub trade_direction: u8, // 0 or 1
    pub collect_fee_mode: u8,
    pub has_referral: bool,

    // Swap parameters
    pub amount_0: u64, // amount0 from params
    pub amount_1: u64, // amount1 from params
    pub swap_mode: u8, // swapMode from params

    // Swap result
    pub included_fee_input_amount: u64,
    pub excluded_fee_input_amount: u64,
    pub amount_left: u64,
    pub output_amount: u64,
    pub next_sqrt_price: u128,
    pub trading_fee: u64,
    pub protocol_fee: u64,
    pub partner_fee: u64,
    pub referral_fee: u64,

    // Transfer fee amounts
    pub included_transfer_fee_amount_in: u64,
    pub included_transfer_fee_amount_out: u64,
    pub excluded_transfer_fee_amount_out: u64,

    // Additional info
    pub current_timestamp: u64,
    pub reserve_a_amount: u64,
    pub reserve_b_amount: u64,

    // 来自 Input Accounts 的数据
    #[borsh(skip)]
    pub pool_authority: Pubkey,
    #[borsh(skip)]
    pub input_token_account: Pubkey,
    #[borsh(skip)]
    pub output_token_account: Pubkey,
    #[borsh(skip)]
    pub token_a_vault: Pubkey,
    #[borsh(skip)]
    pub token_b_vault: Pubkey,
    #[borsh(skip)]
    pub token_a_mint: Pubkey,
    #[borsh(skip)]
    pub token_b_mint: Pubkey,
    #[borsh(skip)]
    pub payer: Pubkey,
    #[borsh(skip)]
    pub token_a_program: Pubkey,
    #[borsh(skip)]
    pub token_b_program: Pubkey,
    #[borsh(skip)]
    pub referral_token_account: Option<Pubkey>,
    #[borsh(skip)]
    pub event_authority: Pubkey,
    #[borsh(skip)]
    pub program: Pubkey,
}

/// Meteora DAMM v2 Swap2 Event (对应 swap2 指令)
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct MeteoraDammV2Swap2Event {
    #[borsh(skip)]
    pub metadata: EventMetadata,

    // 来自 CPI Log Event 的数据
    pub pool: Pubkey,
    pub trade_direction: u8, // 0 or 1
    pub collect_fee_mode: u8,
    pub has_referral: bool,

    // Swap parameters
    pub amount_0: u64, // amount0 from params
    pub amount_1: u64, // amount1 from params
    pub swap_mode: u8, // swapMode from params

    // Swap result
    pub included_fee_input_amount: u64,
    pub excluded_fee_input_amount: u64,
    pub amount_left: u64,
    pub output_amount: u64,
    pub next_sqrt_price: u128,
    pub trading_fee: u64,
    pub protocol_fee: u64,
    pub partner_fee: u64,
    pub referral_fee: u64,

    // Transfer fee amounts
    pub included_transfer_fee_amount_in: u64,
    pub included_transfer_fee_amount_out: u64,
    pub excluded_transfer_fee_amount_out: u64,

    // Additional info
    pub current_timestamp: u64,
    pub reserve_a_amount: u64,
    pub reserve_b_amount: u64,

    // 来自 Input Accounts 的数据
    #[borsh(skip)]
    pub pool_authority: Pubkey,
    #[borsh(skip)]
    pub input_token_account: Pubkey,
    #[borsh(skip)]
    pub output_token_account: Pubkey,
    #[borsh(skip)]
    pub token_a_vault: Pubkey,
    #[borsh(skip)]
    pub token_b_vault: Pubkey,
    #[borsh(skip)]
    pub token_a_mint: Pubkey,
    #[borsh(skip)]
    pub token_b_mint: Pubkey,
    #[borsh(skip)]
    pub payer: Pubkey,
    #[borsh(skip)]
    pub token_a_program: Pubkey,
    #[borsh(skip)]
    pub token_b_program: Pubkey,
    #[borsh(skip)]
    pub referral_token_account: Option<Pubkey>,
    #[borsh(skip)]
    pub event_authority: Pubkey,
    #[borsh(skip)]
    pub program: Pubkey,
    #[borsh(skip)]
    pub sysvar: Pubkey,
}

/// Meteora DAMM v2 Initialize Pool Event (对应 initialize_pool 指令)
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct MeteoraDammV2InitializePoolEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,

    // 来自 CPI Log Event 的数据
    pub pool: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub creator: Pubkey,
    pub payer: Pubkey,
    pub alpha_vault: Pubkey,

    // Pool fees
    pub pool_fees: PoolFeeParameters,

    // Price and liquidity
    pub sqrt_min_price: u128,
    pub sqrt_max_price: u128,
    pub activation_type: u8,
    pub collect_fee_mode: u8,
    pub liquidity: u128,
    pub sqrt_price: u128,
    pub activation_point: u64,

    // Token amounts
    pub token_a_flag: u8,
    pub token_b_flag: u8,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
    pub total_amount_a: u64,
    pub total_amount_b: u64,
    pub pool_type: u8,

    // 来自 Input Accounts 的数据
    #[borsh(skip)]
    pub position_nft_mint: Pubkey,
    #[borsh(skip)]
    pub position_nft_account: Pubkey,
    #[borsh(skip)]
    pub pool_authority: Pubkey,
    #[borsh(skip)]
    pub position: Pubkey,
    #[borsh(skip)]
    pub token_a_vault: Pubkey,
    #[borsh(skip)]
    pub token_b_vault: Pubkey,
    #[borsh(skip)]
    pub payer_token_a: Pubkey,
    #[borsh(skip)]
    pub payer_token_b: Pubkey,
    #[borsh(skip)]
    pub token_a_program: Pubkey,
    #[borsh(skip)]
    pub token_b_program: Pubkey,
    #[borsh(skip)]
    pub event_authority: Pubkey,
    #[borsh(skip)]
    pub program: Pubkey,
    #[borsh(skip)]
    pub config: Pubkey,
    #[borsh(skip)]
    pub remaining_accounts: Vec<Pubkey>,
}

/// Meteora DAMM v2 Initialize Customizable Pool Event (对应 initialize_customizable_pool 指令)
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct MeteoraDammV2InitializeCustomizablePoolEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,

    // 来自 CPI Log Event 的数据
    pub pool: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub creator: Pubkey,
    pub payer: Pubkey,
    pub alpha_vault: Pubkey,

    // Pool fees
    pub pool_fees: PoolFeeParameters,

    // Price and liquidity
    pub sqrt_min_price: u128,
    pub sqrt_max_price: u128,
    pub activation_type: u8,
    pub collect_fee_mode: u8,
    pub liquidity: u128,
    pub sqrt_price: u128,
    pub activation_point: u64,

    // Token amounts
    pub token_a_flag: u8,
    pub token_b_flag: u8,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
    pub total_amount_a: u64,
    pub total_amount_b: u64,
    pub pool_type: u8,

    // 来自 Input Accounts 的数据
    #[borsh(skip)]
    pub position_nft_mint: Pubkey,
    #[borsh(skip)]
    pub position_nft_account: Pubkey,
    #[borsh(skip)]
    pub pool_authority: Pubkey,
    #[borsh(skip)]
    pub position: Pubkey,
    #[borsh(skip)]
    pub token_a_vault: Pubkey,
    #[borsh(skip)]
    pub token_b_vault: Pubkey,
    #[borsh(skip)]
    pub payer_token_a: Pubkey,
    #[borsh(skip)]
    pub payer_token_b: Pubkey,
    #[borsh(skip)]
    pub token_a_program: Pubkey,
    #[borsh(skip)]
    pub token_b_program: Pubkey,
    #[borsh(skip)]
    pub token_2022_program: Pubkey,
    #[borsh(skip)]
    pub system_program: Pubkey,
    #[borsh(skip)]
    pub event_authority: Pubkey,
    #[borsh(skip)]
    pub program: Pubkey,
    #[borsh(skip)]
    pub remaining_accounts: Vec<Pubkey>,
}

/// Meteora DAMM v2 Initialize Pool With Dynamic Config Event (对应 initialize_pool_with_dynamic_config 指令)
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct MeteoraDammV2InitializePoolWithDynamicConfigEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,

    // 来自 CPI Log Event 的数据
    pub pool: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub creator: Pubkey,
    pub payer: Pubkey,
    pub alpha_vault: Pubkey,

    // Pool fees
    pub pool_fees: PoolFeeParameters,

    // Price and liquidity
    pub sqrt_min_price: u128,
    pub sqrt_max_price: u128,
    pub activation_type: u8,
    pub collect_fee_mode: u8,
    pub liquidity: u128,
    pub sqrt_price: u128,
    pub activation_point: u64,

    // Token amounts
    pub token_a_flag: u8,
    pub token_b_flag: u8,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
    pub total_amount_a: u64,
    pub total_amount_b: u64,
    pub pool_type: u8,

    // 来自 Input Accounts 的数据
    #[borsh(skip)]
    pub position_nft_mint: Pubkey,
    #[borsh(skip)]
    pub position_nft_account: Pubkey,
    #[borsh(skip)]
    pub pool_authority: Pubkey,
    #[borsh(skip)]
    pub pool_creator_authority: Pubkey,
    #[borsh(skip)]
    pub position: Pubkey,
    #[borsh(skip)]
    pub token_a_vault: Pubkey,
    #[borsh(skip)]
    pub token_b_vault: Pubkey,
    #[borsh(skip)]
    pub payer_token_a: Pubkey,
    #[borsh(skip)]
    pub payer_token_b: Pubkey,
    #[borsh(skip)]
    pub token_a_program: Pubkey,
    #[borsh(skip)]
    pub token_b_program: Pubkey,
    #[borsh(skip)]
    pub token_2022_program: Pubkey,
    #[borsh(skip)]
    pub system_program: Pubkey,
    #[borsh(skip)]
    pub event_authority: Pubkey,
    #[borsh(skip)]
    pub program: Pubkey,
    #[borsh(skip)]
    pub config: Pubkey,
}

/// DAMM v2 Add Liquidity（parser-sdk / CPI 日志字段对齐）
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct MeteoraDammV2AddLiquidityEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub pool: Pubkey,
    pub position: Pubkey,
    pub owner: Pubkey,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
    #[borsh(skip)]
    pub liquidity_delta: u128,
    #[borsh(skip)]
    pub token_a_amount_threshold: u64,
    #[borsh(skip)]
    pub token_b_amount_threshold: u64,
    #[borsh(skip)]
    pub total_amount_a: u64,
    #[borsh(skip)]
    pub total_amount_b: u64,
}

/// DAMM v2 Remove Liquidity
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct MeteoraDammV2RemoveLiquidityEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub pool: Pubkey,
    pub position: Pubkey,
    pub owner: Pubkey,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
    #[borsh(skip)]
    pub liquidity_delta: u128,
    #[borsh(skip)]
    pub token_a_amount_threshold: u64,
    #[borsh(skip)]
    pub token_b_amount_threshold: u64,
}

/// DAMM v2 Create Position
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct MeteoraDammV2CreatePositionEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub pool: Pubkey,
    pub owner: Pubkey,
    pub position: Pubkey,
    pub position_nft_mint: Pubkey,
}

/// DAMM v2 Close Position
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct MeteoraDammV2ClosePositionEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub pool: Pubkey,
    pub owner: Pubkey,
    pub position: Pubkey,
    pub position_nft_mint: Pubkey,
}

/// Event discriminators
pub mod discriminators {
    // Instruction discriminators
    // 从文档中提取的 instruction data 第一个 8 bytes
    pub const SWAP_IX: &[u8] = &[0xf8, 0xc6, 0x9e, 0x91, 0xe1, 0x75, 0x87, 0xc8]; // swap
    pub const SWAP2_IX: &[u8] = &[0x41, 0x4b, 0x3f, 0x4c, 0xeb, 0x5b, 0x5b, 0x88]; // swap2
    pub const INITIALIZE_CUSTOMIZABLE_POOL_IX: &[u8] =
        &[0x14, 0xa1, 0xf1, 0x18, 0xbd, 0xdd, 0xb4, 0x02]; // initialize_customizable_pool
    pub const INITIALIZE_POOL_IX: &[u8] = &[0x5f, 0xb4, 0x0a, 0xac, 0x54, 0xae, 0xe8, 0x28]; // initialize_pool
    pub const INITIALIZE_POOL_WITH_DYNAMIC_CONFIG_IX: &[u8] =
        &[0x95, 0x52, 0x48, 0xc5, 0xfd, 0xfc, 0x44, 0x0f]; // initialize_pool_with_dynamic_config

    // Event discriminators (CPI Log Event)
    // e445a52e51cb9a1d 是 Meteora 的事件前缀
    // 后面的 8 字节是具体事件类型
    pub const SWAP_EVENT: &[u8] = &[
        0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0xbd, 0x42, 0x33, 0xa8, 0x26, 0x50, 0x75,
        0x99,
    ]; // swap event
    pub const INITIALIZE_POOL_EVENT: &[u8] = &[
        0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0xe4, 0x32, 0xf6, 0x55, 0xcb, 0x42, 0x86,
        0x25,
    ]; // initialize pool event
}

/// Decode swap event from CPI log
pub const METEORA_DAMM_V2_SWAP_EVENT_LOG_SIZE: usize = 180;
