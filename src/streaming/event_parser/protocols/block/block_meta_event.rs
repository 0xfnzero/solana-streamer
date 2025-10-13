use crate::streaming::event_parser::common::{types::EventType, EventMetadata};
use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::signature::Signature;

/// Block元数据事件
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct BlockMetaEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub slot: u64,
    pub block_hash: String,
}

impl BlockMetaEvent {
    pub fn new(
        slot: u64,
        block_hash: String,
        block_time_ms: i64,
        recv_us: i64,
    ) -> Self {
        let metadata = EventMetadata::new(
            Signature::default(),
            slot,
            block_time_ms / 1000,
            block_time_ms,
            crate::streaming::event_parser::common::types::ProtocolType::Common,
            EventType::BlockMeta,
            solana_sdk::pubkey::Pubkey::default(),
            0,
            None,
            recv_us,
            None,
        );
        Self { metadata, slot, block_hash }
    }
}
