use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct GlobalConfig {
    pub admin: Pubkey,
    pub lp_fee_basis_points: u64,
    pub protocol_fee_basis_points: u64,
    pub disable_flags: u8,
    pub protocol_fee_recipients: [Pubkey; 8],
    pub coin_creator_fee_basis_points: u64,
    pub admin_set_coin_creator_authority: Pubkey,
    pub whitelist_pda: Pubkey,
    pub reserved_fee_recipient: Pubkey,
    pub mayhem_mode_enabled: bool,
    pub reserved_fee_recipients: [Pubkey; 7],
}

pub const GLOBAL_CONFIG_SIZE: usize = 32 + 8 + 8 + 1 + 32 * 8 + 8 + 32 + 32 + 32 + 1 + 32 * 7;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct Pool {
    pub pool_bump: u8,
    pub index: u16,
    pub creator: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub lp_mint: Pubkey,
    pub pool_base_token_account: Pubkey,
    pub pool_quote_token_account: Pubkey,
    pub lp_supply: u64,
    pub coin_creator: Pubkey,
    pub is_mayhem_mode: bool,
    pub is_cashback_coin: bool,
    /// On-chain reserved tail (7 bytes); keep in sync with pump_amm pool account layout.
    pub reserved: [u8; 7],
}

/// Legacy pool account body (before `is_cashback_coin` + reserved).
pub const POOL_BODY_LEGACY: usize = 1 + 2 + 32 * 6 + 8 + 32 + 1;
/// Current pool account body including flags and reserved.
pub const POOL_BODY: usize = POOL_BODY_LEGACY + 1 + 7;

pub const POOL_SIZE: usize = POOL_BODY;
