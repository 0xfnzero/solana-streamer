use solana_streamer_sdk::streaming::{
    common::event_processor::parse_grpc_transaction_events,
    event_parser::{
        common::{filter::EventTypeFilter, EventType},
        DexEvent, Protocol,
    },
    grpc::TransactionPretty,
};
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;
use yellowstone_grpc_proto::prelude::SubscribeUpdateTransactionInfo;

fn replay(fixture: &str, event_type: EventType) -> [Vec<DexEvent>; 2] {
    let tx: EncodedConfirmedTransactionWithStatusMeta = serde_json::from_str(fixture).unwrap();
    let filter = EventTypeFilter::include_only([event_type]);
    let rpc = solana_streamer_sdk::parse_encoded_rpc_transaction_as_streamer_events(
        &tx,
        0,
        &[Protocol::PumpSwap],
        Some(&filter),
    )
    .unwrap();
    let (meta, transaction) = solana_streamer_sdk::parser_sdk::convert_rpc_to_grpc(&tx).unwrap();
    let pretty = TransactionPretty {
        slot: tx.slot,
        block_time: tx.block_time.map(|seconds| prost_types::Timestamp { seconds, nanos: 0 }),
        grpc_tx: SubscribeUpdateTransactionInfo {
            signature: transaction.signatures[0].clone(),
            transaction: Some(transaction),
            meta: Some(meta),
            ..Default::default()
        },
        ..Default::default()
    };
    [rpc, parse_grpc_transaction_events(pretty, &[Protocol::PumpSwap], Some(&filter), None)]
}

#[test]
fn boost_vaults_and_token_programs_survive_rpc_and_grpc_bridge() {
    for events in
        replay(include_str!("fixtures/pumpswap_boost_buy_and_burn.json"), EventType::PumpSwapBuy)
    {
        assert_eq!(events.len(), 1);
        let DexEvent::PumpSwapBuyEvent(buy) = &events[0] else { panic!("expected boost buy") };
        for (actual, expected) in [
            (buy.pool, "E5WvAEhX1LKCtCz9BuzBZ4zwEzt4kYS1Du4jxGK1R3Lc"),
            (buy.user, "9SNoU8GVBT7VTg7wfyTbtrgPb53fQTDYmg88EMaGjUQh"),
            (buy.base_mint, "7rmPZgtU22AN1W3kHy8ASw76v46fp8kLgzadnPcEpump"),
            (buy.pool_base_token_account, "KbW4EmdCjfbJefaTDJJ6ww2Gk4Z7nseeVN9KaKZNueE"),
            (buy.pool_quote_token_account, "3Qia2MrJcBaF5ttA4XjqEcoZUr53g58XxTmJopHbMQJG"),
            (buy.base_token_program, "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"),
            (buy.quote_token_program, "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
        ] {
            assert_eq!(actual.to_string(), expected);
        }
    }
}

#[test]
fn precompile_sell_mint_survives_rpc_and_grpc_bridge() {
    for events in
        replay(include_str!("fixtures/pumpswap_precompile_sell.json"), EventType::PumpSwapSell)
    {
        let sells: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                DexEvent::PumpSwapSellEvent(sell)
                    if sell.pool.to_string() == "2NtHe5gwQ1grT3Y89Kf3dRZyWDnTVYk4f4xV9WxuZJBZ" =>
                {
                    Some(sell)
                }
                _ => None,
            })
            .collect();
        assert_eq!(sells.len(), 1);
        assert_eq!(sells[0].base_mint.to_string(), "7hLzF2ahWJKwRNrW4jGSteWBGWJ2eVMBi6BJFvaquant");
    }
}
