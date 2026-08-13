//! Bonk plus Token / Nonce / PumpSwap account event mapping.
use crate::streaming::event_parser::common::EventMetadata;
use crate::streaming::event_parser::core::account_event_parser::{
    NonceAccountEvent, TokenAccountEvent, TokenInfoEvent,
};
use crate::streaming::event_parser::protocols::bonk::events::{
    BonkMigrateToAmmEvent, BonkPoolCreateEvent, BonkTradeEvent,
};
use crate::streaming::event_parser::protocols::bonk::types::{
    CurveParams, MintParams, PoolStatus, TradeDirection as BonkTradeDirection, VestingParams,
};
use crate::streaming::event_parser::protocols::pumpswap::events::{
    PumpSwapGlobalConfigAccountEvent, PumpSwapPoolAccountEvent,
};
use crate::streaming::event_parser::protocols::pumpswap::types::{GlobalConfig, Pool};
use sol_parser_sdk::core::events::{
    RaydiumLaunchlabTradeEvent as PbBonkTrade, TradeDirection as PbBonkTradeDirection,
};
#[inline]
pub(crate) fn pb_bonk_trade_direction(d: PbBonkTradeDirection) -> BonkTradeDirection {
    match d {
        PbBonkTradeDirection::Buy => BonkTradeDirection::Buy,
        PbBonkTradeDirection::Sell => BonkTradeDirection::Sell,
    }
}

/// Align SDK Bonk trades with the four native Bonk trade instruction variants.
#[inline]
pub(crate) fn sdk_bonk_trade_event_type(
    b: &PbBonkTrade,
) -> crate::streaming::event_parser::common::types::EventType {
    use crate::streaming::event_parser::common::types::EventType;
    match (&b.trade_direction, b.exact_in) {
        (&PbBonkTradeDirection::Buy, true) => EventType::BonkBuyExactIn,
        (&PbBonkTradeDirection::Buy, false) => EventType::BonkBuyExactOut,
        (&PbBonkTradeDirection::Sell, true) => EventType::BonkSellExactIn,
        (&PbBonkTradeDirection::Sell, false) => EventType::BonkSellExactOut,
    }
}

/// SDK Bonk trade events do not expose reserves or fee rates; keep those fields at streamer
/// defaults.
pub(crate) fn bonk_trade_from_parser(
    b: sol_parser_sdk::core::events::RaydiumLaunchlabTradeEvent,
    meta: EventMetadata,
) -> BonkTradeEvent {
    BonkTradeEvent {
        metadata: meta,
        pool_state: b.pool_state,
        total_base_sell: 0,
        virtual_base: 0,
        virtual_quote: 0,
        real_base_before: 0,
        real_quote_before: 0,
        real_base_after: 0,
        real_quote_after: 0,
        amount_in: b.amount_in,
        amount_out: b.amount_out,
        protocol_fee: 0,
        platform_fee: 0,
        creator_fee: 0,
        share_fee: 0,
        trade_direction: pb_bonk_trade_direction(b.trade_direction),
        pool_status: PoolStatus::Trade,
        exact_in: b.exact_in,
        payer: b.user,
        global_config: b.global_config,
        platform_config: b.platform_config,
        user_base_token: b.user_base_token,
        user_quote_token: b.user_quote_token,
        base_vault: b.base_vault,
        quote_vault: b.quote_vault,
        base_token_mint: b.base_mint,
        quote_token_mint: b.quote_mint,
        base_token_program: b.base_token_program,
        quote_token_program: b.quote_token_program,
        ..Default::default()
    }
}

pub(crate) fn bonk_pool_create_from_parser(
    p: sol_parser_sdk::core::events::RaydiumLaunchlabPoolCreateEvent,
    meta: EventMetadata,
) -> BonkPoolCreateEvent {
    BonkPoolCreateEvent {
        metadata: meta,
        pool_state: p.pool_state,
        payer: p.payer,
        creator: p.creator,
        config: Default::default(),
        global_config: p.global_config,
        platform_config: p.platform_config,
        base_mint: p.base_mint,
        quote_mint: p.quote_mint,
        base_vault: p.base_vault,
        quote_vault: p.quote_vault,
        base_token_program: p.base_token_program,
        quote_token_program: p.quote_token_program,
        base_mint_param: MintParams {
            decimals: p.base_mint_param.decimals,
            name: p.base_mint_param.name.clone(),
            symbol: p.base_mint_param.symbol.clone(),
            uri: p.base_mint_param.uri.clone(),
        },
        curve_param: CurveParams::default(),
        vesting_param: VestingParams::default(),
        amm_fee_on: None,
    }
}

pub(crate) fn bonk_migrate_to_amm_from_parser(
    m: sol_parser_sdk::core::events::RaydiumLaunchlabMigrateAmmEvent,
    meta: EventMetadata,
) -> BonkMigrateToAmmEvent {
    BonkMigrateToAmmEvent {
        metadata: meta,
        payer: m.user,
        pool_state: m.old_pool,
        amm_pool: m.new_pool,
        ..Default::default()
    }
}

pub(crate) fn token_account_from_parser(
    e: sol_parser_sdk::core::events::TokenAccountEvent,
    meta: EventMetadata,
) -> TokenAccountEvent {
    TokenAccountEvent {
        metadata: meta,
        pubkey: e.pubkey,
        executable: e.executable,
        lamports: e.lamports,
        owner: e.owner,
        rent_epoch: e.rent_epoch,
        amount: e.amount,
        token_owner: e.token_owner,
    }
}

pub(crate) fn token_info_from_parser(
    e: sol_parser_sdk::core::events::TokenInfoEvent,
    meta: EventMetadata,
) -> TokenInfoEvent {
    TokenInfoEvent {
        metadata: meta,
        pubkey: e.pubkey,
        executable: e.executable,
        lamports: e.lamports,
        owner: e.owner,
        rent_epoch: e.rent_epoch,
        supply: e.supply,
        decimals: e.decimals,
    }
}

pub(crate) fn nonce_account_from_parser(
    e: sol_parser_sdk::core::events::NonceAccountEvent,
    meta: EventMetadata,
) -> NonceAccountEvent {
    NonceAccountEvent {
        metadata: meta,
        pubkey: e.pubkey,
        executable: e.executable,
        lamports: e.lamports,
        owner: e.owner,
        rent_epoch: e.rent_epoch,
        nonce: e.nonce,
        authority: e.authority,
    }
}

pub(crate) fn pumpswap_global_config_from_pb(
    g: sol_parser_sdk::core::events::PumpSwapGlobalConfig,
) -> GlobalConfig {
    GlobalConfig {
        admin: g.admin,
        lp_fee_basis_points: g.lp_fee_basis_points,
        protocol_fee_basis_points: g.protocol_fee_basis_points,
        disable_flags: g.disable_flags,
        protocol_fee_recipients: g.protocol_fee_recipients,
        coin_creator_fee_basis_points: g.coin_creator_fee_basis_points,
        admin_set_coin_creator_authority: g.admin_set_coin_creator_authority,
        whitelist_pda: g.whitelist_pda,
        reserved_fee_recipient: g.reserved_fee_recipient,
        mayhem_mode_enabled: g.mayhem_mode_enabled,
        reserved_fee_recipients: g.reserved_fee_recipients,
    }
}

pub(crate) fn pumpswap_pool_from_pb(p: sol_parser_sdk::core::events::PumpSwapPool) -> Pool {
    Pool {
        pool_bump: p.pool_bump,
        index: p.index,
        creator: p.creator,
        base_mint: p.base_mint,
        quote_mint: p.quote_mint,
        lp_mint: p.lp_mint,
        pool_base_token_account: p.pool_base_token_account,
        pool_quote_token_account: p.pool_quote_token_account,
        lp_supply: p.lp_supply,
        coin_creator: p.coin_creator,
        is_mayhem_mode: p.is_mayhem_mode,
        is_cashback_coin: p.is_cashback_coin,
        virtual_quote_reserves: p.virtual_quote_reserves,
    }
}

pub(crate) fn pumpswap_global_config_account_from_parser(
    e: sol_parser_sdk::core::events::PumpSwapGlobalConfigAccountEvent,
    meta: EventMetadata,
) -> PumpSwapGlobalConfigAccountEvent {
    PumpSwapGlobalConfigAccountEvent {
        metadata: meta,
        pubkey: e.pubkey,
        executable: e.executable,
        lamports: e.lamports,
        owner: e.owner,
        rent_epoch: e.rent_epoch,
        global_config: pumpswap_global_config_from_pb(e.global_config),
    }
}

pub(crate) fn pumpswap_pool_account_from_parser(
    e: sol_parser_sdk::core::events::PumpSwapPoolAccountEvent,
    meta: EventMetadata,
) -> PumpSwapPoolAccountEvent {
    PumpSwapPoolAccountEvent {
        metadata: meta,
        pubkey: e.pubkey,
        executable: e.executable,
        lamports: e.lamports,
        owner: e.owner,
        rent_epoch: e.rent_epoch,
        pool: pumpswap_pool_from_pb(e.pool),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sol_parser_sdk::core::events::{
        BaseMintParam, EventMetadata as ParserEventMetadata, RaydiumLaunchlabPoolCreateEvent,
        RaydiumLaunchlabTradeEvent, TradeDirection,
    };
    use solana_sdk::pubkey::Pubkey;

    #[test]
    fn launchlab_trade_preserves_complete_account_context() {
        let keys: Vec<_> = (0..11).map(|_| Pubkey::new_unique()).collect();
        let event = RaydiumLaunchlabTradeEvent {
            metadata: ParserEventMetadata::default(),
            pool_state: keys[0],
            user: keys[1],
            amount_in: 10,
            amount_out: 20,
            is_buy: true,
            trade_direction: TradeDirection::Buy,
            exact_in: true,
            global_config: keys[2],
            platform_config: keys[3],
            user_base_token: keys[4],
            user_quote_token: keys[5],
            base_vault: keys[6],
            quote_vault: keys[7],
            base_mint: keys[8],
            quote_mint: keys[9],
            base_token_program: keys[10],
            quote_token_program: Pubkey::new_unique(),
        };
        let quote_token_program = event.quote_token_program;

        let mapped = bonk_trade_from_parser(event, EventMetadata::default());

        assert_eq!(mapped.pool_state, keys[0]);
        assert_eq!(mapped.payer, keys[1]);
        assert_eq!(mapped.global_config, keys[2]);
        assert_eq!(mapped.platform_config, keys[3]);
        assert_eq!(mapped.user_base_token, keys[4]);
        assert_eq!(mapped.user_quote_token, keys[5]);
        assert_eq!(mapped.base_vault, keys[6]);
        assert_eq!(mapped.quote_vault, keys[7]);
        assert_eq!(mapped.base_token_mint, keys[8]);
        assert_eq!(mapped.quote_token_mint, keys[9]);
        assert_eq!(mapped.base_token_program, keys[10]);
        assert_eq!(mapped.quote_token_program, quote_token_program);
    }

    #[test]
    fn launchlab_pool_create_preserves_complete_account_context() {
        let keys: Vec<_> = (0..11).map(|_| Pubkey::new_unique()).collect();
        let event = RaydiumLaunchlabPoolCreateEvent {
            metadata: ParserEventMetadata::default(),
            base_mint_param: BaseMintParam {
                symbol: "USD1".to_string(),
                name: "USD1 pool".to_string(),
                uri: String::new(),
                decimals: 6,
            },
            pool_state: keys[0],
            payer: keys[1],
            creator: keys[2],
            global_config: keys[3],
            platform_config: keys[4],
            base_mint: keys[5],
            quote_mint: keys[6],
            base_vault: keys[7],
            quote_vault: keys[8],
            base_token_program: keys[9],
            quote_token_program: keys[10],
        };

        let mapped = bonk_pool_create_from_parser(event, EventMetadata::default());

        assert_eq!(mapped.pool_state, keys[0]);
        assert_eq!(mapped.payer, keys[1]);
        assert_eq!(mapped.creator, keys[2]);
        assert_eq!(mapped.config, Pubkey::default());
        assert_eq!(mapped.global_config, keys[3]);
        assert_eq!(mapped.platform_config, keys[4]);
        assert_eq!(mapped.base_mint, keys[5]);
        assert_eq!(mapped.quote_mint, keys[6]);
        assert_eq!(mapped.base_vault, keys[7]);
        assert_eq!(mapped.quote_vault, keys[8]);
        assert_eq!(mapped.base_token_program, keys[9]);
        assert_eq!(mapped.quote_token_program, keys[10]);
    }
}
