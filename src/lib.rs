pub mod common;
pub mod protos;
pub mod streaming;

#[cfg(all(not(feature = "sdk-parse-borsh"), not(feature = "sdk-parse-zero-copy")))]
compile_error!("Enable one SDK parser backend: sdk-parse-borsh or sdk-parse-zero-copy.");

pub use sol_parser_sdk as parser_sdk;
pub use streaming::sdk_bridge;
pub use streaming::{
    fetch_rpc_transaction_as_streamer_events, fetch_rpc_transaction_as_streamer_events_async,
    parse_encoded_rpc_transaction_as_streamer_events, RpcParseError,
};
