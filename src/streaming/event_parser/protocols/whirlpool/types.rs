use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

use crate::streaming::{
    event_parser::{
        common::{EventMetadata, EventType},
        protocols::whirlpool::WhirlpoolAccountEvent,
        DexEvent,
    },
    grpc::AccountPretty,
};

// Number of rewards supported by Whirlpools
pub const NUM_REWARDS: usize = 3;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct WhirlpoolRewardInfo {
    /// Reward token mint.
    pub mint: Pubkey,
    /// Reward vault token account.
    pub vault: Pubkey,
    /// Authority account that has permission to initialize the reward and set emissions.
    pub authority: Pubkey,
    /// Q64.64 number that indicates how many tokens per second are earned per unit of liquidity.
    pub emissions_per_second_x64: u128,
    /// Q64.64 number that tracks the total tokens earned per unit of liquidity since the reward
    /// emissions were turned on.
    pub growth_global_x64: u128,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Whirlpool {
    pub whirlpools_config: Pubkey,
    pub whirlpool_bump: [u8; 1],
    pub tick_spacing: u16,
    pub fee_tier_index_seed: [u8; 2],
    pub fee_rate: u16,
    pub protocol_fee_rate: u16,
    pub liquidity: u128,
    pub sqrt_price: u128,
    pub tick_current_index: i32,
    pub protocol_fee_owed_a: u64,
    pub protocol_fee_owed_b: u64,
    pub token_mint_a: Pubkey,
    pub token_vault_a: Pubkey,
    pub fee_growth_global_a: u128,
    pub token_mint_b: Pubkey,
    pub token_vault_b: Pubkey,
    pub fee_growth_global_b: u128,
    pub reward_last_updated_timestamp: u64,
    pub reward_infos: [WhirlpoolRewardInfo; NUM_REWARDS],
}

// Whirlpool::LEN = 8 (discriminator) + 261 + 384 = 653
// 261 = 32 + 1 + 2 + 2 + 2 + 2 + 16 + 16 + 4 + 8 + 8 + 32 + 32 + 16 + 32 + 32 + 16 + 8
// 384 = 3 * 128 (each WhirlpoolRewardInfo is 128 bytes)
// 总数据大小（不包括 discriminator）= 261 + 384 = 645
pub const WHIRLPOOL_SIZE: usize = 261 + 384; // 645 bytes (不包括 discriminator)
pub const WHIRLPOOL_REWARD_INFO_SIZE: usize = 128;

pub fn whirlpool_decode(data: &[u8]) -> Option<Whirlpool> {
    if data.len() < WHIRLPOOL_SIZE {
        return None;
    }
    
    let mut offset = 0;
    
    // whirlpools_config: Pubkey (32 bytes)
    let whirlpools_config = Pubkey::try_from(&data[offset..offset + 32]).ok()?;
    offset += 32;
    
    // whirlpool_bump: [u8; 1] (1 byte)
    let whirlpool_bump = [data[offset]];
    offset += 1;
    
    // tick_spacing: u16 (2 bytes)
    let tick_spacing = u16::from_le_bytes([data[offset], data[offset + 1]]);
    offset += 2;
    
    // fee_tier_index_seed: [u8; 2] (2 bytes)
    let fee_tier_index_seed = [data[offset], data[offset + 1]];
    offset += 2;
    
    // fee_rate: u16 (2 bytes)
    let fee_rate = u16::from_le_bytes([data[offset], data[offset + 1]]);
    offset += 2;
    
    // protocol_fee_rate: u16 (2 bytes)
    let protocol_fee_rate = u16::from_le_bytes([data[offset], data[offset + 1]]);
    offset += 2;
    
    // liquidity: u128 (16 bytes)
    let mut liquidity_bytes = [0u8; 16];
    liquidity_bytes.copy_from_slice(&data[offset..offset + 16]);
    let liquidity = u128::from_le_bytes(liquidity_bytes);
    offset += 16;
    
    // sqrt_price: u128 (16 bytes)
    let mut sqrt_price_bytes = [0u8; 16];
    sqrt_price_bytes.copy_from_slice(&data[offset..offset + 16]);
    let sqrt_price = u128::from_le_bytes(sqrt_price_bytes);
    offset += 16;
    
    // tick_current_index: i32 (4 bytes)
    let tick_current_index = i32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]);
    offset += 4;
    
    // protocol_fee_owed_a: u64 (8 bytes)
    let protocol_fee_owed_a = u64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ]);
    offset += 8;
    
    // protocol_fee_owed_b: u64 (8 bytes)
    let protocol_fee_owed_b = u64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ]);
    offset += 8;
    
    // token_mint_a: Pubkey (32 bytes)
    let token_mint_a = Pubkey::try_from(&data[offset..offset + 32]).ok()?;
    offset += 32;
    
    // token_vault_a: Pubkey (32 bytes)
    let token_vault_a = Pubkey::try_from(&data[offset..offset + 32]).ok()?;
    offset += 32;
    
    // fee_growth_global_a: u128 (16 bytes)
    let mut fee_growth_global_a_bytes = [0u8; 16];
    fee_growth_global_a_bytes.copy_from_slice(&data[offset..offset + 16]);
    let fee_growth_global_a = u128::from_le_bytes(fee_growth_global_a_bytes);
    offset += 16;
    
    // token_mint_b: Pubkey (32 bytes)
    let token_mint_b = Pubkey::try_from(&data[offset..offset + 32]).ok()?;
    offset += 32;
    
    // token_vault_b: Pubkey (32 bytes)
    let token_vault_b = Pubkey::try_from(&data[offset..offset + 32]).ok()?;
    offset += 32;
    
    // fee_growth_global_b: u128 (16 bytes)
    let mut fee_growth_global_b_bytes = [0u8; 16];
    fee_growth_global_b_bytes.copy_from_slice(&data[offset..offset + 16]);
    let fee_growth_global_b = u128::from_le_bytes(fee_growth_global_b_bytes);
    offset += 16;
    
    // reward_last_updated_timestamp: u64 (8 bytes)
    let reward_last_updated_timestamp = u64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ]);
    offset += 8;
    
    // reward_infos: [WhirlpoolRewardInfo; NUM_REWARDS] (384 bytes)
    // 检查是否有足够的数据来解析所有奖励信息
    if data.len() < offset + (NUM_REWARDS * WHIRLPOOL_REWARD_INFO_SIZE) {
        log::warn!(
            "Whirlpool 奖励信息数据不足: 需要 {} 字节，实际 {} 字节",
            offset + (NUM_REWARDS * WHIRLPOOL_REWARD_INFO_SIZE),
            data.len()
        );
        return None;
    }
    
    let mut reward_infos = [WhirlpoolRewardInfo::default(); NUM_REWARDS];
    for i in 0..NUM_REWARDS {
        
        // mint: Pubkey (32 bytes)
        let mint = Pubkey::try_from(&data[offset..offset + 32]).ok()?;
        offset += 32;
        
        // vault: Pubkey (32 bytes)
        let vault = Pubkey::try_from(&data[offset..offset + 32]).ok()?;
        offset += 32;
        
        // authority: Pubkey (32 bytes)
        let authority = Pubkey::try_from(&data[offset..offset + 32]).ok()?;
        offset += 32;
        
        // emissions_per_second_x64: u128 (16 bytes)
        let mut emissions_bytes = [0u8; 16];
        emissions_bytes.copy_from_slice(&data[offset..offset + 16]);
        let emissions_per_second_x64 = u128::from_le_bytes(emissions_bytes);
        offset += 16;
        
        // growth_global_x64: u128 (16 bytes)
        let mut growth_bytes = [0u8; 16];
        growth_bytes.copy_from_slice(&data[offset..offset + 16]);
        let growth_global_x64 = u128::from_le_bytes(growth_bytes);
        offset += 16;
        
        reward_infos[i] = WhirlpoolRewardInfo {
            mint,
            vault,
            authority,
            emissions_per_second_x64,
            growth_global_x64,
        };
    }
    
    Some(Whirlpool {
        whirlpools_config,
        whirlpool_bump,
        tick_spacing,
        fee_tier_index_seed,
        fee_rate,
        protocol_fee_rate,
        liquidity,
        sqrt_price,
        tick_current_index,
        protocol_fee_owed_a,
        protocol_fee_owed_b,
        token_mint_a,
        token_vault_a,
        fee_growth_global_a,
        token_mint_b,
        token_vault_b,
        fee_growth_global_b,
        reward_last_updated_timestamp,
        reward_infos,
    })
}

pub fn whirlpool_parser(account: &AccountPretty, mut metadata: EventMetadata) -> Option<DexEvent> {
    metadata.event_type = EventType::AccountWhirlpool;

    // 账户总大小应该是 8 (discriminator) + 645 (数据) = 653 字节
    let expected_size = 8 + WHIRLPOOL_SIZE;
    if account.data.len() < expected_size {
        log::warn!(
            "Whirlpool 账户数据长度不足: 需要至少 {} 字节，实际 {} 字节",
            expected_size,
            account.data.len()
        );
        return None;
    }
    
    log::debug!(
        "开始解析 Whirlpool 账户: pubkey={}, 数据长度={}, 期望长度={}",
        account.pubkey,
        account.data.len(),
        expected_size
    );
    
    // 跳过前 8 字节的 discriminator，解析接下来的 645 字节
    if let Some(whirlpool) = whirlpool_decode(&account.data[8..8 + WHIRLPOOL_SIZE]) {
        Some(DexEvent::WhirlpoolAccountEvent(WhirlpoolAccountEvent {
            metadata,
            pubkey: account.pubkey,
            executable: account.executable,
            lamports: account.lamports,
            owner: account.owner,
            rent_epoch: account.rent_epoch,
            whirlpool,
        }))
    } else {
        log::warn!(
            "Whirlpool 账户数据解析失败: pubkey={}, 数据长度={}",
            account.pubkey,
            account.data.len()
        );
        None
    }
}
