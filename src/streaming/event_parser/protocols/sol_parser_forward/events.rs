use crate::streaming::event_parser::common::EventMetadata;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// `sol-parser-sdk` 错误占位（无有效 EventMetadata 字段）
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParserSdkErrorEvent {
    pub metadata: EventMetadata,
    pub message: String,
}

// --- Orca Whirlpool ---

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrcaWhirlpoolSwapEvent {
    pub metadata: EventMetadata,
    pub whirlpool: Pubkey,
    pub input_amount: u64,
    pub output_amount: u64,
    pub a_to_b: bool,
    pub pre_sqrt_price: u128,
    pub post_sqrt_price: u128,
    pub input_transfer_fee: u64,
    pub output_transfer_fee: u64,
    pub lp_fee: u64,
    pub protocol_fee: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrcaWhirlpoolLiquidityIncreasedEvent {
    pub metadata: EventMetadata,
    pub whirlpool: Pubkey,
    pub liquidity: u128,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
    pub position: Pubkey,
    pub tick_lower_index: i32,
    pub tick_upper_index: i32,
    pub token_a_transfer_fee: u64,
    pub token_b_transfer_fee: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrcaWhirlpoolLiquidityDecreasedEvent {
    pub metadata: EventMetadata,
    pub whirlpool: Pubkey,
    pub liquidity: u128,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
    pub position: Pubkey,
    pub tick_lower_index: i32,
    pub tick_upper_index: i32,
    pub token_a_transfer_fee: u64,
    pub token_b_transfer_fee: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrcaWhirlpoolPoolInitializedEvent {
    pub metadata: EventMetadata,
    pub whirlpool: Pubkey,
    pub whirlpools_config: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub tick_spacing: u16,
    pub token_program_a: Pubkey,
    pub token_program_b: Pubkey,
    pub decimals_a: u8,
    pub decimals_b: u8,
    pub initial_sqrt_price: u128,
}

// --- Meteora Pools ---

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeteoraPoolsSwapEvent {
    pub metadata: EventMetadata,
    pub in_amount: u64,
    pub out_amount: u64,
    pub trade_fee: u64,
    pub admin_fee: u64,
    pub host_fee: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeteoraPoolsAddLiquidityEvent {
    pub metadata: EventMetadata,
    pub lp_mint_amount: u64,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeteoraPoolsRemoveLiquidityEvent {
    pub metadata: EventMetadata,
    pub lp_unmint_amount: u64,
    pub token_a_out_amount: u64,
    pub token_b_out_amount: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeteoraPoolsBootstrapLiquidityEvent {
    pub metadata: EventMetadata,
    pub lp_mint_amount: u64,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
    pub pool: Pubkey,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeteoraPoolsPoolCreatedEvent {
    pub metadata: EventMetadata,
    pub lp_mint: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub pool_type: u8,
    pub pool: Pubkey,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeteoraPoolsSetPoolFeesEvent {
    pub metadata: EventMetadata,
    pub trade_fee_numerator: u64,
    pub trade_fee_denominator: u64,
    pub owner_trade_fee_numerator: u64,
    pub owner_trade_fee_denominator: u64,
    pub pool: Pubkey,
}

// --- Meteora DLMM ---

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeteoraDlmmSwapEvent {
    pub metadata: EventMetadata,
    pub pool: Pubkey,
    pub from: Pubkey,
    pub start_bin_id: i32,
    pub end_bin_id: i32,
    pub amount_in: u64,
    pub amount_out: u64,
    pub swap_for_y: bool,
    pub fee: u64,
    pub protocol_fee: u64,
    pub fee_bps: u128,
    pub host_fee: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeteoraDlmmAddLiquidityEvent {
    pub metadata: EventMetadata,
    pub pool: Pubkey,
    pub from: Pubkey,
    pub position: Pubkey,
    pub amounts: [u64; 2],
    pub active_bin_id: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeteoraDlmmRemoveLiquidityEvent {
    pub metadata: EventMetadata,
    pub pool: Pubkey,
    pub from: Pubkey,
    pub position: Pubkey,
    pub amounts: [u64; 2],
    pub active_bin_id: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeteoraDlmmInitializePoolEvent {
    pub metadata: EventMetadata,
    pub pool: Pubkey,
    pub creator: Pubkey,
    pub active_bin_id: i32,
    pub bin_step: u16,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeteoraDlmmInitializeBinArrayEvent {
    pub metadata: EventMetadata,
    pub pool: Pubkey,
    pub bin_array: Pubkey,
    pub index: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeteoraDlmmCreatePositionEvent {
    pub metadata: EventMetadata,
    pub pool: Pubkey,
    pub position: Pubkey,
    pub owner: Pubkey,
    pub lower_bin_id: i32,
    pub width: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeteoraDlmmClosePositionEvent {
    pub metadata: EventMetadata,
    pub pool: Pubkey,
    pub position: Pubkey,
    pub owner: Pubkey,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeteoraDlmmClaimFeeEvent {
    pub metadata: EventMetadata,
    pub pool: Pubkey,
    pub position: Pubkey,
    pub owner: Pubkey,
    pub fee_x: u64,
    pub fee_y: u64,
}
