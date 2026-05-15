use crate::streaming::event_parser::common::high_performance_clock::elapsed_micros_since;
use crate::streaming::event_parser::common::EventMetadata;
use crate::streaming::event_parser::core::traits::DexEvent;
use crate::streaming::event_parser::protocols::block::block_meta_event::BlockMetaEvent;
use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};

/// SetComputeUnitLimit 事件
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct SetComputeUnitLimitEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    /// 请求的计算单元数量
    pub units: u32,
}

/// SetComputeUnitPrice 事件
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct SetComputeUnitPriceEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    /// 每个计算单元的价格 (micro-lamports)
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
}
