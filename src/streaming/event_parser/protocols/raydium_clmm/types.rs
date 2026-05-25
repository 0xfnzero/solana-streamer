use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct AmmConfig {
    pub bump: u8,
    pub index: u16,
    pub owner: Pubkey,
    pub protocol_fee_rate: u32,
    pub trade_fee_rate: u32,
    pub tick_spacing: u16,
    pub fund_fee_rate: u32,
    pub padding_u32: u32,
    pub fund_owner: Pubkey,
    pub padding: [u64; 3],
}

pub const AMM_CONFIG_SIZE: usize = 1 + 2 + 32 + 4 * 2 + 2 + 4 * 2 + 32 + 8 * 3;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct RewardInfo {
    pub reward_state: u8,
    pub open_time: u64,
    pub end_time: u64,
    pub last_update_time: u64,
    pub emissions_per_second_x64: u128,
    pub reward_total_emitted: u64,
    pub reward_claimed: u64,
    pub token_mint: Pubkey,
    pub token_vault: Pubkey,
    pub authority: Pubkey,
    pub reward_growth_global_x64: u128,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct DynamicFeeInfo {
    pub filter_period: u16,
    pub decay_period: u16,
    pub reduction_factor: u16,
    pub dynamic_fee_control: u32,
    pub max_volatility_accumulator: u32,
    pub tick_spacing_index_reference: i32,
    pub volatility_reference: u32,
    pub volatility_accumulator: u32,
    pub last_update_timestamp: u64,
    #[serde(with = "serde_big_array::BigArray")]
    pub padding: [u8; 46],
}

impl Default for DynamicFeeInfo {
    fn default() -> Self {
        Self {
            filter_period: 0,
            decay_period: 0,
            reduction_factor: 0,
            dynamic_fee_control: 0,
            max_volatility_accumulator: 0,
            tick_spacing_index_reference: 0,
            volatility_reference: 0,
            volatility_accumulator: 0,
            last_update_timestamp: 0,
            padding: [0; 46],
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PoolState {
    pub bump: [u8; 1],
    pub amm_config: Pubkey,
    pub owner: Pubkey,
    pub token_mint_0: Pubkey,
    pub token_mint_1: Pubkey,
    pub token_vault_0: Pubkey,
    pub token_vault_1: Pubkey,
    pub observation_key: Pubkey,
    pub mint_decimals_0: u8,
    pub mint_decimals_1: u8,
    pub tick_spacing: u16,
    pub liquidity: u128,
    pub sqrt_price_x64: u128,
    pub tick_current: i32,
    pub padding3: u16,
    pub padding4: u16,
    pub fee_growth_global0_x64: u128,
    pub fee_growth_global1_x64: u128,
    pub protocol_fees_token0: u64,
    pub protocol_fees_token1: u64,
    pub padding5: [u128; 4],
    pub status: u8,
    pub fee_on: u8,
    pub padding: [u8; 6],
    pub reward_infos: [RewardInfo; 3],
    pub tick_array_bitmap: [u64; 16],
    pub padding6: [u64; 4],
    pub fund_fees_token0: u64,
    pub fund_fees_token1: u64,
    pub open_time: u64,
    pub recent_epoch: u64,
    pub dynamic_fee_info: DynamicFeeInfo,
    pub padding1: [u64; 14],
    pub padding2: [u64; 32],
}

pub const POOL_STATE_SIZE: usize = 1536;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct TickState {
    pub tick: i32,
    pub liquidity_net: i128,
    pub liquidity_gross: u128,
    pub fee_growth_outside0_x64: u128,
    pub fee_growth_outside1_x64: u128,
    pub reward_growths_outside_x64: [u128; 3],
    pub order_phase: u64,
    pub orders_amount: u64,
    pub part_filled_orders_remaining: u64,
    pub unfilled_ratio_x64: u128,
    pub padding: [u32; 3],
}

impl Default for TickState {
    fn default() -> Self {
        Self {
            tick: 0,
            liquidity_net: 0,
            liquidity_gross: 0,
            fee_growth_outside0_x64: 0,
            fee_growth_outside1_x64: 0,
            reward_growths_outside_x64: [0; 3],
            order_phase: 0,
            orders_amount: 0,
            part_filled_orders_remaining: 0,
            unfilled_ratio_x64: 0,
            padding: [0; 3],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct TickArrayState {
    pub pool_id: Pubkey,
    pub start_tick_index: i32,
    #[serde(with = "serde_big_array::BigArray")]
    pub ticks: [TickState; 60],
    pub initialized_tick_count: u8,
    pub recent_epoch: u64,
    #[serde(with = "serde_big_array::BigArray")]
    pub padding: [u8; 107],
}

impl Default for TickArrayState {
    fn default() -> Self {
        Self {
            pool_id: Pubkey::default(),
            start_tick_index: 0,
            ticks: core::array::from_fn(|_| TickState::default()),
            initialized_tick_count: 0,
            recent_epoch: 0,
            padding: [0u8; 107],
        }
    }
}

pub const TICK_ARRAY_STATE_SIZE: usize = 10232;
