# solana-streamer v5.0.1

Released 2026-10-26.

Use sol-parser-sdk 0.10.0 and preserve create_v2 method names, unknown quotes and official multi-hop Pump callbacks across the streamer bridge. Update real-transaction replay examples.

升级解析核心到 0.10.0，事件桥接保留 create_v2 指令名、未知报价和官方多跳回调，更新真实交易回放示例。

## Validation / 验证

Native-language regression suites and replay/build checks passed for the synchronized changes. Four trade implementations completed curve buy/sell and pool buy/sell unsigned mainnet simulations (16 successful simulations); these small-amount interface checks do not replace fresh production quotes. No transaction was broadcast.
