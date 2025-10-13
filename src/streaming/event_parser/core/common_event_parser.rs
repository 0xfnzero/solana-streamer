use crate::streaming::event_parser::common::high_performance_clock::elapsed_micros_since;
use crate::streaming::event_parser::core::traits::DexEvent;
use crate::streaming::event_parser::protocols::block::block_meta_event::BlockMetaEvent;

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
