//! Fetch once or replay an RPC fixture through the public streamer facade.
//! With no arguments: replay the reported ALT creates and official multi-hop offline.
//! cargo run --example rpc_transaction_replay -- --fetch SIGNATURE OUTPUT.json
//! cargo run --example rpc_transaction_replay -- --fixture OUTPUT.json
use anyhow::{bail, ensure, Context, Result};
use solana_client::{rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
use solana_sdk::signature::Signature;
use solana_streamer_sdk::{
    parse_encoded_rpc_transaction_as_streamer_events,
    streaming::{
        common::event_processor::process_shred_transaction,
        event_parser::{DexEvent, Protocol},
        shred::TransactionWithSlot,
    },
};
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding};
use std::{fs, str::FromStr, time::Duration};

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let transactions: Vec<EncodedConfirmedTransactionWithStatusMeta> = match args.as_slice() {
        [] => {
            let mut transactions = Vec::new();
            for source in [
                include_str!("../tests/fixtures/pump_create_alt_20261026.json"),
                include_str!("../tests/fixtures/pump_multi_hop_20261025.json"),
            ] {
                let fixture: serde_json::Value = serde_json::from_str(source)?;
                for case in fixture["cases"].as_array().context("Missing cases")? {
                    transactions.push(serde_json::from_value(case["wire_rpc"].clone())?);
                }
            }
            transactions
        }
        [mode, path] if mode == "--fixture" => vec![serde_json::from_slice(&fs::read(path)?)?],
        [mode, signature, path] if mode == "--fetch" => {
            let signature = Signature::from_str(signature)?;
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
                )
                .context("getTransaction failed; retry or use an archive RPC")?;
            let decoded =
                tx.transaction.transaction.decode().context("invalid transaction wire data")?;
            ensure!(
                decoded.signatures.first() == Some(&signature),
                "RPC returned a different signature"
            );
            fs::write(path, serde_json::to_vec_pretty(&tx)?)?;
            vec![tx]
        }
        _ => bail!("usage: --fetch SIGNATURE OUTPUT.json | --fixture INPUT.json"),
    };
    for tx in transactions {
        let signed = tx.transaction.transaction.decode().context("Invalid wire transaction")?;
        let captured = std::sync::Arc::new(std::sync::Mutex::new(Vec::<DexEvent>::new()));
        let output = captured.clone();
        process_shred_transaction(
            TransactionWithSlot::new(signed, tx.slot, 123456, Some(888)),
            &[Protocol::PumpFun],
            None,
            std::sync::Arc::new(move |e| output.lock().unwrap().push(e)),
            None,
        )
        .await?;
        println!(
            "{}",
            serde_json::json!({"source":"shred_static", "slot":tx.slot, "events":*captured.lock().unwrap()})
        );
        let events = parse_encoded_rpc_transaction_as_streamer_events(
            &tx,
            123456,
            &[Protocol::PumpFun],
            None,
        )?;
        println!(
            "{{\"slot\":{},\"execution_error\":{},\"events\":{}}}",
            tx.slot,
            serde_json::to_string(
                &tx.transaction.meta.as_ref().and_then(|meta| meta.err.as_ref())
            )?,
            serde_json::to_string_pretty(&events)?
        );
    }
    Ok(())
}
