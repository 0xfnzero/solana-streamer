//! `sol-parser-sdk` to streamer [`DexEvent`](crate::streaming::event_parser::DexEvent) mapping.
//!
//! The bridge is split by responsibility so the conversion layer stays reviewable.
//!
//! | Module | Responsibility |
//! |------|------|
//! | [`adapt`] | block_time / recv_us alignment with [`EventMetadata`] |
//! | [`program_ids`] | protocol program pubkeys |
//! | [`filter`] | subscribed [`Protocol`](crate::streaming::event_parser::Protocol) checks |
//! | [`pump_pumpswap`] | PumpFun and PumpSwap field mapping |
//! | [`bonk_accounts`] | Bonk plus Token / Nonce / PumpSwap account events |
//! | [`raydium_and_damm`] | Raydium lines and Meteora DAMM v2 |
//! | [`forward_pb`] | Orca / Meteora Pools / Meteora DLMM SDK-shaped events |
//! | [`convert`] | `PbDexEvent` dispatch and batch adaptation |
//! | [`accounts`] | SDK account parser compatibility |

mod accounts;
mod adapt;
mod bonk_accounts;
mod convert;
mod filter;
mod forward_pb;
mod program_ids;
mod pump_pumpswap;
mod raydium_and_damm;

pub(crate) use accounts::{
    parse_account_event as parse_sdk_account_event, parse_account_event_for_streamer,
    AccountParseResult,
};
pub(crate) use adapt::{block_timestamp_from_stream_meta, fuse_streamer_ix_ctx};
pub(crate) use convert::{adapt_parser_event, adapt_parser_events_list, convert_parser_event};

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::filter::event_matches_protocol;
    use super::{adapt_parser_event, convert_parser_event};
    use crate::streaming::event_parser::common::filter::EventTypeFilter;
    use crate::streaming::event_parser::common::types::{EventType, ProtocolType};
    use crate::streaming::event_parser::core::account_event_parser::TokenInfoEvent;
    use crate::streaming::event_parser::{DexEvent, Protocol};
    use sol_parser_sdk::core::events::{
        EventMetadata, MeteoraDlmmSwapEvent as PbDlmmSwap, OrcaWhirlpoolSwapEvent as PbOrcaSwap,
        PumpFunCreateV2TokenEvent as PbPumpCreateV2, PumpFunTradeEvent as PbPumpTrade,
        PumpSwapCreatePoolEvent as PbPumpSwapCreatePool, PumpSwapPool as PbPumpSwapPool,
        PumpSwapPoolAccountEvent as PbPumpSwapPoolAccount,
        RaydiumLaunchlabTradeEvent as PbBonkTrade, TokenInfoEvent as PbTokenInfo,
        TradeDirection as PbBonkDir,
    };
    use sol_parser_sdk::DexEvent as PbDexEvent;
    use solana_sdk::{pubkey::Pubkey, signature::Signature};

    #[test]
    fn converts_pumpfun_trade_preserving_amounts() {
        let mut t = PbPumpTrade::default();
        t.metadata = EventMetadata {
            signature: Signature::default(),
            slot: 42,
            tx_index: 7,
            block_time_us: 1_000_000,
            grpc_recv_us: 88,
            recent_blockhash: None,
        };
        t.mint = Pubkey::new_unique();
        t.user = Pubkey::new_unique();
        t.sol_amount = 100;
        t.token_amount = 200;
        t.amount = 200;
        t.max_sol_cost = 150;
        t.min_sol_output = 0;
        t.spendable_sol_in = 11;
        t.spendable_quote_in = 12;
        t.min_tokens_out = 13;
        t.is_created_buy = true;
        t.global = Pubkey::new_unique();
        t.bonding_curve_v2 = Pubkey::new_unique();
        t.associated_user = Pubkey::new_unique();
        t.system_program = Pubkey::new_unique();
        t.event_authority = Pubkey::new_unique();
        t.program = Pubkey::new_unique();
        t.global_volume_accumulator = Pubkey::new_unique();
        t.user_volume_accumulator = Pubkey::new_unique();
        t.associated_creator_vault = Pubkey::new_unique();
        t.sharing_config = Pubkey::new_unique();
        t.fee_config = Pubkey::new_unique();
        t.fee_program = Pubkey::new_unique();
        t.quote_mint = Pubkey::new_unique();
        t.quote_amount = 14;
        t.virtual_quote_reserves = 15;
        t.real_quote_reserves = 16;
        t.is_buy = true;

        let ev = convert_parser_event(PbDexEvent::PumpFunTrade(t), None, 999).expect("convert");
        match ev {
            DexEvent::PumpFunTradeEvent(st) => {
                assert_eq!(st.metadata.slot, 42);
                assert_eq!(st.metadata.event_type, EventType::PumpFunTrade);
                assert_eq!(st.metadata.recv_us, 999);
                assert_eq!(st.sol_amount, 100);
                assert_eq!(st.token_amount, 200);
                assert_eq!(st.amount, 200);
                assert_eq!(st.max_sol_cost, 150);
                assert_eq!(st.min_sol_output, 0);
                assert_eq!(st.spendable_sol_in, 11);
                assert_eq!(st.spendable_quote_in, 12);
                assert_eq!(st.min_tokens_out, 13);
                assert!(st.is_created_buy);
                assert!(st.is_dev_create_token_trade);
                assert_ne!(st.global, Pubkey::default());
                assert_ne!(st.bonding_curve_v2, Pubkey::default());
                assert_ne!(st.associated_user, Pubkey::default());
                assert_ne!(st.system_program, Pubkey::default());
                assert_ne!(st.event_authority, Pubkey::default());
                assert_ne!(st.program, Pubkey::default());
                assert_ne!(st.global_volume_accumulator, Pubkey::default());
                assert_ne!(st.user_volume_accumulator, Pubkey::default());
                assert_ne!(st.associated_creator_vault, Pubkey::default());
                assert_ne!(st.sharing_config, Pubkey::default());
                assert_ne!(st.fee_config, Pubkey::default());
                assert_ne!(st.fee_program, Pubkey::default());
                assert_ne!(st.quote_mint, Pubkey::default());
                assert_eq!(st.quote_amount, 14);
                assert_eq!(st.virtual_quote_reserves, 15);
                assert_eq!(st.real_quote_reserves, 16);
                assert!(st.is_buy);
            }
            _ => panic!("expected PumpFunTradeEvent"),
        }
    }

    #[test]
    fn converts_pumpfun_buy_exact_sol_in_preserving_event_type() {
        let mut t = PbPumpTrade::default();
        t.metadata = EventMetadata::default();
        t.is_buy = true;

        let ev =
            convert_parser_event(PbDexEvent::PumpFunBuyExactSolIn(t), None, 999).expect("convert");
        match ev {
            DexEvent::PumpFunTradeEvent(st) => {
                assert_eq!(st.metadata.event_type, EventType::PumpFunBuyExactSolIn);
                assert!(st.is_buy);
            }
            _ => panic!("expected PumpFunTradeEvent"),
        }
    }

    #[test]
    fn converts_pumpfun_create_v2_as_canonical_create() {
        let mint = Pubkey::new_unique();
        let global = Pubkey::new_unique();
        let event_authority = Pubkey::new_unique();
        let quote_mint = sol_parser_sdk::core::events::PUMPFUN_WSOL_QUOTE_MINT;
        let c = PbPumpCreateV2 {
            metadata: EventMetadata {
                signature: Signature::default(),
                slot: 42,
                tx_index: 7,
                block_time_us: 1_000_000,
                grpc_recv_us: 88,
                recent_blockhash: None,
            },
            name: "Token".to_string(),
            symbol: "TOK".to_string(),
            mint,
            global,
            event_authority,
            quote_mint,
            is_mayhem_mode: true,
            is_cashback_enabled: true,
            ..Default::default()
        };

        let ev = convert_parser_event(PbDexEvent::PumpFunCreateV2(c), None, 999).expect("convert");
        match ev {
            DexEvent::PumpFunCreateTokenEvent(st) => {
                assert_eq!(st.metadata.event_type, EventType::PumpFunCreateToken);
                assert_eq!(st.mint, mint);
                assert_eq!(st.global, global);
                assert_eq!(st.event_authority, event_authority);
                assert_eq!(st.quote_mint, quote_mint);
                assert_eq!(st.ix_name, "create_v2");
                assert!(st.is_mayhem_mode);
                assert!(st.is_cashback_enabled);
            }
            other => panic!("expected canonical PumpFunCreateTokenEvent, got {other:?}"),
        }
    }

    #[test]
    fn converts_pumpfun_buy_exact_quote_in_v2_as_buy_event() {
        let mut t = PbPumpTrade::default();
        t.metadata = EventMetadata::default();
        t.is_buy = true;
        t.ix_name = "buy_exact_quote_in_v2".to_string();

        let ev = convert_parser_event(PbDexEvent::PumpFunBuy(t), None, 999).expect("convert");
        match ev {
            DexEvent::PumpFunTradeEvent(st) => {
                assert_eq!(st.metadata.event_type, EventType::PumpFunBuy);
                assert_eq!(st.ix_name, "buy_exact_quote_in_v2");
            }
            _ => panic!("expected PumpFunTradeEvent"),
        }
    }

    #[test]
    fn converts_pumpswap_create_pool_flags() {
        let c = PbPumpSwapCreatePool {
            metadata: EventMetadata {
                signature: Signature::default(),
                slot: 42,
                tx_index: 7,
                block_time_us: 1_000_000,
                grpc_recv_us: 88,
                recent_blockhash: None,
            },
            pool: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            is_mayhem_mode: true,
            is_cashback_coin: true,
            ..Default::default()
        };

        let ev =
            convert_parser_event(PbDexEvent::PumpSwapCreatePool(c), None, 999).expect("convert");
        match ev {
            DexEvent::PumpSwapCreatePoolEvent(st) => {
                assert_eq!(st.metadata.event_type, EventType::PumpSwapCreatePool);
                assert!(st.is_mayhem_mode);
                assert!(st.is_cashback_coin);
            }
            _ => panic!("expected PumpSwapCreatePoolEvent"),
        }
    }

    #[test]
    fn converts_pumpswap_pool_account_cashback_flag() {
        let pool_pubkey = Pubkey::new_unique();
        let e = PbPumpSwapPoolAccount {
            metadata: EventMetadata {
                signature: Signature::default(),
                slot: 42,
                tx_index: 7,
                block_time_us: 1_000_000,
                grpc_recv_us: 88,
                recent_blockhash: None,
            },
            pubkey: pool_pubkey,
            executable: false,
            lamports: 123,
            owner: Pubkey::new_unique(),
            rent_epoch: 0,
            pool: PbPumpSwapPool {
                pool_bump: 7,
                index: 42,
                creator: Pubkey::new_unique(),
                base_mint: Pubkey::new_unique(),
                quote_mint: Pubkey::new_unique(),
                lp_mint: Pubkey::new_unique(),
                pool_base_token_account: Pubkey::new_unique(),
                pool_quote_token_account: Pubkey::new_unique(),
                lp_supply: 999,
                coin_creator: Pubkey::new_unique(),
                is_mayhem_mode: true,
                is_cashback_coin: true,
            },
        };

        let ev =
            convert_parser_event(PbDexEvent::PumpSwapPoolAccount(e), None, 999).expect("convert");
        match ev {
            DexEvent::PumpSwapPoolAccountEvent(st) => {
                assert_eq!(st.metadata.event_type, EventType::AccountPumpSwapPool);
                assert_eq!(st.pubkey, pool_pubkey);
                assert!(st.pool.is_mayhem_mode);
                assert!(st.pool.is_cashback_coin);
            }
            _ => panic!("expected PumpSwapPoolAccountEvent"),
        }
    }

    #[test]
    fn protocol_filter_keeps_pumpfun_only_when_requested() {
        let mut t = PbPumpTrade::default();
        t.metadata = EventMetadata::default();
        let dex = convert_parser_event(PbDexEvent::PumpFunTrade(t), None, 0).expect("convert");
        assert!(event_matches_protocol(&[Protocol::PumpFun], &dex));
        assert!(!event_matches_protocol(&[Protocol::PumpSwap], &dex));
    }

    #[test]
    fn converts_parser_sdk_error_preserves_message() {
        let ev = convert_parser_event(PbDexEvent::Error("decode failed".into()), None, 404)
            .expect("convert");
        match ev {
            DexEvent::ParserSdkErrorEvent(e) => {
                assert_eq!(e.message, "decode failed");
                assert_eq!(e.metadata.recv_us, 404);
                assert_eq!(e.metadata.protocol, ProtocolType::Common);
                assert_eq!(e.metadata.event_type, EventType::ParserSdkError);
            }
            _ => panic!("expected ParserSdkErrorEvent"),
        }
    }

    #[test]
    fn converts_token_info_preserving_event_type() {
        let ev = convert_parser_event(PbDexEvent::TokenInfo(PbTokenInfo::default()), None, 404)
            .expect("convert");
        match ev {
            DexEvent::TokenInfoEvent(e) => {
                assert_eq!(e.metadata.event_type, EventType::TokenInfo);
                assert_eq!(e.metadata.protocol, ProtocolType::Common);
            }
            _ => panic!("expected TokenInfoEvent"),
        }
    }

    #[test]
    fn protocol_filter_allows_protocol_independent_events() {
        let mut token_info = TokenInfoEvent::default();
        token_info.metadata.event_type = EventType::TokenInfo;
        token_info.metadata.protocol = ProtocolType::Common;
        let dex = DexEvent::TokenInfoEvent(token_info);

        assert!(event_matches_protocol(&[Protocol::PumpFun], &dex));
        assert!(event_matches_protocol(&[Protocol::RaydiumCpmm], &dex));
    }

    #[test]
    fn adapt_token_info_passes_protocol_and_token_account_filter() {
        let filter = EventTypeFilter::include_only([EventType::TokenAccount]);
        let ev = adapt_parser_event(
            PbDexEvent::TokenInfo(PbTokenInfo::default()),
            None,
            404,
            &[Protocol::PumpFun],
            Some(&filter),
        )
        .expect("token info should pass common protocol and token-account filter");

        assert!(matches!(ev, DexEvent::TokenInfoEvent(_)));
        assert_eq!(ev.metadata().event_type, EventType::TokenInfo);
    }

    #[test]
    fn converts_orca_whirlpool_swap_preserving_amounts() {
        let whirlpool = Pubkey::new_unique();
        let pb = PbOrcaSwap {
            metadata: EventMetadata {
                signature: Signature::default(),
                slot: 9,
                tx_index: 0,
                block_time_us: 0,
                grpc_recv_us: 0,
                recent_blockhash: None,
            },
            whirlpool,
            input_amount: 10,
            output_amount: 20,
            a_to_b: false,
            pre_sqrt_price: 100,
            post_sqrt_price: 200,
            input_transfer_fee: 1,
            output_transfer_fee: 2,
            lp_fee: 3,
            protocol_fee: 4,
        };
        let ev =
            convert_parser_event(PbDexEvent::OrcaWhirlpoolSwap(pb), None, 111).expect("convert");
        match ev {
            DexEvent::OrcaWhirlpoolSwapEvent(e) => {
                assert_eq!(e.whirlpool, whirlpool);
                assert_eq!(e.input_amount, 10);
                assert_eq!(e.output_amount, 20);
                assert!(!e.a_to_b);
                assert_eq!(e.pre_sqrt_price, 100);
                assert_eq!(e.post_sqrt_price, 200);
                assert_eq!(e.lp_fee, 3);
                assert_eq!(e.protocol_fee, 4);
                assert_eq!(e.metadata.slot, 9);
                assert_eq!(e.metadata.recv_us, 111);
            }
            _ => panic!("expected OrcaWhirlpoolSwapEvent"),
        }
    }

    #[test]
    fn converts_meteora_dlmm_swap_preserving_amounts() {
        let pool = Pubkey::new_unique();
        let from = Pubkey::new_unique();
        let pb = PbDlmmSwap {
            metadata: EventMetadata::default(),
            pool,
            from,
            start_bin_id: -5,
            end_bin_id: 12,
            amount_in: 300,
            amount_out: 299,
            swap_for_y: true,
            fee: 1,
            protocol_fee: 2,
            fee_bps: 25,
            host_fee: 0,
        };
        let ev = convert_parser_event(PbDexEvent::MeteoraDlmmSwap(pb), None, 0).expect("convert");
        match ev {
            DexEvent::MeteoraDlmmSwapEvent(e) => {
                assert_eq!(e.pool, pool);
                assert_eq!(e.from, from);
                assert_eq!(e.start_bin_id, -5);
                assert_eq!(e.end_bin_id, 12);
                assert_eq!(e.amount_in, 300);
                assert_eq!(e.amount_out, 299);
                assert!(e.swap_for_y);
                assert_eq!(e.fee_bps, 25);
            }
            _ => panic!("expected MeteoraDlmmSwapEvent"),
        }
    }

    #[test]
    fn protocol_filter_keeps_orca_only_when_requested() {
        let pb = PbOrcaSwap {
            metadata: EventMetadata::default(),
            whirlpool: Pubkey::new_unique(),
            input_amount: 0,
            output_amount: 0,
            a_to_b: true,
            pre_sqrt_price: 0,
            post_sqrt_price: 0,
            input_transfer_fee: 0,
            output_transfer_fee: 0,
            lp_fee: 0,
            protocol_fee: 0,
        };
        let dex =
            convert_parser_event(PbDexEvent::OrcaWhirlpoolSwap(pb), None, 0).expect("convert");
        assert!(event_matches_protocol(&[Protocol::OrcaWhirlpool], &dex));
        assert!(!event_matches_protocol(&[Protocol::PumpFun], &dex));
    }

    #[test]
    fn converts_bonk_trade_maps_event_type_buy_exact_in() {
        let b = PbBonkTrade {
            metadata: EventMetadata::default(),
            pool_state: Pubkey::default(),
            user: Pubkey::default(),
            amount_in: 0,
            amount_out: 0,
            is_buy: true,
            trade_direction: PbBonkDir::Buy,
            exact_in: true,
        };
        let dex =
            convert_parser_event(PbDexEvent::RaydiumLaunchlabTrade(b), None, 0).expect("convert");
        match dex {
            DexEvent::BonkTradeEvent(e) => {
                assert_eq!(e.metadata.event_type, EventType::BonkBuyExactIn)
            }
            _ => panic!("expected BonkTradeEvent"),
        }
    }

    #[test]
    fn converts_bonk_trade_maps_event_type_sell_exact_out() {
        let b = PbBonkTrade {
            metadata: EventMetadata::default(),
            pool_state: Pubkey::default(),
            user: Pubkey::default(),
            amount_in: 0,
            amount_out: 0,
            is_buy: false,
            trade_direction: PbBonkDir::Sell,
            exact_in: false,
        };
        let dex =
            convert_parser_event(PbDexEvent::RaydiumLaunchlabTrade(b), None, 0).expect("convert");
        match dex {
            DexEvent::BonkTradeEvent(e) => {
                assert_eq!(e.metadata.event_type, EventType::BonkSellExactOut)
            }
            _ => panic!("expected BonkTradeEvent"),
        }
    }
}
