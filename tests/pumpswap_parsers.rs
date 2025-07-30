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
async fn test_pumpswap_buy_cpi_transaction_parsing() -> Result<()> {
    use solana_streamer_sdk::streaming::event_parser::{
        protocols::pumpswap::PumpSwapBuyEvent, EventParserFactory, Protocol,
    };
    use solana_transaction_status::EncodedTransactionWithStatusMeta;

    // Transaction with PumpSwap called via CPI (currently failing to parse correctly)
    let signature = "56RbkzmAEtd88ZeiBigh41kPThpoFqZoxj9tULQJe7xRBAcdRYxREuNBRUW5f2jJASZ81aNhxe8EBej258q76AuH";
    let fixture_path = Path::new("tests/fixtures/pumpswap_cpi_tx.json");
    let tx = fetch_transaction_fixture(signature, fixture_path).await?;

    let parser = EventParserFactory::create_parser(Protocol::PumpSwap);
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

    // The transaction should parse at least one buy event
    assert!(!events.is_empty(), "Should find at least one event");

    let buy_event = events
        .iter()
        .find_map(|e| e.as_any().downcast_ref::<PumpSwapBuyEvent>())
        .expect("Should find at least one PumpSwap buy event");

    // Verify all numeric fields
    assert_eq!(buy_event.timestamp, 1753789195);
    assert_eq!(buy_event.base_amount_out, 5544465819);
    assert_eq!(buy_event.max_quote_amount_in, 963427853);
    assert_eq!(buy_event.user_base_token_reserves, 1);
    assert_eq!(buy_event.user_quote_token_reserves, 2856323143);
    assert_eq!(buy_event.pool_base_token_reserves, 30819310537017);
    assert_eq!(buy_event.pool_quote_token_reserves, 5338304399191);
    assert_eq!(buy_event.quote_amount_in, 960546212);
    assert_eq!(buy_event.lp_fee_basis_points, 20);
    assert_eq!(buy_event.lp_fee, 1921093);
    assert_eq!(buy_event.protocol_fee_basis_points, 5);
    assert_eq!(buy_event.protocol_fee, 480274);
    assert_eq!(buy_event.quote_amount_in_with_lp_fee, 962467305);
    assert_eq!(buy_event.user_quote_amount_in, 963427853);
    assert_eq!(buy_event.coin_creator_fee_basis_points, 5);
    assert_eq!(buy_event.coin_creator_fee, 480274);
    
    // Verify all account fields
    assert_eq!(buy_event.pool.to_string(), "4w2cysotX6czaUGmmWg13hDpY4QEMG2CzeKYEQyK9Ama");
    assert_eq!(buy_event.user.to_string(), "6TYDxGmVxkBPBmEfnmLXx6jVff9LknsjRHqdTjVyZmG8");
    assert_eq!(buy_event.user_base_token_account.to_string(), "4yFkSKTzpbDhFpFEHiooKg9XmoNbQGfbe4ZPxK7CRtHB");
    assert_eq!(buy_event.user_quote_token_account.to_string(), "13Ma7KdfbBrF8NmDBnVTkwny7tjrZxYHCfn6a3FFbRUa");
    assert_eq!(buy_event.protocol_fee_recipient.to_string(), "62qc2CNXwrYqQScmEdiZFFAnJR262PxWEuNQtxfafNgV");
    assert_eq!(buy_event.protocol_fee_recipient_token_account.to_string(), "94qWNrtmfn42h3ZjUZwWvK1MEo9uVmmrBPd2hpNjYDjb");
    assert_eq!(buy_event.coin_creator.to_string(), "hQmkJVU4iWgxCHZvVACpVPa1KN28oGxXpRmcYzYNwck");
    
    // Verify additional account fields marked with #[borsh(skip)]
    assert_eq!(buy_event.base_mint.to_string(), "5UUH9RTDiSpq6HKS6bp4NdU9PNJpXRXuiw6ShBTBhgH2");
    assert_eq!(buy_event.quote_mint.to_string(), "So11111111111111111111111111111111111111112");
    assert_eq!(buy_event.pool_base_token_account.to_string(), "8SmW82qNZ7BpdBp5PZk7znqt3VswFzX3nkRgHBBE1x4e");
    assert_eq!(buy_event.pool_quote_token_account.to_string(), "657gpdF5TtxfXaW88MwuideK2pWhwRyoiVNnLDzS5q2K");
    assert_eq!(buy_event.coin_creator_vault_ata.to_string(), "DGGpxm8H8Bj5Dc1wZaLzPPVuiXsJTHoxvP5iR8tje1BK");
    assert_eq!(buy_event.coin_creator_vault_authority.to_string(), "CUdmpcWRWqE6Q3wkGeFXrQ4WyukBzhiuVeDTNV5sMknJ");

    Ok(())
}

#[tokio::test]
async fn test_pumpswap_sell_cpi_transaction_parsing() -> Result<()> {
    use solana_streamer_sdk::streaming::event_parser::{
        protocols::pumpswap::PumpSwapSellEvent, EventParserFactory, Protocol,
    };
    use solana_transaction_status::EncodedTransactionWithStatusMeta;

    // Transaction with PumpSwap sell called via CPI
    let signature = "27f6P1sV4sDJqvgbto5yyqRVn86pcDW1sqfMMSP2p3yNdFM27aBqr4Sn6ZDvkhAG12Dxm4Ehy1T8xDFQ3GywMAxA";
    let fixture_path = Path::new("tests/fixtures/pumpswap_sell_cpi_tx.json");
    let tx = fetch_transaction_fixture(signature, fixture_path).await?;

    let parser = EventParserFactory::create_parser(Protocol::PumpSwap);
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

    // The transaction should parse at least one sell event
    assert!(!events.is_empty(), "Should find at least one event");

    let sell_event = events
        .iter()
        .find_map(|e| e.as_any().downcast_ref::<PumpSwapSellEvent>())
        .expect("Should find at least one PumpSwap sell event");

    // Verify all numeric fields for sell event
    assert_eq!(sell_event.timestamp, 1753789184);
    assert_eq!(sell_event.base_amount_in, 134018503080);
    assert_eq!(sell_event.min_quote_amount_out, 0);
    assert_eq!(sell_event.quote_amount_out, 112810637);
    assert_eq!(sell_event.user_base_token_reserves, 134018503080);
    assert_eq!(sell_event.user_quote_token_reserves, 0);
    assert_eq!(sell_event.pool_base_token_reserves, 228938854683754);
    assert_eq!(sell_event.pool_quote_token_reserves, 192823053310);
    assert_eq!(sell_event.lp_fee_basis_points, 20);
    assert_eq!(sell_event.lp_fee, 225622);
    assert_eq!(sell_event.protocol_fee_basis_points, 5);
    assert_eq!(sell_event.protocol_fee, 56406);
    assert_eq!(sell_event.quote_amount_out_without_lp_fee, 112585015);
    assert_eq!(sell_event.user_quote_amount_out, 112472203);
    assert_eq!(sell_event.coin_creator_fee_basis_points, 5);
    assert_eq!(sell_event.coin_creator_fee, 56406);
    
    // Verify all account fields
    assert_eq!(sell_event.pool.to_string(), "EGTHgmHac1ZLgwVQjzXyGAyN8SKjgMbA6vsrFe6pB5dz");
    assert_eq!(sell_event.user.to_string(), "BvoxKEUeCkbWV7geTs74z8bhExE8FjVfaR4STkGe8tBi");
    assert_eq!(sell_event.user_base_token_account.to_string(), "9a5yJFXMNQVWGpK3SPrNoGxT7tr8HDCrwfhcb9bDviAj");
    assert_eq!(sell_event.user_quote_token_account.to_string(), "Bv9y8jM3JqH3mdSPdDKWEi1vuQBkUaK2gK7WdD6BeA4x");
    assert_eq!(sell_event.protocol_fee_recipient.to_string(), "JCRGumoE9Qi5BBgULTgdgTLjSgkCMSbF62ZZfGs84JeU");
    assert_eq!(sell_event.protocol_fee_recipient_token_account.to_string(), "DWpvfqzGWuVy9jVSKSShdM2733nrEsnnhsUStYbkj6Nn");
    assert_eq!(sell_event.coin_creator.to_string(), "JE8eFoVR5ZDpT7au7DdNuca1YrmNmSCDUoLy6rhzoY8Q");
    
    // Verify additional account fields marked with #[borsh(skip)]
    assert_eq!(sell_event.base_mint.to_string(), "5w68xqRpVFqi7hCXuN7QLc5ReLv1PfpmT9TVm11Cpump");
    assert_eq!(sell_event.quote_mint.to_string(), "So11111111111111111111111111111111111111112");
    assert_eq!(sell_event.pool_base_token_account.to_string(), "C6GSAdjdHfJTo8vVfULqLkzvXJjAPwEwbxkNq9xyShSL");
    assert_eq!(sell_event.pool_quote_token_account.to_string(), "2BpXbTgo2TpoexM3HX58SXjvT1T18EbmkmJwjRtvnEzs");
    assert_eq!(sell_event.coin_creator_vault_ata.to_string(), "8hudeyGY92sUn12dmUTMKDKSn9fq6zJUCRJBMbstis36");
    assert_eq!(sell_event.coin_creator_vault_authority.to_string(), "Ex512LiimjSvh5ex1ypMyL19wzvKVrr4Dbzg4Ai5vaW9");

    Ok(())
}