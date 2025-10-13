use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

use crate::streaming::{
    event_parser::{
        common::EventMetadata,
        protocols::raydium_clmm::{
            RaydiumClmmAmmConfigAccountEvent, RaydiumClmmPoolStateAccountEvent,
            RaydiumClmmTickArrayStateAccountEvent,
        },
        DexEvent,
    },
    grpc::AccountPretty,
};

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

pub fn amm_config_decode(data: &[u8]) -> Option<AmmConfig> {
    if data.len() < AMM_CONFIG_SIZE {
        return None;
    }
    borsh::from_slice::<AmmConfig>(&data[..AMM_CONFIG_SIZE]).ok()
}

pub fn amm_config_parser(account: &AccountPretty, metadata: EventMetadata) -> Option<DexEvent> {
    if account.data.len() < AMM_CONFIG_SIZE + 8 {
        return None;
    }
    if let Some(amm_config) = amm_config_decode(&account.data[8..AMM_CONFIG_SIZE + 8]) {
        Some(DexEvent::RaydiumClmmAmmConfigAccountEvent(RaydiumClmmAmmConfigAccountEvent {
            metadata,
            pubkey: account.pubkey,
            executable: account.executable,
            lamports: account.lamports,
            owner: account.owner,
            rent_epoch: account.rent_epoch,
            amm_config: amm_config,
        }))
    } else {
        None
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct RewardInfo {
    pub reward_state: u8,
    pub open_time: u64,
    pub end_time: u64,
    pub last_update_time: u64,
    pub emissions_per_second_x64: u128,
    pub reward_total_emissioned: u64,
    pub reward_claimed: u64,
    pub token_mint: Pubkey,
    pub token_vault: Pubkey,
    pub authority: Pubkey,
    pub reward_growth_global_x64: u128,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PoolState {
    pub bump: [u8; 1],
    pub amm_config: Pubkey,
    pub owner: Pubkey,
    pub token_mint0: Pubkey,
    pub token_mint1: Pubkey,
    pub token_vault0: Pubkey,
    pub token_vault1: Pubkey,
    pub observation_key: Pubkey,
    pub mint_decimals0: u8,
    pub mint_decimals1: u8,
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
    pub swap_in_amount_token0: u128,
    pub swap_out_amount_token1: u128,
    pub swap_in_amount_token1: u128,
    pub swap_out_amount_token0: u128,
    pub status: u8,
    pub padding: [u8; 7],
    pub reward_infos: [RewardInfo; 3],
    pub tick_array_bitmap: [u64; 16],
    pub total_fees_token0: u64,
    pub total_fees_claimed_token0: u64,
    pub total_fees_token1: u64,
    pub total_fees_claimed_token1: u64,
    pub fund_fees_token0: u64,
    pub fund_fees_token1: u64,
    pub open_time: u64,
    pub recent_epoch: u64,
    pub padding1: [u64; 24],
    pub padding2: [u64; 32],
}

pub const POOL_STATE_SIZE: usize = 1536;

pub fn pool_state_decode(data: &[u8]) -> Option<PoolState> {
    if data.len() < POOL_STATE_SIZE {
        return None;
    }
    borsh::from_slice::<PoolState>(&data[..POOL_STATE_SIZE]).ok()
}

pub fn pool_state_parser(account: &AccountPretty, metadata: EventMetadata) -> Option<DexEvent> {
    if account.data.len() < POOL_STATE_SIZE + 8 {
        return None;
    }
    if let Some(pool_state) = pool_state_decode(&account.data[8..POOL_STATE_SIZE + 8]) {
        Some(DexEvent::RaydiumClmmPoolStateAccountEvent(RaydiumClmmPoolStateAccountEvent {
            metadata,
            pubkey: account.pubkey,
            executable: account.executable,
            lamports: account.lamports,
            owner: account.owner,
            rent_epoch: account.rent_epoch,
            pool_state: pool_state,
        }))
    } else {
        None
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct TickState {
    pub tick: i32,
    pub liquidity_net: i128,
    pub liquidity_gross: u128,
    pub fee_growth_outside0_x64: u128,
    pub fee_growth_outside1_x64: u128,
    pub reward_growths_outside_x64: [u128; 3],
    pub padding: [u32; 13],
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
            padding: [0; 13],
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

pub fn tick_array_state_decode(data: &[u8]) -> Option<TickArrayState> {
    if data.len() < TICK_ARRAY_STATE_SIZE {
        return None;
    }
    borsh::from_slice::<TickArrayState>(&data[..TICK_ARRAY_STATE_SIZE]).ok()
}

pub fn tick_array_state_parser(
    account: &AccountPretty,
    metadata: EventMetadata,
) -> Option<DexEvent> {
    if account.data.len() < TICK_ARRAY_STATE_SIZE + 8 {
        return None;
    }
    if let Some(tick_array_state) =
        tick_array_state_decode(&account.data[8..TICK_ARRAY_STATE_SIZE + 8])
    {
        Some(DexEvent::RaydiumClmmTickArrayStateAccountEvent(
            RaydiumClmmTickArrayStateAccountEvent {
                metadata,
                pubkey: account.pubkey,
                executable: account.executable,
                lamports: account.lamports,
                owner: account.owner,
                rent_epoch: account.rent_epoch,
                tick_array_state: tick_array_state,
            },
        ))
    } else {
        None
    }
}
