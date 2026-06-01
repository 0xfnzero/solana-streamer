//! Transaction parser entry point.
//!
//! gRPC and ShredStream parsing are delegated to `sol-parser-sdk`; this layer
//! only adapts SDK events into streamer event structs and applies streamer
//! metadata enrichment.

pub(crate) mod helpers;

pub struct EventParser;

impl EventParser {
    #[allow(clippy::too_many_arguments)]
    pub async fn parse_grpc_transaction(
        protocols: &[crate::streaming::event_parser::Protocol],
        event_type_filter: Option<&crate::streaming::event_parser::common::filter::EventTypeFilter>,
        mut grpc_tx: yellowstone_grpc_proto::geyser::SubscribeUpdateTransactionInfo,
        signature: solana_sdk::signature::Signature,
        slot: Option<u64>,
        block_time: Option<prost_types::Timestamp>,
        recv_us: i64,
        bot_wallet: Option<solana_sdk::pubkey::Pubkey>,
        tx_index: Option<u64>,
        callback: std::sync::Arc<dyn Fn(crate::streaming::event_parser::DexEvent) + Send + Sync>,
    ) -> anyhow::Result<()> {
        if grpc_tx.signature.is_empty() {
            grpc_tx.signature = signature.as_ref().to_vec();
        }
        if grpc_tx.index == 0 {
            grpc_tx.index = tx_index.unwrap_or(0);
        }

        let block_us = block_time.map(|t| t.seconds * 1_000_000 + t.nanos as i64 / 1_000);
        let update = yellowstone_grpc_proto::geyser::SubscribeUpdateTransaction {
            slot: slot.unwrap_or(0),
            transaction: Some(grpc_tx),
        };
        let sdk_parse_filter =
            crate::streaming::event_parser::common::filter::build_sdk_parse_event_filter(
                event_type_filter,
            );
        let sdk_events = sol_parser_sdk::grpc::parse_subscribe_update_transaction_low_latency(
            &update,
            recv_us,
            block_us,
            sdk_parse_filter.as_ref(),
        );
        for sdk_event in sdk_events {
            if let Some(mut event) = crate::streaming::parser_sdk_bridge::adapt_parser_event(
                sdk_event,
                block_time.as_ref(),
                recv_us,
                protocols,
                event_type_filter,
            ) {
                event.metadata_mut().handle_us =
                    crate::streaming::event_parser::common::high_performance_clock::elapsed_micros_since(
                        recv_us,
                    );
                event = helpers::process_event(event, bot_wallet);
                callback(event);
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn parse_instruction_events_from_versioned_transaction(
        protocols: &[crate::streaming::event_parser::Protocol],
        event_type_filter: Option<&crate::streaming::event_parser::common::filter::EventTypeFilter>,
        transaction: &solana_sdk::transaction::VersionedTransaction,
        signature: solana_sdk::signature::Signature,
        slot: Option<u64>,
        _block_time: Option<prost_types::Timestamp>,
        recv_us: i64,
        _accounts: &[solana_sdk::pubkey::Pubkey],
        _inner_instructions: &[solana_transaction_status::InnerInstructions],
        bot_wallet: Option<solana_sdk::pubkey::Pubkey>,
        tx_index: Option<u64>,
        callback: std::sync::Arc<dyn Fn(crate::streaming::event_parser::DexEvent) + Send + Sync>,
    ) -> anyhow::Result<()> {
        crate::streaming::common::parse_shred_transaction_events(
            transaction,
            signature,
            slot.unwrap_or(0),
            tx_index,
            recv_us,
            protocols,
            event_type_filter,
            bot_wallet,
            |event| {
                callback(event);
            },
        );
        Ok(())
    }
}
