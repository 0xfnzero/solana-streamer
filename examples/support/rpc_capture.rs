//! Bounded RPC discovery and wire checks, kept outside the streaming hot path.
use anyhow::{bail, ensure, Context, Result};
use serde_json::{json, Value};
use solana_client::{rpc_client::RpcClient, rpc_request::RpcRequest};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;
use std::{fs, path::Path, str::FromStr, thread, time::Duration};

enum CaptureFilter<'a> {
    LogName(&'a str),
    Instruction { program: &'a str, discriminator: [u8; 8] },
}

fn parse_discriminator(hex: &str) -> Result<[u8; 8]> {
    ensure!(hex.is_ascii() && hex.len() == 16, "discriminator must be exactly 16 hex digits");
    let mut bytes = [0; 8];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16)
            .context("invalid discriminator hex")?;
    }
    Ok(bytes)
}

/// Match compiled wire instructions, resolving ALT program IDs as well as
/// static keys. A program merely present in the account list does not match.
pub fn matches_instruction(wire: &Value, program: &str, discriminator: &[u8; 8]) -> Result<bool> {
    let mut keys = wire["transaction"]["message"]["accountKeys"]
        .as_array()
        .context("missing static keys")?
        .iter()
        .collect::<Vec<_>>();
    for kind in ["writable", "readonly"] {
        if let Some(loaded) = wire["meta"]["loadedAddresses"][kind].as_array() {
            keys.extend(loaded);
        }
    }
    let outer = wire["transaction"]["message"]["instructions"]
        .as_array()
        .context("missing outer instructions")?;
    let inner = wire["meta"]["innerInstructions"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|group| group["instructions"].as_array().into_iter().flatten());
    for ix in outer.iter().chain(inner) {
        let index =
            usize::try_from(ix["programIdIndex"].as_u64().context("invalid program index")?)?;
        let key = keys
            .get(index)
            .context("program index exceeds resolved keys")?
            .as_str()
            .context("invalid program key")?;
        if key == program {
            let data =
                solana_sdk::bs58::decode(ix["data"].as_str().context("missing instruction data")?)
                    .into_vec()?;
            if data.starts_with(discriminator) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn fetch<T: serde::de::DeserializeOwned>(
    client: &RpcClient,
    request: RpcRequest,
    params: Value,
) -> Result<T> {
    for attempt in 0..3 {
        if let Ok(value) = client.send(request.clone(), params.clone()) {
            return Ok(value);
        }
        if attempt < 2 {
            thread::sleep(Duration::from_secs(attempt + 1));
        }
    }
    bail!("RPC request failed after 3 attempts; retry or select another SOLANA_RPC_URL")
}

pub fn verify_wire(tx: &EncodedConfirmedTransactionWithStatusMeta, wire: &Value) -> Result<()> {
    let decoded = tx.transaction.transaction.decode().context("invalid base64 transaction")?;
    decoded.verify_and_hash_message().context("invalid transaction signatures")?;
    ensure!(wire["slot"].as_u64() == Some(tx.slot), "slot mismatch");
    ensure!(wire["blockTime"].as_i64() == tx.block_time, "block time mismatch");
    let signatures: Vec<_> = decoded.signatures.iter().map(ToString::to_string).collect();
    ensure!(json!(signatures) == wire["transaction"]["signatures"], "signature mismatch");
    let message = &wire["transaction"]["message"];
    ensure!(
        message["header"] == serde_json::to_value(decoded.message.header())?,
        "message header mismatch"
    );
    ensure!(
        message["recentBlockhash"].as_str()
            == Some(decoded.message.recent_blockhash().to_string().as_str()),
        "blockhash mismatch"
    );
    let lookups = decoded.message.address_table_lookups().map(|lookups| {
        lookups
            .iter()
            .map(|lookup| {
                json!({
                    "accountKey": lookup.account_key.to_string(),
                    "writableIndexes": lookup.writable_indexes,
                    "readonlyIndexes": lookup.readonly_indexes,
                })
            })
            .collect::<Vec<_>>()
    });
    ensure!(message["addressTableLookups"] == json!(lookups), "ALT lookups mismatch");
    let keys: Vec<_> =
        decoded.message.static_account_keys().iter().map(ToString::to_string).collect();
    ensure!(json!(keys) == message["accountKeys"], "static account keys mismatch");
    let instructions = message["instructions"].as_array().context("missing wire instructions")?;
    ensure!(
        instructions.len() == decoded.message.instructions().len(),
        "instruction count mismatch"
    );
    for (raw, compiled) in instructions.iter().zip(decoded.message.instructions()) {
        ensure!(
            raw["programIdIndex"].as_u64() == Some(u64::from(compiled.program_id_index)),
            "program index mismatch"
        );
        ensure!(raw["accounts"] == json!(compiled.accounts), "instruction account order mismatch");
        let data =
            solana_sdk::bs58::decode(raw["data"].as_str().context("missing instruction data")?)
                .into_vec()?;
        ensure!(data == compiled.data, "instruction data mismatch");
    }
    let metadata = serde_json::to_value(
        tx.transaction.meta.as_ref().context("missing transaction metadata")?,
    )?;
    for field in [
        "err",
        "fee",
        "loadedAddresses",
        "innerInstructions",
        "preTokenBalances",
        "postTokenBalances",
    ] {
        ensure!(metadata[field] == wire["meta"][field], "metadata mismatch: {field}");
    }
    Ok(())
}

/// The callback serializes events directly, preserving full-width u128 fields.
pub fn run(
    parse: impl Fn(&EncodedConfirmedTransactionWithStatusMeta) -> Result<(usize, String)>,
) -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if let [mode, path, wire_path] = args.as_slice() {
        if mode == "--fixture" {
            let tx = serde_json::from_slice(&fs::read(path)?)?;
            let wire = serde_json::from_slice(&fs::read(wire_path)?)?;
            verify_wire(&tx, &wire)?;
            let (count, events) = parse(&tx)?;
            ensure!(
                wire["meta"]["err"].is_null() || count == 0,
                "failed transaction emitted DEX events"
            );
            println!("wire=PASS slot={} events={count}\n{events}", tx.slot);
            return Ok(());
        }
    }
    let url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".into());
    let client = RpcClient::new_with_timeout(url, Duration::from_secs(30));
    let (signatures, directory, filter): (Vec<String>, &str, Option<CaptureFilter<'_>>) = match args.as_slice() {
        [mode, signature, directory] if mode == "--signature" => {
            Signature::from_str(signature)?;
            (vec![signature.clone()], directory, None)
        }
        [mode, address, limit, directory, rest @ ..] if mode == "--address" => {
            Pubkey::from_str(address)?;
            let limit: usize = limit.parse()?;
            ensure!((1..=25).contains(&limit), "limit must be 1..=25");
            let filter = match rest {
                [] => None,
                [name] => Some(CaptureFilter::LogName(name)),
                [flag, program, hex] if flag == "--discriminator" => {
                    Pubkey::from_str(program)?;
                    Some(CaptureFilter::Instruction { program, discriminator: parse_discriminator(hex)? })
                }
                _ => bail!("filter: INSTRUCTION or --discriminator PROGRAM_ID HEX"),
            };
            let rows: Value = fetch(&client, RpcRequest::GetSignaturesForAddress,
                json!([address, {"limit":limit, "commitment":"finalized"}]))?;
            let signatures = rows.as_array().context("invalid signature response")?.iter()
                .map(|row| Ok(row["signature"].as_str().context("missing signature")?.to_owned()))
                .collect::<Result<Vec<_>>>()?;
            (signatures, directory, filter)
        }
        _ => bail!("usage: --address ADDRESS LIMIT DIRECTORY [INSTRUCTION] | --signature SIGNATURE DIRECTORY | --fixture BASE64.json WIRE.json"),
    };
    fs::create_dir_all(directory)?;
    let mut saved = 0;
    for signature in signatures {
        // Validate server-provided filenames and pace archival RPC calls.
        Signature::from_str(&signature)?;
        let wire: Value = fetch(
            &client,
            RpcRequest::GetTransaction,
            json!([signature,
            {"encoding":"json", "commitment":"finalized", "maxSupportedTransactionVersion":1}]),
        )?;
        if wire.is_null() {
            println!("unavailable {signature}");
            thread::sleep(Duration::from_secs(1));
            continue;
        }
        if let Some(filter) = &filter {
            let matched = match filter {
                CaptureFilter::LogName(instruction) => {
                    let expected = format!("Program log: Instruction: {instruction}");
                    wire["meta"]["logMessages"].as_array().is_some_and(|logs| {
                        logs.iter().any(|line| line.as_str() == Some(expected.as_str()))
                    })
                }
                CaptureFilter::Instruction { program, discriminator } => {
                    matches_instruction(&wire, program, discriminator)?
                }
            };
            if !matched {
                thread::sleep(Duration::from_secs(1));
                continue;
            }
        }
        let base = Path::new(directory).join(&signature);
        fs::write(base.with_extension("wire.json"), serde_json::to_vec_pretty(&wire)?)?;
        thread::sleep(Duration::from_secs(1));
        let tx: EncodedConfirmedTransactionWithStatusMeta = fetch(
            &client,
            RpcRequest::GetTransaction,
            json!([signature, {"encoding":"base64", "commitment":"finalized", "maxSupportedTransactionVersion":1}]),
        )?;
        fs::write(base.with_extension("json"), serde_json::to_vec_pretty(&tx)?)?;
        verify_wire(&tx, &wire)?;
        ensure!(
            wire["transaction"]["signatures"][0].as_str() == Some(signature.as_str()),
            "RPC returned a different signature"
        );
        let (count, events) = parse(&tx)?;
        ensure!(
            wire["meta"]["err"].is_null() || count == 0,
            "failed transaction emitted DEX events"
        );
        fs::write(base.with_extension("events.json"), events)?;
        println!(
            "wire=PASS {signature} slot={} failed={} events={count}",
            tx.slot,
            !wire["meta"]["err"].is_null()
        );
        saved += 1;
        thread::sleep(Duration::from_secs(1));
    }
    println!("Saved {saved} captures; wire checks compare encodings, DEX amount checks use rpc_corpus_validate.");
    Ok(())
}
