//! SDK-backed event dispatcher kept for compatibility with older streamer APIs.
//!
//! Streamer no longer owns protocol parsers here. The dispatcher only adapts
//! streamer metadata and routes to `sol-parser-sdk` parsers, then converts SDK
//! events back to streamer `DexEvent`.

use crate::streaming::event_parser::{common::EventMetadata, DexEvent, Protocol};
use crate::streaming::parser_sdk_bridge::{
    block_timestamp_from_stream_meta, convert_parser_event, fuse_streamer_ix_ctx,
    parse_sdk_account_event,
};
use sol_parser_sdk::core::events::EventMetadata as PbEventMetadata;
use sol_parser_sdk::instr::{
    all_inner, program_ids, pump_amm_inner, pump_inner, raydium_clmm_inner,
};
use solana_sdk::pubkey::Pubkey;

pub struct EventDispatcher;

impl EventDispatcher {
    #[inline]
    pub fn dispatch_instruction(
        protocol: Protocol,
        instruction_discriminator: &[u8],
        instruction_data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<DexEvent> {
        let mut full = Vec::with_capacity(instruction_discriminator.len() + instruction_data.len());
        full.extend_from_slice(instruction_discriminator);
        full.extend_from_slice(instruction_data);

        let program_id = Self::get_program_id(protocol);
        let pb = sol_parser_sdk::instr::parse_instruction_unified(
            &full,
            accounts,
            metadata.signature,
            metadata.slot,
            metadata.tx_index.unwrap_or(0),
            Some(metadata.block_time_ms.saturating_mul(1000)),
            metadata.recv_us,
            None,
            &program_id,
        )?;

        let ts = block_timestamp_from_stream_meta(&metadata);
        let ev = convert_parser_event(pb, Some(&ts), metadata.recv_us)?;
        Some(fuse_streamer_ix_ctx(ev, &metadata))
    }

    #[inline]
    pub fn dispatch_inner_instruction(
        protocol: Protocol,
        inner_instruction_discriminator: &[u8],
        inner_instruction_data: &[u8],
        metadata: EventMetadata,
    ) -> Option<DexEvent> {
        let disc: [u8; 16] = inner_instruction_discriminator.try_into().ok()?;
        let pm = pb_meta_from_streamer(&metadata);

        let pb = match protocol {
            Protocol::PumpFun => pump_inner::parse_pumpfun_inner_instruction(
                &disc,
                inner_instruction_data,
                pm,
                false,
            ),
            Protocol::PumpSwap => {
                pump_amm_inner::parse_pumpswap_inner_instruction(&disc, inner_instruction_data, pm)
            }
            Protocol::PumpFees => all_inner::pump_fees::parse(&disc, inner_instruction_data, pm),
            Protocol::Bonk | Protocol::RaydiumLaunchpad => {
                all_inner::raydium_launchlab::parse(&disc, inner_instruction_data, pm)
            }
            Protocol::RaydiumCpmm => {
                all_inner::raydium_cpmm::parse(&disc, inner_instruction_data, pm)
            }
            Protocol::RaydiumClmm => raydium_clmm_inner::parse_raydium_clmm_inner_instruction(
                &disc,
                inner_instruction_data,
                pm,
            ),
            Protocol::RaydiumAmmV4 => {
                all_inner::raydium_amm::parse(&disc, inner_instruction_data, pm)
            }
            Protocol::MeteoraDammV2 => {
                all_inner::meteora_damm::parse(&disc, inner_instruction_data, pm)
            }
            // Meteora DBC is currently log-side in sol-parser-sdk; direct inner dispatch is absent.
            Protocol::MeteoraDbc => None,
            Protocol::OrcaWhirlpool => all_inner::orca::parse(&disc, inner_instruction_data, pm),
            Protocol::MeteoraPools => {
                all_inner::meteora_amm::parse(&disc, inner_instruction_data, pm)
            }
            Protocol::MeteoraDlmm => {
                all_inner::meteora_dlmm::parse(&disc, inner_instruction_data, pm)
            }
        }?;

        let ts = block_timestamp_from_stream_meta(&metadata);
        let ev = convert_parser_event(pb, Some(&ts), metadata.recv_us)?;
        Some(fuse_streamer_ix_ctx(ev, &metadata))
    }

    #[inline]
    pub fn match_protocol_by_program_id(program_id: &Pubkey) -> Option<Protocol> {
        if program_id == &program_ids::PUMPFUN_PROGRAM_ID {
            Some(Protocol::PumpFun)
        } else if program_id == &program_ids::PUMP_FEES_PROGRAM_ID {
            Some(Protocol::PumpFees)
        } else if program_id == &program_ids::PUMPSWAP_PROGRAM_ID {
            Some(Protocol::PumpSwap)
        } else if program_id == &program_ids::RAYDIUM_LAUNCHLAB_PROGRAM_ID {
            Some(Protocol::Bonk)
        } else if program_id == &program_ids::RAYDIUM_CPMM_PROGRAM_ID {
            Some(Protocol::RaydiumCpmm)
        } else if program_id == &program_ids::RAYDIUM_CLMM_PROGRAM_ID {
            Some(Protocol::RaydiumClmm)
        } else if program_id == &program_ids::RAYDIUM_AMM_V4_PROGRAM_ID {
            Some(Protocol::RaydiumAmmV4)
        } else if program_id == &program_ids::METEORA_DAMM_V2_PROGRAM_ID {
            Some(Protocol::MeteoraDammV2)
        } else if program_id == &program_ids::METEORA_DBC_PROGRAM_ID {
            Some(Protocol::MeteoraDbc)
        } else if program_id == &program_ids::ORCA_WHIRLPOOL_PROGRAM_ID {
            Some(Protocol::OrcaWhirlpool)
        } else if program_id == &program_ids::METEORA_POOLS_PROGRAM_ID {
            Some(Protocol::MeteoraPools)
        } else if program_id == &program_ids::METEORA_DLMM_PROGRAM_ID {
            Some(Protocol::MeteoraDlmm)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_compute_budget_program(program_id: &Pubkey) -> bool {
        program_id == &solana_sdk::pubkey!("ComputeBudget111111111111111111111111111111")
    }

    #[inline]
    pub fn dispatch_compute_budget_instruction(
        _instruction_data: &[u8],
        _metadata: EventMetadata,
    ) -> Option<DexEvent> {
        None
    }

    #[inline]
    pub fn get_program_id(protocol: Protocol) -> Pubkey {
        match protocol {
            Protocol::PumpFun => program_ids::PUMPFUN_PROGRAM_ID,
            Protocol::PumpFees => program_ids::PUMP_FEES_PROGRAM_ID,
            Protocol::PumpSwap => program_ids::PUMPSWAP_PROGRAM_ID,
            Protocol::Bonk | Protocol::RaydiumLaunchpad => {
                program_ids::RAYDIUM_LAUNCHLAB_PROGRAM_ID
            }
            Protocol::RaydiumCpmm => program_ids::RAYDIUM_CPMM_PROGRAM_ID,
            Protocol::RaydiumClmm => program_ids::RAYDIUM_CLMM_PROGRAM_ID,
            Protocol::RaydiumAmmV4 => program_ids::RAYDIUM_AMM_V4_PROGRAM_ID,
            Protocol::MeteoraDammV2 => program_ids::METEORA_DAMM_V2_PROGRAM_ID,
            Protocol::MeteoraDbc => program_ids::METEORA_DBC_PROGRAM_ID,
            Protocol::OrcaWhirlpool => program_ids::ORCA_WHIRLPOOL_PROGRAM_ID,
            Protocol::MeteoraPools => program_ids::METEORA_POOLS_PROGRAM_ID,
            Protocol::MeteoraDlmm => program_ids::METEORA_DLMM_PROGRAM_ID,
        }
    }

    pub fn get_program_ids(protocols: &[Protocol]) -> Vec<Pubkey> {
        let mut ids = Vec::with_capacity(protocols.len());
        for protocol in protocols {
            let id = Self::get_program_id(protocol.clone());
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
        ids
    }

    pub fn dispatch_account(
        protocol: Protocol,
        _discriminator: &[u8],
        account: &crate::streaming::grpc::AccountPretty,
        _metadata: EventMetadata,
    ) -> Option<DexEvent> {
        parse_sdk_account_event(account, &[protocol], None)
    }
}

#[inline]
fn pb_meta_from_streamer(sm: &EventMetadata) -> PbEventMetadata {
    PbEventMetadata {
        signature: sm.signature,
        slot: sm.slot,
        tx_index: sm.tx_index.unwrap_or(0),
        block_time_us: sm.block_time_ms.saturating_mul(1000),
        grpc_recv_us: sm.recv_us,
        recent_blockhash: sm.recent_blockhash.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming::event_parser::common::EventType;

    #[test]
    fn pumpswap_current_sell_tail_survives_dispatch_and_bridge() {
        let mut payload = vec![0u8; 409];
        payload[352..360].copy_from_slice(&11u64.to_le_bytes());
        payload[360..368].copy_from_slice(&22u64.to_le_bytes());
        payload[368..376].copy_from_slice(&33u64.to_le_bytes());
        payload[376..384].copy_from_slice(&44u64.to_le_bytes());
        payload[384..400].copy_from_slice(&i128::MIN.to_le_bytes());
        payload[400] = 1;
        payload[401..409].copy_from_slice(&55u64.to_le_bytes());

        let event = EventDispatcher::dispatch_inner_instruction(
            Protocol::PumpSwap,
            &pump_amm_inner::discriminators::SELL,
            &payload,
            EventMetadata::default(),
        )
        .expect("current PumpSwap sell should dispatch");

        let DexEvent::PumpSwapSellEvent(event) = event else {
            panic!("expected PumpSwapSellEvent");
        };
        assert_eq!(event.metadata.event_type, EventType::PumpSwapSell);
        assert_eq!(event.cashback_fee_basis_points, 11);
        assert_eq!(event.cashback, 22);
        assert_eq!(event.buyback_fee_basis_points, 33);
        assert_eq!(event.buyback_fee, 44);
        assert_eq!(event.virtual_quote_reserves, i128::MIN);
        assert!(event.can_boost);
        assert_eq!(event.base_supply, 55);

        assert!(EventDispatcher::dispatch_inner_instruction(
            Protocol::PumpSwap,
            &pump_amm_inner::discriminators::SELL,
            &payload[..408],
            EventMetadata::default(),
        )
        .is_none());
    }
}
