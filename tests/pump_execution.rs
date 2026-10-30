use solana_streamer_sdk::{
    parse_encoded_rpc_transaction_as_streamer_events, streaming::event_parser::DexEvent,
};
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;

#[tokio::test]
async fn reported_alt_creates_and_multi_hop_survive_the_streamer_facade() {
    use solana_sdk::pubkey::Pubkey;
    use solana_streamer_sdk::streaming::{
        common::event_processor::process_shred_transaction,
        event_parser::{
            common::{filter::EventTypeFilter, types::EventType},
            Protocol,
        },
        shred::TransactionWithSlot,
    };
    use std::sync::{Arc, Mutex};
    let filter =
        EventTypeFilter::include_only(vec![EventType::PumpFunCreateToken, EventType::PumpFunTrade]);
    for source in [
        include_str!("fixtures/pump_create_alt_20261026.json"),
        include_str!("fixtures/pump_multi_hop_20261025.json"),
    ] {
        let fixture: serde_json::Value = serde_json::from_str(source).unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let rpc: EncodedConfirmedTransactionWithStatusMeta =
                serde_json::from_value(case["wire_rpc"].clone()).unwrap();
            let signed = rpc.transaction.transaction.decode().unwrap();
            signed.verify_and_hash_message().unwrap();
            let expected_alt_addresses =
                signed.message.address_table_lookups().filter(|lookups| !lookups.is_empty()).map(
                    |lookups| lookups.iter().map(|lookup| lookup.account_key).collect::<Vec<_>>(),
                );
            let captured = Arc::new(Mutex::new(Vec::new()));
            let output = captured.clone();
            process_shred_transaction(
                TransactionWithSlot::new(signed, rpc.slot, 123456, Some(888)),
                &[Protocol::PumpFun],
                Some(&filter),
                Arc::new(move |e| output.lock().unwrap().push(e)),
                None,
            )
            .await
            .unwrap();
            let events = captured.lock().unwrap();
            assert_eq!(events.len(), 2, "{}", case["signature"]);
            let DexEvent::PumpFunCreateTokenEvent(create) = &events[0] else {
                panic!("create must be first")
            };
            let DexEvent::PumpFunTradeEvent(trade) = &events[1] else { panic!("trade missing") };
            assert_eq!(create.ix_name, "create_v2");
            assert_eq!(create.alt_addresses, expected_alt_addresses);
            assert_eq!(create.mint, trade.mint);
            assert!(trade.is_created_buy && trade.is_buy);
            for event in &*events {
                assert_eq!(event.metadata().tx_index, Some(888));
                assert_eq!(event.metadata().recv_us, 123456);
            }
            let multi_hop = trade.ix_name == "multi_hop_swap";
            if multi_hop {
                assert!(trade.third_program.is_some());
                assert_eq!((trade.sol_amount, trade.token_amount, trade.quote_amount), (0, 0, 0));
            } else {
                assert_eq!(trade.token_program, Pubkey::default());
                if trade.ix_name != "buy_exact_sol_in" {
                    assert_eq!(
                        trade.quote_mint,
                        Pubkey::default(),
                        "unknown quote must stay unknown"
                    );
                }
            }
            for mode in 0..3 {
                let mut rpc = rpc.clone();
                if let Some(meta) = rpc.transaction.meta.as_mut() {
                    if let solana_transaction_status::option_serializer::OptionSerializer::Some(
                        logs,
                    ) = &mut meta.log_messages
                    {
                        if mode == 1 {
                            logs.clear();
                        }
                        if mode == 2 {
                            logs.truncate(logs.len() / 2);
                        }
                    }
                }
                let rpc_events = parse_encoded_rpc_transaction_as_streamer_events(
                    &rpc,
                    123456,
                    &[Protocol::PumpFun],
                    Some(&filter),
                )
                .unwrap();
                let (meta, transaction) =
                    solana_streamer_sdk::parser_sdk::convert_rpc_to_grpc(&rpc).unwrap();
                let signature = transaction.signatures[0].clone();
                let grpc_events = solana_streamer_sdk::streaming::common::event_processor::parse_grpc_transaction_events(
                    solana_streamer_sdk::streaming::grpc::TransactionPretty {
                        slot: rpc.slot, tx_index: Some(888), recv_us: 123456,
                        grpc_tx: yellowstone_grpc_proto::prelude::SubscribeUpdateTransactionInfo {
                            signature, transaction: Some(transaction), meta: Some(meta), index: 888,
                            ..Default::default()
                        },
                        ..Default::default()
                    }, &[Protocol::PumpFun], Some(&filter), None);
                for executed in [rpc_events, grpc_events] {
                    let DexEvent::PumpFunCreateTokenEvent(c) = &executed[0] else {
                        panic!("create event")
                    };
                    assert_eq!(c.alt_addresses, expected_alt_addresses);
                    assert!(
                        matches!(&executed[0], DexEvent::PumpFunCreateTokenEvent(c) if c.ix_name == "create_v2"),
                        "{} mode {mode}: {:?}",
                        case["signature"],
                        executed[0]
                    );
                    let fill = executed
                        .iter()
                        .find_map(|e| match e {
                            DexEvent::PumpFunTradeEvent(t) if t.mint == trade.mint => Some(t),
                            _ => None,
                        })
                        .unwrap();
                    assert_eq!(fill.ix_name, trade.ix_name);
                    if multi_hop {
                        assert!(fill.third_program.is_some());
                        assert_eq!(
                            (fill.sol_amount, fill.token_amount, fill.quote_amount),
                            (0, 255705136795915, 11728308720756)
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn delta_sell_execution_survives_truncated_logs() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/pump_execution_20261019.json")).unwrap();
    let mut expected = None;
    for mode in 0..3 {
        let mut raw = fixture.clone();
        let logs = raw["meta"]["logMessages"].as_array_mut().unwrap();
        if mode == 1 {
            logs.truncate(logs.len() / 2);
        }
        if mode == 2 {
            logs.clear();
        }
        let tx: EncodedConfirmedTransactionWithStatusMeta = serde_json::from_value(raw).unwrap();
        let events = parse_encoded_rpc_transaction_as_streamer_events(&tx, 0, &[], None).unwrap();
        let trades: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let DexEvent::PumpFunTradeEvent(t) = e {
                    assert!(!t.is_buy);
                    assert!(t.token_amount > 0);
                    Some((t.mint, t.user, t.token_amount, t.quote_amount, t.sol_amount))
                } else {
                    None
                }
            })
            .collect();
        assert!(!trades.is_empty());
        if let Some(want) = &expected {
            assert_eq!(&trades, want);
        } else {
            expected = Some(trades);
        }
    }
}
