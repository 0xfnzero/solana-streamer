//! Independent oracle: JSON wire instructions, SPL movements and balance deltas.
//! Does not use SDK instruction/event parsers to compute expected fields.
use serde_json::{json, Value};
use std::{fs, path::Path};

pub const CASES: [&str; 13] = [
    "cpmm_0",
    "cpmm_1",
    "pumpswap_0",
    "pumpswap_1",
    "dlmm_route",
    "pumpfun",
    "dlmm_0",
    "cpmm_route",
    "failed_route",
    "cpmm_collect",
    "cpmm_collect_token2022",
    "pumpswap_buy_exact_quote",
    "pumpfun_sell_v2",
];

pub fn fixture(name: &str) -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("tests/fixtures/rpc_corpus/{name}.json")),
    )
    .unwrap()
}

fn bytes(text: &str) -> Vec<u8> {
    solana_sdk::bs58::decode(text).into_vec().unwrap()
}

pub fn verify(name: &str, events: &Value, streamer: bool) -> usize {
    let wire: Value = serde_json::from_str(
        &fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("tests/fixtures/rpc_corpus/{name}_wire.json")),
        )
        .unwrap(),
    )
    .unwrap();
    if !wire["meta"]["err"].is_null() {
        assert!(events.as_array().unwrap().is_empty(), "failed transaction emitted DEX events");
        return 0;
    }
    let mut keys = wire["transaction"]["message"]["accountKeys"].as_array().unwrap().clone();
    for kind in ["writable", "readonly"] {
        keys.extend(wire["meta"]["loadedAddresses"][kind].as_array().unwrap().iter().cloned());
    }
    let mut instructions =
        wire["transaction"]["message"]["instructions"].as_array().unwrap().clone();
    for group in wire["meta"]["innerInstructions"].as_array().unwrap() {
        instructions.extend(group["instructions"].as_array().unwrap().iter().cloned());
    }
    let resolved: Vec<_> = instructions
        .iter()
        .map(|ix| {
            let program = keys[ix["programIdIndex"].as_u64().unwrap() as usize].as_str().unwrap();
            let data = bytes(ix["data"].as_str().unwrap());
            let accounts: Vec<_> = ix["accounts"]
                .as_array()
                .unwrap()
                .iter()
                .map(|i| keys[i.as_u64().unwrap() as usize].as_str().unwrap())
                .collect();
            (program, data, accounts)
        })
        .collect();
    let moves: Vec<_> = resolved
        .iter()
        .filter_map(|(program, data, a)| {
            if [
                "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
                "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
            ]
            .contains(program)
                && matches!(data.first(), Some(3 | 12))
            {
                Some((
                    a[0],
                    a[if data[0] == 3 { 1 } else { 2 }],
                    u64::from_le_bytes(data[1..9].try_into().unwrap()),
                ))
            } else {
                None
            }
        })
        .collect();
    let movement = |src: &str, dst: &str| -> u64 {
        let found: Vec<_> = moves.iter().filter(|(s, d, _)| *s == src && *d == dst).collect();
        assert_eq!(found.len(), 1, "ambiguous SPL movement {name}: {src} -> {dst}");
        found[0].2
    };
    let delta = |account: &str| -> i128 {
        let amount = |side: &str| -> i128 {
            let found: Vec<_> = wire["meta"][side]
                .as_array()
                .unwrap()
                .iter()
                .filter(|b| keys[b["accountIndex"].as_u64().unwrap() as usize] == account)
                .collect();
            assert!(found.len() <= 1);
            found
                .first()
                .map(|b| b["uiTokenAmount"]["amount"].as_str().unwrap().parse::<i128>().unwrap())
                .unwrap_or(0)
        };
        amount("postTokenBalances") - amount("preTokenBalances")
    };
    let all = events.as_array().unwrap();
    for event in all {
        let payload = event.as_object().unwrap().values().next().unwrap();
        assert_eq!(payload["metadata"]["slot"], wire["slot"]);
        assert_eq!(
            payload["metadata"]["signature"],
            json!(bytes(wire["transaction"]["signatures"][0].as_str().unwrap()))
        );
    }
    let mut checked = 0;
    let mut expected_counts = std::collections::BTreeMap::<&str, usize>::new();
    for (program, data, a) in &resolved {
        let disc = data.get(..8).unwrap_or_default();
        let (kind, fields, pool_field) = if *program
            == "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C"
            && disc == [143, 190, 90, 218, 196, 30, 51, 222]
        {
            let gross_in = movement(a[4], a[6]);
            let gross_out = movement(a[7], a[5]);
            assert_eq!(delta(a[7]), -i128::from(gross_out));
            let net_in = u64::try_from(delta(a[6])).unwrap();
            // A routed output can fund the next leg or close a wrapped-SOL
            // account. Its final balance delta is not this leg's payout. For
            // classic tokens there is no transfer tax. For these Token-2022
            // fixtures require one incoming edge, then add later gross debits.
            let net_out = if a[9] == "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" {
                gross_out
            } else {
                assert_eq!(moves.iter().filter(|(_, dst, _)| *dst == a[5]).count(), 1);
                let later_debits: i128 = moves
                    .iter()
                    .filter(|(src, _, _)| *src == a[5])
                    .map(|(_, _, amount)| i128::from(*amount))
                    .sum();
                u64::try_from(delta(a[5]) + later_debits).unwrap()
            };
            assert_eq!(
                gross_in,
                u64::from_le_bytes(data[8..16].try_into().unwrap()),
                "{name}: exact-input must debit the instructed gross amount"
            );
            assert!(
                net_out >= u64::from_le_bytes(data[16..24].try_into().unwrap()),
                "{name}: successful swap must satisfy its net output limit"
            );
            let mut fields = json!({"input_amount": net_in, "output_amount": gross_out,
                "input_transfer_fee": gross_in.checked_sub(net_in).unwrap(),
                "output_transfer_fee": gross_out.checked_sub(net_out).unwrap(),
                "amount_in": u64::from_le_bytes(data[8..16].try_into().unwrap()),
                "minimum_amount_out": u64::from_le_bytes(data[16..24].try_into().unwrap()),
                "max_amount_in":0, "amount_out":0, "base_input":true,
                "ix_name":"swap_base_input"});
            for (field, index) in [
                ("payer", 0),
                ("authority", 1),
                ("amm_config", 2),
                ("input_token_account", 4),
                ("output_token_account", 5),
                ("input_vault", 6),
                ("output_vault", 7),
                ("input_token_program", 8),
                ("output_token_program", 9),
                ("input_token_mint", 10),
                ("output_token_mint", 11),
                ("observation_state", 12),
            ] {
                fields[field] = json!(bytes(a[index]));
            }
            let pool_field = if streamer { "pool_state" } else { "pool_id" };
            fields[pool_field] = json!(bytes(a[3]));
            (if streamer { "RaydiumCpmmSwapEvent" } else { "RaydiumCpmmSwap" }, fields, pool_field)
        } else if *program == "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc"
            && disc == [43, 4, 237, 11, 26, 201, 30, 98]
        {
            let a_to_b = data[41] == 1;
            let (owner_in, vault_in, vault_out, owner_out) =
                if a_to_b { (a[7], a[8], a[10], a[9]) } else { (a[9], a[10], a[8], a[7]) };
            let gross_in = movement(owner_in, vault_in);
            let gross_out = movement(vault_out, owner_out);
            let net_in = u64::try_from(delta(vault_in)).unwrap();
            // The captured output can fund another leg. For Token-2022,
            // require a unique incoming edge and reconstruct the credited net.
            let output_program = a[usize::from(a_to_b)];
            let net_out = if output_program == "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" {
                gross_out
            } else {
                assert_eq!(moves.iter().filter(|(_, dst, _)| *dst == owner_out).count(), 1);
                let debits: i128 = moves
                    .iter()
                    .filter(|(src, _, _)| *src == owner_out)
                    .map(|(_, _, amount)| i128::from(*amount))
                    .sum();
                u64::try_from(delta(owner_out) + debits).unwrap()
            };
            let mut fields = json!({"whirlpool":bytes(a[4]), "a_to_b":a_to_b,
                "input_amount":gross_in, "output_amount":gross_out,
                "input_transfer_fee":gross_in.checked_sub(net_in).unwrap(),
                "output_transfer_fee":gross_out.checked_sub(net_out).unwrap(),
                "amount":u64::from_le_bytes(data[8..16].try_into().unwrap()),
                "other_amount_threshold":u64::from_le_bytes(data[16..24].try_into().unwrap()),
                "amount_specified_is_input":data[40] == 1, "ix_name":"swap_v2"});
            for (field, index) in [
                ("token_program_a", 0),
                ("token_program_b", 1),
                ("token_authority", 3),
                ("token_mint_a", 5),
                ("token_mint_b", 6),
                ("token_owner_account_a", 7),
                ("token_vault_a", 8),
                ("token_owner_account_b", 9),
                ("token_vault_b", 10),
                ("tick_array_0", 11),
                ("tick_array_1", 12),
                ("tick_array_2", 13),
                ("oracle", 14),
            ] {
                fields[field] = json!(bytes(a[index]));
            }
            (
                if streamer { "OrcaWhirlpoolSwapEvent" } else { "OrcaWhirlpoolSwap" },
                fields,
                "whirlpool",
            )
        } else if *program == "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK"
            && disc == [248, 198, 158, 145, 225, 117, 135, 200]
        {
            let mut fields = json!({"pool_state":bytes(a[2]), "sender":bytes(a[0]),
                "amount":u64::from_le_bytes(data[8..16].try_into().unwrap()),
                "other_amount_threshold":u64::from_le_bytes(data[16..24].try_into().unwrap()),
                "sqrt_price_limit_x64":u128::from_le_bytes(data[24..40].try_into().unwrap()),
                "is_base_input":data[40] == 1, "ix_name":"swap"});
            for (field, index) in [
                ("amm_config", 1),
                ("input_vault", 5),
                ("output_vault", 6),
                ("observation_state", 7),
            ] {
                fields[field] = json!(bytes(a[index]));
            }
            let mint = |account: &str| {
                wire["meta"]["preTokenBalances"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|balance| {
                        keys[balance["accountIndex"].as_u64().unwrap() as usize] == account
                    })
                    .unwrap()["mint"]
                    .as_str()
                    .unwrap()
            };
            // This captured v1 swap uses classic SPL tokens. Canonical mint
            // ordering determines direction independently of the event log.
            assert_eq!(a[8], "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
            let zero_for_one = bytes(mint(a[5])) < bytes(mint(a[6]));
            let gross_in = movement(a[3], a[5]);
            let gross_out = movement(a[6], a[4]);
            fields["zero_for_one"] = json!(zero_for_one);
            fields["amount_0"] = json!(if zero_for_one { gross_in } else { gross_out });
            fields["amount_1"] = json!(if zero_for_one { gross_out } else { gross_in });
            fields["transfer_fee_0"] = json!(0);
            fields["transfer_fee_1"] = json!(0);
            fields["input_token_account"] = json!(bytes(a[3]));
            fields["output_token_account"] = json!(bytes(a[4]));
            if streamer {
                fields["payer"] = json!(bytes(a[0]));
                fields["input_token_account"] = json!(bytes(a[3]));
                fields["output_token_account"] = json!(bytes(a[4]));
            }
            (
                if streamer { "RaydiumClmmSwapEvent" } else { "RaydiumClmmSwap" },
                fields,
                "pool_state",
            )
        } else if *program == "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C"
            && disc == [20, 22, 86, 123, 198, 28, 219, 132]
        {
            assert_eq!(a.len(), 15, "upgraded collection must include the share PDA");
            let mut fields = json!({"permissionless":false});
            for (field, index) in [
                ("payer", 0),
                ("creator", 0),
                ("authority", 1),
                ("pool_state", 2),
                ("amm_config", 3),
                ("token_0_vault", 4),
                ("token_1_vault", 5),
                ("vault_0_mint", 6),
                ("vault_1_mint", 7),
                ("creator_token_0", 8),
                ("creator_token_1", 9),
                ("token_0_program", 10),
                ("token_1_program", 11),
                ("associated_token_program", 12),
                ("system_program", 13),
                ("creator_fee_share", 14),
            ] {
                fields[field] = json!(bytes(a[index]));
            }
            let key = |index: usize| {
                solana_sdk::pubkey::Pubkey::new_from_array(bytes(a[index]).try_into().unwrap())
            };
            let program_key =
                solana_sdk::pubkey::Pubkey::new_from_array(bytes(program).try_into().unwrap());
            let (share, _) = solana_sdk::pubkey::Pubkey::find_program_address(
                &[b"creator_fee_share", key(0).as_ref(), key(3).as_ref()],
                &program_key,
            );
            assert_eq!(share, key(14), "collection share PDA seeds");
            (
                if streamer {
                    "RaydiumCpmmCollectCreatorFeeEvent"
                } else {
                    "RaydiumCpmmCollectCreatorFee"
                },
                fields,
                "pool_state",
            )
        } else if *program == "pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA"
            && disc == [198, 46, 21, 82, 180, 217, 232, 112]
        {
            let pool_credit = movement(a[6], a[8]);
            let creator_fee = movement(a[6], a[17]);
            let buyback_fee = movement(a[6], a[25]);
            let protocol_fee = movement(a[6], a[10]) + buyback_fee;
            let gross_quote: u64 =
                moves.iter().filter(|(src, _, _)| *src == a[6]).map(|(_, _, amount)| amount).sum();
            assert_eq!(gross_quote, pool_credit + creator_fee + protocol_fee);
            assert_eq!(gross_quote, u64::from_le_bytes(data[8..16].try_into().unwrap()));
            let gross_base = movement(a[7], a[5]);
            assert!(gross_base >= u64::from_le_bytes(data[16..24].try_into().unwrap()));
            assert_eq!(
                delta(a[5]),
                i128::from(gross_base),
                "captured base output has no transfer tax"
            );
            // Read the dynamic LP rate from the raw Anchor event instruction,
            // independently of SDK decoding, then invert the pool credit.
            let raw_buy = resolved
                .iter()
                .find(|(pg, d, _)| {
                    *pg == *program
                        && d.get(..8) == Some(&[228, 69, 165, 46, 81, 203, 154, 29])
                        && d.get(8..16) == Some(&[103, 244, 82, 31, 44, 245, 119, 119])
                })
                .unwrap();
            let lp_bps = u64::from_le_bytes(raw_buy.1[80..88].try_into().unwrap());
            let curve_quote =
                u64::try_from(u128::from(pool_credit) * 10_000 / (10_000 + u128::from(lp_bps)))
                    .unwrap();
            let lp_fee = pool_credit - curve_quote;
            assert_eq!(
                u128::from(lp_fee),
                (u128::from(curve_quote) * u128::from(lp_bps)).div_ceil(10_000)
            );
            let mut fields = json!({"base_amount_out":gross_base,
                "max_quote_amount_in":gross_quote, "min_base_amount_out":u64::from_le_bytes(data[16..24].try_into().unwrap()),
                "quote_amount_in":gross_quote, "quote_amount_in_with_lp_fee":pool_credit,
                "user_quote_amount_in":curve_quote, "lp_fee":lp_fee, "lp_fee_basis_points":lp_bps,
                "protocol_fee":protocol_fee, "coin_creator_fee":creator_fee,
                "buyback_fee":buyback_fee, "ix_name":"buy_exact_quote_in"});
            for (field, index) in [
                ("pool", 0),
                ("user", 1),
                ("base_mint", 3),
                ("quote_mint", 4),
                ("user_base_token_account", 5),
                ("user_quote_token_account", 6),
                ("pool_base_token_account", 7),
                ("pool_quote_token_account", 8),
                ("protocol_fee_recipient", 9),
                ("protocol_fee_recipient_token_account", 10),
                ("base_token_program", 11),
                ("quote_token_program", 12),
                ("coin_creator_vault_ata", 17),
                ("coin_creator_vault_authority", 18),
                ("pool_v2", 23),
                ("fee_recipient", 24),
                ("fee_recipient_quote_token_account", 25),
            ] {
                fields[field] = json!(bytes(a[index]));
            }
            (if streamer { "PumpSwapBuyEvent" } else { "PumpSwapBuy" }, fields, "pool")
        } else if *program == "pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA"
            && disc == [51, 230, 133, 164, 1, 127, 131, 173]
        {
            let mut fields = json!({"base_amount_in": movement(a[5],a[7]), "user_quote_amount_out":movement(a[8],a[6]),
                "min_quote_amount_out":u64::from_le_bytes(data[16..24].try_into().unwrap())});
            for (field, index) in [
                ("pool", 0),
                ("user", 1),
                ("base_mint", 3),
                ("quote_mint", 4),
                ("user_base_token_account", 5),
                ("user_quote_token_account", 6),
                ("pool_base_token_account", 7),
                ("pool_quote_token_account", 8),
            ] {
                fields[field] = json!(bytes(a[index]));
            }
            (if streamer { "PumpSwapSellEvent" } else { "PumpSwapSell" }, fields, "pool")
        } else if *program == "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo"
            && disc == [248, 198, 158, 145, 225, 117, 135, 200]
        {
            let fields = json!({"pool":bytes(a[0]), "token_x_mint":bytes(a[6]), "token_y_mint":bytes(a[7]),
                "amount_in":movement(a[4],a[2]), "amount_out":movement(a[3],a[5]),
                "min_amount_out":u64::from_le_bytes(data[16..24].try_into().unwrap())});
            (if streamer { "MeteoraDlmmSwapEvent" } else { "MeteoraDlmmSwap" }, fields, "pool")
        } else if *program == "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P"
            && disc == [93, 246, 130, 60, 231, 233, 64, 178]
        {
            let lamport_delta = |account: &str| -> i128 {
                let index = keys.iter().position(|key| key == account).unwrap();
                i128::from(wire["meta"]["postBalances"][index].as_u64().unwrap())
                    - i128::from(wire["meta"]["preBalances"][index].as_u64().unwrap())
            };
            let gross_quote = u64::try_from(-lamport_delta(a[10])).unwrap();
            let buyback = u64::try_from(lamport_delta(a[8])).unwrap();
            let fee = u64::try_from(lamport_delta(a[6])).unwrap() + buyback;
            let creator_fee = u64::try_from(lamport_delta(a[16])).unwrap();
            let base = movement(a[14], a[11]);
            assert_eq!(base, u64::from_le_bytes(data[8..16].try_into().unwrap()));
            assert!(
                gross_quote - fee - creator_fee
                    >= u64::from_le_bytes(data[16..24].try_into().unwrap())
            );
            let mut fields = json!({"mint":bytes(a[1]),"user":bytes(a[13]),
                "token_amount":base, "amount":base, "is_buy":false,
                "sol_amount":gross_quote, "quote_amount":gross_quote,
                "fee":fee, "buyback_fee":buyback, "creator_fee":creator_fee,
                "min_sol_output":u64::from_le_bytes(data[16..24].try_into().unwrap())});
            for (field, index) in [
                ("global", 0),
                ("quote_mint", 2),
                ("token_program", 3),
                ("quote_token_program", 4),
                ("associated_token_program", 5),
                ("fee_recipient", 6),
                ("associated_quote_fee_recipient", 7),
                ("buyback_fee_recipient", 8),
                ("associated_quote_buyback_fee_recipient", 9),
                ("bonding_curve", 10),
                ("associated_bonding_curve", 11),
                ("associated_quote_bonding_curve", 12),
                ("associated_user", 14),
                ("associated_quote_user", 15),
                ("creator_vault", 16),
                ("associated_creator_vault", 17),
                ("sharing_config", 18),
                ("user_volume_accumulator", 19),
                ("associated_user_volume_accumulator", 20),
                ("fee_config", 21),
                ("fee_program", 22),
                ("system_program", 23),
                ("event_authority", 24),
                ("program", 25),
            ] {
                fields[field] = json!(bytes(a[index]));
            }
            (if streamer { "PumpFunTradeEvent" } else { "PumpFunSell" }, fields, "mint")
        } else if *program == "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P"
            && disc == [102, 6, 61, 18, 1, 218, 235, 234]
        {
            let fields = json!({"mint":bytes(a[2]),"bonding_curve":bytes(a[3]),"user":bytes(a[6]),"token_amount":movement(a[4],a[5])});
            (if streamer { "PumpFunTradeEvent" } else { "PumpFunBuy" }, fields, "mint")
        } else {
            continue;
        };
        *expected_counts.entry(kind).or_default() += 1;
        let found: Vec<_> = all
            .iter()
            .filter_map(|e| e.get(kind))
            .filter(|e| e[pool_field] == fields[pool_field])
            .collect();
        assert_eq!(found.len(), 1, "{name}: missing/duplicate {kind}");
        for (field, expected) in fields.as_object().unwrap() {
            assert_eq!(&found[0][field], expected, "{name} {kind}.{field}");
        }
        checked += 1;
    }
    // Reject extra events for the instruction families this oracle checks,
    // including a phantom family that never executed in this transaction.
    for kind in if streamer {
        vec![
            "RaydiumCpmmSwapEvent",
            "PumpSwapSellEvent",
            "MeteoraDlmmSwapEvent",
            "PumpFunTradeEvent",
            "RaydiumCpmmCollectCreatorFeeEvent",
            "OrcaWhirlpoolSwapEvent",
            "RaydiumClmmSwapEvent",
            "PumpSwapBuyEvent",
        ]
    } else {
        vec![
            "RaydiumCpmmSwap",
            "PumpSwapSell",
            "MeteoraDlmmSwap",
            "PumpFunBuy",
            "RaydiumCpmmCollectCreatorFee",
            "OrcaWhirlpoolSwap",
            "RaydiumClmmSwap",
            "PumpSwapBuy",
            "PumpFunSell",
        ]
    } {
        assert_eq!(
            all.iter().filter(|event| event.get(kind).is_some()).count(),
            expected_counts.get(kind).copied().unwrap_or(0),
            "{name}: unexpected number of {kind} events"
        );
    }
    if name == "dlmm_0" {
        assert!(all.is_empty(), "referencing a program is not executing it");
    } else {
        assert!(checked > 0, "oracle must exercise a real DEX instruction");
    }
    checked
}

/// Display a checked fee decomposition without confusing curve input with
/// the user's total debit, or counting the buyback split twice.
#[allow(dead_code)]
pub fn print_buy_details(events: &Value, streamer: bool) {
    let kind = if streamer { "PumpSwapBuyEvent" } else { "PumpSwapBuy" };
    for event in events.as_array().unwrap() {
        if let Some(buy) = event.get(kind).filter(|buy| buy["ix_name"] == "buy_exact_quote_in") {
            println!("  base_out={} quote_total={} curve_quote={} pool_quote={} lp={} protocol_total={} (buyback={}) creator={}",
                buy["base_amount_out"], buy["quote_amount_in"], buy["user_quote_amount_in"],
                buy["quote_amount_in_with_lp_fee"], buy["lp_fee"], buy["protocol_fee"],
                buy["buyback_fee"], buy["coin_creator_fee"]);
        }
    }
}
