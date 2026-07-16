//! Protocol program ids used by bridged events.
use sol_parser_sdk::instr::program_ids;
use solana_sdk::pubkey::Pubkey;

pub(crate) fn pumpswap_program() -> Pubkey {
    program_ids::PUMPSWAP_PROGRAM_ID
}
pub(crate) fn pump_program() -> Pubkey {
    program_ids::PUMPFUN_PROGRAM_ID
}
pub(crate) fn pump_fees_program() -> Pubkey {
    program_ids::PUMP_FEES_PROGRAM_ID
}
pub(crate) fn bonk_program() -> Pubkey {
    program_ids::RAYDIUM_LAUNCHLAB_PROGRAM_ID
}
pub(crate) fn raydium_cpmm_program() -> Pubkey {
    program_ids::RAYDIUM_CPMM_PROGRAM_ID
}
pub(crate) fn raydium_clmm_program() -> Pubkey {
    program_ids::RAYDIUM_CLMM_PROGRAM_ID
}
pub(crate) fn raydium_amm_v4_program() -> Pubkey {
    program_ids::RAYDIUM_AMM_V4_PROGRAM_ID
}
pub(crate) fn meteora_damm_program() -> Pubkey {
    program_ids::METEORA_DAMM_V2_PROGRAM_ID
}

pub(crate) fn meteora_dbc_program() -> Pubkey {
    program_ids::METEORA_DBC_PROGRAM_ID
}

pub(crate) fn orca_whirlpool_program() -> Pubkey {
    program_ids::ORCA_WHIRLPOOL_PROGRAM_ID
}

pub(crate) fn meteora_pools_program() -> Pubkey {
    program_ids::METEORA_POOLS_PROGRAM_ID
}

pub(crate) fn meteora_dlmm_program() -> Pubkey {
    program_ids::METEORA_DLMM_PROGRAM_ID
}
