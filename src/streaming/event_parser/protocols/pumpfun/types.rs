use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

use crate::streaming::{
    event_parser::{
        common::EventMetadata,
        protocols::pumpfun::{PumpFunBondingCurveAccountEvent, PumpFunGlobalAccountEvent},
        DexEvent,
    },
    grpc::AccountPretty,
};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct BondingCurve {
    pub virtual_token_reserves: u64,
    pub virtual_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub token_total_supply: u64,
    pub complete: bool,
    pub creator: Pubkey,
}

pub const BONDING_CURVE_SIZE: usize = 8 * 5 + 1 + 32;

pub fn bonding_curve_decode(data: &[u8]) -> Option<BondingCurve> {
    if data.len() < BONDING_CURVE_SIZE {
        return None;
    }
    borsh::from_slice::<BondingCurve>(&data[..BONDING_CURVE_SIZE]).ok()
}

pub fn bonding_curve_parser(
    account: &AccountPretty,
    metadata: EventMetadata,
) -> Option<DexEvent> {
    if account.data.len() < BONDING_CURVE_SIZE + 8 {
        return None;
    }
    if let Some(bonding_curve) = bonding_curve_decode(&account.data[8..BONDING_CURVE_SIZE + 8]) {
        Some(DexEvent::PumpFunBondingCurveAccountEvent(PumpFunBondingCurveAccountEvent {
            metadata,
            pubkey: account.pubkey,
            executable: account.executable,
            lamports: account.lamports,
            owner: account.owner,
            rent_epoch: account.rent_epoch,
            bonding_curve,
        }))
    } else {
        None
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct Global {
    pub initialized: bool,
    pub authority: Pubkey,
    pub fee_recipient: Pubkey,
    pub initial_virtual_token_reserves: u64,
    pub initial_virtual_sol_reserves: u64,
    pub initial_real_token_reserves: u64,
    pub token_total_supply: u64,
    pub fee_basis_points: u64,
    pub withdraw_authority: Pubkey,
    pub enable_migrate: bool,
    pub pool_migration_fee: u64,
    pub creator_fee_basis_points: u64,
    pub fee_recipients: [Pubkey; 7],
    pub set_creator_authority: Pubkey,
    pub admin_set_creator_authority: Pubkey,
}

pub const GLOBAL_SIZE: usize = 1 + 32 * 2 + 8 * 5 + 32 + 1 + 8 * 2 + 32 * 7 + 32 * 2;

pub fn global_decode(data: &[u8]) -> Option<Global> {
    if data.len() < GLOBAL_SIZE {
        return None;
    }
    borsh::from_slice::<Global>(&data[..GLOBAL_SIZE]).ok()
}

pub fn global_parser(account: &AccountPretty, metadata: EventMetadata) -> Option<DexEvent> {
    if account.data.len() < GLOBAL_SIZE + 8 {
        return None;
    }
    if let Some(global) = global_decode(&account.data[8..GLOBAL_SIZE + 8]) {
        Some(DexEvent::PumpFunGlobalAccountEvent(PumpFunGlobalAccountEvent {
            metadata,
            pubkey: account.pubkey,
            executable: account.executable,
            lamports: account.lamports,
            owner: account.owner,
            rent_epoch: account.rent_epoch,
            global,
        }))
    } else {
        None
    }
}
