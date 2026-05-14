//! Parse a PumpSwap transaction from RPC using solana-streamer EventParser.
//!
//! Usage:
//!   cargo run --example parse_pumpswap_tx --release
//!   TX_SIGNATURE=<sig> SOLANA_RPC_URL=<url> cargo run --example parse_pumpswap_tx --release

use anyhow::Result;
use solana_commitment_config::CommitmentConfig;
use solana_streamer_sdk::streaming::event_parser::core::event_parser::EventParser;
use solana_streamer_sdk::streaming::event_parser::{DexEvent, Protocol};
use std::str::FromStr;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let default_sig =
        "3zsihbygW7hoKGtduAyDDFzp4E1eis8gaBzEzzNKr8ma39baffpFcphok9wHFgR3EauDe9vYYsVf4Puh5pZ6UJiS";
    let tx_sig = std::env::var("TX_SIGNATURE").unwrap_or_else(|_| default_sig.to_string());
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());

    println!("=== PumpSwap Transaction Parser (solana-streamer) ===\n");
    println!("Transaction: {}\n", tx_sig);
    println!("RPC: {}\n", rpc_url);

    parse_one_tx(&tx_sig, &rpc_url).await?;
    println!("\n✓ Done.");
    Ok(())
}

async fn parse_one_tx(signature_str: &str, rpc_url: &str) -> Result<()> {
    use prost_types::Timestamp;
    use solana_sdk::{
        message::compiled_instruction::CompiledInstruction, pubkey::Pubkey, signature::Signature,
    };
    use solana_transaction_status::{
        InnerInstruction, InnerInstructions, UiInstruction, UiTransactionEncoding,
    };

    let signature = Signature::from_str(signature_str)?;
    let client = solana_client::nonblocking::rpc_client::RpcClient::new(rpc_url.to_string());

    let transaction = match client
        .get_transaction_with_config(
            &signature,
            solana_client::rpc_config::RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::Base64),
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(0),
            },
        )
        .await
    {
        Ok(tx) => tx,
        Err(e) => anyhow::bail!("Failed to fetch transaction: {}", e),
    };

    let versioned_tx = match transaction.transaction.transaction.decode() {
        Some(tx) => tx,
        None => anyhow::bail!("Failed to decode transaction"),
    };

    let mut inner_instructions_vec: Vec<InnerInstructions> = Vec::new();
    if let Some(meta) = &transaction.transaction.meta {
        if let solana_transaction_status::option_serializer::OptionSerializer::Some(
            ui_inner_insts,
        ) = &meta.inner_instructions
        {
            for ui_inner in ui_inner_insts {
                let mut converted = Vec::new();
                for ui_instruction in &ui_inner.instructions {
                    if let UiInstruction::Compiled(ui_compiled) = ui_instruction {
                        if let Ok(data) = solana_sdk::bs58::decode(&ui_compiled.data).into_vec() {
                            converted.push(InnerInstruction {
                                instruction: CompiledInstruction {
                                    program_id_index: ui_compiled.program_id_index,
                                    accounts: ui_compiled.accounts.to_vec(),
                                    data,
                                },
                                stack_height: ui_compiled.stack_height,
                            });
                        }
                    }
                }
                inner_instructions_vec
                    .push(InnerInstructions { index: ui_inner.index, instructions: converted });
            }
        }
    }

    let mut address_table_lookups: Vec<Pubkey> = vec![];
    if let Some(meta) = &transaction.transaction.meta {
        if let solana_transaction_status::option_serializer::OptionSerializer::Some(loaded) =
            &meta.loaded_addresses
        {
            for s in loaded.writable.iter().chain(loaded.readonly.iter()) {
                if let Ok(p) = s.parse::<Pubkey>() {
                    address_table_lookups.push(p);
                }
            }
        }
    }

    let mut accounts: Vec<Pubkey> = versioned_tx.message.static_account_keys().to_vec();
    accounts.extend(address_table_lookups);

    let slot = transaction.slot;
    let block_time = transaction.block_time.map(|t| Timestamp { seconds: t as i64, nanos: 0 });
    let recv_us =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros()
            as i64;

    let protocols = vec![
        Protocol::PumpFun,
        Protocol::PumpSwap,
        Protocol::Bonk,
        Protocol::RaydiumClmm,
        Protocol::RaydiumCpmm,
        Protocol::RaydiumAmmV4,
        Protocol::MeteoraDammV2,
    ];

    let callback = Arc::new(|event: DexEvent| {
        println!("Event: {:?}\n", event);
    });

    EventParser::parse_instruction_events_from_versioned_transaction(
        &protocols,
        None,
        &versioned_tx,
        signature,
        Some(slot),
        block_time,
        recv_us,
        &accounts,
        &inner_instructions_vec,
        None,
        None,
        callback,
    )
    .await?;

    Ok(())
}
