//! Main dispatch from [`sol_parser_sdk::DexEvent`] to streamer [`DexEvent`].
use crate::streaming::event_parser::common::filter::passes_event_type_filter;
use crate::streaming::event_parser::common::types::{EventType, ProtocolType};
use crate::streaming::event_parser::common::EventMetadata;
use crate::streaming::event_parser::protocols::block::block_meta_event::BlockMetaEvent;
use crate::streaming::event_parser::protocols::sol_parser_forward::events::ParserSdkErrorEvent;
use crate::streaming::event_parser::{DexEvent, Protocol};
use prost_types::Timestamp;
use sol_parser_sdk::DexEvent as PbDexEvent;
use solana_sdk::pubkey::Pubkey;

use super::adapt::adapt_pm;
use super::bonk_accounts::*;
use super::filter::event_matches_protocol;
use super::forward_pb::*;
use super::program_ids::*;
use super::pump_pumpswap::*;
use super::raydium_and_damm::*;

pub(crate) fn convert_parser_event(
    ev: PbDexEvent,
    bt: Option<&Timestamp>,
    recv_wall_us: i64,
) -> Option<DexEvent> {
    match ev {
        PbDexEvent::PumpFunTrade(t) => Some(pumpfun_trade_from_parser(t, bt, recv_wall_us)),
        PbDexEvent::PumpFunBuy(t) => Some(pumpfun_trade_from_parser_with_event_type(
            t,
            bt,
            recv_wall_us,
            EventType::PumpFunBuy,
        )),
        PbDexEvent::PumpFunSell(t) => Some(pumpfun_trade_from_parser_with_event_type(
            t,
            bt,
            recv_wall_us,
            EventType::PumpFunSell,
        )),
        PbDexEvent::PumpFunBuyExactSolIn(t) => Some(pumpfun_trade_from_parser_with_event_type(
            t,
            bt,
            recv_wall_us,
            EventType::PumpFunBuyExactSolIn,
        )),

        PbDexEvent::PumpFunCreate(c) => {
            let meta = adapt_pm(
                c.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::PumpFun,
                EventType::PumpFunCreateToken,
                pump_program(),
            );
            Some(DexEvent::PumpFunCreateTokenEvent(pumpfun_create_token_from_parser(c, meta)))
        }
        PbDexEvent::PumpFunCreateV2(c) => {
            let meta = adapt_pm(
                c.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::PumpFun,
                EventType::PumpFunCreateToken,
                pump_program(),
            );
            Some(DexEvent::PumpFunCreateTokenEvent(pumpfun_create_token_from_parser_v2(c, meta)))
        }
        PbDexEvent::PumpFunMigrate(m) => {
            let meta = adapt_pm(
                m.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::PumpFun,
                EventType::PumpFunMigrate,
                pump_program(),
            );
            Some(DexEvent::PumpFunMigrateEvent(pumpfun_migrate_from_parser(m, meta)))
        }
        PbDexEvent::PumpFeesCreateFeeSharingConfig(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::PumpFees,
                EventType::PumpFeesCreateFeeSharingConfig,
                pump_fees_program(),
            );
            Some(DexEvent::PumpFeesCreateFeeSharingConfigEvent(
                pump_fees_create_sharing_config_from_parser(e, meta),
            ))
        }
        PbDexEvent::PumpFeesInitializeFeeConfig(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::PumpFees,
                EventType::PumpFeesInitializeFeeConfig,
                pump_fees_program(),
            );
            Some(DexEvent::PumpFeesInitializeFeeConfigEvent(
                pump_fees_initialize_fee_config_from_parser(e, meta),
            ))
        }
        PbDexEvent::PumpFeesResetFeeSharingConfig(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::PumpFees,
                EventType::PumpFeesResetFeeSharingConfig,
                pump_fees_program(),
            );
            Some(DexEvent::PumpFeesResetFeeSharingConfigEvent(
                pump_fees_reset_sharing_config_from_parser(e, meta),
            ))
        }
        PbDexEvent::PumpFeesRevokeFeeSharingAuthority(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::PumpFees,
                EventType::PumpFeesRevokeFeeSharingAuthority,
                pump_fees_program(),
            );
            Some(DexEvent::PumpFeesRevokeFeeSharingAuthorityEvent(
                pump_fees_revoke_authority_from_parser(e, meta),
            ))
        }
        PbDexEvent::PumpFeesTransferFeeSharingAuthority(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::PumpFees,
                EventType::PumpFeesTransferFeeSharingAuthority,
                pump_fees_program(),
            );
            Some(DexEvent::PumpFeesTransferFeeSharingAuthorityEvent(
                pump_fees_transfer_authority_from_parser(e, meta),
            ))
        }
        PbDexEvent::PumpFeesUpdateAdmin(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::PumpFees,
                EventType::PumpFeesUpdateAdmin,
                pump_fees_program(),
            );
            Some(DexEvent::PumpFeesUpdateAdminEvent(pump_fees_update_admin_from_parser(e, meta)))
        }
        PbDexEvent::PumpFeesUpdateFeeConfig(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::PumpFees,
                EventType::PumpFeesUpdateFeeConfig,
                pump_fees_program(),
            );
            Some(DexEvent::PumpFeesUpdateFeeConfigEvent(pump_fees_update_fee_config_from_parser(
                e, meta,
            )))
        }
        PbDexEvent::PumpFeesUpdateFeeShares(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::PumpFees,
                EventType::PumpFeesUpdateFeeShares,
                pump_fees_program(),
            );
            Some(DexEvent::PumpFeesUpdateFeeSharesEvent(pump_fees_update_fee_shares_from_parser(
                e, meta,
            )))
        }
        PbDexEvent::PumpFeesUpsertFeeTiers(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::PumpFees,
                EventType::PumpFeesUpsertFeeTiers,
                pump_fees_program(),
            );
            Some(DexEvent::PumpFeesUpsertFeeTiersEvent(pump_fees_upsert_fee_tiers_from_parser(
                e, meta,
            )))
        }
        PbDexEvent::PumpFunMigrateBondingCurveCreator(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::PumpFun,
                EventType::PumpFunMigrateBondingCurveCreator,
                pump_program(),
            );
            Some(DexEvent::PumpFunMigrateBondingCurveCreatorEvent(
                pumpfun_migrate_bonding_creator_from_parser(e, meta),
            ))
        }
        PbDexEvent::PumpFunGlobalAccount(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::PumpFun,
                EventType::AccountPumpFunGlobal,
                pump_program(),
            );
            Some(DexEvent::PumpFunGlobalAccountEvent(pumpfun_global_account_from_parser(e, meta)))
        }
        PbDexEvent::PumpFunBondingCurveAccount(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::PumpFun,
                EventType::AccountPumpFunBondingCurve,
                pump_program(),
            );
            Some(DexEvent::PumpFunBondingCurveAccountEvent(
                pumpfun_bonding_curve_account_from_parser(e, meta),
            ))
        }
        PbDexEvent::PumpFunFeeConfigAccount(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::PumpFun,
                EventType::AccountPumpFunFeeConfig,
                pump_program(),
            );
            Some(DexEvent::PumpFunFeeConfigAccountEvent(pumpfun_fee_config_account_from_parser(
                e, meta,
            )))
        }
        PbDexEvent::PumpFunSharingConfigAccount(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::PumpFun,
                EventType::AccountPumpFunSharingConfig,
                pump_program(),
            );
            Some(DexEvent::PumpFunSharingConfigAccountEvent(
                pumpfun_sharing_config_account_from_parser(e, meta),
            ))
        }
        PbDexEvent::PumpFunGlobalVolumeAccumulatorAccount(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::PumpFun,
                EventType::AccountPumpFunGlobalVolumeAccumulator,
                pump_program(),
            );
            Some(DexEvent::PumpFunGlobalVolumeAccumulatorAccountEvent(
                pumpfun_global_volume_account_from_parser(e, meta),
            ))
        }
        PbDexEvent::PumpFunUserVolumeAccumulatorAccount(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::PumpFun,
                EventType::AccountPumpFunUserVolumeAccumulator,
                pump_program(),
            );
            Some(DexEvent::PumpFunUserVolumeAccumulatorAccountEvent(
                pumpfun_user_volume_account_from_parser(e, meta),
            ))
        }

        PbDexEvent::PumpSwapTrade(t) => pumpswap_trade_from_parser(t, bt, recv_wall_us),
        PbDexEvent::PumpSwapBuy(b) => {
            let meta = adapt_pm(
                b.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::PumpSwap,
                EventType::PumpSwapBuy,
                pumpswap_program(),
            );
            Some(DexEvent::PumpSwapBuyEvent(pumpswap_buy_full_from_parser(b, meta)))
        }
        PbDexEvent::PumpSwapSell(s) => {
            let meta = adapt_pm(
                s.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::PumpSwap,
                EventType::PumpSwapSell,
                pumpswap_program(),
            );
            Some(DexEvent::PumpSwapSellEvent(pumpswap_sell_full_from_parser(s, meta)))
        }
        PbDexEvent::PumpSwapCreatePool(c) => {
            let meta = adapt_pm(
                c.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::PumpSwap,
                EventType::PumpSwapCreatePool,
                pumpswap_program(),
            );
            Some(DexEvent::PumpSwapCreatePoolEvent(pumpswap_create_pool_from_parser(c, meta)))
        }
        PbDexEvent::PumpSwapLiquidityAdded(a) => {
            let meta = adapt_pm(
                a.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::PumpSwap,
                EventType::PumpSwapDeposit,
                pumpswap_program(),
            );
            Some(DexEvent::PumpSwapDepositEvent(pumpswap_liquidity_added_to_deposit(a, meta)))
        }
        PbDexEvent::PumpSwapLiquidityRemoved(r) => {
            let meta = adapt_pm(
                r.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::PumpSwap,
                EventType::PumpSwapWithdraw,
                pumpswap_program(),
            );
            Some(DexEvent::PumpSwapWithdrawEvent(pumpswap_liquidity_removed_to_withdraw(r, meta)))
        }

        PbDexEvent::BonkTrade(b) => {
            let et = sdk_bonk_trade_event_type(&b);
            let meta = adapt_pm(
                b.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::Bonk,
                et,
                bonk_program(),
            );
            Some(DexEvent::BonkTradeEvent(bonk_trade_from_parser(b, meta)))
        }
        PbDexEvent::BonkPoolCreate(p) => {
            let meta = adapt_pm(
                p.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::Bonk,
                EventType::BonkInitialize,
                bonk_program(),
            );
            Some(DexEvent::BonkPoolCreateEvent(bonk_pool_create_from_parser(p, meta)))
        }
        PbDexEvent::BonkMigrateAmm(m) => {
            let meta = adapt_pm(
                m.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::Bonk,
                EventType::BonkMigrateToAmm,
                bonk_program(),
            );
            Some(DexEvent::BonkMigrateToAmmEvent(bonk_migrate_to_amm_from_parser(m, meta)))
        }

        PbDexEvent::RaydiumCpmmSwap(e) => {
            let event_type = if e.base_input {
                EventType::RaydiumCpmmSwapBaseInput
            } else {
                EventType::RaydiumCpmmSwapBaseOutput
            };
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumCpmm,
                event_type,
                raydium_cpmm_program(),
            );
            Some(DexEvent::RaydiumCpmmSwapEvent(raydium_cpmm_swap_from_parser(e, meta)))
        }
        PbDexEvent::RaydiumCpmmDeposit(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumCpmm,
                EventType::RaydiumCpmmDeposit,
                raydium_cpmm_program(),
            );
            Some(DexEvent::RaydiumCpmmDepositEvent(raydium_cpmm_deposit_from_parser(e, meta)))
        }
        PbDexEvent::RaydiumCpmmWithdraw(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumCpmm,
                EventType::RaydiumCpmmWithdraw,
                raydium_cpmm_program(),
            );
            Some(DexEvent::RaydiumCpmmWithdrawEvent(raydium_cpmm_withdraw_from_parser(e, meta)))
        }
        PbDexEvent::RaydiumCpmmInitialize(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumCpmm,
                EventType::RaydiumCpmmInitialize,
                raydium_cpmm_program(),
            );
            Some(DexEvent::RaydiumCpmmInitializeEvent(raydium_cpmm_initialize_from_parser(e, meta)))
        }

        PbDexEvent::RaydiumClmmSwap(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumClmm,
                EventType::RaydiumClmmSwap,
                raydium_clmm_program(),
            );
            Some(DexEvent::RaydiumClmmSwapEvent(raydium_clmm_swap_from_parser(e, meta)))
        }
        PbDexEvent::RaydiumClmmCreatePool(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumClmm,
                EventType::RaydiumClmmCreatePool,
                raydium_clmm_program(),
            );
            Some(DexEvent::RaydiumClmmCreatePoolEvent(raydium_clmm_create_pool_from_parser(
                e, meta,
            )))
        }
        PbDexEvent::RaydiumClmmOpenPosition(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumClmm,
                EventType::RaydiumClmmOpenPositionV2,
                raydium_clmm_program(),
            );
            Some(DexEvent::RaydiumClmmOpenPositionV2Event(
                raydium_clmm_open_position_v2_from_parser(e, meta),
            ))
        }
        PbDexEvent::RaydiumClmmOpenPositionWithTokenExtNft(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumClmm,
                EventType::RaydiumClmmOpenPositionWithToken22Nft,
                raydium_clmm_program(),
            );
            Some(DexEvent::RaydiumClmmOpenPositionWithToken22NftEvent(
                raydium_clmm_open_position_token22_from_parser(e, meta),
            ))
        }
        PbDexEvent::RaydiumClmmClosePosition(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumClmm,
                EventType::RaydiumClmmClosePosition,
                raydium_clmm_program(),
            );
            Some(DexEvent::RaydiumClmmClosePositionEvent(raydium_clmm_close_position_from_parser(
                e, meta,
            )))
        }
        PbDexEvent::RaydiumClmmIncreaseLiquidity(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumClmm,
                EventType::RaydiumClmmIncreaseLiquidityV2,
                raydium_clmm_program(),
            );
            Some(DexEvent::RaydiumClmmIncreaseLiquidityV2Event(
                raydium_clmm_increase_liquidity_v2_from_parser(e, meta),
            ))
        }
        PbDexEvent::RaydiumClmmDecreaseLiquidity(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumClmm,
                EventType::RaydiumClmmDecreaseLiquidityV2,
                raydium_clmm_program(),
            );
            Some(DexEvent::RaydiumClmmDecreaseLiquidityV2Event(
                raydium_clmm_decrease_liquidity_v2_from_parser(e, meta),
            ))
        }
        PbDexEvent::RaydiumClmmLiquidityChange(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumClmm,
                EventType::RaydiumClmmLiquidityChange,
                raydium_clmm_program(),
            );
            Some(DexEvent::RaydiumClmmLiquidityChangeEvent(
                raydium_clmm_liquidity_change_from_parser(e, meta),
            ))
        }
        PbDexEvent::RaydiumClmmConfigChange(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumClmm,
                EventType::RaydiumClmmConfigChange,
                raydium_clmm_program(),
            );
            Some(DexEvent::RaydiumClmmConfigChangeEvent(raydium_clmm_config_change_from_parser(
                e, meta,
            )))
        }
        PbDexEvent::RaydiumClmmCreatePersonalPosition(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumClmm,
                EventType::RaydiumClmmCreatePersonalPosition,
                raydium_clmm_program(),
            );
            Some(DexEvent::RaydiumClmmCreatePersonalPositionEvent(
                raydium_clmm_create_personal_position_from_parser(e, meta),
            ))
        }
        PbDexEvent::RaydiumClmmLiquidityCalculate(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumClmm,
                EventType::RaydiumClmmLiquidityCalculate,
                raydium_clmm_program(),
            );
            Some(DexEvent::RaydiumClmmLiquidityCalculateEvent(
                raydium_clmm_liquidity_calculate_from_parser(e, meta),
            ))
        }
        PbDexEvent::RaydiumClmmOpenLimitOrder(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumClmm,
                EventType::RaydiumClmmOpenLimitOrder,
                raydium_clmm_program(),
            );
            Some(DexEvent::RaydiumClmmOpenLimitOrderEvent(
                raydium_clmm_open_limit_order_from_parser(e, meta),
            ))
        }
        PbDexEvent::RaydiumClmmIncreaseLimitOrder(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumClmm,
                EventType::RaydiumClmmIncreaseLimitOrder,
                raydium_clmm_program(),
            );
            Some(DexEvent::RaydiumClmmIncreaseLimitOrderEvent(
                raydium_clmm_increase_limit_order_from_parser(e, meta),
            ))
        }
        PbDexEvent::RaydiumClmmDecreaseLimitOrder(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumClmm,
                EventType::RaydiumClmmDecreaseLimitOrder,
                raydium_clmm_program(),
            );
            Some(DexEvent::RaydiumClmmDecreaseLimitOrderEvent(
                raydium_clmm_decrease_limit_order_from_parser(e, meta),
            ))
        }
        PbDexEvent::RaydiumClmmSettleLimitOrder(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumClmm,
                EventType::RaydiumClmmSettleLimitOrder,
                raydium_clmm_program(),
            );
            Some(DexEvent::RaydiumClmmSettleLimitOrderEvent(
                raydium_clmm_settle_limit_order_from_parser(e, meta),
            ))
        }
        PbDexEvent::RaydiumClmmUpdateRewardInfos(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumClmm,
                EventType::RaydiumClmmUpdateRewardInfos,
                raydium_clmm_program(),
            );
            Some(DexEvent::RaydiumClmmUpdateRewardInfosEvent(
                raydium_clmm_update_reward_infos_from_parser(e, meta),
            ))
        }
        PbDexEvent::RaydiumClmmCollectFee(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumClmm,
                EventType::RaydiumClmmCollectFee,
                raydium_clmm_program(),
            );
            Some(DexEvent::RaydiumClmmCollectFeeEvent(raydium_clmm_collect_fee_from_parser(
                e, meta,
            )))
        }
        PbDexEvent::RaydiumClmmAmmConfigAccount(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumClmm,
                EventType::AccountRaydiumClmmAmmConfig,
                raydium_clmm_program(),
            );
            Some(DexEvent::RaydiumClmmAmmConfigAccountEvent(
                raydium_clmm_amm_config_account_from_parser(e, meta),
            ))
        }
        PbDexEvent::RaydiumClmmPoolStateAccount(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumClmm,
                EventType::AccountRaydiumClmmPoolState,
                raydium_clmm_program(),
            );
            Some(DexEvent::RaydiumClmmPoolStateAccountEvent(
                raydium_clmm_pool_state_account_from_parser(e, meta),
            ))
        }
        PbDexEvent::RaydiumClmmTickArrayStateAccount(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumClmm,
                EventType::AccountRaydiumClmmTickArrayState,
                raydium_clmm_program(),
            );
            Some(DexEvent::RaydiumClmmTickArrayStateAccountEvent(
                raydium_clmm_tick_array_state_account_from_parser(e, meta),
            ))
        }

        PbDexEvent::RaydiumAmmV4Swap(e) => {
            let event_type = if e.max_amount_in != 0 || (e.amount_out != 0 && e.amount_in == 0) {
                EventType::RaydiumAmmV4SwapBaseOut
            } else {
                EventType::RaydiumAmmV4SwapBaseIn
            };
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumAmmV4,
                event_type,
                raydium_amm_v4_program(),
            );
            Some(DexEvent::RaydiumAmmV4SwapEvent(raydium_amm_v4_swap_from_parser(e, meta)))
        }
        PbDexEvent::RaydiumAmmV4Deposit(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumAmmV4,
                EventType::RaydiumAmmV4Deposit,
                raydium_amm_v4_program(),
            );
            Some(DexEvent::RaydiumAmmV4DepositEvent(raydium_amm_v4_deposit_from_parser(e, meta)))
        }
        PbDexEvent::RaydiumAmmV4Withdraw(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumAmmV4,
                EventType::RaydiumAmmV4Withdraw,
                raydium_amm_v4_program(),
            );
            Some(DexEvent::RaydiumAmmV4WithdrawEvent(raydium_amm_v4_withdraw_from_parser(e, meta)))
        }
        PbDexEvent::RaydiumAmmV4WithdrawPnl(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumAmmV4,
                EventType::RaydiumAmmV4WithdrawPnl,
                raydium_amm_v4_program(),
            );
            Some(DexEvent::RaydiumAmmV4WithdrawPnlEvent(raydium_amm_v4_withdraw_pnl_from_parser(
                e, meta,
            )))
        }
        PbDexEvent::RaydiumAmmV4Initialize2(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::RaydiumAmmV4,
                EventType::RaydiumAmmV4Initialize2,
                raydium_amm_v4_program(),
            );
            Some(DexEvent::RaydiumAmmV4Initialize2Event(raydium_amm_v4_initialize2_from_parser(
                e, meta,
            )))
        }

        PbDexEvent::MeteoraDammV2Swap(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::MeteoraDammV2,
                EventType::MeteoraDammV2Swap,
                meteora_damm_program(),
            );
            Some(DexEvent::MeteoraDammV2SwapEvent(meteora_damm_v2_swap_from_parser(e, meta)))
        }
        PbDexEvent::MeteoraDammV2AddLiquidity(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::MeteoraDammV2,
                EventType::MeteoraDammV2AddLiquidity,
                meteora_damm_program(),
            );
            Some(DexEvent::MeteoraDammV2AddLiquidityEvent(meteora_damm_v2_add_liquidity_from_pb(
                e, meta,
            )))
        }
        PbDexEvent::MeteoraDammV2RemoveLiquidity(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::MeteoraDammV2,
                EventType::MeteoraDammV2RemoveLiquidity,
                meteora_damm_program(),
            );
            Some(DexEvent::MeteoraDammV2RemoveLiquidityEvent(
                meteora_damm_v2_remove_liquidity_from_pb(e, meta),
            ))
        }
        PbDexEvent::MeteoraDammV2CreatePosition(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::MeteoraDammV2,
                EventType::MeteoraDammV2CreatePosition,
                meteora_damm_program(),
            );
            Some(DexEvent::MeteoraDammV2CreatePositionEvent(
                meteora_damm_v2_create_position_from_pb(e, meta),
            ))
        }
        PbDexEvent::MeteoraDammV2ClosePosition(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::MeteoraDammV2,
                EventType::MeteoraDammV2ClosePosition,
                meteora_damm_program(),
            );
            Some(DexEvent::MeteoraDammV2ClosePositionEvent(meteora_damm_v2_close_position_from_pb(
                e, meta,
            )))
        }

        PbDexEvent::TokenAccount(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::Common,
                EventType::TokenAccount,
                Pubkey::default(),
            );
            Some(DexEvent::TokenAccountEvent(token_account_from_parser(e, meta)))
        }
        PbDexEvent::TokenInfo(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::Common,
                EventType::TokenInfo,
                Pubkey::default(),
            );
            Some(DexEvent::TokenInfoEvent(token_info_from_parser(e, meta)))
        }
        PbDexEvent::NonceAccount(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::Common,
                EventType::NonceAccount,
                Pubkey::default(),
            );
            Some(DexEvent::NonceAccountEvent(nonce_account_from_parser(e, meta)))
        }
        PbDexEvent::PumpSwapGlobalConfigAccount(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::PumpSwap,
                EventType::AccountPumpSwapGlobalConfig,
                pumpswap_program(),
            );
            Some(DexEvent::PumpSwapGlobalConfigAccountEvent(
                pumpswap_global_config_account_from_parser(e, meta),
            ))
        }
        PbDexEvent::PumpSwapPoolAccount(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::PumpSwap,
                EventType::AccountPumpSwapPool,
                pumpswap_program(),
            );
            Some(DexEvent::PumpSwapPoolAccountEvent(pumpswap_pool_account_from_parser(e, meta)))
        }
        PbDexEvent::BlockMeta(m) => {
            let meta = adapt_pm(
                m.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::Common,
                EventType::BlockMeta,
                Pubkey::default(),
            );
            Some(DexEvent::BlockMetaEvent(BlockMetaEvent {
                metadata: meta,
                slot: m.metadata.slot,
                block_hash: m.metadata.recent_blockhash.clone().unwrap_or_default(),
            }))
        }

        PbDexEvent::Error(msg) => Some(DexEvent::ParserSdkErrorEvent(ParserSdkErrorEvent {
            metadata: EventMetadata {
                recv_us: recv_wall_us,
                protocol: ProtocolType::Common,
                event_type: EventType::ParserSdkError,
                ..Default::default()
            },
            message: msg,
        })),
        PbDexEvent::OrcaWhirlpoolSwap(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::OrcaWhirlpool,
                EventType::OrcaWhirlpoolSwap,
                orca_whirlpool_program(),
            );
            Some(DexEvent::OrcaWhirlpoolSwapEvent(orca_swap_from_pb(e, meta)))
        }
        PbDexEvent::OrcaWhirlpoolLiquidityIncreased(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::OrcaWhirlpool,
                EventType::OrcaWhirlpoolLiquidityIncreased,
                orca_whirlpool_program(),
            );
            Some(DexEvent::OrcaWhirlpoolLiquidityIncreasedEvent(orca_liquidity_increased_from_pb(
                e, meta,
            )))
        }
        PbDexEvent::OrcaWhirlpoolLiquidityDecreased(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::OrcaWhirlpool,
                EventType::OrcaWhirlpoolLiquidityDecreased,
                orca_whirlpool_program(),
            );
            Some(DexEvent::OrcaWhirlpoolLiquidityDecreasedEvent(orca_liquidity_decreased_from_pb(
                e, meta,
            )))
        }
        PbDexEvent::OrcaWhirlpoolPoolInitialized(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::OrcaWhirlpool,
                EventType::OrcaWhirlpoolPoolInitialized,
                orca_whirlpool_program(),
            );
            Some(DexEvent::OrcaWhirlpoolPoolInitializedEvent(orca_pool_initialized_from_pb(
                e, meta,
            )))
        }
        PbDexEvent::MeteoraPoolsSwap(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::MeteoraPools,
                EventType::MeteoraPoolsSwap,
                meteora_pools_program(),
            );
            Some(DexEvent::MeteoraPoolsSwapEvent(meteora_pools_swap_from_pb(e, meta)))
        }
        PbDexEvent::MeteoraPoolsAddLiquidity(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::MeteoraPools,
                EventType::MeteoraPoolsAddLiquidity,
                meteora_pools_program(),
            );
            Some(DexEvent::MeteoraPoolsAddLiquidityEvent(meteora_pools_add_liquidity_from_pb(
                e, meta,
            )))
        }
        PbDexEvent::MeteoraPoolsRemoveLiquidity(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::MeteoraPools,
                EventType::MeteoraPoolsRemoveLiquidity,
                meteora_pools_program(),
            );
            Some(DexEvent::MeteoraPoolsRemoveLiquidityEvent(
                meteora_pools_remove_liquidity_from_pb(e, meta),
            ))
        }
        PbDexEvent::MeteoraPoolsBootstrapLiquidity(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::MeteoraPools,
                EventType::MeteoraPoolsBootstrapLiquidity,
                meteora_pools_program(),
            );
            Some(DexEvent::MeteoraPoolsBootstrapLiquidityEvent(meteora_pools_bootstrap_from_pb(
                e, meta,
            )))
        }
        PbDexEvent::MeteoraPoolsPoolCreated(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::MeteoraPools,
                EventType::MeteoraPoolsPoolCreated,
                meteora_pools_program(),
            );
            Some(DexEvent::MeteoraPoolsPoolCreatedEvent(meteora_pools_pool_created_from_pb(
                e, meta,
            )))
        }
        PbDexEvent::MeteoraPoolsSetPoolFees(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::MeteoraPools,
                EventType::MeteoraPoolsSetPoolFees,
                meteora_pools_program(),
            );
            Some(DexEvent::MeteoraPoolsSetPoolFeesEvent(meteora_pools_set_fees_from_pb(e, meta)))
        }
        PbDexEvent::MeteoraDlmmSwap(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::MeteoraDlmm,
                EventType::MeteoraDlmmSwap,
                meteora_dlmm_program(),
            );
            Some(DexEvent::MeteoraDlmmSwapEvent(meteora_dlmm_swap_from_pb(e, meta)))
        }
        PbDexEvent::MeteoraDlmmAddLiquidity(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::MeteoraDlmm,
                EventType::MeteoraDlmmAddLiquidity,
                meteora_dlmm_program(),
            );
            Some(DexEvent::MeteoraDlmmAddLiquidityEvent(meteora_dlmm_add_liquidity_from_pb(
                e, meta,
            )))
        }
        PbDexEvent::MeteoraDlmmRemoveLiquidity(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::MeteoraDlmm,
                EventType::MeteoraDlmmRemoveLiquidity,
                meteora_dlmm_program(),
            );
            Some(DexEvent::MeteoraDlmmRemoveLiquidityEvent(meteora_dlmm_remove_liquidity_from_pb(
                e, meta,
            )))
        }
        PbDexEvent::MeteoraDlmmInitializePool(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::MeteoraDlmm,
                EventType::MeteoraDlmmInitializePool,
                meteora_dlmm_program(),
            );
            Some(DexEvent::MeteoraDlmmInitializePoolEvent(meteora_dlmm_init_pool_from_pb(e, meta)))
        }
        PbDexEvent::MeteoraDlmmInitializeBinArray(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::MeteoraDlmm,
                EventType::MeteoraDlmmInitializeBinArray,
                meteora_dlmm_program(),
            );
            Some(DexEvent::MeteoraDlmmInitializeBinArrayEvent(meteora_dlmm_init_bin_array_from_pb(
                e, meta,
            )))
        }
        PbDexEvent::MeteoraDlmmCreatePosition(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::MeteoraDlmm,
                EventType::MeteoraDlmmCreatePosition,
                meteora_dlmm_program(),
            );
            Some(DexEvent::MeteoraDlmmCreatePositionEvent(meteora_dlmm_create_position_from_pb(
                e, meta,
            )))
        }
        PbDexEvent::MeteoraDlmmClosePosition(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::MeteoraDlmm,
                EventType::MeteoraDlmmClosePosition,
                meteora_dlmm_program(),
            );
            Some(DexEvent::MeteoraDlmmClosePositionEvent(meteora_dlmm_close_position_from_pb(
                e, meta,
            )))
        }
        PbDexEvent::MeteoraDlmmClaimFee(e) => {
            let meta = adapt_pm(
                e.metadata.clone(),
                bt,
                recv_wall_us,
                ProtocolType::MeteoraDlmm,
                EventType::MeteoraDlmmClaimFee,
                meteora_dlmm_program(),
            );
            Some(DexEvent::MeteoraDlmmClaimFeeEvent(meteora_dlmm_claim_fee_from_pb(e, meta)))
        }
        _ => None,
    }
}

pub(crate) fn adapt_parser_events_list(
    pb: Vec<PbDexEvent>,
    bt: Option<&Timestamp>,
    recv_wall_us: i64,
    protocols: &[Protocol],
    event_type_filter: Option<&crate::streaming::event_parser::common::filter::EventTypeFilter>,
) -> Vec<DexEvent> {
    pb.into_iter()
        .filter_map(|e| adapt_parser_event(e, bt, recv_wall_us, protocols, event_type_filter))
        .collect()
}

pub(crate) fn adapt_parser_event(
    pb: PbDexEvent,
    bt: Option<&Timestamp>,
    recv_wall_us: i64,
    protocols: &[Protocol],
    event_type_filter: Option<&crate::streaming::event_parser::common::filter::EventTypeFilter>,
) -> Option<DexEvent> {
    let ev = convert_parser_event(pb, bt, recv_wall_us)?;
    if !event_matches_protocol(protocols, &ev) {
        return None;
    }
    if !passes_event_type_filter(event_type_filter, &ev) {
        return None;
    }
    Some(ev)
}
