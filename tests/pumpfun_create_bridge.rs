use solana_streamer_sdk::streaming::{
    common::event_processor::parse_grpc_transaction_events,
    event_parser::{DexEvent, Protocol},
    grpc::TransactionPretty,
};
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;
use yellowstone_grpc_proto::prelude::SubscribeUpdateTransactionInfo;

#[test]
fn real_create_accounts_and_failed_status_survive_rpc_and_grpc_bridge() {
    let directory =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pumpfun_create");
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(directory.join("manifest.json")).unwrap())
            .unwrap();
    let mut successful = 0;
    let mut failed = 0;
    for case in manifest["cases"].as_array().unwrap() {
        let tx: EncodedConfirmedTransactionWithStatusMeta = serde_json::from_str(
            &std::fs::read_to_string(directory.join(case["file"].as_str().unwrap())).unwrap(),
        )
        .unwrap();
        let rpc = solana_streamer_sdk::parse_encoded_rpc_transaction_as_streamer_events(
            &tx,
            0,
            &[Protocol::PumpFun],
            None,
        )
        .unwrap();
        let (meta, transaction) =
            solana_streamer_sdk::parser_sdk::convert_rpc_to_grpc(&tx).unwrap();
        let pretty = TransactionPretty {
            slot: tx.slot,
            grpc_tx: SubscribeUpdateTransactionInfo {
                signature: transaction.signatures[0].clone(),
                transaction: Some(transaction),
                meta: Some(meta),
                ..Default::default()
            },
            ..Default::default()
        };
        let grpc = parse_grpc_transaction_events(pretty, &[Protocol::PumpFun], None, None);
        for events in [rpc, grpc] {
            if case["failed"].as_bool().unwrap() {
                assert!(events.is_empty(), "{}", case["signature"]);
                continue;
            }
            let creates: Vec<_> = events
                .iter()
                .filter_map(|event| match event {
                    DexEvent::PumpFunCreateTokenEvent(c) => Some(c),
                    _ => None,
                })
                .collect();
            assert_eq!(creates.len(), 1, "{}", case["signature"]);
            let c = creates[0];
            assert_eq!(c.mint.to_string(), case["mint"].as_str().unwrap());
            assert_eq!(c.user.to_string(), case["user"].as_str().unwrap());
            assert_eq!(c.token_program.to_string(), case["token_program"].as_str().unwrap());
            assert!([
                "So11111111111111111111111111111111111111111",
                "So11111111111111111111111111111111111111112"
            ]
            .contains(&c.quote_mint.to_string().as_str()));
            assert_eq!(c.quote_vault.to_string(), "11111111111111111111111111111111");
        }
        if case["failed"].as_bool().unwrap() {
            failed += 1;
        } else {
            successful += 1;
        }
    }
    assert_eq!((successful, failed), (28, 8));
}
