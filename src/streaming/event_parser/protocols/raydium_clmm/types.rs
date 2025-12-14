use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

use crate::streaming::{
    event_parser::{
        common::{EventMetadata, EventType},
        protocols::raydium_clmm::{
            RaydiumClmmAmmConfigAccountEvent, RaydiumClmmPoolStateAccountEvent,
            RaydiumClmmTickArrayBitmapExtensionAccountEvent, RaydiumClmmTickArrayStateAccountEvent,
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

pub fn amm_config_parser(account: &AccountPretty, mut metadata: EventMetadata) -> Option<DexEvent> {
    metadata.event_type = EventType::AccountRaydiumClmmAmmConfig;

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

pub fn pool_state_parser(account: &AccountPretty, mut metadata: EventMetadata) -> Option<DexEvent> {
    metadata.event_type = EventType::AccountRaydiumClmmPoolState;

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
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::AccountRaydiumClmmTickArrayState;

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

// EXTENSION_TICKARRAY_BITMAP_SIZE 常量，根据 Raydium CLMM 实现，通常为 14
pub const EXTENSION_TICKARRAY_BITMAP_SIZE: usize = 14;

#[repr(C, packed)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct TickArrayBitmapExtension {
    pub pool_id: Pubkey,
    /// Packed initialized tick array state for start_tick_index is positive
    pub positive_tick_array_bitmap: [[u64; 8]; EXTENSION_TICKARRAY_BITMAP_SIZE],
    /// Packed initialized tick array state for start_tick_index is negative
    pub negative_tick_array_bitmap: [[u64; 8]; EXTENSION_TICKARRAY_BITMAP_SIZE],
}

impl Default for TickArrayBitmapExtension {
    fn default() -> Self {
        Self {
            pool_id: Pubkey::default(),
            positive_tick_array_bitmap: [[0u64; 8]; EXTENSION_TICKARRAY_BITMAP_SIZE],
            negative_tick_array_bitmap: [[0u64; 8]; EXTENSION_TICKARRAY_BITMAP_SIZE],
        }
    }
}

pub const TICK_ARRAY_BITMAP_EXTENSION_SIZE: usize = 32 + (EXTENSION_TICKARRAY_BITMAP_SIZE * 8 * 8) + (EXTENSION_TICKARRAY_BITMAP_SIZE * 8 * 8);

pub fn tick_array_bitmap_extension_decode(data: &[u8]) -> Option<TickArrayBitmapExtension> {
    if data.len() < TICK_ARRAY_BITMAP_EXTENSION_SIZE {
        return None;
    }
    
    // 由于使用了 #[repr(C, packed)]，我们需要手动解析
    let mut offset = 0;
    
    // 读取 pool_id (32 bytes)
    if data.len() < offset + 32 {
        return None;
    }
    let pool_id = Pubkey::try_from(&data[offset..offset + 32]).ok()?;
    offset += 32;
    
    // 读取 positive_tick_array_bitmap
    let mut positive_tick_array_bitmap = [[0u64; 8]; EXTENSION_TICKARRAY_BITMAP_SIZE];
    for i in 0..EXTENSION_TICKARRAY_BITMAP_SIZE {
        for j in 0..8 {
            if data.len() < offset + 8 {
                return None;
            }
            positive_tick_array_bitmap[i][j] = u64::from_le_bytes(
                data[offset..offset + 8].try_into().ok()?
            );
            offset += 8;
        }
    }
    
    // 读取 negative_tick_array_bitmap
    let mut negative_tick_array_bitmap = [[0u64; 8]; EXTENSION_TICKARRAY_BITMAP_SIZE];
    for i in 0..EXTENSION_TICKARRAY_BITMAP_SIZE {
        for j in 0..8 {
            if data.len() < offset + 8 {
                return None;
            }
            negative_tick_array_bitmap[i][j] = u64::from_le_bytes(
                data[offset..offset + 8].try_into().ok()?
            );
            offset += 8;
        }
    }
    
    Some(TickArrayBitmapExtension {
        pool_id,
        positive_tick_array_bitmap,
        negative_tick_array_bitmap,
    })
}

pub fn tick_array_bitmap_extension_parser(
    account: &AccountPretty,
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::AccountRaydiumClmmTickArrayBitmapExtension;

    if account.data.len() < TICK_ARRAY_BITMAP_EXTENSION_SIZE + 8 {
        return None;
    }
    if let Some(tick_array_bitmap_extension) =
        tick_array_bitmap_extension_decode(&account.data[8..TICK_ARRAY_BITMAP_EXTENSION_SIZE + 8])
    {
        Some(DexEvent::RaydiumClmmTickArrayBitmapExtensionAccountEvent(
            RaydiumClmmTickArrayBitmapExtensionAccountEvent {
                metadata,
                pubkey: account.pubkey,
                executable: account.executable,
                lamports: account.lamports,
                owner: account.owner,
                rent_epoch: account.rent_epoch,
                tick_array_bitmap_extension,
            },
        ))
    } else {
        None
    }
}
