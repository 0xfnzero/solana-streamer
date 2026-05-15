use anyhow::{anyhow, Result};
use sol_parser_sdk::instr::program_ids::{
    BONK_PROGRAM_ID, METEORA_DAMM_V2_PROGRAM_ID, METEORA_DLMM_PROGRAM_ID, METEORA_POOLS_PROGRAM_ID,
    ORCA_WHIRLPOOL_PROGRAM_ID, PUMPFUN_PROGRAM_ID, PUMPSWAP_PROGRAM_ID, PUMP_FEES_PROGRAM_ID,
    RAYDIUM_AMM_V4_PROGRAM_ID, RAYDIUM_CLMM_PROGRAM_ID, RAYDIUM_CPMM_PROGRAM_ID,
};
use solana_sdk::pubkey::Pubkey;

/// 支持的协议
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Protocol {
    PumpSwap,
    PumpFun,
    PumpFees,
    /// Backward-compatible alias for Raydium Launchpad / LaunchLab.
    Bonk,
    RaydiumLaunchpad,
    RaydiumCpmm,
    RaydiumClmm,
    RaydiumAmmV4,
    MeteoraDammV2,
    OrcaWhirlpool,
    MeteoraPools,
    MeteoraDlmm,
}

impl Protocol {
    pub fn get_program_id(&self) -> Vec<Pubkey> {
        match self {
            Protocol::PumpSwap => vec![PUMPSWAP_PROGRAM_ID],
            Protocol::PumpFun => vec![PUMPFUN_PROGRAM_ID],
            Protocol::PumpFees => vec![PUMP_FEES_PROGRAM_ID],
            Protocol::Bonk => vec![BONK_PROGRAM_ID],
            Protocol::RaydiumLaunchpad => vec![BONK_PROGRAM_ID],
            Protocol::RaydiumCpmm => vec![RAYDIUM_CPMM_PROGRAM_ID],
            Protocol::RaydiumClmm => vec![RAYDIUM_CLMM_PROGRAM_ID],
            Protocol::RaydiumAmmV4 => vec![RAYDIUM_AMM_V4_PROGRAM_ID],
            Protocol::MeteoraDammV2 => vec![METEORA_DAMM_V2_PROGRAM_ID],
            Protocol::OrcaWhirlpool => vec![ORCA_WHIRLPOOL_PROGRAM_ID],
            Protocol::MeteoraPools => vec![METEORA_POOLS_PROGRAM_ID],
            Protocol::MeteoraDlmm => vec![METEORA_DLMM_PROGRAM_ID],
        }
    }
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::PumpSwap => write!(f, "PumpSwap"),
            Protocol::PumpFun => write!(f, "PumpFun"),
            Protocol::PumpFees => write!(f, "PumpFees"),
            Protocol::Bonk => write!(f, "Bonk"),
            Protocol::RaydiumLaunchpad => write!(f, "RaydiumLaunchpad"),
            Protocol::RaydiumCpmm => write!(f, "RaydiumCpmm"),
            Protocol::RaydiumClmm => write!(f, "RaydiumClmm"),
            Protocol::RaydiumAmmV4 => write!(f, "RaydiumAmmV4"),
            Protocol::MeteoraDammV2 => write!(f, "MeteoraDammV2"),
            Protocol::OrcaWhirlpool => write!(f, "OrcaWhirlpool"),
            Protocol::MeteoraPools => write!(f, "MeteoraPools"),
            Protocol::MeteoraDlmm => write!(f, "MeteoraDlmm"),
        }
    }
}

impl std::str::FromStr for Protocol {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pumpswap" => Ok(Protocol::PumpSwap),
            "pumpfun" => Ok(Protocol::PumpFun),
            "pumpfees" | "pump_fees" => Ok(Protocol::PumpFees),
            "bonk" => Ok(Protocol::Bonk),
            "raydiumlaunchpad" | "raydium_launchpad" | "raydium_launchlab" | "launchpad"
            | "launchlab" => Ok(Protocol::RaydiumLaunchpad),
            "raydiumcpmm" | "raydium_cpmm" => Ok(Protocol::RaydiumCpmm),
            "raydiumclmm" | "raydium_clmm" => Ok(Protocol::RaydiumClmm),
            "raydiumammv4" | "raydium_amm_v4" => Ok(Protocol::RaydiumAmmV4),
            "meteoradammv2" | "meteoradamm_v2" | "meteora_damm_v2" => Ok(Protocol::MeteoraDammV2),
            "orcawhirlpool" | "orca_whirlpool" | "orca" => Ok(Protocol::OrcaWhirlpool),
            "meteorapools" | "meteora_pools" => Ok(Protocol::MeteoraPools),
            "meteoradlmm" | "meteora_dlmm" => Ok(Protocol::MeteoraDlmm),
            _ => Err(anyhow!("Unsupported protocol: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Protocol;
    use std::str::FromStr;

    #[test]
    fn parses_display_style_protocol_names() {
        for protocol in [
            Protocol::PumpFees,
            Protocol::RaydiumLaunchpad,
            Protocol::RaydiumCpmm,
            Protocol::RaydiumClmm,
            Protocol::RaydiumAmmV4,
            Protocol::MeteoraDammV2,
            Protocol::OrcaWhirlpool,
            Protocol::MeteoraPools,
            Protocol::MeteoraDlmm,
        ] {
            let parsed = Protocol::from_str(&protocol.to_string()).unwrap();
            assert_eq!(parsed, protocol);
        }
    }

    #[test]
    fn parses_snake_case_protocol_aliases() {
        assert_eq!(Protocol::from_str("raydium_cpmm").unwrap(), Protocol::RaydiumCpmm);
        assert_eq!(Protocol::from_str("pump_fees").unwrap(), Protocol::PumpFees);
        assert_eq!(Protocol::from_str("raydium_launchpad").unwrap(), Protocol::RaydiumLaunchpad);
        assert_eq!(Protocol::from_str("raydium_clmm").unwrap(), Protocol::RaydiumClmm);
        assert_eq!(Protocol::from_str("raydium_amm_v4").unwrap(), Protocol::RaydiumAmmV4);
        assert_eq!(Protocol::from_str("meteora_damm_v2").unwrap(), Protocol::MeteoraDammV2);
        assert_eq!(Protocol::from_str("orca_whirlpool").unwrap(), Protocol::OrcaWhirlpool);
        assert_eq!(Protocol::from_str("meteora_pools").unwrap(), Protocol::MeteoraPools);
        assert_eq!(Protocol::from_str("meteora_dlmm").unwrap(), Protocol::MeteoraDlmm);
    }
}
