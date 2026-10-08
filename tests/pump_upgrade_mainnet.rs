use serde_json::Value;
use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;

fn normalize(value: &mut Value) {
    match value {
        Value::Array(a)
            if a.len() == 32 && a.iter().all(|n| n.as_u64().is_some_and(|n| n <= 255)) =>
        {
            let mut bytes = [0; 32];
            for (i, n) in a.iter().enumerate() {
                bytes[i] = n.as_u64().unwrap() as u8;
            }
            *value = Pubkey::new_from_array(bytes).to_string().into();
        }
        Value::Array(a) => a.iter_mut().for_each(normalize),
        Value::Object(o) => o.values_mut().for_each(normalize),
        _ => {}
    }
}

#[test]
fn captured_mainnet_matches_official_idl_and_wire() {
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/pump_upgrade/mainnet.json")).unwrap();
    let simulated: Value =
        serde_json::from_str(include_str!("fixtures/pump_upgrade/simulation_events.json")).unwrap();
    for case in
        fixture["cases"].as_array().unwrap().iter().chain(simulated["cases"].as_array().unwrap())
    {
        let tx: EncodedConfirmedTransactionWithStatusMeta =
            serde_json::from_value(case["encoded"].clone()).unwrap();
        let parsed = solana_streamer_sdk::parse_encoded_rpc_transaction_as_streamer_events(
            &tx,
            0,
            &[],
            None,
        )
        .unwrap();
        if !case["raw"]["meta"]["err"].is_null() {
            assert!(parsed.is_empty());
            continue;
        }
        let mut events: Value =
            serde_json::from_str(&serde_json::to_string(&parsed).unwrap()).unwrap();
        normalize(&mut events);
        let bodies: Vec<_> = events
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|e| e.as_object().unwrap().values())
            .collect();
        for expected in case["expected"].as_array().unwrap() {
            let d = &expected["data"];
            let identity = if expected["name"] == "TradeEvent" { "mint" } else { "pool" };
            let matches: Vec<_> = bodies
                .iter()
                .copied()
                .filter(|a| {
                    a[identity] == d[identity]
                        && a["user"] == d["user"]
                        && (d["is_buy"].is_null() || a["is_buy"] == d["is_buy"])
                        && (expected["name"] != "BuyEvent" || !a["base_amount_out"].is_null())
                        && (expected["name"] != "SellEvent" || !a["base_amount_in"].is_null())
                })
                .collect();
            assert_eq!(matches.len(), 1, "{}", case["signature"]);
            for (field, value) in d.as_object().unwrap() {
                if field == "shareholders" {
                    continue;
                }
                let got = &matches[0][field];
                if field == "quote_mint" && value == "11111111111111111111111111111111" {
                    assert!([
                        "11111111111111111111111111111111",
                        "So11111111111111111111111111111111111111111",
                        "So11111111111111111111111111111111111111112"
                    ]
                    .contains(&got.as_str().unwrap()));
                } else {
                    let text =
                        |v: &Value| v.as_str().map(str::to_owned).unwrap_or_else(|| v.to_string());
                    assert_eq!(text(got), text(value), "{} {field}", case["signature"]);
                }
            }
        }
    }
}
