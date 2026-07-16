//! Public extension bridge for advanced `sol-parser-sdk` interop.
//!
//! Existing streamer subscription APIs remain unchanged. Use this module when code needs direct
//! access to raw SDK parsers/events but still wants streamer `DexEvent` compatibility.
use prost_types::Timestamp;

use crate::streaming::event_parser::common::filter::{
    build_sdk_parse_event_filter, EventTypeFilter,
};
use crate::streaming::event_parser::{DexEvent, Protocol};
use crate::streaming::grpc::AccountPretty;

pub use sol_parser_sdk as raw;

pub type SdkClientConfig = sol_parser_sdk::grpc::types::ClientConfig;
pub type SdkDexEvent = sol_parser_sdk::DexEvent;
pub type SdkEventType = sol_parser_sdk::grpc::types::EventType;
pub type SdkEventTypeFilter = sol_parser_sdk::grpc::types::EventTypeFilter;
pub type SdkOrderMode = sol_parser_sdk::grpc::types::OrderMode;
pub type SdkProtocol = sol_parser_sdk::grpc::types::Protocol;

pub fn event_type_filter_to_sdk(filter: Option<&EventTypeFilter>) -> Option<SdkEventTypeFilter> {
    build_sdk_parse_event_filter(filter)
}

pub fn adapt_event(
    sdk_event: SdkDexEvent,
    block_time: Option<&Timestamp>,
    recv_wall_us: i64,
    protocols: &[Protocol],
    event_type_filter: Option<&EventTypeFilter>,
) -> Option<DexEvent> {
    crate::streaming::parser_sdk_bridge::adapt_parser_event(
        sdk_event,
        block_time,
        recv_wall_us,
        protocols,
        event_type_filter,
    )
}

pub fn adapt_events(
    sdk_events: Vec<SdkDexEvent>,
    block_time: Option<&Timestamp>,
    recv_wall_us: i64,
    protocols: &[Protocol],
    event_type_filter: Option<&EventTypeFilter>,
) -> Vec<DexEvent> {
    crate::streaming::parser_sdk_bridge::adapt_parser_events_list(
        sdk_events,
        block_time,
        recv_wall_us,
        protocols,
        event_type_filter,
    )
}

pub fn parse_account_event(
    account: &AccountPretty,
    protocols: &[Protocol],
    event_type_filter: Option<&EventTypeFilter>,
) -> Option<DexEvent> {
    crate::streaming::parser_sdk_bridge::parse_sdk_account_event(
        account,
        protocols,
        event_type_filter,
    )
}
