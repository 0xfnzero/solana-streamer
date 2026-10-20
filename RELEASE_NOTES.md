# solana-streamer v4.0.0

Trade provenance bridge synchronized with sol-parser-sdk 0.8.0.

## Changes

- Preserve original router payloads and parsing evidence through supported trade event bridges.
- Use sol-parser-sdk =0.8.0; retain versioned official instruction names and transaction/receive metadata.
- Organize examples and documentation; tests use the SDK's own language. Offline captured transactions do not represent fresh mainnet execution or production latency measurements.

## Migration

third_program now contains RawInstructionPayload rather than a rule identifier. It does not contain duplicated metadata/dexes; metadata remains on the event. Check parse_status: Unresolved is only a possible DEX interaction, InstructionDecoded establishes input decoding, and ExecutionConfirmed reflects supplied execution evidence, not necessarily known fill amounts. Do not treat zero defaults as proven zero fills. Rust callers constructing trade structs must initialize the new fields or use ..Default::default().

No RPC is performed by the parsing hot path. Explicit analysis/discovery tools can fetch RPC data outside that path. Wrapper reconstruction remains opt-in and is not proof of future third-party program behavior.
