use anyhow::Result;
use solana_commitment_config::CommitmentConfig;
use solana_streamer_sdk::streaming::event_parser::core::event_parser::EventParser;
use solana_streamer_sdk::streaming::event_parser::Protocol;
use solana_streamer_sdk::streaming::event_parser::UnifiedEvent;
use std::str::FromStr;
use std::sync::Arc;
/// Get transaction data based on transaction signature
#[tokio::main]
async fn main() -> Result<()> {
    let signatures = vec![
        "5sDWrTTkE69CNc6nrAX7SqPS7FiajJTg8TMog3Gve7KjVfrqYn8YZcX1kAoyKok976S4RTnK1EdCV8hRiDWg68Aj",
    ];
    // Validate signature format
    let mut valid_signatures = Vec::new();
    for sig_str in &signatures {
        match solana_sdk::signature::Signature::from_str(sig_str) {
            Ok(_) => valid_signatures.push(*sig_str),
            Err(e) => println!("Invalid signature format: {}", e),
        }
    }
    if valid_signatures.is_empty() {
        println!("No valid transaction signatures");
        return Ok(());
    }
    for signature in valid_signatures {
        println!("Starting transaction parsing: {}", signature);
        get_single_transaction_details(signature).await?;
        println!("Transaction parsing completed: {}\n", signature);
        println!("Visit link to compare data: \nhttps://solscan.io/tx/{}\n", signature);
        println!("--------------------------------");
    }

    Ok(())
}

/// Get details of a single transaction
async fn get_single_transaction_details(signature_str: &str) -> Result<()> {
    use solana_sdk::signature::Signature;
    use solana_transaction_status::UiTransactionEncoding;

    let signature = Signature::from_str(signature_str)?;

    // Create Solana RPC client
    let rpc_url = "https://api.mainnet-beta.solana.com";
    println!("Connecting to Solana RPC: {}", rpc_url);

    let client = solana_client::nonblocking::rpc_client::RpcClient::new(rpc_url.to_string());

    match client
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
        Ok(transaction) => {
            println!("Transaction signature: {}", signature_str);
            println!("Block slot: {}", transaction.slot);

            if let Some(block_time) = transaction.block_time {
                println!("Block time: {}", block_time);
            }

            if let Some(meta) = &transaction.transaction.meta {
                println!("Transaction fee: {} lamports", meta.fee);
                println!("Status: {}", if meta.err.is_none() { "Success" } else { "Failed" });
                if let Some(err) = &meta.err {
                    println!("Error details: {:?}", err);
                }
                // Compute units consumed
                if let solana_transaction_status::option_serializer::OptionSerializer::Some(units) =
                    &meta.compute_units_consumed
                {
                    println!("Compute units consumed: {}", units);
                }
                // Display logs (all)
                if let solana_transaction_status::option_serializer::OptionSerializer::Some(logs) =
                    &meta.log_messages
                {
                    println!("Transaction logs (all {} entries):", logs.len());
                    for (i, log) in logs.iter().enumerate() {
                        println!("  [{}] {}", i + 1, log);
                    }
                }
            }
            let protocols = vec![
                Protocol::Bonk,
                Protocol::RaydiumClmm,
                Protocol::PumpSwap,
                Protocol::PumpFun,
                Protocol::RaydiumCpmm,
                Protocol::RaydiumAmmV4,
            ];
            let parser: Arc<EventParser> = Arc::new(EventParser::new(protocols, None));
            parser
                .parse_encoded_confirmed_transaction_with_status_meta(
                    signature,
                    transaction,
                    Arc::new(move |event: &Box<dyn UnifiedEvent>| {
                        println!("{:?}\n", event);
                    }),
                )
                .await?;
        }
        Err(e) => {
            println!("Failed to get transaction: {}", e);
        }
    }

    println!("Press Ctrl+C to exit example...");
    tokio::signal::ctrl_c().await?;

    Ok(())
}
