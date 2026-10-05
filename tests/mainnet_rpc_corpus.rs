#[path = "support/rpc_corpus.rs"]
mod corpus;

#[test]
fn real_creator_fee_collection_survives_protocol_and_event_filters() {
    use solana_streamer_sdk::streaming::event_parser::common::{
        filter::EventTypeFilter, EventType, ProtocolType,
    };
    use solana_streamer_sdk::streaming::event_parser::Protocol;
    let tx = serde_json::from_str(&corpus::fixture("cpmm_collect")).unwrap();
    let parse = |protocols: &[Protocol], filter: Option<&EventTypeFilter>| {
        solana_streamer_sdk::parse_encoded_rpc_transaction_as_streamer_events(
            &tx, 0, protocols, filter,
        )
        .unwrap()
    };
    let include = EventTypeFilter::include_only([EventType::RaydiumCpmmCollectCreatorFee]);
    for protocols in [vec![], vec![Protocol::RaydiumCpmm]] {
        let events = parse(&protocols, Some(&include));
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].metadata().event_type, EventType::RaydiumCpmmCollectCreatorFee);
        assert_eq!(events[0].metadata().protocol, ProtocolType::RaydiumCpmm);
        let value = serde_json::from_str(&serde_json::to_string(&events).unwrap()).unwrap();
        assert_eq!(corpus::verify("cpmm_collect", &value, true), 1);
    }
    assert!(parse(&[Protocol::PumpSwap], Some(&include)).is_empty());
    let exclude = EventTypeFilter::exclude_only([EventType::RaydiumCpmmCollectCreatorFee]);
    assert!(parse(&[Protocol::RaydiumCpmm], Some(&exclude)).is_empty());
}

#[test]
fn independent_oracle_rejects_corrupted_real_transaction_events() {
    let tx = serde_json::from_str(&corpus::fixture("cpmm_0")).unwrap();
    let events =
        solana_streamer_sdk::parse_encoded_rpc_transaction_as_streamer_events(&tx, 0, &[], None)
            .unwrap();
    let original: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&events).unwrap()).unwrap();
    corpus::verify("cpmm_0", &original, true);
    let index = original
        .as_array()
        .unwrap()
        .iter()
        .position(|event| event.get("RaydiumCpmmSwapEvent").is_some())
        .unwrap();
    let mut bad_amount = original.clone();
    bad_amount[index]["RaydiumCpmmSwapEvent"]["output_transfer_fee"] = serde_json::json!(0);
    let mut duplicate = original.clone();
    duplicate.as_array_mut().unwrap().push(original[index].clone());
    let mut bad_account = original.clone();
    bad_account[index]["RaydiumCpmmSwapEvent"]["input_vault"] = serde_json::json!(vec![0; 32]);
    for corrupted in [bad_amount, duplicate, bad_account, serde_json::json!([])] {
        assert!(std::panic::catch_unwind(|| corpus::verify("cpmm_0", &corrupted, true)).is_err());
    }
}

#[test]
fn real_rpc_corpus_preserves_accounts_limits_execution_and_token_fees() {
    let mut checked = 0;
    for name in corpus::CASES {
        let tx = serde_json::from_str(&corpus::fixture(name)).unwrap();
        let events = solana_streamer_sdk::parse_encoded_rpc_transaction_as_streamer_events(
            &tx,
            0,
            &[],
            None,
        )
        .unwrap();
        let events = serde_json::from_str(&serde_json::to_string(&events).unwrap()).unwrap();
        checked += corpus::verify(name, &events, true);
    }
    assert_eq!(checked, 17);
}

#[test]
fn failed_real_rpc_preserves_cost_only_and_combined_selection() {
    use solana_streamer_sdk::streaming::event_parser::common::{
        filter::EventTypeFilter, EventType,
    };
    use solana_streamer_sdk::streaming::event_parser::{DexEvent, Protocol};
    let tx: solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta =
        serde_json::from_str(&corpus::fixture("failed_route")).unwrap();
    assert!(tx.transaction.meta.as_ref().unwrap().err.is_some());
    let parse = |filter: Option<&EventTypeFilter>| {
        solana_streamer_sdk::parse_encoded_rpc_transaction_as_streamer_events(
            &tx,
            0,
            &[Protocol::PumpSwap],
            filter,
        )
        .unwrap()
    };
    assert!(parse(None).is_empty());
    for filter in [
        EventTypeFilter::include_only([EventType::TransactionCost]),
        EventTypeFilter::include_only([EventType::PumpSwapSell, EventType::TransactionCost]),
    ] {
        let events = parse(Some(&filter));
        assert_eq!(events.len(), 1);
        let DexEvent::TransactionCostEvent(cost) = &events[0] else { panic!("expected cost only") };
        assert_eq!(cost.transaction_fee_lamports, tx.transaction.meta.as_ref().map(|m| m.fee));
    }
}

#[allow(dead_code)]
#[path = "../examples/support/rpc_capture.rs"]
mod capture;

#[test]
fn captured_base64_and_json_encodings_agree_and_reject_wire_corruption() {
    for name in corpus::CASES {
        let tx = serde_json::from_str(&corpus::fixture(name)).unwrap();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("tests/fixtures/rpc_corpus/{name}_wire.json"));
        let wire: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        capture::verify_wire(&tx, &wire).unwrap();
        let mut bad_slot = wire.clone();
        bad_slot["slot"] = serde_json::json!(0);
        let mut bad_key = wire.clone();
        bad_key["transaction"]["message"]["accountKeys"][0] =
            serde_json::json!("11111111111111111111111111111111");
        let mut bad_data = wire.clone();
        bad_data["transaction"]["message"]["instructions"][0]["data"] = serde_json::json!("1");
        let mut bad_loaded_keys = wire.clone();
        bad_loaded_keys["meta"]["loadedAddresses"] =
            serde_json::json!({"writable":[],"readonly":[]});
        for corrupted in [bad_slot, bad_key, bad_data] {
            assert!(capture::verify_wire(&tx, &corrupted).is_err(), "{name}: corruption accepted");
        }
        if bad_loaded_keys["meta"]["loadedAddresses"] != wire["meta"]["loadedAddresses"] {
            assert!(capture::verify_wire(&tx, &bad_loaded_keys).is_err());
        }
    }
}

#[test]
fn real_orca_price_limit_preserves_full_u128_precision() {
    let wire_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/rpc_corpus/cpmm_0_wire.json");
    let wire: serde_json::Value =
        serde_json::from_slice(&std::fs::read(wire_path).unwrap()).unwrap();
    let mut keys = wire["transaction"]["message"]["accountKeys"].as_array().unwrap().clone();
    keys.extend(wire["meta"]["loadedAddresses"]["writable"].as_array().unwrap().iter().cloned());
    keys.extend(wire["meta"]["loadedAddresses"]["readonly"].as_array().unwrap().iter().cloned());
    let raw = wire["meta"]["innerInstructions"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|group| group["instructions"].as_array().unwrap())
        .find(|ix| {
            keys[ix["programIdIndex"].as_u64().unwrap() as usize]
                == "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc"
        })
        .unwrap();
    let data = solana_sdk::bs58::decode(raw["data"].as_str().unwrap()).into_vec().unwrap();
    let expected = u128::from_le_bytes(data[24..40].try_into().unwrap());
    assert!(expected > u128::from(u64::MAX));
    let tx = serde_json::from_str(&corpus::fixture("cpmm_0")).unwrap();
    let events =
        solana_streamer_sdk::parse_encoded_rpc_transaction_as_streamer_events(&tx, 0, &[], None)
            .unwrap();
    let event = events
        .iter()
        .find_map(|event| match event {
            solana_streamer_sdk::streaming::event_parser::DexEvent::OrcaWhirlpoolSwapEvent(
                swap,
            ) => Some(swap),
            _ => None,
        })
        .unwrap();
    assert_eq!(event.sqrt_price_limit, expected);
}

#[test]
fn real_clmm_instruction_accounts_survive_missing_logs() {
    let mut payload: serde_json::Value =
        serde_json::from_str(&corpus::fixture("dlmm_route")).unwrap();
    payload["meta"]["logMessages"] = serde_json::json!([]);
    let tx = serde_json::from_value(payload).unwrap();
    let events =
        solana_streamer_sdk::parse_encoded_rpc_transaction_as_streamer_events(&tx, 0, &[], None)
            .unwrap();
    let event = events
        .iter()
        .find_map(|event| match event {
            solana_streamer_sdk::streaming::event_parser::DexEvent::RaydiumClmmSwapEvent(swap) => {
                Some(swap)
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(
        event.input_token_account,
        "7tZ1fHSRmwbBQXT3fgNrzQNFufJx36yBJswgpTzyKTHs".parse().unwrap()
    );
    assert_eq!(
        event.output_token_account,
        "2mi8e3FM7iAqAKnLFJ3TXabPTm9ZrWB58GGpoqiVszg2".parse().unwrap()
    );
    assert!(event.is_base_input);
    assert_eq!(event.other_amount_threshold, 0);
    assert_eq!(event.sqrt_price_limit_x64, 0);
    assert_eq!(event.ix_name, "swap");
}

#[test]
fn discovery_matches_wire_program_and_discriminator_without_logs() {
    let load = |name: &str| -> serde_json::Value {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("tests/fixtures/rpc_corpus/{name}_wire.json"));
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
    };
    for (name, program, disc) in [
        (
            "cpmm_0",
            "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C",
            [143, 190, 90, 218, 196, 30, 51, 222],
        ),
        (
            "cpmm_collect",
            "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C",
            [20, 22, 86, 123, 198, 28, 219, 132],
        ),
        (
            "pumpfun_sell_v2",
            "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P",
            [93, 246, 130, 60, 231, 233, 64, 178],
        ),
    ] {
        let mut wire = load(name);
        wire["meta"]["logMessages"] = serde_json::Value::Null;
        assert!(capture::matches_instruction(&wire, program, &disc).unwrap());
        assert!(!capture::matches_instruction(&wire, "11111111111111111111111111111111", &disc)
            .unwrap());
        assert!(!capture::matches_instruction(&wire, program, &[0; 8]).unwrap());
        wire["transaction"]["message"]["instructions"][0]["programIdIndex"] =
            serde_json::json!(999999);
        assert!(capture::matches_instruction(&wire, program, &disc).is_err());
    }
    let mut wire = load("dlmm_0");
    wire["meta"]["logMessages"] = serde_json::json!(["Program log: Instruction: Swap"]);
    assert!(!capture::matches_instruction(
        &wire,
        "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo",
        &[248, 198, 158, 145, 225, 117, 135, 200]
    )
    .unwrap());
}
