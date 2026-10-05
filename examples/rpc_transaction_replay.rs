//! Fetch once or replay an RPC fixture through the public streamer facade.
//! cargo run --example rpc_transaction_replay -- --fetch SIGNATURE OUTPUT.json
//! cargo run --example rpc_transaction_replay -- --fixture OUTPUT.json
use anyhow::{bail, ensure, Context, Result};
use solana_client::{rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
use solana_sdk::signature::Signature;
use solana_streamer_sdk::parse_encoded_rpc_transaction_as_streamer_events;
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding};
use std::{fs, str::FromStr, time::Duration};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let tx: EncodedConfirmedTransactionWithStatusMeta = match args.as_slice() {
        [mode, path] if mode == "--fixture" => serde_json::from_slice(&fs::read(path)?)?,
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
            tx
        }
        _ => bail!("usage: --fetch SIGNATURE OUTPUT.json | --fixture INPUT.json"),
    };
    let events = parse_encoded_rpc_transaction_as_streamer_events(&tx, 0, &[], None)?;
    println!(
        "{{\"slot\":{},\"execution_error\":{},\"events\":{}}}",
        tx.slot,
        serde_json::to_string(&tx.transaction.meta.as_ref().and_then(|meta| meta.err.as_ref()))?,
        serde_json::to_string_pretty(&events)?
    );
    Ok(())
}
