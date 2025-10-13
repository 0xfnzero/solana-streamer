use crate::streaming::event_parser::common::EventMetadata;
use crate::{
    streaming::event_parser::protocols::raydium_amm_v4::types::AmmInfo,
};
use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// 交易
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct RaydiumAmmV4SwapEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    // base in
    pub amount_in: u64,
    pub minimum_amount_out: u64,
    // base out
    pub max_amount_in: u64,
    pub amount_out: u64,

    pub token_program: Pubkey,
    pub amm: Pubkey,
    pub amm_authority: Pubkey,
    pub amm_open_orders: Pubkey,
    pub amm_target_orders: Option<Pubkey>,
    pub pool_coin_token_account: Pubkey,
    pub pool_pc_token_account: Pubkey,
    pub serum_program: Pubkey,
    pub serum_market: Pubkey,
    pub serum_bids: Pubkey,
    pub serum_asks: Pubkey,
    pub serum_event_queue: Pubkey,
    pub serum_coin_vault_account: Pubkey,
    pub serum_pc_vault_account: Pubkey,
    pub serum_vault_signer: Pubkey,
    pub user_source_token_account: Pubkey,
    pub user_destination_token_account: Pubkey,
    pub user_source_owner: Pubkey,
}

/// 添加流动性
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct RaydiumAmmV4DepositEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub max_coin_amount: u64,
    pub max_pc_amount: u64,
    pub base_side: u64,

    pub token_program: Pubkey,
    pub amm: Pubkey,
    pub amm_authority: Pubkey,
    pub amm_open_orders: Pubkey,
    pub amm_target_orders: Pubkey,
    pub lp_mint_address: Pubkey,
    pub pool_coin_token_account: Pubkey,
    pub pool_pc_token_account: Pubkey,
    pub serum_market: Pubkey,
    pub user_coin_token_account: Pubkey,
    pub user_pc_token_account: Pubkey,
    pub user_lp_token_account: Pubkey,
    pub user_owner: Pubkey,
    pub serum_event_queue: Pubkey,
}

/// 初始化
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct RaydiumAmmV4Initialize2Event {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub nonce: u8,
    pub open_time: u64,
    pub init_pc_amount: u64,
    pub init_coin_amount: u64,

    pub token_program: Pubkey,
    pub spl_associated_token_account: Pubkey,
    pub system_program: Pubkey,
    pub rent: Pubkey,
    pub amm: Pubkey,
    pub amm_authority: Pubkey,
    pub amm_open_orders: Pubkey,
    pub lp_mint: Pubkey,
    pub coin_mint: Pubkey,
    pub pc_mint: Pubkey,
    pub pool_coin_token_account: Pubkey,
    pub pool_pc_token_account: Pubkey,
    pub pool_withdraw_queue: Pubkey,
    pub amm_target_orders: Pubkey,
    pub pool_temp_lp: Pubkey,
    pub serum_program: Pubkey,
    pub serum_market: Pubkey,
    pub user_wallet: Pubkey,
    pub user_token_coin: Pubkey,
    pub user_token_pc: Pubkey,
    pub user_lp_token_account: Pubkey,
}

/// 移除流动性
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct RaydiumAmmV4WithdrawEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub amount: u64,

    pub token_program: Pubkey,
    pub amm: Pubkey,
    pub amm_authority: Pubkey,
    pub amm_open_orders: Pubkey,
    pub amm_target_orders: Pubkey,
    pub lp_mint_address: Pubkey,
    pub pool_coin_token_account: Pubkey,
    pub pool_pc_token_account: Pubkey,
    pub pool_withdraw_queue: Pubkey,
    pub pool_temp_lp_token_account: Pubkey,
    pub serum_program: Pubkey,
    pub serum_market: Pubkey,
    pub serum_coin_vault_account: Pubkey,
    pub serum_pc_vault_account: Pubkey,
    pub serum_vault_signer: Pubkey,
    pub user_lp_token_account: Pubkey,
    pub user_coin_token_account: Pubkey,
    pub user_pc_token_account: Pubkey,
    pub user_owner: Pubkey,
    pub serum_event_queue: Pubkey,
    pub serum_bids: Pubkey,
    pub serum_asks: Pubkey,
}

/// 提现
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct RaydiumAmmV4WithdrawPnlEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,

    pub token_program: Pubkey,
    pub amm: Pubkey,
    pub amm_config: Pubkey,
    pub amm_authority: Pubkey,
    pub amm_open_orders: Pubkey,
    pub pool_coin_token_account: Pubkey,
    pub pool_pc_token_account: Pubkey,
    pub coin_pnl_token_account: Pubkey,
    pub pc_pnl_token_account: Pubkey,
    pub pnl_owner_account: Pubkey,
    pub amm_target_orders: Pubkey,
    pub serum_program: Pubkey,
    pub serum_market: Pubkey,
    pub serum_event_queue: Pubkey,
    pub serum_coin_vault_account: Pubkey,
    pub serum_pc_vault_account: Pubkey,
    pub serum_vault_signer: Pubkey,
}

/// 池信息
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct RaydiumAmmV4AmmInfoAccountEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub amm_info: AmmInfo,
}

/// 事件鉴别器常量
pub mod discriminators {
    // 指令鉴别器
    pub const SWAP_BASE_IN: &[u8] = &[9];
    pub const SWAP_BASE_OUT: &[u8] = &[11];
    pub const DEPOSIT: &[u8] = &[03];
    pub const INITIALIZE2: &[u8] = &[01];
    pub const WITHDRAW: &[u8] = &[04];
    pub const WITHDRAW_PNL: &[u8] = &[07];

    /// 池信息鉴别器
    pub const AMM_INFO: &[u8] = &[6];
}
