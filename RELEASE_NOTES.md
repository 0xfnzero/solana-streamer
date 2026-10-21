# solana-streamer v5.0.0

Released 2026-10-22. The Rust, Node.js, Python and Go parser packages use the same family version, 0.9.0. Streamer 5.0.0 and Shred 6.0.1 are separately versioned integrations.

## Default stock pools / 默认股票代币池子

Remove five pools from the default catalog: two have captured single-transaction withdrawals reducing both vault balances by at least 50%; four have less than $500 in USDC/WSOL-side reserves at the 2026-10-22 04:25 UTC+8 snapshot (one meets both criteria). Defaults now contain 58 stock pools across 33 mints; trading adds the SOL/USDC conversion pool for 59 pools total. COPX.US is no longer a default mint. See [the synchronized catalog notes](docs/STOCK_BRIDGE_SYNC.md) for addresses and evidence.

This is a static catalog refresh, not continuous liquidity monitoring, certified executable depth, or an LP-lock guarantee. Explicit custom catalogs remain supported. No RPC or history scanning was added to parsing or trading hot paths.

中文：默认目录剔除 5 个池子，涵盖整池双边储备单笔下降至少 50% 的历史记录，以及快照中 USDC/WSOL 一侧储备不足 500 美元的池子。保留 58 个股票池、33 个 mint；交易侧加 SOL/USDC 共 59 个池子。该目录为静态快照，不保证实时深度或流动性锁定。

## Changes / 更新

- Remove standalone PumpRouteIntent and RawInstruction callbacks and their obsolete decoding helpers/examples. Wrapped Pump.fun callbacks retain the original outer program, ordered accounts and data in third_program.
- Preserve the distinction between unresolved Shred candidates, decoded instruction inputs and confirmed execution. The parser hot path performs no RPC and does not infer opaque third-party arguments.
- Synchronize Pump.fun V3 instruction handling, gRPC execution evidence and create-before-trade ordering. Preserve supplied transaction-position and receive-time metadata.
- Clean up examples and README navigation; maintain the shared stock catalog and native-language validation.

## Migration / 升级

Subscribe to PumpFunTrade and inspect third_program and parse_status. Remove matches/filters for the retired PumpRouteIntent and RawInstruction events. Unresolved is only a possible interaction; defaults are not a proven direction or zero fill. ExecutionConfirmed does not guarantee all amounts are available. Unresolved ALT addresses remain unknown.

中文：独立的路由意图和原始指令回调已删除。第三方包装信息保留在交易事件的 third_program 中；Shred 候选与实际成交必须按 parse_status 区分。SDK 不猜测第三方合约参数，热路径不调用 RPC。

Version alignment does not imply identical APIs across languages. Offline fixtures validate captured cases, not current chain state or production latency. See README and docs/wrapped-pumpfun.md for the supported surface.

Pins sol-parser-sdk =0.9.1 and forwards the simplified callback model. Update exhaustive streamer event matches when upgrading.
