//! Central event parsing dispatcher
//!
//! Routes to corresponding parsing functions based on protocol type, replacing the original static CONFIGS array architecture
//!
//! ## Design Principles
//! - **Single Responsibility**: Each function is responsible for one thing (routing, parsing, merging separated)
//! - **Flexibility**: Callers can choose whether to merge or customize merge logic
//! - **Testability**: Each function can be tested independently

use crate::streaming::event_parser::{
    common::EventMetadata,
    core::common_event_parser::{CommonEventParser, COMPUTE_BUDGET_PROGRAM_ID},
    protocols::{
        bonk::parser as bonk, meteora_damm_v2::parser as meteora_damm_v2, pumpfun::parser as pumpfun,
        pumpswap::parser as pumpswap, raydium_amm_v4::parser as raydium_amm_v4,
        raydium_clmm::parser as raydium_clmm, raydium_cpmm::parser as raydium_cpmm,
    },
    DexEvent, Protocol,
};
use solana_sdk::pubkey::Pubkey;

/// Central event parsing dispatcher
///
/// Responsible for routing parsing requests to corresponding protocol parsing functions
pub struct EventDispatcher;

impl EventDispatcher {
    /// Parse instruction event (parse only, no merging)
    ///
    /// # Parameters
    /// - `protocol`: Protocol type
    /// - `instruction_discriminator`: Instruction discriminator (8 bytes)
    /// - `instruction_data`: Instruction data
    /// - `accounts`: Account public key list
    /// - `metadata`: Event metadata
    ///
    /// # Returns
    /// Returns `Some(DexEvent)` on successful parsing, otherwise `None`
    #[inline]
    pub fn dispatch_instruction(
        protocol: Protocol,
        instruction_discriminator: &[u8],
        instruction_data: &[u8],
        accounts: &[Pubkey],
        mut metadata: EventMetadata,
    ) -> Option<DexEvent> {
        // Set metadata.protocol based on protocol type
        use crate::streaming::event_parser::common::ProtocolType;
        metadata.protocol = match protocol {
            Protocol::PumpFun => ProtocolType::PumpFun,
            Protocol::PumpSwap => ProtocolType::PumpSwap,
            Protocol::Bonk => ProtocolType::Bonk,
            Protocol::RaydiumCpmm => ProtocolType::RaydiumCpmm,
            Protocol::RaydiumClmm => ProtocolType::RaydiumClmm,
            Protocol::RaydiumAmmV4 => ProtocolType::RaydiumAmmV4,
            Protocol::MeteoraDammV2 => ProtocolType::MeteoraDammV2,
        };

        match protocol {
            Protocol::PumpFun => pumpfun::parse_pumpfun_instruction_data(
                instruction_discriminator,
                instruction_data,
                accounts,
                metadata,
            ),
            Protocol::PumpSwap => pumpswap::parse_pumpswap_instruction_data(
                instruction_discriminator,
                instruction_data,
                accounts,
                metadata,
            ),
            Protocol::Bonk => bonk::parse_bonk_instruction_data(
                instruction_discriminator,
                instruction_data,
                accounts,
                metadata,
            ),
            Protocol::RaydiumCpmm => raydium_cpmm::parse_raydium_cpmm_instruction_data(
                instruction_discriminator,
                instruction_data,
                accounts,
                metadata,
            ),
            Protocol::RaydiumClmm => raydium_clmm::parse_raydium_clmm_instruction_data(
                instruction_discriminator,
                instruction_data,
                accounts,
                metadata,
            ),
            Protocol::RaydiumAmmV4 => raydium_amm_v4::parse_raydium_amm_v4_instruction_data(
                instruction_discriminator,
                instruction_data,
                accounts,
                metadata,
            ),
            Protocol::MeteoraDammV2 => meteora_damm_v2::parse_meteora_damm_v2_instruction_data(
                instruction_discriminator,
                instruction_data,
                accounts,
                metadata,
            ),
        }
    }

    /// Parse inner instruction event (parse only, no merging)
    ///
    /// # Parameters
    /// - `protocol`: Protocol type
    /// - `inner_instruction_discriminator`: Inner instruction discriminator (16 bytes)
    /// - `inner_instruction_data`: Inner instruction data
    /// - `metadata`: Event metadata
    ///
    /// # Returns
    /// Returns `Some(DexEvent)` on successful parsing, otherwise `None`
    #[inline]
    pub fn dispatch_inner_instruction(
        protocol: Protocol,
        inner_instruction_discriminator: &[u8],
        inner_instruction_data: &[u8],
        mut metadata: EventMetadata,
    ) -> Option<DexEvent> {
        // Set metadata.protocol based on protocol type
        use crate::streaming::event_parser::common::ProtocolType;
        metadata.protocol = match protocol {
            Protocol::PumpFun => ProtocolType::PumpFun,
            Protocol::PumpSwap => ProtocolType::PumpSwap,
            Protocol::Bonk => ProtocolType::Bonk,
            Protocol::RaydiumCpmm => ProtocolType::RaydiumCpmm,
            Protocol::RaydiumClmm => ProtocolType::RaydiumClmm,
            Protocol::RaydiumAmmV4 => ProtocolType::RaydiumAmmV4,
            Protocol::MeteoraDammV2 => ProtocolType::MeteoraDammV2,
        };

        match protocol {
            Protocol::PumpFun => pumpfun::parse_pumpfun_inner_instruction_data(
                inner_instruction_discriminator,
                inner_instruction_data,
                metadata,
            ),
            Protocol::PumpSwap => pumpswap::parse_pumpswap_inner_instruction_data(
                inner_instruction_discriminator,
                inner_instruction_data,
                metadata,
            ),
            Protocol::Bonk => bonk::parse_bonk_inner_instruction_data(
                inner_instruction_discriminator,
                inner_instruction_data,
                metadata,
            ),
            Protocol::RaydiumCpmm => raydium_cpmm::parse_raydium_cpmm_inner_instruction_data(
                inner_instruction_discriminator,
                inner_instruction_data,
                metadata,
            ),
            Protocol::RaydiumClmm => raydium_clmm::parse_raydium_clmm_inner_instruction_data(
                inner_instruction_discriminator,
                inner_instruction_data,
                metadata,
            ),
            Protocol::RaydiumAmmV4 => raydium_amm_v4::parse_raydium_amm_v4_inner_instruction_data(
                inner_instruction_discriminator,
                inner_instruction_data,
                metadata,
            ),
            Protocol::MeteoraDammV2 => meteora_damm_v2::parse_meteora_damm_v2_inner_instruction_data(
                inner_instruction_discriminator,
                inner_instruction_data,
                metadata,
            ),
        }
    }

    /// Match protocol type by program_id
    #[inline]
    pub fn match_protocol_by_program_id(program_id: &Pubkey) -> Option<Protocol> {
        if program_id == &pumpfun::PUMPFUN_PROGRAM_ID {
            Some(Protocol::PumpFun)
        } else if program_id == &pumpswap::PUMPSWAP_PROGRAM_ID {
            Some(Protocol::PumpSwap)
        } else if program_id == &bonk::BONK_PROGRAM_ID {
            Some(Protocol::Bonk)
        } else if program_id == &raydium_cpmm::RAYDIUM_CPMM_PROGRAM_ID {
            Some(Protocol::RaydiumCpmm)
        } else if program_id == &raydium_clmm::RAYDIUM_CLMM_PROGRAM_ID {
            Some(Protocol::RaydiumClmm)
        } else if program_id == &raydium_amm_v4::RAYDIUM_AMM_V4_PROGRAM_ID {
            Some(Protocol::RaydiumAmmV4)
        } else if program_id == &meteora_damm_v2::METEORA_DAMM_V2_PROGRAM_ID {
            Some(Protocol::MeteoraDammV2)
        } else {
            None
        }
    }

    /// Check if it's a Compute Budget Program
    #[inline]
    pub fn is_compute_budget_program(program_id: &Pubkey) -> bool {
        program_id == &COMPUTE_BUDGET_PROGRAM_ID
    }

    /// Parse Compute Budget instruction
    ///
    /// # Parameters
    /// - `instruction_data`: Instruction data
    /// - `metadata`: Event metadata
    ///
    /// # Returns
    /// Returns `Some(DexEvent)` on successful parsing, otherwise `None`
    #[inline]
    pub fn dispatch_compute_budget_instruction(
        instruction_data: &[u8],
        metadata: EventMetadata,
    ) -> Option<DexEvent> {
        CommonEventParser::parse_compute_budget_instruction(instruction_data, metadata)
    }

    /// Get program_id for specified protocol
    #[inline]
    pub fn get_program_id(protocol: Protocol) -> Pubkey {
        match protocol {
            Protocol::PumpFun => pumpfun::PUMPFUN_PROGRAM_ID,
            Protocol::PumpSwap => pumpswap::PUMPSWAP_PROGRAM_ID,
            Protocol::Bonk => bonk::BONK_PROGRAM_ID,
            Protocol::RaydiumCpmm => raydium_cpmm::RAYDIUM_CPMM_PROGRAM_ID,
            Protocol::RaydiumClmm => raydium_clmm::RAYDIUM_CLMM_PROGRAM_ID,
            Protocol::RaydiumAmmV4 => raydium_amm_v4::RAYDIUM_AMM_V4_PROGRAM_ID,
            Protocol::MeteoraDammV2 => meteora_damm_v2::METEORA_DAMM_V2_PROGRAM_ID,
        }
    }

    /// Batch get program_ids
    pub fn get_program_ids(protocols: &[Protocol]) -> Vec<Pubkey> {
        protocols.iter().map(|p| Self::get_program_id(p.clone())).collect()
    }

    /// Parse account data
    ///
    /// Route to corresponding protocol account parsing function based on account discriminator
    ///
    /// # Parameters
    /// - `protocol`: Protocol type
    /// - `discriminator`: Account discriminator
    /// - `account`: Account information
    /// - `metadata`: Event metadata
    ///
    /// # Returns
    /// Returns `Some(DexEvent)` on successful parsing, otherwise `None`
    pub fn dispatch_account(
        protocol: Protocol,
        discriminator: &[u8],
        account: &crate::streaming::grpc::AccountPretty,
        mut metadata: crate::streaming::event_parser::common::EventMetadata,
    ) -> Option<DexEvent> {
        // Set metadata.protocol based on protocol type
        use crate::streaming::event_parser::common::ProtocolType;
        metadata.protocol = match protocol {
            Protocol::PumpFun => ProtocolType::PumpFun,
            Protocol::PumpSwap => ProtocolType::PumpSwap,
            Protocol::Bonk => ProtocolType::Bonk,
            Protocol::RaydiumCpmm => ProtocolType::RaydiumCpmm,
            Protocol::RaydiumClmm => ProtocolType::RaydiumClmm,
            Protocol::RaydiumAmmV4 => ProtocolType::RaydiumAmmV4,
            Protocol::MeteoraDammV2 => ProtocolType::MeteoraDammV2,
        };

        match protocol {
            Protocol::PumpFun => {
                pumpfun::parse_pumpfun_account_data(discriminator, account, metadata)
            }
            Protocol::PumpSwap => {
                pumpswap::parse_pumpswap_account_data(discriminator, account, metadata)
            }
            Protocol::Bonk => bonk::parse_bonk_account_data(discriminator, account, metadata),
            Protocol::RaydiumCpmm => {
                raydium_cpmm::parse_raydium_cpmm_account_data(discriminator, account, metadata)
            }
            Protocol::RaydiumClmm => {
                raydium_clmm::parse_raydium_clmm_account_data(discriminator, account, metadata)
            }
            Protocol::RaydiumAmmV4 => {
                raydium_amm_v4::parse_raydium_amm_v4_account_data(discriminator, account, metadata)
            }
            Protocol::MeteoraDammV2 => {
                // Meteora DAMM currently doesn't need to parse account data, return None
                None
            }
        }
    }
}
