use crate::streaming::event_parser::protocols::{
    bonk::parser::BONK_PROGRAM_ID, meteora_damm_v2::parser::METEORA_DAMM_V2_PROGRAM_ID,
    meteora_dlmm::parser::METEORA_DLMM_PROGRAM_ID, pumpfun::parser::PUMPFUN_PROGRAM_ID,
    pumpswap::parser::PUMPSWAP_PROGRAM_ID, raydium_amm_v4::parser::RAYDIUM_AMM_V4_PROGRAM_ID,
    raydium_clmm::parser::RAYDIUM_CLMM_PROGRAM_ID, raydium_cpmm::parser::RAYDIUM_CPMM_PROGRAM_ID,
    whirlpool::parser::WHIRLPOOL_PROGRAM_ID,
};
use anyhow::{anyhow, Result};
use solana_sdk::pubkey::Pubkey;

/// 支持的协议
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Protocol {
    PumpSwap,
    PumpFun,
    Bonk,
    RaydiumCpmm,
    RaydiumClmm,
    RaydiumAmmV4,
    MeteoraDammV2,
    MeteoraDlmm,
    Whirlpool,
}

impl Protocol {
    pub fn get_program_id(&self) -> Vec<Pubkey> {
        match self {
            Protocol::PumpSwap => vec![PUMPSWAP_PROGRAM_ID],
            Protocol::PumpFun => vec![PUMPFUN_PROGRAM_ID],
            Protocol::Bonk => vec![BONK_PROGRAM_ID],
            Protocol::RaydiumCpmm => vec![RAYDIUM_CPMM_PROGRAM_ID],
            Protocol::RaydiumClmm => vec![RAYDIUM_CLMM_PROGRAM_ID],
            Protocol::RaydiumAmmV4 => vec![RAYDIUM_AMM_V4_PROGRAM_ID],
            Protocol::MeteoraDammV2 => vec![METEORA_DAMM_V2_PROGRAM_ID],
            Protocol::MeteoraDlmm => vec![METEORA_DLMM_PROGRAM_ID],
            Protocol::Whirlpool => vec![WHIRLPOOL_PROGRAM_ID],
        }
    }
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::PumpSwap => write!(f, "PumpSwap"),
            Protocol::PumpFun => write!(f, "PumpFun"),
            Protocol::Bonk => write!(f, "Bonk"),
            Protocol::RaydiumCpmm => write!(f, "RaydiumCpmm"),
            Protocol::RaydiumClmm => write!(f, "RaydiumClmm"),
            Protocol::RaydiumAmmV4 => write!(f, "RaydiumAmmV4"),
            Protocol::MeteoraDammV2 => write!(f, "MeteoraDammV2"),
            Protocol::MeteoraDlmm => write!(f, "MeteoraDlmm"),
            Protocol::Whirlpool => write!(f, "Whirlpool"),
        }
    }
}

impl std::str::FromStr for Protocol {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pumpswap" => Ok(Protocol::PumpSwap),
            "pumpfun" => Ok(Protocol::PumpFun),
            "bonk" => Ok(Protocol::Bonk),
            "raydiumcpmm" => Ok(Protocol::RaydiumCpmm),
            "raydiumclmm" => Ok(Protocol::RaydiumClmm),
            "raydiumammv4" => Ok(Protocol::RaydiumAmmV4),
            "meteoradamm_v2" => Ok(Protocol::MeteoraDammV2),
            "meteoradlmm" => Ok(Protocol::MeteoraDlmm),
            "whirlpool" => Ok(Protocol::Whirlpool),
            _ => Err(anyhow!("Unsupported protocol: {}", s)),
        }
    }
}
