# Wrapped Pump.fun instructions (solana-streamer-sdk 4.0.0)

## Boundaries

The streaming hot path never calls RPC. gRPC uses the supplied execution metadata and actual CPI instructions. Shred sees only signed outer instructions and locally resolved ALT addresses. Never populate a missing ALT address with a default key.

## Callback update (v4.0.0)

Trade provenance is now uniform across Pump.fun/PumpSwap, LaunchLab, Raydium
AMM v4/CPMM/CLMM, Orca Whirlpool and Meteora Pools/DAMM v2/DLMM/DBC. Their gRPC
trade callbacks preserve `third_program` and `TradeParseStatus` from the parser.
Use `DexEvent::third_program()` and `DexEvent::trade_parse_status()` for common
access. Direct calls have no wrapper payload; successful wrapped CPI trades
retain the original outer instruction. This does not turn unknown Shred wrappers
into confirmed trades. Existing protocol subscriptions still select the executed
DEX; callers do not need to subscribe to a particular third-party program.

Shred now emits a PumpFunTrade **candidate** for an unknown outer program whose own instruction accounts reference the Pump.fun program. Its `parse_status` is `Unresolved`; the default direction, mint, pool and amounts are unknown, not an actual sell or zero-size fill. This is not evidence that a trade executed: the wrapper might only create or prepare accounts. Transaction-wide account references alone do not qualify. Unresolved ALT addresses remain None; if the Pump program reference itself is unresolved, the parser cannot classify the instruction.

Subscribe to the broad PumpFunTrade family (or no event filter) to receive candidates. Buy/sell-only subscriptions do not receive unknown directions. Each candidate produces one callback. Explicit RawInstruction subscriptions retain the old raw API instead of producing duplicate candidate callbacks; other DEX raw subscriptions remain supported.

gRPC/RPC preserve actual Pump CPI trades and attach their originating outer instruction before merging logs and instructions. `ExecutionConfirmed` means the successful execution confirms the invocation, not that every amount is available. Native direct Pump instructions have no third_program. One outer wrapper can correspond to zero, one, or several actual Pump trades: compare signature + outer instruction_index, not total Shred/gRPC event counts.

Third-party parameter reconstruction remains **explicit opt-in**. Load reviewed rules once at startup. Adapt a candidate using `candidate.third_program.as_ref().unwrap().with_metadata(candidate.metadata.clone())`, then call the wrapper parser with the subscription filter. A successful reconstruction has `InstructionDecoded` status; a miss leaves the original unresolved candidate available to the caller. Do not feed confirmed gRPC trades through these rules.

`third_program` is now a RawInstructionPayload containing `program_id`, `instruction_index`, original ordered `accounts` (including duplicates and unresolved None entries), `account_indices`, and byte-vector `instruction_data`. It has no metadata or dexes fields: metadata remains on the enclosing trade. JSON uses arrays for instruction bytes and account indices, and null for unresolved accounts. Rule IDs remain in the caller's rule registry.

For an explicitly reconstructed trade, `ix_name` uses the Pump.fun IDL method name and input fields such as `spendable_quote_in` and `min_tokens_out` are available. `sol_amount`, `quote_amount`, and `token_amount` stay zero because Shred cannot establish executed amounts. Metadata, including tx_index, receive time, and recent_blockhash, is retained.

Migration: third_program's shape changed from the previously published program/index/rule_id structure. Update struct literals and consumers reading rule_id. Read parse_status before interpreting defaults. Old serialized trades without parse_status default to InstructionDecoded; they are not retroactively treated as execution-confirmed. RawInstructionEvent and its protocol filters remain available for compatibility.

The initial rule engine supports fixed two-u64 v2 inputs. Runtime-balance-dependent parameters and unsupported wrapper layouts remain unresolved candidates. Existing official multi-hop/CurveSwap route intents are unchanged. Exact program, data length and guarded bytes, account count, canonical keys and aliases must match. A program upgrade can change semantics without changing this shape; rules are not a guarantee of future CPI behavior.

## Offline analysis

In the Rust sol-parser-sdk repository:

```sh
cargo run --example wrapped_pumpfun_rpc_analyzer
SOLANA_RPC_URL=https://YOUR_RPC cargo run --example wrapped_pumpfun_rpc_analyzer -- SIGNATURE
cargo run --example wrapped_pumpfun_replay
cargo test --test wrapped_pumpfun --test pump_execution
```

Only the analyzer example's signature-fetch branch accesses RPC. The library analyzer consumes an already fetched transaction. It matches actual Pump CPI instruction bytes (not executed TradeEvent amounts), infers account maps and exact byte guards, and emits a **candidate** rule. Missing, ambiguous or runtime-derived parameters produce an explicit rejection. Candidates never install themselves. Validate each candidate on independent signatures and retain separate rules for different byte patterns; do not wildcard unexplained bytes.

`wrapped_pumpfun_replay` uses signed saved mainnet wire data and compares the gRPC execution path with the Shred outer-only path. It accepts `--fixture FILE` and `--rules FILE`. Saved execution metadata is an offline oracle only, never an input to the hot wrapper parser.

## Verified transactions

Reported signatures:

- `5hKonDfdNj283yidV77Ptfe1oJsPB9XXBQ6mJPtpbXVe9b6UhwBedVkX6tAaTwschTHYMeqUwt7bXeiHAzRob62p`
- `2r4uXUJtSsphTBuJHJm2T5kKe9c215MCVL4bZ1mUZEZ6cT5698s3xFUfHb4TfU4ZfJ6cuFppQfYZUpMP5QL9pZ1Q`
- `3nMXPjA7xRFmCeq3jypeWaXe9bmP5Fkwkvy6u74bBZHe316qMXNaE5aDQwDB1HRVLSkmk3BjgRCKBNcPEMeXrGWj`

Additional mainnet captures, slots 458801397–458801420:

- `5wM2Jk6eDSc42mtaSWrVavAuo6rZDpR3LmWK4BeJMqrt6B5xiEE6YzpHv5qdHJsHt7crLEM1L8P5r53ndRBespvC`
- `2KBsKB7xmfFz3qLB6U3YMTuc2Z8W3WYrEJhxqvtNytqw2co2W3zQizTPzMtC1c37NnaCuKpaoaxiT6QT4pqmF5nW`
- `5rY6dXEjgr7m5KymyNUP1GQxegrFf8GsM9263y35b6AxQ4e5d1nmHDpj2QACqnE5eZoFPn5vNmz17nbXGXtDZR9w`
- `4dG9FmeZjtKDiH1asJgWy8PJZAfqd96hiZRyFfjNn1g2JosLG1iwgpMgi95bygVX9jUgNW3oRDwt61q4yrjKSe2B`

The original Rapid transaction has three reconstructable buys, with u64 arguments at outer offsets 1 and 33. Recent buys differ at guarded byte 13 (15 versus 255), so the examples keep two distinct rules. The recent rule is inferred from 5rY6… and tested against independent signature 4dG9…. Five total buys match their actual CPI data and ordered accounts. The older rule is demonstrated by three calls in one signature; it does not have independent-signature validation in this corpus.

J7's first instruction in 5hKon… performs token-account/transfer work, with no Pump trade CPI. Its later buy passes 3,551,960 quote units to Pump, a value absent from its outer data. This cannot be restored by replacing the discriminator; the analyzer returns no rule. The captured Rapid sells also fail parameter inference. These wrappers can still produce unresolved Shred candidates when their own accounts reference Pump; gRPC reports only actual Pump trades with original wrapper data.

gRPC regression tests cover the reported transactions, recent captures and existing multi-hop fixtures; full/missing/truncated logs, repeated trades, failed transactions, metadata and filtering. Fixed: create-buy context is derived from outer/CPI create instructions per mint, not a transaction-wide log hint; versioned instruction names are no longer overwritten by generic event-log names.

## Other repositories

- sol-shred-sdk: `cargo run --example wrapped_pumpfun_replay` verifies signed wires and exact CPI correspondence offline.
- TypeScript: `npx tsx examples/wrapped_pumpfun_replay.ts`; `npx tsx examples/pump_execution_replay.ts [SIGNATURE]`.
- Python: `python examples/wrapped_pumpfun_replay.py`; `python examples/pump_execution_replay.py [SIGNATURE]`.
- Go (repository root): `go run ./examples/wrapped_pumpfun_replay`; `go run ./examples/pump_execution_replay [SIGNATURE]`.
- solana-streamer: uses the corrected local parser and preserves `third_program` through its event bridge. It does not enable third-party rules automatically.

The Rust analyzer's JSON rules can be consumed by all four parser implementations. Tests and wrapper examples are offline by default. Passing a signature to an execution/analyzer tool enables an explicit offline RPC fetch, not a request from the parsing hot path.
