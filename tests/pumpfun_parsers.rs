use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, signature::Signature};
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding};
use std::fs;
use std::path::Path;
use std::str::FromStr;

async fn fetch_transaction_fixture(
    signature: &str,
    fixture_path: &Path,
) -> Result<EncodedConfirmedTransactionWithStatusMeta> {
    if fixture_path.exists() {
        let data = fs::read_to_string(fixture_path)?;
        return Ok(serde_json::from_str(&data)?);
    }

    let client = RpcClient::new_with_commitment(
        "https://api.mainnet-beta.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    );

    let tx = client
        .get_transaction_with_config(
            &Signature::from_str(signature)?,
            solana_client::rpc_config::RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::Base64),
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(0),
            },
        )
        .await?;

    fs::write(fixture_path, serde_json::to_string_pretty(&tx)?)?;
    Ok(tx)
}

#[tokio::test]
async fn test_pumpfun_cpi_transaction_parsing() -> Result<()> {
    use solana_streamer_sdk::streaming::event_parser::{
        protocols::pumpfun::PumpFunTradeEvent, EventParserFactory, Protocol,
    };
    use solana_transaction_status::EncodedTransactionWithStatusMeta;

    // Transaction with PumpFun called via CPI (inner instruction)
    let signature = "3QLV6eN1W1pCWvS8pFYK4x7DCbQNsv16zNmjdpFvqbnzYCnMYEEfVWDsewmLJF1MByFBncaTcHXbqSpQhYHzqGEo";
    let fixture_path = Path::new("tests/fixtures/pumpfun_cpi_tx.json");
    let tx = fetch_transaction_fixture(signature, fixture_path).await?;

    let parser = EventParserFactory::create_parser(Protocol::PumpFun);
    let encoded_tx = EncodedTransactionWithStatusMeta {
        transaction: tx.transaction.transaction.clone(),
        meta: tx.transaction.meta.clone(),
        version: tx.transaction.version,
    };

    let events = parser
        .parse_transaction(
            encoded_tx,
            signature,
            Some(tx.slot),
            tx.block_time.map(|bt| prost_types::Timestamp {
                seconds: bt / 1000,
                nanos: ((bt % 1000) * 1_000_000) as i32,
            }),
            0, // program_received_time_ms for testing
            None,
        )
        .await?;

    // The transaction should parse at least one trade event
    assert!(!events.is_empty(), "Should find at least one event");

    let trade_event = events
        .iter()
        .find_map(|e| e.as_any().downcast_ref::<PumpFunTradeEvent>())
        .expect("Should find at least one PumpFun trade event");

    // Verify trade data from CPI logs is properly parsed
    assert_eq!(trade_event.mint.to_string(), "Ac9UhxTAvhbqC6e9LbKvHkMgtBt8kZSpmRQrVBqJpump");
    assert_eq!(trade_event.sol_amount, 98019);
    assert_eq!(trade_event.token_amount, 1864792795);
    assert_eq!(trade_event.is_buy, true);
    assert_eq!(trade_event.user.to_string(), "DRUujjQPsCqNFnaqY1c6FaSbDsS6BwLqm7hGPUMaJPF6");
    assert_eq!(trade_event.timestamp, 1753747643);
    
    // Verify reserve data from logs
    assert_eq!(trade_event.virtual_sol_reserves, 41133990957);
    assert_eq!(trade_event.virtual_token_reserves, 782564474266348);
    assert_eq!(trade_event.real_sol_reserves, 11133990957);
    assert_eq!(trade_event.real_token_reserves, 502664474266348);
    
    // Verify fee data from logs
    assert_eq!(trade_event.fee_recipient.to_string(), "62qc2CNXwrYqQScmEdiZFFAnJR262PxWEuNQtxfafNgV");
    assert_eq!(trade_event.fee_basis_points, 95);
    assert_eq!(trade_event.fee, 932);
    assert_eq!(trade_event.creator.to_string(), "3RUXFybjGtpQ49418Kcx1yaqgEeghrX3NvAwZPZxothu");
    assert_eq!(trade_event.creator_fee_basis_points, 5);
    assert_eq!(trade_event.creator_fee, 50);

    // Verify additional transaction fields
    assert_eq!(trade_event.bonding_curve.to_string(), "5YRpiJSMQwnG3Jpq1byRGedtXgrKQx2dJnsR1ZcDz3pH");
    assert_eq!(trade_event.associated_bonding_curve.to_string(), "ERwVHpVisT7WHFLGKKqXzWibxuP3dKVz75MYhdxXj31J");
    assert_eq!(trade_event.associated_user.to_string(), "3tUeBaJjPvzr2Ny71ShTrvNbaHvbwiCxCvDC8YKzq6JY");
    assert_eq!(trade_event.creator_vault.to_string(), "FKTHRr1NfAdGJHgfM5k3TsmXaTVQ74gNgS2AiYKjrXwT");
    assert_eq!(trade_event.max_sol_cost, 99979);
    assert_eq!(trade_event.min_sol_output, 0);
    assert_eq!(trade_event.amount, 1864792795);
    assert_eq!(trade_event.is_bot, false);
    assert_eq!(trade_event.is_dev_create_token_trade, false);

    Ok(())
}

#[tokio::test]
async fn test_pumpfun_direct_transaction_parsing() -> Result<()> {
    use solana_streamer_sdk::streaming::event_parser::{
        protocols::pumpfun::PumpFunTradeEvent, EventParserFactory, Protocol,
    };
    use solana_transaction_status::EncodedTransactionWithStatusMeta;

    // Add test for direct PumpFun transaction (non-CPI)
    let signature = "2ghHZXwyU6K1Q8KMJbLJg37ktmyctKmdzzZKGDvHk1MR865dDYyo8SfrKvmvijT43P6hdu6ozPtATiMeg2STszhc";
    let fixture_path = Path::new("tests/fixtures/pumpfun_direct_tx.json");
    
    let tx = fetch_transaction_fixture(signature, fixture_path).await?;
    let parser = EventParserFactory::create_parser(Protocol::PumpFun);
    let encoded_tx = EncodedTransactionWithStatusMeta {
        transaction: tx.transaction.transaction.clone(),
        meta: tx.transaction.meta.clone(),
        version: tx.transaction.version,
    };

    let events = parser
        .parse_transaction(
            encoded_tx,
            signature,
            Some(tx.slot),
            tx.block_time.map(|bt| prost_types::Timestamp {
                seconds: bt / 1000,
                nanos: ((bt % 1000) * 1_000_000) as i32,
            }),
            0, // program_received_time_ms for testing
            None,
        )
        .await?;

    assert!(!events.is_empty(), "Should find at least one event");
    
    let trade_event = events
        .iter()
        .find_map(|e| e.as_any().downcast_ref::<PumpFunTradeEvent>())
        .expect("Should find at least one PumpFun trade event");

    // Verify trade data for direct transaction
    assert_eq!(trade_event.mint.to_string(), "7k2255ueF3Ecnnjf9odEu7so3gmXKS8E29atDWmFpump");
    assert_eq!(trade_event.sol_amount, 129814469);
    assert_eq!(trade_event.token_amount, 2878556000000);
    assert_eq!(trade_event.is_buy, true);
    assert_eq!(trade_event.user.to_string(), "3HeEuccBzrTvWBvQGuiVgqbJcCpzTd4mFZjbKQoz5BYg");
    assert_eq!(trade_event.timestamp, 1753751878);
    
    // Verify reserve data
    assert_eq!(trade_event.virtual_sol_reserves, 38165815301);
    assert_eq!(trade_event.virtual_token_reserves, 843424927552285);
    assert_eq!(trade_event.real_sol_reserves, 8165815301);
    assert_eq!(trade_event.real_token_reserves, 563524927552285);
    
    // Verify fee data
    assert_eq!(trade_event.fee_recipient.to_string(), "CebN5WGQ4jvEPvsVU4EoHEpgzq1VV7AbicfhtW4xC9iM");
    assert_eq!(trade_event.fee_basis_points, 95);
    assert_eq!(trade_event.fee, 1233238);
    assert_eq!(trade_event.creator.to_string(), "3JCST4EZJoJT6rJhjNn6e4KRK2pg79JoAye7zdCPMMdK");
    assert_eq!(trade_event.creator_fee_basis_points, 5);
    assert_eq!(trade_event.creator_fee, 64908);

    // Verify additional transaction fields
    assert_eq!(trade_event.bonding_curve.to_string(), "9A5TEByiBsj1RXZfe6cDr3tLn89vXtrZ11QsFWf13Njm");
    assert_eq!(trade_event.associated_bonding_curve.to_string(), "DNvVRTVBPVzQymdBJasyEhycJazS4pLTvpR9XNPADqvN");
    assert_eq!(trade_event.associated_user.to_string(), "BCQfw1AGhiDr4gaFsLoYiUZh9GABSJopsWV5o9AyhZek");
    // assert_eq!(trade_event.creator_vault.to_string(), "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
    assert_eq!(trade_event.creator_vault.to_string(), "ib3XMvvrzqqrzvt2ejXQmToEBjQsaP157xRhKtAAQYj");
    assert_eq!(trade_event.max_sol_cost, 195000000);
    assert_eq!(trade_event.min_sol_output, 0);
    assert_eq!(trade_event.amount, 2878556000000);
    assert_eq!(trade_event.is_bot, false);
    assert_eq!(trade_event.is_dev_create_token_trade, false);

    Ok(())
}