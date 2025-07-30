pub mod common;
pub mod core;
pub mod factory;
pub mod protocols;

pub use core::traits::{EventParser, UnifiedEvent};
pub use factory::{EventParserFactory, Protocol};

/// Macro: Simplify downcast_ref pattern matching
/// 
/// # Usage Example
/// ```ignore
/// use sol_trade_sdk::streaming::event_parser::match_event;
/// 
/// match_event!(event, {
///     PumpSwapCreatePoolEvent => |typed_event| {
///         println!("CreatePool event: {:?}", typed_event);
///     },
///     PumpSwapDepositEvent => |typed_event| {
///         // Handle deposit event
///     },
/// });
/// ```
#[macro_export]
macro_rules! match_event {
    ($event:expr, {
        $($event_type:ty => $handler:expr),* $(,)?
    }) => {
        $(
            if let Some(typed_event) = $event.as_any().downcast_ref::<$event_type>() {
                $handler(typed_event.clone());
            } else
        )*
        {
            // 默认情况：什么都不做
        }
    };
}

// 重新导出宏以便于使用
pub use match_event;
