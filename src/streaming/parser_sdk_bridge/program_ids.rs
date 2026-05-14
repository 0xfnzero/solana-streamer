//! Protocol program ids used by bridged events.
use crate::streaming::event_parser::protocols::sol_parser_forward;

use solana_sdk::pubkey::Pubkey;

pub(crate) fn pumpswap_program() -> Pubkey {
    crate::streaming::event_parser::protocols::pumpswap::parser::PUMPSWAP_PROGRAM_ID
}
pub(crate) fn pump_program() -> Pubkey {
    crate::streaming::event_parser::protocols::pumpfun::parser::PUMPFUN_PROGRAM_ID
}
pub(crate) fn pump_fees_program() -> Pubkey {
    sol_parser_sdk::instr::program_ids::PUMP_FEES_PROGRAM_ID
}
pub(crate) fn bonk_program() -> Pubkey {
    crate::streaming::event_parser::protocols::bonk::parser::BONK_PROGRAM_ID
}
pub(crate) fn raydium_cpmm_program() -> Pubkey {
    crate::streaming::event_parser::protocols::raydium_cpmm::parser::RAYDIUM_CPMM_PROGRAM_ID
}
pub(crate) fn raydium_clmm_program() -> Pubkey {
    crate::streaming::event_parser::protocols::raydium_clmm::parser::RAYDIUM_CLMM_PROGRAM_ID
}
pub(crate) fn raydium_amm_v4_program() -> Pubkey {
    crate::streaming::event_parser::protocols::raydium_amm_v4::parser::RAYDIUM_AMM_V4_PROGRAM_ID
}
pub(crate) fn meteora_damm_program() -> Pubkey {
    crate::streaming::event_parser::protocols::meteora_damm_v2::parser::METEORA_DAMM_V2_PROGRAM_ID
}

pub(crate) fn orca_whirlpool_program() -> Pubkey {
    sol_parser_forward::ORCA_WHIRLPOOL_PROGRAM_ID
}

pub(crate) fn meteora_pools_program() -> Pubkey {
    sol_parser_forward::METEORA_POOLS_PROGRAM_ID
}

pub(crate) fn meteora_dlmm_program() -> Pubkey {
    sol_parser_forward::METEORA_DLMM_PROGRAM_ID
}
