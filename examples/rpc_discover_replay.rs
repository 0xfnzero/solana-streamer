//! Discover real signatures, preserve both RPC encodings and parse offline.
//! cargo run --example rpc_discover_replay -- --address 7Ljfc4c9KJ6wm2mSmbUAmt2aK5h3TX4S3Rg1sKAc45nM 1 /tmp/cpmm-captures CollectCreatorFee
//! cargo run --example rpc_discover_replay -- --signature SIGNATURE /tmp/captures
//! cargo run --example rpc_discover_replay -- --fixture CAPTURE.json CAPTURE.wire.json
//! Wire filter (works without logs): --address ADDRESS LIMIT DIRECTORY --discriminator PROGRAM_ID HEX
//! PumpFun sell_v2 discriminator: 5df6823ce7e940b2
//! SOLANA_RPC_URL overrides the default mainnet RPC. No transactions are sent.
#[path = "support/rpc_capture.rs"]
mod capture;

fn main() -> anyhow::Result<()> {
    capture::run(|tx| {
        let events = solana_streamer_sdk::parse_encoded_rpc_transaction_as_streamer_events(
            tx,
            0,
            &[],
            None,
        )?;
        Ok((events.len(), serde_json::to_string_pretty(&events)?))
    })
}
