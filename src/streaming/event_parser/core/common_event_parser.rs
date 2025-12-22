use crate::streaming::event_parser::common::high_performance_clock::elapsed_micros_since;
use crate::streaming::event_parser::common::types::{EventType, ProtocolType};
use crate::streaming::event_parser::common::EventMetadata;
use crate::streaming::event_parser::core::traits::DexEvent;
use crate::streaming::event_parser::protocols::block::block_meta_event::BlockMetaEvent;
use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

// Compute Budget Program ID
pub const COMPUTE_BUDGET_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("ComputeBudget111111111111111111111111111111");

/// SetComputeUnitLimit event
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct SetComputeUnitLimitEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    /// Number of compute units requested
    pub units: u32,
}

/// SetComputeUnitPrice event
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct SetComputeUnitPriceEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    /// Price per compute unit (micro-lamports)
    pub micro_lamports: u64,
}

pub struct CommonEventParser {}

impl CommonEventParser {
    pub fn generate_block_meta_event(
        slot: u64,
        block_hash: String,
        block_time_ms: i64,
        recv_us: i64,
    ) -> DexEvent {
        let mut block_meta_event = BlockMetaEvent::new(slot, block_hash, block_time_ms, recv_us);
        block_meta_event.metadata.handle_us = elapsed_micros_since(recv_us);
        DexEvent::BlockMetaEvent(block_meta_event)
    }

    /// Parse Compute Budget instruction
    pub fn parse_compute_budget_instruction(
        instruction_data: &[u8],
        mut metadata: EventMetadata,
    ) -> Option<DexEvent> {
        if instruction_data.is_empty() {
            return None;
        }

        // Set protocol to Common
        metadata.protocol = ProtocolType::Common;

        // Compute Budget instructions use single-byte discriminator
        match instruction_data[0] {
            // SetComputeUnitLimit: discriminator = 2
            2 => {
                if instruction_data.len() < 5 {
                    return None;
                }
                let units = u32::from_le_bytes(instruction_data[1..5].try_into().ok()?);
                metadata.event_type = EventType::SetComputeUnitLimit;
                let event = SetComputeUnitLimitEvent { metadata, units };
                Some(DexEvent::SetComputeUnitLimitEvent(event))
            }
            // SetComputeUnitPrice: discriminator = 3
            3 => {
                if instruction_data.len() < 9 {
                    return None;
                }
                let micro_lamports = u64::from_le_bytes(instruction_data[1..9].try_into().ok()?);
                metadata.event_type = EventType::SetComputeUnitPrice;
                let event = SetComputeUnitPriceEvent { metadata, micro_lamports };
                Some(DexEvent::SetComputeUnitPriceEvent(event))
            }
            _ => None,
        }
    }
}
