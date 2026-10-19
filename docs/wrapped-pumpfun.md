# Wrapped Pump.fun instructions (solana-streamer-sdk 3.0.17)

## Boundaries

The streaming hot path never calls RPC. gRPC uses the supplied execution metadata and actual CPI instructions. Shred sees only signed outer instructions and locally resolved ALT addresses. Never populate a missing ALT address with a default key.

Third-party reconstruction is **explicit opt-in**. Existing subscriptions keep RawInstruction as their default fallback. Load reviewed rules once at startup, then call the wrapper parser from a RawInstruction callback. Pass the subscription filter through; only deliver the restored event on a successful match. On a miss keep the raw event. Do not feed gRPC events through these rules.

A restored PumpFunTrade includes `third_program: { program_id, instruction_index, rule_id }`. `ix_name` uses the Pump.fun IDL method name. Input fields such as `spendable_quote_in` and `min_tokens_out` are available. `sol_amount`, `quote_amount`, and `token_amount` stay zero because Shred cannot establish executed amounts. Metadata, including tx_index, receive time, and recent_blockhash, is retained.

The initial rule engine supports fixed two-u64 v2 inputs. Multi-hop/CurveSwap, runtime-balance-dependent parameters, and unsupported layouts remain raw. Exact program, data length and guarded bytes, account count, canonical keys and aliases must match. A program upgrade can change semantics without changing this shape; rules are not a guarantee of future CPI behavior.

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

J7's first instruction in 5hKon… performs token-account/transfer work, with no Pump trade CPI. Its later buy passes 3,551,960 quote units to Pump, a value absent from its outer data. This cannot be restored by replacing the discriminator; the analyzer returns no rule. The captured Rapid sells also fail parameter inference and remain raw.

gRPC regression tests cover the reported transactions, recent captures and existing multi-hop fixtures; full/missing/truncated logs, repeated trades, failed transactions, metadata and filtering. Fixed: create-buy context is derived from outer/CPI create instructions per mint, not a transaction-wide log hint; versioned instruction names are no longer overwritten by generic event-log names.

## Other repositories

- sol-shred-sdk: `cargo run --example wrapped_pumpfun_replay` verifies signed wires and exact CPI correspondence offline.
- TypeScript: `npx tsx examples/wrapped_pumpfun_replay.ts`; `npx tsx examples/pump_execution_replay.ts [SIGNATURE]`.
- Python: `python examples/wrapped_pumpfun_replay.py`; `python examples/pump_execution_replay.py [SIGNATURE]`.
- Go (repository root): `go run ./examples/wrapped_pumpfun_replay`; `go run ./examples/pump_execution_replay [SIGNATURE]`.
- solana-streamer: uses the corrected local parser and preserves `third_program` through its event bridge. It does not enable third-party rules automatically.

The Rust analyzer's JSON rules can be consumed by all four parser implementations. Tests and wrapper examples are offline by default. Passing a signature to an execution/analyzer tool enables an explicit offline RPC fetch, not a request from the parsing hot path.
