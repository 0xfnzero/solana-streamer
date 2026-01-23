use crate::streaming::event_parser::common::EventMetadata;
use crate::streaming::event_parser::protocols::raydium_cpmm::types::PoolState;
use crate::{
    streaming::event_parser::protocols::raydium_cpmm::types::AmmConfig,
};
use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// 交易
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct RaydiumCpmmSwapEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub amount_in: u64,
    pub minimum_amount_out: u64,
    pub max_amount_in: u64,
    pub amount_out: u64,
    pub payer: Pubkey,
    pub authority: Pubkey,
    pub amm_config: Pubkey,
    pub pool_state: Pubkey,
    pub input_token_account: Pubkey,
    pub output_token_account: Pubkey,
    pub input_vault: Pubkey,
    pub output_vault: Pubkey,
    pub input_token_program: Pubkey,
    pub output_token_program: Pubkey,
    pub input_token_mint: Pubkey,
    pub output_token_mint: Pubkey,
    pub observation_state: Pubkey,
}


/// 存款
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct RaydiumCpmmDepositEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub lp_token_amount: u64,
    pub maximum_token0_amount: u64,
    pub maximum_token1_amount: u64,

    pub owner: Pubkey,
    pub authority: Pubkey,
    pub pool_state: Pubkey,
    pub owner_lp_token: Pubkey,
    pub token_0_account: Pubkey,
    pub token_1_account: Pubkey,
    pub token_0_vault: Pubkey,
    pub token_1_vault: Pubkey,
    pub token_program: Pubkey,
    pub token_program2022: Pubkey,
    pub vault_0_mint: Pubkey,
    pub vault_1_mint: Pubkey,
    pub lp_mint: Pubkey,
}

/// 初始化
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct RaydiumCpmmInitializeEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub init_amount0: u64,
    pub init_amount1: u64,
    pub open_time: u64,

    pub creator: Pubkey,
    pub amm_config: Pubkey,
    pub authority: Pubkey,
    pub pool_state: Pubkey,
    pub token_0_mint: Pubkey,
    pub token_1_mint: Pubkey,
    pub lp_mint: Pubkey,
    pub creator_token_0: Pubkey,
    pub creator_token_1: Pubkey,
    pub creator_lp_token: Pubkey,
    pub token_0_vault: Pubkey,
    pub token_1_vault: Pubkey,
    pub create_pool_fee: Pubkey,
    pub observation_state: Pubkey,
    pub token_program: Pubkey,
    pub token_0_program: Pubkey,
    pub token_1_program: Pubkey,
    pub associated_token_program: Pubkey,
    pub system_program: Pubkey,
    pub rent: Pubkey,
}

/// 提款
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct RaydiumCpmmWithdrawEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub lp_token_amount: u64,
    pub minimum_token0_amount: u64,
    pub minimum_token1_amount: u64,

    pub owner: Pubkey,
    pub authority: Pubkey,
    pub pool_state: Pubkey,
    pub owner_lp_token: Pubkey,
    pub token_0_account: Pubkey,
    pub token_1_account: Pubkey,
    pub token_0_vault: Pubkey,
    pub token_1_vault: Pubkey,
    pub token_program: Pubkey,
    pub token_program2022: Pubkey,
    pub vault_0_mint: Pubkey,
    pub vault_1_mint: Pubkey,
    pub lp_mint: Pubkey,
    pub memo_program: Pubkey,
}

/// 池配置
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct RaydiumCpmmAmmConfigAccountEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub amm_config: AmmConfig,
}

/// 池状态
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct RaydiumCpmmPoolStateAccountEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub pool_state: PoolState,
}

/// 事件鉴别器常量
pub mod discriminators {
    // 指令鉴别器
    pub const SWAP_BASE_IN: &[u8] = &[143, 190, 90, 218, 196, 30, 51, 222];
    pub const SWAP_BASE_OUT: &[u8] = &[55, 217, 98, 86, 163, 74, 180, 173];
    pub const DEPOSIT: &[u8] = &[242, 35, 198, 137, 82, 225, 242, 182];
    pub const INITIALIZE: &[u8] = &[175, 175, 109, 31, 13, 152, 155, 237];
    pub const WITHDRAW: &[u8] = &[183, 18, 70, 156, 148, 109, 161, 34];

    // 账号鉴别器
    pub const AMM_CONFIG: &[u8] = &[218, 244, 33, 104, 203, 203, 43, 111];
    pub const POOL_STATE: &[u8] = &[247, 237, 227, 245, 215, 195, 222, 70];
}
