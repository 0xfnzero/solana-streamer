use solana_streamer_sdk::{
    parse_encoded_rpc_transaction_as_streamer_events, streaming::event_parser::DexEvent,
};
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;

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
