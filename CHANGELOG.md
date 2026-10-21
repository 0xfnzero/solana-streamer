# Changelog

## v5.0.0 — 2026-10-22

Synchronize SDK code, examples, version navigation and AI release rules. Read the [release and migration notes](RELEASE_NOTES.md) for callback removals, V3 defaults and the Go /v9 migration.

Release and integration notes preserved from the previous README. APIs, dependencies and validation results describe those releases, not the current contract. See the [README](README.md) and [release and migration notes](RELEASE_NOTES.md) for current usage.

## v4.0.0

See [release and migration notes](RELEASE_NOTES.md).

## v3.0.17 — Wrapped Pump.fun provenance and parser fixes

Updates sol-parser-sdk to 0.7.19 and preserves optional `third_program` provenance through the Pump.fun event bridge. Includes mint-scoped create-buy context and versioned IDL method-name fixes. Third-party rules remain explicit opt-in; the parsing hot path performs no RPC.

[Wrapped Pump.fun: offline rules, hot-path boundaries, and real-transaction replays](docs/wrapped-pumpfun.md)

## v3.0.16 — Shred metadata passthrough and ordering scope

Preserve and verify transaction position and receive time through raw instruction callbacks. Entries indexes are message-local, unlike Yellowstone node indexes; use signatures for cross-message deduplication. This release clarifies API documentation and adds regression protection without changing metadata fields or numbering behavior.

## v3.0.16 — DEX-scoped RawInstruction and typed DEX identifiers

Raw instructions now respect DEX subscriptions using programs referenced by each instruction. `RawInstructionDexType` provides typed identifiers with unchanged JSON strings. Unknown router layouts do not establish pool addresses, trade direction, or method versions.

## v3.0.16 — Pump execution replay

Current Anchor TradeEvent CPI decoding is supported alongside the historical encoding. RPC/gRPC instruction budgets and minimum outputs remain in instruction fields. Execution fields `sol_amount`, `quote_amount`, and `token_amount` come from execution events; without logs or event CPIs they remain zero (unavailable, not a proven zero fill).

Run `cargo run --offline --example rpc_transaction_replay` from this repository to replay real transactions fetched on 2026-10-19 offline. The default case is Delta SellV2. Fetch another using `--fetch SIGNATURE OUTPUT.json`; replay with `--fixture OUTPUT.json`. Set `SOLANA_RPC_URL` for your RPC; transaction v1 is supported.

The current Delta sell signature is `4ARos3Yq1SMNE3izDpy9YdKrRgMmD7UCDxvnVrBRaGCJHiifPHSmQWc4DPfjUaEUcxS7puGmEN64K7EZdeXLJaj2` (slot 458542546).  Older five-combination multi-hop fixtures are retained as regression tests, not labeled as new mainnet captures.

## v3.0.16 — raw Shred instructions

Protocol filters apply to raw DEX associations as well as decoded events.

Default Shred parsing emits `RawInstruction` for unsupported third-party programs, including Rapid/Delta, without inferring direction, amounts, or internal buy methods. Infrastructure programs (System, Token, ATA, ComputeBudget, etc.) are omitted by default. Supported official DEX parsers are unchanged; official Pump/PumpSwap multi-hop instructions retain `PumpRouteIntent`.

The payload contains `program_id`, `instruction_index`, `accounts`, `account_indices`, `instruction_data`, and transaction metadata. Account order and duplicates are preserved. Unresolved ALT addresses are null/None, with original indices retained; supply cached loaded addresses or a resolver. No hot-path RPC is performed. Data is copied verbatim without requiring a known layout.

The raw event retains `metadata.tx_index` and `metadata.grpc_recv_us` from the parser arguments (streamer names the receive field `metadata.recv_us`). The default Entries client index is local to one received Entries message and restarts at zero for the next message; it is not a confirmed slot-wide transaction index. `instruction_index` orders outer instructions within a transaction. Receive time is a local microsecond timestamp, not an execution-order guarantee. Standalone replay APIs may default these values to zero; pass the upstream values when available.

Without a filter, unknown third-party instructions are returned. Broad trade/swap filters (for example `PumpFunTrade` or `RaydiumCpmmSwap`) also return raw instructions associated with that DEX; direction-specific filters do not infer a buy or sell. Explicit `RawInstruction` selects all outer instructions only when no protocol restriction is set; streamer protocol restrictions always apply. Excluding `RawInstruction` wins. `dex_type` uses the public `RawInstructionDexType` enum (typed string constants in Go); JSON values remain unchanged, such as `"pumpfun"` and `"raydium_cpmm"`. `dexes` contains `{ dex_type, program_id, pool }` for each recognized DEX program targeted or referenced by this instruction, not other instructions in the transaction. Multi-DEX wrappers retain every association. Unknown or unresolved ALT references cannot satisfy a DEX subscription. `pool` is currently null: an unknown wrapper layout cannot establish the pool account. A program reference is not proof of an executed CPI. Raw callbacks are not fills.

Wrapper adapter functions remain explicit compatibility helpers only; default Shred parsing does not call them. The released-version history below describes previous behavior.

## 🔄 Migration Guide

## v3.0.13 — Pump route event support

Route intents expose `is_buy` and `ix_name`, using existing trade field names and official IDL names. RapidBuy reports a buy intent and `buy_exact_quote_in_v2`; BuyPumpDelta reports only a buy intent. Multi-hop direction and unknown methods remain null. Currencies remain in `hops[].base_mint` / `quote_mint`. These hints do not confirm execution; filtering does not convert intents into trades.

Streamer forwards complete route payloads as `DexEvent::PumpRouteIntentEvent` with protocol/event filtering. Old JSON defaults missing hints to unknown. Rust struct literals need the new fields; exhaustive streamer event matches need the new variant.

## v3.0.12 — Pump routes and offline replay

Pins sol-parser-sdk 0.7.14 and includes the latest streaming facade code and native hot-path benchmark. The raw parser re-export provides the new Pump route-intent and cached-ALT APIs.

Route intents describe instructions before execution; use execution metadata for confirmed fills. Offline CPU timings do not measure production transport or transaction landing.

## v3.0.11 — Pump depth-2 compatibility

Pins sol-parser-sdk 0.7.13 and inherits signed Pump depth-2 create-event compatibility coverage through the parser bridge. Existing trading and event interfaces remain unchanged.

Validation includes official Pump SDK 4.1.0 instruction comparisons, signed local LiteSVM simulations and SDK regressions. Local simulation state/config is synthetic; no funded transactions were broadcast and production latency was not measured.

### v3.0.10

Pins `sol-parser-sdk =0.7.12` and inherits the PumpFun migration event CPI parsing fix. Publishes the current bridge support for compact Pump trades, quote-coin fields, completion and fee-sweep events, protocol fee fields and synthetic migration handling. Includes captured mainnet compact-trade regression fixtures.

Validation covers the streaming facade, parser bridge and offline transaction fixtures. Network transaction-landing latency is not an SLA.

### v3.0.9

- Pins `sol-parser-sdk 0.7.10` and inherits creation account matching, quote-account and historical-layout fixes, verified through the RPC facade and gRPC bridge.

### v3.0.8

Pins `sol-parser-sdk 0.7.9` and inherits PumpSwap boost account backfills and precompile log-index fixes. Preserves strict trade matching and ALT account handling. Adds offline mainnet regressions verifying pool vaults, token programs and sell mints through both the RPC facade and gRPC event bridge.

### v3.0.7

Uses `sol-parser-sdk 0.7.8`. Preserves CPMM creator-fee collection events, the AmmConfig share rate, swap instruction arguments and Orca/CLMM accounts through the bridge. Includes independent RPC corpus validation for 13 mainnet transactions and 17 target operations. Run `cargo run --example rpc_corpus_validate -- --list` to view signatures.

### Upgrading to v3.0.6

Version 3.0.6 pins `sol-parser-sdk 0.7.7`. Advanced callers can access StonkFun route analysis, migration registry and raw account subscriptions through `parser_sdk` / `sdk_bridge::raw`. Raw snapshots retain their parser SDK type and should be consumed directly rather than passed through the legacy streamer event adapter. Bridge regression fixtures are synchronized with the current Whirlpool/DLMM account fields.

### Upgrading to v3.0.5

Version 3.0.5 uses `sol-parser-sdk 0.7.5` and forwards the recovered Meteora DAMM v2 swap instruction accounts, including token mints, vaults, payer, token programs, and optional referral. Live mainnet regressions cover both swaps reported in issue #82.

### Upgrading to v3.0.4

Version 3.0.4 uses `sol-parser-sdk 0.7.4`, adds the user-facing `Protocol::LaunchLab`, `Protocol::StonkFun`, `ProtocolType::LaunchLab`, and `ProtocolType::StonkFun` identifiers, and filters shared LaunchLab traffic by StonkFun's official platform configuration accounts. LaunchLab trade events now forward complete reserves, fee legs, pool status, and the current three appended accounts. `Protocol::RaydiumLaunchpad` remains a source-compatible alias and displays as `LaunchLab`.

### Upgrading to v3.0.3

Version 3.0.3 uses `sol-parser-sdk 0.7.3` and synchronizes the current PumpFun, PumpSwap, and Pump Fees protocol schemas. PumpFun create/trade events and PumpSwap buy/sell/create-pool events now preserve creator-fee and holder-reward fields through the streamer bridge. PumpFun bonding-curve and PumpSwap pool account events also expose the new creator-fee editability and holder-reward state while retaining defaults for legacy accounts and JSON payloads. The parser rejects truncated current PumpSwap trade and CreatePool tails instead of returning partially populated events.

### Upgrading to v3.0.2

Version 3.0.2 uses `sol-parser-sdk 0.7.2` and forwards the new Meteora DAMM v2 events: `UpdateDelegatePermission`, `WithdrawDeadLiquidityReward`, `CreateConfig`, and `CreateDynamicConfig`. Transaction-cost events preserve all four Solana V1 config requests, and Yellowstone V1 messages are distinguished from V0 through `Message.config`. V1 ingestion requires a Yellowstone geyser server plugin version of at least `15.1.1`; older servers silently discard the config while downgrading V1 to V0.

### Upgrading to v3.0.1

Version 3.0.1 uses `sol-parser-sdk 0.7.1` and fixes current Meteora DAMM v2 Swap and AddLiquidity delivery. The bridge now preserves all current swap fee, transfer-fee, reserve, mode, and liquidity fields; exact AddLiquidity filtering follows the unified on-chain `EvtLiquidityChange` event. Streamer no longer maintains a separate DAMM swap payload decoder, keeping `sol-parser-sdk` as the single parsing implementation. Mainnet RPC regression tests cover both reported event types.

### Upgrading to v3.0.0

Version 3.0.0 uses `sol-parser-sdk 0.7.0` and Solana 4 public types. ShredStream Entry decoding now uses `wincode 0.5.5` in production and supports Solana V1 transactions. The normal dependency graph dropped from 677 to 544 package/version entries, while duplicated crate names dropped from 119 to 30 by removing redundant direct Solana/SPL dependencies and aligning versions with the parser. This release requires Rust 1.91 or newer and also forwards Meteora DLMM user token input/output accounts.

### Upgrading to v2.0.3

Version 2.0.3 uses `sol-parser-sdk 0.6.6` and pins `spl-pod 0.7.2` to keep fresh, lock-free builds compatible with `spl-list-view 0.1.0`. PumpFun trade events preserve the parser SDK's optional pre/post token balances in raw mint units and pre/post SOL balances in lamports, populated from transaction metadata without additional RPC calls. Meteora DLMM swaps also preserve `min_amount_out` separately from the executed `amount_out`.

### Upgrading to v2.0.1

Version 2.0.1 uses `sol-parser-sdk 0.6.1` from crates.io. PumpSwap Buy/Sell events now preserve the complete current cashback, buyback, signed virtual quote reserve, boost, and base-supply tail through dispatcher and SDK bridge paths. Truncated current layouts are rejected consistently by the default and zero-copy parser backends while historical layouts remain supported.

### Upgrading to v1.5.16

Version 1.5.16 fixes `StreamingOrdered` delivery for transactions that parse into multiple events. Events from the same `(slot, tx_index)` are admitted as a group, the streaming watermark advances once per transaction, and stable sorting preserves parser order for equal transaction indexes. It also pins `yellowstone-grpc-proto` to the `12.4.x` API used by `sol-parser-sdk 0.5.15` so CI and clean installs do not resolve an incompatible newer proto crate.

### Upgrading to v1.5.15

Version 1.5.15 tracks `sol-parser-sdk 0.5.15` at GitHub rev `36ec202`. Pump.fun `create_v2` now distinguishes 16-account SOL-sentinel creates from 19/20-account quote-pool creates, models Yellowstone gRPC loaded writable/readonly addresses correctly, rejects PumpFun program-account fillers as quote mints, and keeps ShredStream from guessing ALT-loaded quote mints when the static transaction keys do not contain them. gRPC/RPC inner instruction parsing now handles guarded 8-byte ordinary CPI instructions for all SDK-supported DEX parsers, so PumpSwap `create_pool` reads `is_cashback_coin` from instruction args before merging with log-derived `CreatePoolEvent` output. ShredStream remains outer-instruction only, but its outer discriminator gate is shared with the SDK parser and no longer diverges for supported DEX protocols.

### Upgrading to v1.5.14

Version 1.5.14 uses `sol-parser-sdk 0.5.14` from crates.io. Pump.fun canonical create events now expose `quote_mint`, `quote_vault`, and `quote_token_program` for `create_v2` quote pools, including USDC pools, across gRPC/RPC parser bridge and ShredStream-backed SDK output.

### Upgrading to v1.5.13

Version 1.5.13 uses `sol-parser-sdk 0.5.13` from crates.io. Pump.fun gRPC and ShredStream create/trade outputs now preserve real WSOL quote mints (`So11111111111111111111111111111111111111112`). The Solscan SOL sentinel (`So11111111111111111111111111111111111111111`) is used only when legacy data omits a quote mint.

### Upgrading to v1.5.11

Version 1.5.11 uses `sol-parser-sdk 0.5.11` from crates.io. PumpSwap `PumpSwapCreatePoolEvent` now carries `is_cashback_coin` when it is available from the `create_pool` instruction args, including ShredStream outer-instruction parsing. Log-only `CreatePoolEvent` payloads still default this field to `false` because the on-chain log event IDL does not carry it. `AccountPumpSwapPool` remains the authoritative account-state source for the pool flag.

### Upgrading to v1.5.10

Version 1.5.10 uses `sol-parser-sdk 0.5.10` from crates.io. PumpSwap `CreatePoolEvent` now matches the on-chain IDL: it exposes `is_mayhem_mode` but does not expose `is_cashback_coin`. To read the cashback flag, subscribe to `AccountPumpSwapPool` and use `PumpSwapPoolAccountEvent.pool.is_cashback_coin`. The PumpSwap CreatePool log payload length check now includes the final `is_mayhem_mode` byte.

### Upgrading to v1.5.9

Version 1.5.9 uses `sol-parser-sdk 0.5.9` from crates.io. It inherits the Yellowstone gRPC stop lifecycle fix: `stop()` now signals, aborts, and awaits the active subscription task, subscription lifecycle transitions are serialized, and gRPC stream errors are labeled as `Grpc Stream error` for easier log separation from ShredStream.

### Upgrading to v1.5.8

Version 1.5.8 uses `sol-parser-sdk 0.5.8` from crates.io. It inherits the Pump.fun ShredStream filter-family semantics from the parser SDK: `PumpFunBuy` covers `buy`, `buy_v2`, `buy_exact_sol_in`, and `buy_exact_quote_in_v2`; `PumpFunSell` covers `sell` and `sell_v2`; and `PumpFunTrade` covers all buy/sell instructions while the parser can emit unified trade events when only the generic trade filter is requested.

### Upgrading to v1.5.5

Version 1.5.5 uses `sol-parser-sdk 0.5.5` from crates.io. The SDK exposes LaunchLab as `RaydiumLaunchlab*`; streamer keeps the existing `Bonk*` event structs and `Protocol::Bonk` / `Protocol::RaydiumLaunchpad` aliases for source compatibility, while routing parser calls and upstream gRPC event filters to the new LaunchLab SDK variants. This release also syncs the CLMM/CPMM/Orca account bridges, Meteora DAMM v2 initialize-pool events, Meteora DBC events, and parser warmup on client creation.

### Upgrading to v1.5.4

Version 1.5.4 uses `sol-parser-sdk 0.5.4` from crates.io. Pump.fun `create` and `create_v2` are delivered as one canonical `PumpFunCreateTokenEvent`; subscribing to either `PumpFunCreateToken` or `PumpFunCreateV2Token` receives the same complete create-family data. This prevents duplicate new-mint callbacks from gRPC log + instruction parsing while preserving create_v2 account fields on the canonical create event.

### Upgrading to v1.5.3

Version 1.5.3 uses `sol-parser-sdk 0.5.3` from crates.io. It preserves real Pump.fun v2 `ix_name` values through the streamer bridge, improves ShredStream Pump.fun v2 best-effort parsing for short account lists, and treats `PumpFunBuy` and `PumpFunBuyExactSolIn` subscriptions as compatible buy-family filters. ShredStream still uses the direct entry-reading path with automatic reconnect; callbacks should remain non-blocking and `tx_index` remains entry-local best-effort.

### Upgrading to v1.5.2

Version 1.5.2 uses `sol-parser-sdk 0.5.2` from crates.io. ShredStream delivery uses the SDK parser on a direct entry-reading path with automatic reconnect, avoiding the extra queue consumer task while keeping parser event buffers reused. ShredStream `tx_index` is entry-local best-effort, not the Yellowstone slot-level transaction index. User callbacks on the direct path run on the read task and should avoid blocking work.

### Upgrading to v1.5.1

Version 1.5.1 uses `sol-parser-sdk 0.5.1` from crates.io. It improves ShredStream ALT handling by parsing outer instructions best-effort with default placeholders for ALT-loaded accounts, adds dropped-event queue observability, and keeps CPI/inner-only events documented as a ShredStream limitation.

### Upgrading to v1.5.0

Version 1.5.0 uses `sol-parser-sdk 0.5.0`, wires Raydium CLMM account parsing through the SDK bridge, and reduces ordered-buffer allocation/move overhead in low-latency delivery paths. `sol-parser-sdk 0.4.19` and `solana-streamer-sdk 1.4.14` were superseded because adding public parser event variants belongs in the 0.5 line.

### Upgrading to v1.4.14

Version 1.4.14 was superseded by v1.5.0.

### Upgrading to v1.4.13

Version 1.4.13 uses `sol-parser-sdk 0.4.18` and updates Raydium CLMM integration to the official upgraded IDL: current log-side event discriminators, official Swap/Liquidity/Create/Collect layouts, limit-order events, dynamic-fee related events, and the reshaped PoolState/TickState account structs.

### Upgrading to v1.4.12

Version 1.4.12 uses `sol-parser-sdk 0.4.17` and normalizes legacy PumpFun SOL quote mints to the Solscan SOL sentinel while preserving real USDC quote mints and quote-reserve fields. Streamer bridge and merger paths keep the parser SDK semantics aligned for Yellowstone gRPC and ShredStream output.

New optional capabilities:

- `solana_streamer_sdk::parser_sdk` re-exports the raw `sol-parser-sdk` crate.
- `solana_streamer_sdk::sdk_bridge` adapts raw SDK events back into streamer `DexEvent`.
- `fetch_rpc_transaction_as_streamer_events` and `parse_encoded_rpc_transaction_as_streamer_events` parse RPC transactions into streamer events.
- `grpc::ClientConfig::order_mode` supports `Unordered`, `Ordered`, `StreamingOrdered`, and `MicroBatch`.
- `sdk-parse-zero-copy` enables the SDK zero-copy parser backend.

### Migrating from v0.5.x to v1.x.x

Version 1.0.0 introduces a major architectural change from trait-based event handling to enum-based events. This provides better type safety, improved performance, and simpler code patterns.

**Key Changes:**

1. **Event Type Changed** - `Box<dyn UnifiedEvent>` → `DexEvent` enum
2. **Callback Signature** - Callbacks now receive concrete `DexEvent` instead of trait objects
3. **Event Matching** - Use standard Rust `match` instead of `match_event!` macro
4. **Metadata Access** - Event properties now accessed through `.metadata()` method

**Quick Migration Example:**

```rust
// Old (v0.5.x)
let callback = |event: Box<dyn UnifiedEvent>| {
    println!("Event: {:?}", event.event_type());
};

// New (v1.x.x)
let callback = |event: DexEvent| {
    println!("Event: {:?}", event.metadata().event_type);
};
```
