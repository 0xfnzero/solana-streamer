//! cargo run --example rpc_corpus_validate
//! cargo run --example rpc_corpus_validate -- --list
//! cargo run --example rpc_corpus_validate -- --case cpmm_route
//! cargo run --example rpc_corpus_validate -- --case dlmm_route
//! dlmm_route independently checks DLMM, CLMM and Orca execution.
//! cargo run --example rpc_corpus_validate -- --case cpmm_collect
//! cargo run --example rpc_corpus_validate -- --case pumpswap_buy_exact_quote
//! cargo run --example rpc_corpus_validate -- --case pumpfun_sell_v2
//! Creator collection sample: 5PpDgbQKov8W6vJZdV4BdX1DL9aiDVP5P74mo4cAnTgZzUMSV9jnCzDcGVa8UjZiqrRJm5VSAMvS4hGgS2KAbkmX
//! cargo run --example rpc_corpus_validate -- --fetch-case cpmm_0 /tmp/live.json
#[path = "../tests/support/rpc_corpus.rs"]
mod corpus;
use anyhow::{bail, ensure, Result};
use solana_client::{rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
use solana_sdk::signature::Signature;
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding};
use std::{fs, path::Path, str::FromStr, time::Duration};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.as_slice() == ["--list"] {
        for name in corpus::CASES {
            let wire: serde_json::Value = serde_json::from_slice(&fs::read(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join(format!("tests/fixtures/rpc_corpus/{name}_wire.json")),
            )?)?;
            let signature = wire["transaction"]["signatures"][0]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("missing signature in {name}"))?;
            let slot = wire["slot"]
                .as_u64()
                .ok_or_else(|| anyhow::anyhow!("missing slot in {name}"))?;
            let status = if wire["meta"]["err"].is_null() { "success" } else { "failed" };
            println!("{name}: {signature}, slot={slot}, status={status}");
        }
        return Ok(());
    }
    let (names, output): (Vec<&str>, Option<&str>) = match args.as_slice() {
        [] => (corpus::CASES.to_vec(), None),
        [mode, name] if mode == "--case" => (vec![name], None),
        [mode, name, path] if mode == "--fetch-case" => (vec![name], Some(path)),
        _ => bail!("usage: [--list | --case NAME | --fetch-case NAME OUTPUT.json]"),
    };
    let mut total = 0;
    for name in names {
        ensure!(corpus::CASES.contains(&name), "unknown corpus case: {name}");
        let wire: serde_json::Value = serde_json::from_slice(&fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("tests/fixtures/rpc_corpus/{name}_wire.json")),
        )?)?;
        let signature =
            Signature::from_str(wire["transaction"]["signatures"][0].as_str().unwrap())?;
        let tx: EncodedConfirmedTransactionWithStatusMeta = if let Some(output) = output {
            let url = std::env::var("SOLANA_RPC_URL")
                .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".into());
            let tx = RpcClient::new_with_timeout(url, Duration::from_secs(30))
                .get_transaction_with_config(
                    &signature,
                    RpcTransactionConfig {
                        encoding: Some(UiTransactionEncoding::Base64),
                        max_supported_transaction_version: Some(1),
                        ..Default::default()
                    },
                )?;
            fs::write(output, serde_json::to_vec_pretty(&tx)?)?;
            tx
        } else {
            serde_json::from_str(&corpus::fixture(name))?
        };
        let decoded = tx
            .transaction
            .transaction
            .decode()
            .ok_or_else(|| anyhow::anyhow!("invalid transaction"))?;
        ensure!(decoded.signatures.first() == Some(&signature), "RPC signature mismatch");
        let events = solana_streamer_sdk::parse_encoded_rpc_transaction_as_streamer_events(
            &tx,
            0,
            &[],
            None,
        )?;
        let value = serde_json::from_str(&serde_json::to_string(&events)?)?;
        let checked = corpus::verify(name, &value, true);
        corpus::print_buy_details(&value, true);
        total += checked;
        println!(
            "PASS {name}: {signature}, slot={}, checked={checked}, events={}",
            tx.slot,
            events.len()
        );
    }
    println!("Verified {total} target operations against raw RPC data");
    Ok(())
}
