//! PumpFun / PumpSwap field mapping and aggregate trade conversion.
use crate::streaming::event_parser::common::types::{EventType, ProtocolType};
use crate::streaming::event_parser::common::EventMetadata;
use crate::streaming::event_parser::protocols::pumpfun::events::{
    PumpFeesConfigStatus, PumpFeesCreateFeeSharingConfigEvent, PumpFeesFeeTier, PumpFeesFees,
    PumpFeesInitializeFeeConfigEvent, PumpFeesResetFeeSharingConfigEvent,
    PumpFeesRevokeFeeSharingAuthorityEvent, PumpFeesShareholder,
    PumpFeesTransferFeeSharingAuthorityEvent, PumpFeesUpdateAdminEvent,
    PumpFeesUpdateFeeConfigEvent, PumpFeesUpdateFeeSharesEvent, PumpFeesUpsertFeeTiersEvent,
    PumpFunBondingCurveAccountEvent, PumpFunCreateTokenEvent, PumpFunFeeConfig,
    PumpFunFeeConfigAccountEvent, PumpFunGlobalAccountEvent, PumpFunGlobalVolumeAccumulator,
    PumpFunGlobalVolumeAccumulatorAccountEvent, PumpFunMigrateBondingCurveCreatorEvent,
    PumpFunMigrateEvent, PumpFunSharingConfig, PumpFunSharingConfigAccountEvent, PumpFunTradeEvent,
    PumpFunUserVolumeAccumulator, PumpFunUserVolumeAccumulatorAccountEvent,
};
use crate::streaming::event_parser::protocols::pumpfun::types::{BondingCurve, Global};
use crate::streaming::event_parser::protocols::pumpswap::events::{
    PumpSwapBuyEvent, PumpSwapCreatePoolEvent, PumpSwapDepositEvent, PumpSwapSellEvent,
    PumpSwapWithdrawEvent,
};
use crate::streaming::event_parser::DexEvent;
use prost_types::Timestamp;
use sol_parser_sdk::core::events::normalize_pumpfun_quote_mint as normalize_sdk_pumpfun_quote_mint;
use solana_sdk::pubkey::Pubkey;

use super::adapt::adapt_pm;
use super::program_ids::{pump_program, pumpswap_program};

#[inline]
fn normalize_pumpfun_quote_mint(quote_mint: Pubkey) -> Pubkey {
    normalize_sdk_pumpfun_quote_mint(quote_mint)
}

pub(crate) fn pumpfun_create_token_from_parser(
    c: sol_parser_sdk::core::events::PumpFunCreateTokenEvent,
    meta: EventMetadata,
) -> PumpFunCreateTokenEvent {
    PumpFunCreateTokenEvent {
        metadata: meta,
        name: c.name,
        symbol: c.symbol,
        uri: c.uri,
        mint: c.mint,
        bonding_curve: c.bonding_curve,
        user: c.user,
        creator: c.creator,
        timestamp: c.timestamp,
        virtual_token_reserves: c.virtual_token_reserves,
        virtual_sol_reserves: c.virtual_sol_reserves,
        real_token_reserves: c.real_token_reserves,
        token_total_supply: c.token_total_supply,
        token_program: c.token_program,
        is_mayhem_mode: c.is_mayhem_mode,
        is_cashback_enabled: c.is_cashback_enabled,
        quote_mint: normalize_pumpfun_quote_mint(c.quote_mint),
        quote_vault: c.quote_vault,
        quote_token_program: c.quote_token_program,
        virtual_quote_reserves: c.virtual_quote_reserves,
        ix_name: "create".to_string(),
        ..Default::default()
    }
}

pub(crate) fn pumpfun_create_token_from_parser_v2(
    c: sol_parser_sdk::core::events::PumpFunCreateV2TokenEvent,
    meta: EventMetadata,
) -> PumpFunCreateTokenEvent {
    PumpFunCreateTokenEvent {
        metadata: meta,
        name: c.name,
        symbol: c.symbol,
        uri: c.uri,
        mint: c.mint,
        bonding_curve: c.bonding_curve,
        user: c.user,
        creator: c.creator,
        timestamp: c.timestamp,
        virtual_token_reserves: c.virtual_token_reserves,
        virtual_sol_reserves: c.virtual_sol_reserves,
        real_token_reserves: c.real_token_reserves,
        token_total_supply: c.token_total_supply,
        token_program: c.token_program,
        is_mayhem_mode: c.is_mayhem_mode,
        is_cashback_enabled: c.is_cashback_enabled,
        quote_mint: normalize_pumpfun_quote_mint(c.quote_mint),
        quote_vault: c.quote_vault,
        quote_token_program: c.quote_token_program,
        virtual_quote_reserves: c.virtual_quote_reserves,
        ix_name: "create_v2".to_string(),
        mint_authority: c.mint_authority,
        associated_bonding_curve: c.associated_bonding_curve,
        global: c.global,
        system_program: c.system_program,
        associated_token_program: c.associated_token_program,
        mayhem_program_id: c.mayhem_program_id,
        global_params: c.global_params,
        sol_vault: c.sol_vault,
        mayhem_state: c.mayhem_state,
        mayhem_token_vault: c.mayhem_token_vault,
        event_authority: c.event_authority,
        program: c.program,
        ..Default::default()
    }
}

pub(crate) fn pumpfun_migrate_from_parser(
    m: sol_parser_sdk::core::events::PumpFunMigrateEvent,
    meta: EventMetadata,
) -> PumpFunMigrateEvent {
    PumpFunMigrateEvent {
        metadata: meta,
        user: m.user,
        mint: m.mint,
        mint_amount: m.mint_amount,
        sol_amount: m.sol_amount,
        pool_migration_fee: m.pool_migration_fee,
        bonding_curve: m.bonding_curve,
        timestamp: m.timestamp,
        pool: m.pool,
        ..Default::default()
    }
}

fn pump_fees_status_from_parser(
    s: sol_parser_sdk::core::events::PumpFeesConfigStatus,
) -> PumpFeesConfigStatus {
    match s {
        sol_parser_sdk::core::events::PumpFeesConfigStatus::Paused => PumpFeesConfigStatus::Paused,
        sol_parser_sdk::core::events::PumpFeesConfigStatus::Active => PumpFeesConfigStatus::Active,
    }
}

fn pump_fees_shareholder_from_parser(
    s: sol_parser_sdk::core::events::PumpFeesShareholder,
) -> PumpFeesShareholder {
    PumpFeesShareholder { address: s.address, share_bps: s.share_bps }
}

fn pump_fees_fees_from_parser(f: sol_parser_sdk::core::events::PumpFeesFees) -> PumpFeesFees {
    PumpFeesFees {
        lp_fee_bps: f.lp_fee_bps,
        protocol_fee_bps: f.protocol_fee_bps,
        creator_fee_bps: f.creator_fee_bps,
    }
}

fn pump_fees_tier_from_parser(t: sol_parser_sdk::core::events::PumpFeesFeeTier) -> PumpFeesFeeTier {
    PumpFeesFeeTier {
        market_cap_lamports_threshold: t.market_cap_lamports_threshold,
        fees: pump_fees_fees_from_parser(t.fees),
    }
}

pub(crate) fn pump_fees_create_sharing_config_from_parser(
    e: sol_parser_sdk::core::events::PumpFeesCreateFeeSharingConfigEvent,
    meta: EventMetadata,
) -> PumpFeesCreateFeeSharingConfigEvent {
    PumpFeesCreateFeeSharingConfigEvent {
        metadata: meta,
        timestamp: e.timestamp,
        mint: e.mint,
        bonding_curve: e.bonding_curve,
        pool: e.pool,
        sharing_config: e.sharing_config,
        admin: e.admin,
        initial_shareholders: e
            .initial_shareholders
            .into_iter()
            .map(pump_fees_shareholder_from_parser)
            .collect(),
        status: pump_fees_status_from_parser(e.status),
    }
}

pub(crate) fn pump_fees_initialize_fee_config_from_parser(
    e: sol_parser_sdk::core::events::PumpFeesInitializeFeeConfigEvent,
    meta: EventMetadata,
) -> PumpFeesInitializeFeeConfigEvent {
    PumpFeesInitializeFeeConfigEvent {
        metadata: meta,
        timestamp: e.timestamp,
        admin: e.admin,
        fee_config: e.fee_config,
    }
}

pub(crate) fn pump_fees_reset_sharing_config_from_parser(
    e: sol_parser_sdk::core::events::PumpFeesResetFeeSharingConfigEvent,
    meta: EventMetadata,
) -> PumpFeesResetFeeSharingConfigEvent {
    PumpFeesResetFeeSharingConfigEvent {
        metadata: meta,
        timestamp: e.timestamp,
        mint: e.mint,
        sharing_config: e.sharing_config,
        old_admin: e.old_admin,
        old_shareholders: e
            .old_shareholders
            .into_iter()
            .map(pump_fees_shareholder_from_parser)
            .collect(),
        new_admin: e.new_admin,
        new_shareholders: e
            .new_shareholders
            .into_iter()
            .map(pump_fees_shareholder_from_parser)
            .collect(),
    }
}

pub(crate) fn pump_fees_revoke_authority_from_parser(
    e: sol_parser_sdk::core::events::PumpFeesRevokeFeeSharingAuthorityEvent,
    meta: EventMetadata,
) -> PumpFeesRevokeFeeSharingAuthorityEvent {
    PumpFeesRevokeFeeSharingAuthorityEvent {
        metadata: meta,
        timestamp: e.timestamp,
        mint: e.mint,
        sharing_config: e.sharing_config,
        admin: e.admin,
    }
}

pub(crate) fn pump_fees_transfer_authority_from_parser(
    e: sol_parser_sdk::core::events::PumpFeesTransferFeeSharingAuthorityEvent,
    meta: EventMetadata,
) -> PumpFeesTransferFeeSharingAuthorityEvent {
    PumpFeesTransferFeeSharingAuthorityEvent {
        metadata: meta,
        timestamp: e.timestamp,
        mint: e.mint,
        sharing_config: e.sharing_config,
        old_admin: e.old_admin,
        new_admin: e.new_admin,
    }
}

pub(crate) fn pump_fees_update_admin_from_parser(
    e: sol_parser_sdk::core::events::PumpFeesUpdateAdminEvent,
    meta: EventMetadata,
) -> PumpFeesUpdateAdminEvent {
    PumpFeesUpdateAdminEvent {
        metadata: meta,
        timestamp: e.timestamp,
        old_admin: e.old_admin,
        new_admin: e.new_admin,
    }
}

pub(crate) fn pump_fees_update_fee_config_from_parser(
    e: sol_parser_sdk::core::events::PumpFeesUpdateFeeConfigEvent,
    meta: EventMetadata,
) -> PumpFeesUpdateFeeConfigEvent {
    PumpFeesUpdateFeeConfigEvent {
        metadata: meta,
        timestamp: e.timestamp,
        admin: e.admin,
        fee_config: e.fee_config,
        fee_tiers: e.fee_tiers.into_iter().map(pump_fees_tier_from_parser).collect(),
        flat_fees: pump_fees_fees_from_parser(e.flat_fees),
    }
}

pub(crate) fn pump_fees_update_fee_shares_from_parser(
    e: sol_parser_sdk::core::events::PumpFeesUpdateFeeSharesEvent,
    meta: EventMetadata,
) -> PumpFeesUpdateFeeSharesEvent {
    PumpFeesUpdateFeeSharesEvent {
        metadata: meta,
        timestamp: e.timestamp,
        mint: e.mint,
        sharing_config: e.sharing_config,
        admin: e.admin,
        bonding_curve: e.bonding_curve,
        pump_creator_vault: e.pump_creator_vault,
        new_shareholders: e
            .new_shareholders
            .into_iter()
            .map(pump_fees_shareholder_from_parser)
            .collect(),
    }
}

pub(crate) fn pump_fees_upsert_fee_tiers_from_parser(
    e: sol_parser_sdk::core::events::PumpFeesUpsertFeeTiersEvent,
    meta: EventMetadata,
) -> PumpFeesUpsertFeeTiersEvent {
    PumpFeesUpsertFeeTiersEvent {
        metadata: meta,
        timestamp: e.timestamp,
        admin: e.admin,
        fee_config: e.fee_config,
        fee_tiers: e.fee_tiers.into_iter().map(pump_fees_tier_from_parser).collect(),
        offset: e.offset,
    }
}

pub(crate) fn pumpfun_migrate_bonding_creator_from_parser(
    e: sol_parser_sdk::core::events::PumpFunMigrateBondingCurveCreatorEvent,
    meta: EventMetadata,
) -> PumpFunMigrateBondingCurveCreatorEvent {
    PumpFunMigrateBondingCurveCreatorEvent {
        metadata: meta,
        timestamp: e.timestamp,
        mint: e.mint,
        bonding_curve: e.bonding_curve,
        sharing_config: e.sharing_config,
        old_creator: e.old_creator,
        new_creator: e.new_creator,
    }
}

pub(crate) fn pumpfun_global_account_from_parser(
    e: sol_parser_sdk::core::events::PumpFunGlobalAccountEvent,
    meta: EventMetadata,
) -> PumpFunGlobalAccountEvent {
    let mut fee_recipients = [Pubkey::default(); 7];
    for (dst, src) in fee_recipients.iter_mut().zip(e.global.fee_recipients.iter()) {
        *dst = *src;
    }

    PumpFunGlobalAccountEvent {
        metadata: meta,
        pubkey: e.pubkey,
        executable: false,
        lamports: 0,
        owner: pump_program(),
        rent_epoch: 0,
        global: Global {
            initialized: e.global.initialized,
            authority: e.global.authority,
            fee_recipient: e.global.fee_recipient,
            initial_virtual_token_reserves: e.global.initial_virtual_token_reserves,
            initial_virtual_sol_reserves: e.global.initial_virtual_sol_reserves,
            initial_real_token_reserves: e.global.initial_real_token_reserves,
            token_total_supply: e.global.token_total_supply,
            fee_basis_points: e.global.fee_basis_points,
            withdraw_authority: e.global.withdraw_authority,
            enable_migrate: e.global.enable_migrate,
            pool_migration_fee: e.global.pool_migration_fee,
            creator_fee_basis_points: e.global.creator_fee_basis_points,
            fee_recipients,
            set_creator_authority: e.global.set_creator_authority,
            admin_set_creator_authority: e.global.admin_set_creator_authority,
            create_v2_enabled: e.global.create_v2_enabled,
            whitelist_pda: e.global.whitelist_pda,
            reserved_fee_recipient: e.global.reserved_fee_recipient,
            mayhem_mode_enabled: e.global.mayhem_mode_enabled,
            reserved_fee_recipients: e.global.reserved_fee_recipients,
            is_cashback_enabled: e.global.is_cashback_enabled,
            buyback_fee_recipients: e.global.buyback_fee_recipients,
            buyback_basis_points: e.global.buyback_basis_points,
            initial_virtual_quote_reserves: e.global.initial_virtual_quote_reserves,
            whitelisted_quote_mints: e.global.whitelisted_quote_mints,
        },
    }
}

pub(crate) fn pumpfun_bonding_curve_account_from_parser(
    e: sol_parser_sdk::core::events::PumpFunBondingCurveAccountEvent,
    meta: EventMetadata,
) -> PumpFunBondingCurveAccountEvent {
    PumpFunBondingCurveAccountEvent {
        metadata: meta,
        pubkey: e.pubkey,
        executable: false,
        lamports: 0,
        owner: pump_program(),
        rent_epoch: 0,
        bonding_curve: BondingCurve {
            virtual_token_reserves: e.bonding_curve.virtual_token_reserves,
            virtual_quote_reserves: e.bonding_curve.virtual_quote_reserves,
            real_token_reserves: e.bonding_curve.real_token_reserves,
            real_quote_reserves: e.bonding_curve.real_quote_reserves,
            token_total_supply: e.bonding_curve.token_total_supply,
            complete: e.bonding_curve.complete,
            creator: e.bonding_curve.creator,
            is_mayhem_mode: e.bonding_curve.is_mayhem_mode,
            is_cashback_coin: e.bonding_curve.is_cashback_coin,
            quote_mint: normalize_pumpfun_quote_mint(e.bonding_curve.quote_mint),
        },
    }
}

pub(crate) fn pumpfun_fee_config_account_from_parser(
    e: sol_parser_sdk::core::events::PumpFunFeeConfigAccountEvent,
    meta: EventMetadata,
) -> PumpFunFeeConfigAccountEvent {
    PumpFunFeeConfigAccountEvent {
        metadata: meta,
        pubkey: e.pubkey,
        executable: false,
        lamports: 0,
        owner: pump_program(),
        rent_epoch: 0,
        fee_config: PumpFunFeeConfig {
            bump: e.fee_config.bump,
            admin: e.fee_config.admin,
            flat_fees: pump_fees_fees_from_parser(e.fee_config.flat_fees),
            fee_tiers: e.fee_config.fee_tiers.into_iter().map(pump_fees_tier_from_parser).collect(),
            stable_fee_tiers: e
                .fee_config
                .stable_fee_tiers
                .into_iter()
                .map(pump_fees_tier_from_parser)
                .collect(),
        },
    }
}

pub(crate) fn pumpfun_sharing_config_account_from_parser(
    e: sol_parser_sdk::core::events::PumpFunSharingConfigAccountEvent,
    meta: EventMetadata,
) -> PumpFunSharingConfigAccountEvent {
    PumpFunSharingConfigAccountEvent {
        metadata: meta,
        pubkey: e.pubkey,
        executable: false,
        lamports: 0,
        owner: pump_program(),
        rent_epoch: 0,
        sharing_config: PumpFunSharingConfig {
            bump: e.sharing_config.bump,
            version: e.sharing_config.version,
            status: pump_fees_status_from_parser(e.sharing_config.status),
            mint: e.sharing_config.mint,
            admin: e.sharing_config.admin,
            admin_revoked: e.sharing_config.admin_revoked,
            shareholders: e
                .sharing_config
                .shareholders
                .into_iter()
                .map(pump_fees_shareholder_from_parser)
                .collect(),
        },
    }
}

pub(crate) fn pumpfun_global_volume_account_from_parser(
    e: sol_parser_sdk::core::events::PumpFunGlobalVolumeAccumulatorAccountEvent,
    meta: EventMetadata,
) -> PumpFunGlobalVolumeAccumulatorAccountEvent {
    PumpFunGlobalVolumeAccumulatorAccountEvent {
        metadata: meta,
        pubkey: e.pubkey,
        executable: false,
        lamports: 0,
        owner: pump_program(),
        rent_epoch: 0,
        global_volume_accumulator: PumpFunGlobalVolumeAccumulator {
            start_time: e.global_volume_accumulator.start_time,
            end_time: e.global_volume_accumulator.end_time,
            seconds_in_a_day: e.global_volume_accumulator.seconds_in_a_day,
            mint: e.global_volume_accumulator.mint,
            total_token_supply: e.global_volume_accumulator.total_token_supply,
            sol_volumes: e.global_volume_accumulator.sol_volumes,
        },
    }
}

pub(crate) fn pumpfun_user_volume_account_from_parser(
    e: sol_parser_sdk::core::events::PumpFunUserVolumeAccumulatorAccountEvent,
    meta: EventMetadata,
) -> PumpFunUserVolumeAccumulatorAccountEvent {
    PumpFunUserVolumeAccumulatorAccountEvent {
        metadata: meta,
        pubkey: e.pubkey,
        executable: false,
        lamports: 0,
        owner: pump_program(),
        rent_epoch: 0,
        user_volume_accumulator: PumpFunUserVolumeAccumulator {
            user: e.user_volume_accumulator.user,
            needs_claim: e.user_volume_accumulator.needs_claim,
            total_unclaimed_tokens: e.user_volume_accumulator.total_unclaimed_tokens,
            total_claimed_tokens: e.user_volume_accumulator.total_claimed_tokens,
            current_sol_volume: e.user_volume_accumulator.current_sol_volume,
            last_update_timestamp: e.user_volume_accumulator.last_update_timestamp,
            has_total_claimed_tokens: e.user_volume_accumulator.has_total_claimed_tokens,
            cashback_earned: e.user_volume_accumulator.cashback_earned,
            total_cashback_claimed: e.user_volume_accumulator.total_cashback_claimed,
            stable_cashback_earned: e.user_volume_accumulator.stable_cashback_earned,
            total_stable_cashback_claimed: e.user_volume_accumulator.total_stable_cashback_claimed,
        },
    }
}

pub(crate) fn pumpswap_buy_full_from_parser(
    b: sol_parser_sdk::core::events::PumpSwapBuyEvent,
    meta: EventMetadata,
) -> PumpSwapBuyEvent {
    PumpSwapBuyEvent {
        metadata: meta,
        timestamp: b.timestamp,
        base_amount_out: b.base_amount_out,
        max_quote_amount_in: b.max_quote_amount_in,
        user_base_token_reserves: b.user_base_token_reserves,
        user_quote_token_reserves: b.user_quote_token_reserves,
        pool_base_token_reserves: b.pool_base_token_reserves,
        pool_quote_token_reserves: b.pool_quote_token_reserves,
        quote_amount_in: b.quote_amount_in,
        lp_fee_basis_points: b.lp_fee_basis_points,
        lp_fee: b.lp_fee,
        protocol_fee_basis_points: b.protocol_fee_basis_points,
        protocol_fee: b.protocol_fee,
        quote_amount_in_with_lp_fee: b.quote_amount_in_with_lp_fee,
        user_quote_amount_in: b.user_quote_amount_in,
        pool: b.pool,
        user: b.user,
        user_base_token_account: b.user_base_token_account,
        user_quote_token_account: b.user_quote_token_account,
        protocol_fee_recipient: b.protocol_fee_recipient,
        protocol_fee_recipient_token_account: b.protocol_fee_recipient_token_account,
        coin_creator: b.coin_creator,
        coin_creator_fee_basis_points: b.coin_creator_fee_basis_points,
        coin_creator_fee: b.coin_creator_fee,
        track_volume: b.track_volume,
        total_unclaimed_tokens: b.total_unclaimed_tokens,
        total_claimed_tokens: b.total_claimed_tokens,
        current_sol_volume: b.current_sol_volume,
        last_update_timestamp: b.last_update_timestamp,
        min_base_amount_out: b.min_base_amount_out,
        ix_name: b.ix_name,
        cashback_fee_basis_points: b.cashback_fee_basis_points,
        cashback: b.cashback,
        is_pump_pool: b.is_pump_pool,
        base_mint: b.base_mint,
        quote_mint: b.quote_mint,
        pool_base_token_account: b.pool_base_token_account,
        pool_quote_token_account: b.pool_quote_token_account,
        coin_creator_vault_ata: b.coin_creator_vault_ata,
        coin_creator_vault_authority: b.coin_creator_vault_authority,
        base_token_program: b.base_token_program,
        quote_token_program: b.quote_token_program,
        pool_v2: b.pool_v2,
        fee_recipient: b.fee_recipient,
        fee_recipient_quote_token_account: b.fee_recipient_quote_token_account,
    }
}

pub(crate) fn pumpswap_sell_full_from_parser(
    s: sol_parser_sdk::core::events::PumpSwapSellEvent,
    meta: EventMetadata,
) -> PumpSwapSellEvent {
    PumpSwapSellEvent {
        metadata: meta,
        timestamp: s.timestamp,
        base_amount_in: s.base_amount_in,
        min_quote_amount_out: s.min_quote_amount_out,
        user_base_token_reserves: s.user_base_token_reserves,
        user_quote_token_reserves: s.user_quote_token_reserves,
        pool_base_token_reserves: s.pool_base_token_reserves,
        pool_quote_token_reserves: s.pool_quote_token_reserves,
        quote_amount_out: s.quote_amount_out,
        lp_fee_basis_points: s.lp_fee_basis_points,
        lp_fee: s.lp_fee,
        protocol_fee_basis_points: s.protocol_fee_basis_points,
        protocol_fee: s.protocol_fee,
        quote_amount_out_without_lp_fee: s.quote_amount_out_without_lp_fee,
        user_quote_amount_out: s.user_quote_amount_out,
        pool: s.pool,
        user: s.user,
        user_base_token_account: s.user_base_token_account,
        user_quote_token_account: s.user_quote_token_account,
        protocol_fee_recipient: s.protocol_fee_recipient,
        protocol_fee_recipient_token_account: s.protocol_fee_recipient_token_account,
        coin_creator: s.coin_creator,
        coin_creator_fee_basis_points: s.coin_creator_fee_basis_points,
        coin_creator_fee: s.coin_creator_fee,
        cashback_fee_basis_points: s.cashback_fee_basis_points,
        cashback: s.cashback,
        is_pump_pool: s.is_pump_pool,
        base_mint: s.base_mint,
        quote_mint: s.quote_mint,
        pool_base_token_account: s.pool_base_token_account,
        pool_quote_token_account: s.pool_quote_token_account,
        coin_creator_vault_ata: s.coin_creator_vault_ata,
        coin_creator_vault_authority: s.coin_creator_vault_authority,
        base_token_program: s.base_token_program,
        quote_token_program: s.quote_token_program,
        pool_v2: s.pool_v2,
        fee_recipient: s.fee_recipient,
        fee_recipient_quote_token_account: s.fee_recipient_quote_token_account,
    }
}

pub(crate) fn pumpswap_create_pool_from_parser(
    c: sol_parser_sdk::core::events::PumpSwapCreatePoolEvent,
    meta: EventMetadata,
) -> PumpSwapCreatePoolEvent {
    PumpSwapCreatePoolEvent {
        metadata: meta,
        timestamp: c.timestamp,
        index: c.index,
        creator: c.creator,
        base_mint: c.base_mint,
        quote_mint: c.quote_mint,
        base_mint_decimals: c.base_mint_decimals,
        quote_mint_decimals: c.quote_mint_decimals,
        base_amount_in: c.base_amount_in,
        quote_amount_in: c.quote_amount_in,
        pool_base_amount: c.pool_base_amount,
        pool_quote_amount: c.pool_quote_amount,
        minimum_liquidity: c.minimum_liquidity,
        initial_liquidity: c.initial_liquidity,
        lp_token_amount_out: c.lp_token_amount_out,
        pool_bump: c.pool_bump,
        pool: c.pool,
        lp_mint: c.lp_mint,
        user_base_token_account: c.user_base_token_account,
        user_quote_token_account: c.user_quote_token_account,
        coin_creator: c.coin_creator,
        is_mayhem_mode: c.is_mayhem_mode,
        is_cashback_coin: c.is_cashback_coin,
        ..Default::default()
    }
}

pub(crate) fn pumpswap_liquidity_added_to_deposit(
    a: sol_parser_sdk::core::events::PumpSwapLiquidityAdded,
    meta: EventMetadata,
) -> PumpSwapDepositEvent {
    PumpSwapDepositEvent {
        metadata: meta,
        timestamp: a.timestamp,
        lp_token_amount_out: a.lp_token_amount_out,
        max_base_amount_in: a.max_base_amount_in,
        max_quote_amount_in: a.max_quote_amount_in,
        user_base_token_reserves: a.user_base_token_reserves,
        user_quote_token_reserves: a.user_quote_token_reserves,
        pool_base_token_reserves: a.pool_base_token_reserves,
        pool_quote_token_reserves: a.pool_quote_token_reserves,
        base_amount_in: a.base_amount_in,
        quote_amount_in: a.quote_amount_in,
        lp_mint_supply: a.lp_mint_supply,
        pool: a.pool,
        user: a.user,
        user_base_token_account: a.user_base_token_account,
        user_quote_token_account: a.user_quote_token_account,
        user_pool_token_account: a.user_pool_token_account,
        ..Default::default()
    }
}

pub(crate) fn pumpswap_liquidity_removed_to_withdraw(
    r: sol_parser_sdk::core::events::PumpSwapLiquidityRemoved,
    meta: EventMetadata,
) -> PumpSwapWithdrawEvent {
    PumpSwapWithdrawEvent {
        metadata: meta,
        timestamp: r.timestamp,
        lp_token_amount_in: r.lp_token_amount_in,
        min_base_amount_out: r.min_base_amount_out,
        min_quote_amount_out: r.min_quote_amount_out,
        user_base_token_reserves: r.user_base_token_reserves,
        user_quote_token_reserves: r.user_quote_token_reserves,
        pool_base_token_reserves: r.pool_base_token_reserves,
        pool_quote_token_reserves: r.pool_quote_token_reserves,
        base_amount_out: r.base_amount_out,
        quote_amount_out: r.quote_amount_out,
        lp_mint_supply: r.lp_mint_supply,
        pool: r.pool,
        user: r.user,
        user_base_token_account: r.user_base_token_account,
        user_quote_token_account: r.user_quote_token_account,
        user_pool_token_account: r.user_pool_token_account,
        ..Default::default()
    }
}
pub(crate) fn pumpfun_trade_from_parser_with_event_type(
    t: sol_parser_sdk::core::events::PumpFunTradeEvent,
    bt: Option<&Timestamp>,
    recv_wall_us: i64,
    event_type: EventType,
) -> DexEvent {
    let pm = t.metadata.clone();
    let meta = adapt_pm(pm, bt, recv_wall_us, ProtocolType::PumpFun, event_type, pump_program());
    let st = PumpFunTradeEvent {
        metadata: meta,
        mint: t.mint,
        sol_amount: t.sol_amount,
        token_amount: t.token_amount,
        is_buy: t.is_buy,
        user: t.user,
        timestamp: t.timestamp,
        virtual_sol_reserves: t.virtual_sol_reserves,
        virtual_token_reserves: t.virtual_token_reserves,
        real_sol_reserves: t.real_sol_reserves,
        real_token_reserves: t.real_token_reserves,
        fee_recipient: t.fee_recipient,
        fee_basis_points: t.fee_basis_points,
        fee: t.fee,
        creator: t.creator,
        creator_fee_basis_points: t.creator_fee_basis_points,
        creator_fee: t.creator_fee,
        track_volume: t.track_volume,
        total_unclaimed_tokens: t.total_unclaimed_tokens,
        total_claimed_tokens: t.total_claimed_tokens,
        current_sol_volume: t.current_sol_volume,
        last_update_timestamp: t.last_update_timestamp,
        global: t.global,
        bonding_curve: t.bonding_curve,
        bonding_curve_v2: t.bonding_curve_v2,
        associated_bonding_curve: t.associated_bonding_curve,
        associated_user: t.associated_user,
        system_program: t.system_program,
        token_program: t.token_program,
        quote_token_program: t.quote_token_program,
        associated_token_program: t.associated_token_program,
        creator_vault: t.creator_vault,
        associated_quote_fee_recipient: t.associated_quote_fee_recipient,
        buyback_fee_recipient: t.buyback_fee_recipient,
        associated_quote_buyback_fee_recipient: t.associated_quote_buyback_fee_recipient,
        associated_quote_bonding_curve: t.associated_quote_bonding_curve,
        associated_quote_user: t.associated_quote_user,
        associated_creator_vault: t.associated_creator_vault,
        sharing_config: t.sharing_config,
        event_authority: t.event_authority,
        program: t.program,
        global_volume_accumulator: t.global_volume_accumulator,
        user_volume_accumulator: t.user_volume_accumulator,
        associated_user_volume_accumulator: t.associated_user_volume_accumulator,
        fee_config: t.fee_config,
        fee_program: t.fee_program,
        account: t.account,
        ix_name: t.ix_name,
        mayhem_mode: t.mayhem_mode,
        cashback_fee_basis_points: t.cashback_fee_basis_points,
        cashback: t.cashback,
        buyback_fee_basis_points: t.buyback_fee_basis_points,
        buyback_fee: t.buyback_fee,
        shareholders: t.shareholders.into_iter().map(pump_fees_shareholder_from_parser).collect(),
        quote_mint: normalize_pumpfun_quote_mint(t.quote_mint),
        quote_amount: t.quote_amount,
        virtual_quote_reserves: t.virtual_quote_reserves,
        real_quote_reserves: t.real_quote_reserves,
        is_cashback_coin: t.is_cashback_coin,
        is_created_buy: t.is_created_buy,
        is_dev_create_token_trade: t.is_created_buy,
        amount: t.amount,
        max_sol_cost: t.max_sol_cost,
        min_sol_output: t.min_sol_output,
        spendable_sol_in: t.spendable_sol_in,
        spendable_quote_in: t.spendable_quote_in,
        min_tokens_out: t.min_tokens_out,
        ..Default::default()
    };
    DexEvent::PumpFunTradeEvent(st)
}

pub(crate) fn pumpswap_trade_from_parser(
    t: sol_parser_sdk::core::events::PumpSwapTradeEvent,
    bt: Option<&Timestamp>,
    recv_wall_us: i64,
) -> Option<DexEvent> {
    let pm = t.metadata.clone();
    let meta = adapt_pm(
        pm,
        bt,
        recv_wall_us,
        ProtocolType::PumpSwap,
        if t.is_buy { EventType::PumpSwapBuy } else { EventType::PumpSwapSell },
        pumpswap_program(),
    );
    if t.is_buy {
        Some(DexEvent::PumpSwapBuyEvent(PumpSwapBuyEvent {
            metadata: meta,
            timestamp: t.timestamp,
            base_amount_out: t.token_amount,
            max_quote_amount_in: t.sol_amount,
            user_base_token_reserves: t.virtual_token_reserves,
            user_quote_token_reserves: t.virtual_sol_reserves,
            pool_base_token_reserves: t.real_token_reserves,
            pool_quote_token_reserves: t.real_sol_reserves,
            quote_amount_in: t.sol_amount,
            lp_fee_basis_points: t.fee_basis_points,
            lp_fee: t.fee,
            protocol_fee_basis_points: 0,
            protocol_fee: 0,
            quote_amount_in_with_lp_fee: t.sol_amount,
            user_quote_amount_in: t.sol_amount,
            pool: Pubkey::default(),
            user: t.user,
            user_base_token_account: Pubkey::default(),
            user_quote_token_account: Pubkey::default(),
            protocol_fee_recipient: t.fee_recipient,
            protocol_fee_recipient_token_account: Pubkey::default(),
            coin_creator: t.creator,
            coin_creator_fee_basis_points: t.creator_fee_basis_points,
            coin_creator_fee: t.creator_fee,
            track_volume: t.track_volume,
            total_unclaimed_tokens: t.total_unclaimed_tokens,
            total_claimed_tokens: t.total_claimed_tokens,
            current_sol_volume: t.current_sol_volume,
            last_update_timestamp: t.last_update_timestamp,
            min_base_amount_out: 0,
            ix_name: t.ix_name.clone(),
            cashback_fee_basis_points: 0,
            cashback: 0,
            is_pump_pool: false,
            base_mint: t.mint,
            ..Default::default()
        }))
    } else {
        Some(DexEvent::PumpSwapSellEvent(PumpSwapSellEvent {
            metadata: meta,
            timestamp: t.timestamp,
            base_amount_in: t.token_amount,
            min_quote_amount_out: t.sol_amount,
            user_base_token_reserves: t.virtual_token_reserves,
            user_quote_token_reserves: t.virtual_sol_reserves,
            pool_base_token_reserves: t.real_token_reserves,
            pool_quote_token_reserves: t.real_sol_reserves,
            quote_amount_out: t.sol_amount,
            lp_fee_basis_points: t.fee_basis_points,
            lp_fee: t.fee,
            protocol_fee_basis_points: 0,
            protocol_fee: 0,
            quote_amount_out_without_lp_fee: t.sol_amount,
            user_quote_amount_out: t.sol_amount,
            pool: Pubkey::default(),
            user: t.user,
            user_base_token_account: Pubkey::default(),
            user_quote_token_account: Pubkey::default(),
            protocol_fee_recipient: t.fee_recipient,
            protocol_fee_recipient_token_account: Pubkey::default(),
            coin_creator: t.creator,
            coin_creator_fee_basis_points: t.creator_fee_basis_points,
            coin_creator_fee: t.creator_fee,
            cashback_fee_basis_points: 0,
            cashback: 0,
            is_pump_pool: false,
            base_mint: t.mint,
            ..Default::default()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pumpfun_default_quote_mint_uses_solscan_sol_sentinel() {
        let ev = pumpfun_trade_from_parser_with_event_type(
            sol_parser_sdk::core::events::PumpFunTradeEvent { is_buy: true, ..Default::default() },
            None,
            0,
            EventType::PumpFunBuy,
        );

        match ev {
            DexEvent::PumpFunTradeEvent(t) => {
                assert_eq!(t.quote_mint.to_string(), "So11111111111111111111111111111111111111111");
            }
            _ => panic!("expected PumpFunTradeEvent"),
        }
    }

    #[test]
    fn pumpfun_wsol_quote_mint_is_preserved() {
        let quote_mint = sol_parser_sdk::core::events::PUMPFUN_WSOL_QUOTE_MINT;
        let ev = pumpfun_trade_from_parser_with_event_type(
            sol_parser_sdk::core::events::PumpFunTradeEvent {
                is_buy: true,
                quote_mint,
                ..Default::default()
            },
            None,
            0,
            EventType::PumpFunBuy,
        );

        match ev {
            DexEvent::PumpFunTradeEvent(t) => {
                assert_eq!(t.quote_mint, quote_mint);
            }
            _ => panic!("expected PumpFunTradeEvent"),
        }
    }
}
