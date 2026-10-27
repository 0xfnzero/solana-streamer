# Wrapped Pump.fun transactions

The streamer preserves `PumpFunTradeEvent.third_program` and transaction metadata from sol-parser-sdk. Subscribe to the PumpFunTrade family
to receive unresolved Shred candidates as well as gRPC trades.

Shred candidates only establish that an unknown outer instruction references
Pump.fun. They do not prove an executed trade, buy/sell direction or amount.
Direction-specific subscriptions exclude candidates. gRPC uses actual execution
metadata and CPI instructions, retaining the original outer instruction on
wrapped trades; native direct calls have no third-party payload.

`third_program` preserves the outer program, instruction index, ordered accounts,
account indices and unchanged instruction data. Metadata stays on the event.
No RPC is added to the parsing hot path.

The route-intent and raw-event variants, bridge conversions and subscription
filters have been removed together with the corresponding parser SDK interfaces.
Wrapper interpretation belongs to the caller. Existing supported native DEX
events and their gRPC provenance remain unchanged.

See the parser SDK's [callback and migration contract](https://github.com/0xfnzero/sol-parser-sdk/blob/main/docs/wrapped-pumpfun.md).
The focused offline regression lives in
`sol-parser-sdk/src/wrapped_pumpfun_rpc_analyzer.rs`; run
`cargo test --offline --lib wrapped_pumpfun_rpc_analyzer` in that repository.
It uses saved signed transactions, not a live Shred connection.
