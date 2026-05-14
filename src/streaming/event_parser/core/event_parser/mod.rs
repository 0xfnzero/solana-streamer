//! Transaction parser entry point with separate gRPC and standard ix paths.
//!
//! | Module | Path |
//! |--------|------|
//! | [`grpc_path`] | Yellowstone gRPC |
//! | [`compiled_path`] | standard transaction / RPC replay |
//! | [`helpers`] | `should_handle`、`process_event` |

mod compiled_path;
mod grpc_path;
pub(crate) mod helpers;

pub struct EventParser;

impl EventParser {
    pub async fn parse_grpc_transaction(
        protocols: &[crate::streaming::event_parser::Protocol],
        event_type_filter: Option<&crate::streaming::event_parser::common::filter::EventTypeFilter>,
        grpc_tx: yellowstone_grpc_proto::geyser::SubscribeUpdateTransactionInfo,
        signature: solana_sdk::signature::Signature,
        slot: Option<u64>,
        block_time: Option<prost_types::Timestamp>,
        recv_us: i64,
        bot_wallet: Option<solana_sdk::pubkey::Pubkey>,
        tx_index: Option<u64>,
        callback: std::sync::Arc<dyn Fn(crate::streaming::event_parser::DexEvent) + Send + Sync>,
    ) -> anyhow::Result<()> {
        grpc_path::parse_grpc_transaction(
            protocols,
            event_type_filter,
            grpc_tx,
            signature,
            slot,
            block_time,
            recv_us,
            bot_wallet,
            tx_index,
            callback,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn parse_instruction_events_from_versioned_transaction(
        protocols: &[crate::streaming::event_parser::Protocol],
        event_type_filter: Option<&crate::streaming::event_parser::common::filter::EventTypeFilter>,
        transaction: &solana_sdk::transaction::VersionedTransaction,
        signature: solana_sdk::signature::Signature,
        slot: Option<u64>,
        block_time: Option<prost_types::Timestamp>,
        recv_us: i64,
        accounts: &[solana_sdk::pubkey::Pubkey],
        inner_instructions: &[solana_transaction_status::InnerInstructions],
        bot_wallet: Option<solana_sdk::pubkey::Pubkey>,
        tx_index: Option<u64>,
        callback: std::sync::Arc<dyn Fn(crate::streaming::event_parser::DexEvent) + Send + Sync>,
    ) -> anyhow::Result<()> {
        compiled_path::parse_instruction_events_from_versioned_transaction(
            protocols,
            event_type_filter,
            transaction,
            signature,
            slot,
            block_time,
            recv_us,
            accounts,
            inner_instructions,
            bot_wallet,
            tx_index,
            callback,
        )
        .await
    }
}
