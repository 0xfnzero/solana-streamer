//! Parse Pump.fun quote-mint edge cases through solana-streamer's parser SDK re-export.
//!
//! Defaults cover:
//! - 19-account `create_v2` with explicit WSOL quote mint.
//! - 16-account `create_v2` with no quote mint account.
//!
//! Usage:
//! ```bash
//! cargo run --example parse_pumpfun_quote_cases --release
//! SOLANA_RPC_URL=https://api.mainnet-beta.solana.com cargo run --example parse_pumpfun_quote_cases --release
//! TX_SIGNATURES=sig1,sig2 cargo run --example parse_pumpfun_quote_cases --release
//! ```

use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_streamer_sdk::parser_sdk::core::events::{
    DexEvent, PUMPFUN_SOLSCAN_SOL_QUOTE_MINT, PUMPFUN_WSOL_QUOTE_MINT,
};
use solana_streamer_sdk::parser_sdk::parse_transaction_from_rpc;
use std::str::FromStr;

struct Case {
    name: &'static str,
    signature: &'static str,
    expected_create_v2_quote: Option<Pubkey>,
}

fn default_cases() -> Vec<Case> {
    vec![
        Case {
            name: "19-account create_v2 explicit WSOL quote",
            signature: "4GCVgY2FnT1s4q5zemnPL4mzSbuhUTgQo9mc9jewhLZzsCXKe8ehz6xD4QDJE853CLrF6doJbf4JNwJVeEYLA4De",
            expected_create_v2_quote: Some(PUMPFUN_WSOL_QUOTE_MINT),
        },
        Case {
            name: "16-account create_v2 no quote account",
            signature: "H6azwLqtRtrnVNC5iwcjYM9idU3e9SRyLZXTwjfJGJxA4X7dZL7vyhFAJNvQy7bb6bmQNmFHUt1KkkPPmhdge3G",
            expected_create_v2_quote: Some(PUMPFUN_SOLSCAN_SOL_QUOTE_MINT),
        },
        Case {
            name: "19-account create_v2 explicit USDC quote",
            signature: "3MVawF6EPtG7rEPXdsyQfQUBLv3epRVNpNS4tRE4uwTPMqLNPqhuABwxU3QZH4uD6CuVupcpGchpNRK5HTbHRLNK",
            expected_create_v2_quote: Some(usdc_mint()),
        },
        Case {
            name: "20-account create_v2 explicit WSOL quote",
            signature: "oY9YQbie16Bw11GsqbAPVnW6YjMHAj3kP9sufjcuQjdfcU86iUY8CiSaDrvu4QXJFnGY4jqQc2Kc1YVuAzujvyv",
            expected_create_v2_quote: Some(PUMPFUN_WSOL_QUOTE_MINT),
        },
        Case {
            name: "19-account create_v2 explicit WSOL quote with later buy",
            signature: "3jWGFYXT5V33Qc2roEBFDRAWHeybDowr53dSdnYSRkrPdYybU7oyEH9BfgSRxkgFHVKmUjv4e5T33AEnhJvBCuP2",
            expected_create_v2_quote: Some(PUMPFUN_WSOL_QUOTE_MINT),
        },
        Case {
            name: "19-account create_v2 explicit USDC quote exact quote buy",
            signature: "2dZAucKwr4n5Lqu3BtJ4P8JsjCDtUXJzthadddfURraEJRTgn6XWaTNUNBbgUfP5c2wcVdubqViQhr48eWsgRqPX",
            expected_create_v2_quote: Some(usdc_mint()),
        },
        Case {
            name: "20-account create_v2 explicit WSOL quote with jit account",
            signature: "4h9kYjzYpqqyYZuFnjf14zRwrGyChCuKAYVy6a4ZBig19bydEYsHwp6VbiKqTzT3pLf6NXnf6E25dn1NiU8LR4YB",
            expected_create_v2_quote: Some(PUMPFUN_WSOL_QUOTE_MINT),
        },
    ]
}

fn usdc_mint() -> Pubkey {
    Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap()
}

fn cases_from_env() -> Option<Vec<Case>> {
    let raw = std::env::var("TX_SIGNATURES").ok()?;
    let cases: Vec<Case> = raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|signature| Case {
            name: "custom signature",
            signature: Box::leak(signature.to_string().into_boxed_str()),
            expected_create_v2_quote: None,
        })
        .collect();

    if cases.is_empty() {
        None
    } else {
        Some(cases)
    }
}

fn print_event_summary(index: usize, event: &DexEvent) -> Option<Pubkey> {
    match event {
        DexEvent::PumpFunCreate(e) => {
            println!(
                "  #{index} PumpFunCreate ix={} mint={} quote_mint={} token_program={} user={}",
                e.ix_name, e.mint, e.quote_mint, e.token_program, e.user
            );
            if e.ix_name == "create_v2" {
                Some(e.quote_mint)
            } else {
                None
            }
        }
        DexEvent::PumpFunCreateV2(e) => {
            println!(
                "  #{index} PumpFunCreateV2 ix={} mint={} quote_mint={} token_program={} user={}",
                e.ix_name, e.mint, e.quote_mint, e.token_program, e.user
            );
            Some(e.quote_mint)
        }
        DexEvent::PumpFunBuy(e)
        | DexEvent::PumpFunBuyExactSolIn(e)
        | DexEvent::PumpFunSell(e)
        | DexEvent::PumpFunTrade(e) => {
            println!(
                "  #{index} PumpFunTrade ix={} mint={} quote_mint={} quote_amount={} sol_amount={} token_amount={}",
                e.ix_name, e.mint, e.quote_mint, e.quote_amount, e.sol_amount, e.token_amount
            );
            None
        }
        _ => None,
    }
}

fn main() {
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
    let client = RpcClient::new(rpc_url.clone());
    let cases = cases_from_env().unwrap_or_else(default_cases);
    let mut failed = false;

    println!("RPC: {rpc_url}");
    println!("Cases: {}\n", cases.len());

    for case in cases {
        println!("=== {} ===", case.name);
        println!("signature={}", case.signature);

        let signature = Signature::from_str(case.signature).expect("invalid signature");
        let events = match parse_transaction_from_rpc(&client, &signature, None) {
            Ok(events) => events,
            Err(err) => {
                eprintln!("  ERROR: {err}");
                failed = true;
                println!();
                continue;
            }
        };

        println!("  events={}", events.len());
        let mut create_v2_quotes = Vec::new();
        for (i, event) in events.iter().enumerate() {
            if let Some(quote) = print_event_summary(i + 1, event) {
                create_v2_quotes.push(quote);
            }
        }

        if let Some(expected) = case.expected_create_v2_quote {
            let ok = create_v2_quotes.contains(&expected);
            println!(
                "  expected_create_v2_quote={} result={}",
                expected,
                if ok { "ok" } else { "FAIL" }
            );
            failed |= !ok;
        }
        println!();
    }

    if failed {
        std::process::exit(1);
    }
}
