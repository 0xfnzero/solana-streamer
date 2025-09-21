use crate::streaming::event_parser::common::EventType;
use crate::streaming::event_parser::common::SwapData;
use solana_sdk::signature::Signature;
use std::fmt::Debug;

/// Unified Event Interface - All protocol events must implement this trait
pub trait UnifiedEvent: Debug + Send + Sync {
    /// Get event type
    fn event_type(&self) -> EventType;

    /// Get transaction signature
    fn signature(&self) -> &Signature;

    /// Get slot number
    fn slot(&self) -> u64;

    /// Get program received timestamp (milliseconds)
    fn recv_us(&self) -> i64;

    /// Processing time consumption (milliseconds)
    fn handle_us(&self) -> i64;

    /// Set processing time consumption (milliseconds)
    fn set_handle_us(&mut self, handle_us: i64);

    /// Convert event to Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;

    /// Convert event to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    /// Clone the event
    fn clone_boxed(&self) -> Box<dyn UnifiedEvent>;

    /// Merge events (optional implementation)
    fn merge(&mut self, _other: &dyn UnifiedEvent) {
        // Default implementation: no merging operation
    }

    /// Set swap data
    fn set_swap_data(&mut self, swap_data: SwapData);

    /// swap_data is parsed
    fn swap_data_is_parsed(&self) -> bool;

    /// Get index
    fn outer_index(&self) -> i64;
    fn inner_index(&self) -> Option<i64>;

    /// Get transaction index in slot
    fn transaction_index(&self) -> Option<u64>;
}

// 为Box<dyn UnifiedEvent>实现Clone
impl Clone for Box<dyn UnifiedEvent> {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}
