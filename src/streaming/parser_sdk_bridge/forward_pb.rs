//! Orca Whirlpool and Meteora Pools / DLMM events with SDK-shaped payloads.
use crate::streaming::event_parser::common::EventMetadata;
use crate::streaming::event_parser::protocols::sol_parser_forward::events::{
    MeteoraDbcCurveCompleteEvent, MeteoraDbcInitializePoolEvent, MeteoraDbcSwapEvent,
    MeteoraDlmmAddLiquidityEvent, MeteoraDlmmClaimFeeEvent, MeteoraDlmmClosePositionEvent,
    MeteoraDlmmCreatePositionEvent, MeteoraDlmmInitializeBinArrayEvent,
    MeteoraDlmmInitializePoolEvent, MeteoraDlmmRemoveLiquidityEvent, MeteoraDlmmSwapEvent,
    MeteoraPoolsAddLiquidityEvent, MeteoraPoolsBootstrapLiquidityEvent,
    MeteoraPoolsPoolCreatedEvent, MeteoraPoolsRemoveLiquidityEvent, MeteoraPoolsSetPoolFeesEvent,
    MeteoraPoolsSwapEvent, OrcaFeeTierAccount, OrcaFeeTierAccountEvent, OrcaPositionAccount,
    OrcaPositionAccountEvent, OrcaPositionRewardInfo, OrcaTick, OrcaTickArrayAccount,
    OrcaTickArrayAccountEvent, OrcaWhirlpoolAccount, OrcaWhirlpoolAccountEvent,
    OrcaWhirlpoolLiquidityDecreasedEvent, OrcaWhirlpoolLiquidityIncreasedEvent,
    OrcaWhirlpoolPoolInitializedEvent, OrcaWhirlpoolRewardInfo, OrcaWhirlpoolSwapEvent,
    OrcaWhirlpoolsConfigAccount, OrcaWhirlpoolsConfigAccountEvent,
};

pub(crate) fn orca_swap_from_pb(
    e: sol_parser_sdk::core::events::OrcaWhirlpoolSwapEvent,
    meta: EventMetadata,
) -> OrcaWhirlpoolSwapEvent {
    OrcaWhirlpoolSwapEvent {
        metadata: meta,
        whirlpool: e.whirlpool,
        input_amount: e.input_amount,
        output_amount: e.output_amount,
        a_to_b: e.a_to_b,
        pre_sqrt_price: e.pre_sqrt_price,
        post_sqrt_price: e.post_sqrt_price,
        input_transfer_fee: e.input_transfer_fee,
        output_transfer_fee: e.output_transfer_fee,
        lp_fee: e.lp_fee,
        protocol_fee: e.protocol_fee,
    }
}

pub(crate) fn orca_liquidity_increased_from_pb(
    e: sol_parser_sdk::core::events::OrcaWhirlpoolLiquidityIncreasedEvent,
    meta: EventMetadata,
) -> OrcaWhirlpoolLiquidityIncreasedEvent {
    OrcaWhirlpoolLiquidityIncreasedEvent {
        metadata: meta,
        whirlpool: e.whirlpool,
        liquidity: e.liquidity,
        token_a_amount: e.token_a_amount,
        token_b_amount: e.token_b_amount,
        position: e.position,
        tick_lower_index: e.tick_lower_index,
        tick_upper_index: e.tick_upper_index,
        token_a_transfer_fee: e.token_a_transfer_fee,
        token_b_transfer_fee: e.token_b_transfer_fee,
    }
}

pub(crate) fn orca_liquidity_decreased_from_pb(
    e: sol_parser_sdk::core::events::OrcaWhirlpoolLiquidityDecreasedEvent,
    meta: EventMetadata,
) -> OrcaWhirlpoolLiquidityDecreasedEvent {
    OrcaWhirlpoolLiquidityDecreasedEvent {
        metadata: meta,
        whirlpool: e.whirlpool,
        liquidity: e.liquidity,
        token_a_amount: e.token_a_amount,
        token_b_amount: e.token_b_amount,
        position: e.position,
        tick_lower_index: e.tick_lower_index,
        tick_upper_index: e.tick_upper_index,
        token_a_transfer_fee: e.token_a_transfer_fee,
        token_b_transfer_fee: e.token_b_transfer_fee,
    }
}

pub(crate) fn orca_pool_initialized_from_pb(
    e: sol_parser_sdk::core::events::OrcaWhirlpoolPoolInitializedEvent,
    meta: EventMetadata,
) -> OrcaWhirlpoolPoolInitializedEvent {
    OrcaWhirlpoolPoolInitializedEvent {
        metadata: meta,
        whirlpool: e.whirlpool,
        whirlpools_config: e.whirlpools_config,
        token_mint_a: e.token_mint_a,
        token_mint_b: e.token_mint_b,
        tick_spacing: e.tick_spacing,
        token_program_a: e.token_program_a,
        token_program_b: e.token_program_b,
        decimals_a: e.decimals_a,
        decimals_b: e.decimals_b,
        initial_sqrt_price: e.initial_sqrt_price,
    }
}

fn orca_whirlpool_reward_from_pb(
    r: sol_parser_sdk::core::events::OrcaWhirlpoolRewardInfo,
) -> OrcaWhirlpoolRewardInfo {
    OrcaWhirlpoolRewardInfo {
        mint: r.mint,
        vault: r.vault,
        authority: r.authority,
        emissions_per_second_x64: r.emissions_per_second_x64,
        growth_global_x64: r.growth_global_x64,
    }
}

fn orca_position_reward_from_pb(
    r: sol_parser_sdk::core::events::OrcaPositionRewardInfo,
) -> OrcaPositionRewardInfo {
    OrcaPositionRewardInfo {
        growth_inside_checkpoint: r.growth_inside_checkpoint,
        amount_owed: r.amount_owed,
    }
}

fn orca_tick_from_pb(t: sol_parser_sdk::core::events::OrcaTick) -> OrcaTick {
    OrcaTick {
        initialized: t.initialized,
        liquidity_net: t.liquidity_net,
        liquidity_gross: t.liquidity_gross,
        fee_growth_outside_a: t.fee_growth_outside_a,
        fee_growth_outside_b: t.fee_growth_outside_b,
        reward_growths_outside: t.reward_growths_outside,
    }
}

pub(crate) fn orca_whirlpool_account_from_pb(
    e: sol_parser_sdk::core::events::OrcaWhirlpoolAccountEvent,
    meta: EventMetadata,
) -> OrcaWhirlpoolAccountEvent {
    OrcaWhirlpoolAccountEvent {
        metadata: meta,
        pubkey: e.pubkey,
        whirlpool: OrcaWhirlpoolAccount {
            whirlpools_config: e.whirlpool.whirlpools_config,
            whirlpool_bump: e.whirlpool.whirlpool_bump,
            tick_spacing: e.whirlpool.tick_spacing,
            tick_spacing_seed: e.whirlpool.tick_spacing_seed,
            fee_rate: e.whirlpool.fee_rate,
            protocol_fee_rate: e.whirlpool.protocol_fee_rate,
            liquidity: e.whirlpool.liquidity,
            sqrt_price: e.whirlpool.sqrt_price,
            tick_current_index: e.whirlpool.tick_current_index,
            protocol_fee_owed_a: e.whirlpool.protocol_fee_owed_a,
            protocol_fee_owed_b: e.whirlpool.protocol_fee_owed_b,
            token_mint_a: e.whirlpool.token_mint_a,
            token_vault_a: e.whirlpool.token_vault_a,
            fee_growth_global_a: e.whirlpool.fee_growth_global_a,
            token_mint_b: e.whirlpool.token_mint_b,
            token_vault_b: e.whirlpool.token_vault_b,
            fee_growth_global_b: e.whirlpool.fee_growth_global_b,
            reward_last_updated_timestamp: e.whirlpool.reward_last_updated_timestamp,
            reward_infos: e.whirlpool.reward_infos.map(orca_whirlpool_reward_from_pb),
        },
        ..Default::default()
    }
}

pub(crate) fn orca_position_account_from_pb(
    e: sol_parser_sdk::core::events::OrcaPositionAccountEvent,
    meta: EventMetadata,
) -> OrcaPositionAccountEvent {
    OrcaPositionAccountEvent {
        metadata: meta,
        pubkey: e.pubkey,
        position: OrcaPositionAccount {
            whirlpool: e.position.whirlpool,
            position_mint: e.position.position_mint,
            liquidity: e.position.liquidity,
            tick_lower_index: e.position.tick_lower_index,
            tick_upper_index: e.position.tick_upper_index,
            fee_growth_checkpoint_a: e.position.fee_growth_checkpoint_a,
            fee_owed_a: e.position.fee_owed_a,
            fee_growth_checkpoint_b: e.position.fee_growth_checkpoint_b,
            fee_owed_b: e.position.fee_owed_b,
            reward_infos: e.position.reward_infos.map(orca_position_reward_from_pb),
        },
        ..Default::default()
    }
}

pub(crate) fn orca_tick_array_account_from_pb(
    e: sol_parser_sdk::core::events::OrcaTickArrayAccountEvent,
    meta: EventMetadata,
) -> OrcaTickArrayAccountEvent {
    OrcaTickArrayAccountEvent {
        metadata: meta,
        pubkey: e.pubkey,
        tick_array: OrcaTickArrayAccount {
            start_tick_index: e.tick_array.start_tick_index,
            ticks: e.tick_array.ticks.into_iter().map(orca_tick_from_pb).collect(),
            whirlpool: e.tick_array.whirlpool,
        },
        ..Default::default()
    }
}

pub(crate) fn orca_fee_tier_account_from_pb(
    e: sol_parser_sdk::core::events::OrcaFeeTierAccountEvent,
    meta: EventMetadata,
) -> OrcaFeeTierAccountEvent {
    OrcaFeeTierAccountEvent {
        metadata: meta,
        pubkey: e.pubkey,
        fee_tier: OrcaFeeTierAccount {
            whirlpools_config: e.fee_tier.whirlpools_config,
            tick_spacing: e.fee_tier.tick_spacing,
            default_fee_rate: e.fee_tier.default_fee_rate,
        },
        ..Default::default()
    }
}

pub(crate) fn orca_whirlpools_config_account_from_pb(
    e: sol_parser_sdk::core::events::OrcaWhirlpoolsConfigAccountEvent,
    meta: EventMetadata,
) -> OrcaWhirlpoolsConfigAccountEvent {
    OrcaWhirlpoolsConfigAccountEvent {
        metadata: meta,
        pubkey: e.pubkey,
        config: OrcaWhirlpoolsConfigAccount {
            fee_authority: e.config.fee_authority,
            collect_protocol_fees_authority: e.config.collect_protocol_fees_authority,
            reward_emissions_super_authority: e.config.reward_emissions_super_authority,
            default_protocol_fee_rate: e.config.default_protocol_fee_rate,
        },
        ..Default::default()
    }
}

pub(crate) fn meteora_dbc_swap_from_pb(
    e: sol_parser_sdk::core::events::MeteoraDbcSwapEvent,
    meta: EventMetadata,
) -> MeteoraDbcSwapEvent {
    MeteoraDbcSwapEvent {
        metadata: meta,
        pool: e.pool,
        config: e.config,
        trade_direction: e.trade_direction,
        has_referral: e.has_referral,
        amount_in: e.amount_in,
        minimum_amount_out: e.minimum_amount_out,
        actual_input_amount: e.actual_input_amount,
        output_amount: e.output_amount,
        next_sqrt_price: e.next_sqrt_price,
        trading_fee: e.trading_fee,
        protocol_fee: e.protocol_fee,
        referral_fee: e.referral_fee,
        current_timestamp: e.current_timestamp,
    }
}

pub(crate) fn meteora_dbc_initialize_pool_from_pb(
    e: sol_parser_sdk::core::events::MeteoraDbcInitializePoolEvent,
    meta: EventMetadata,
) -> MeteoraDbcInitializePoolEvent {
    MeteoraDbcInitializePoolEvent {
        metadata: meta,
        pool: e.pool,
        config: e.config,
        creator: e.creator,
        base_mint: e.base_mint,
        pool_type: e.pool_type,
        activation_point: e.activation_point,
    }
}

pub(crate) fn meteora_dbc_curve_complete_from_pb(
    e: sol_parser_sdk::core::events::MeteoraDbcCurveCompleteEvent,
    meta: EventMetadata,
) -> MeteoraDbcCurveCompleteEvent {
    MeteoraDbcCurveCompleteEvent {
        metadata: meta,
        pool: e.pool,
        config: e.config,
        base_reserve: e.base_reserve,
        quote_reserve: e.quote_reserve,
    }
}

pub(crate) fn meteora_pools_swap_from_pb(
    e: sol_parser_sdk::core::events::MeteoraPoolsSwapEvent,
    meta: EventMetadata,
) -> MeteoraPoolsSwapEvent {
    MeteoraPoolsSwapEvent {
        metadata: meta,
        in_amount: e.in_amount,
        out_amount: e.out_amount,
        trade_fee: e.trade_fee,
        admin_fee: e.admin_fee,
        host_fee: e.host_fee,
    }
}

pub(crate) fn meteora_pools_add_liquidity_from_pb(
    e: sol_parser_sdk::core::events::MeteoraPoolsAddLiquidityEvent,
    meta: EventMetadata,
) -> MeteoraPoolsAddLiquidityEvent {
    MeteoraPoolsAddLiquidityEvent {
        metadata: meta,
        lp_mint_amount: e.lp_mint_amount,
        token_a_amount: e.token_a_amount,
        token_b_amount: e.token_b_amount,
    }
}

pub(crate) fn meteora_pools_remove_liquidity_from_pb(
    e: sol_parser_sdk::core::events::MeteoraPoolsRemoveLiquidityEvent,
    meta: EventMetadata,
) -> MeteoraPoolsRemoveLiquidityEvent {
    MeteoraPoolsRemoveLiquidityEvent {
        metadata: meta,
        lp_unmint_amount: e.lp_unmint_amount,
        token_a_out_amount: e.token_a_out_amount,
        token_b_out_amount: e.token_b_out_amount,
    }
}

pub(crate) fn meteora_pools_bootstrap_from_pb(
    e: sol_parser_sdk::core::events::MeteoraPoolsBootstrapLiquidityEvent,
    meta: EventMetadata,
) -> MeteoraPoolsBootstrapLiquidityEvent {
    MeteoraPoolsBootstrapLiquidityEvent {
        metadata: meta,
        lp_mint_amount: e.lp_mint_amount,
        token_a_amount: e.token_a_amount,
        token_b_amount: e.token_b_amount,
        pool: e.pool,
    }
}

pub(crate) fn meteora_pools_pool_created_from_pb(
    e: sol_parser_sdk::core::events::MeteoraPoolsPoolCreatedEvent,
    meta: EventMetadata,
) -> MeteoraPoolsPoolCreatedEvent {
    MeteoraPoolsPoolCreatedEvent {
        metadata: meta,
        lp_mint: e.lp_mint,
        token_a_mint: e.token_a_mint,
        token_b_mint: e.token_b_mint,
        pool_type: e.pool_type,
        pool: e.pool,
    }
}

pub(crate) fn meteora_pools_set_fees_from_pb(
    e: sol_parser_sdk::core::events::MeteoraPoolsSetPoolFeesEvent,
    meta: EventMetadata,
) -> MeteoraPoolsSetPoolFeesEvent {
    MeteoraPoolsSetPoolFeesEvent {
        metadata: meta,
        trade_fee_numerator: e.trade_fee_numerator,
        trade_fee_denominator: e.trade_fee_denominator,
        owner_trade_fee_numerator: e.owner_trade_fee_numerator,
        owner_trade_fee_denominator: e.owner_trade_fee_denominator,
        pool: e.pool,
    }
}

pub(crate) fn meteora_dlmm_swap_from_pb(
    e: sol_parser_sdk::core::events::MeteoraDlmmSwapEvent,
    meta: EventMetadata,
) -> MeteoraDlmmSwapEvent {
    MeteoraDlmmSwapEvent {
        metadata: meta,
        pool: e.pool,
        from: e.from,
        start_bin_id: e.start_bin_id,
        end_bin_id: e.end_bin_id,
        amount_in: e.amount_in,
        amount_out: e.amount_out,
        swap_for_y: e.swap_for_y,
        fee: e.fee,
        protocol_fee: e.protocol_fee,
        fee_bps: e.fee_bps,
        host_fee: e.host_fee,
    }
}

pub(crate) fn meteora_dlmm_add_liquidity_from_pb(
    e: sol_parser_sdk::core::events::MeteoraDlmmAddLiquidityEvent,
    meta: EventMetadata,
) -> MeteoraDlmmAddLiquidityEvent {
    MeteoraDlmmAddLiquidityEvent {
        metadata: meta,
        pool: e.pool,
        from: e.from,
        position: e.position,
        amounts: e.amounts,
        active_bin_id: e.active_bin_id,
    }
}

pub(crate) fn meteora_dlmm_remove_liquidity_from_pb(
    e: sol_parser_sdk::core::events::MeteoraDlmmRemoveLiquidityEvent,
    meta: EventMetadata,
) -> MeteoraDlmmRemoveLiquidityEvent {
    MeteoraDlmmRemoveLiquidityEvent {
        metadata: meta,
        pool: e.pool,
        from: e.from,
        position: e.position,
        amounts: e.amounts,
        active_bin_id: e.active_bin_id,
    }
}

pub(crate) fn meteora_dlmm_init_pool_from_pb(
    e: sol_parser_sdk::core::events::MeteoraDlmmInitializePoolEvent,
    meta: EventMetadata,
) -> MeteoraDlmmInitializePoolEvent {
    MeteoraDlmmInitializePoolEvent {
        metadata: meta,
        pool: e.pool,
        creator: e.creator,
        active_bin_id: e.active_bin_id,
        bin_step: e.bin_step,
    }
}

pub(crate) fn meteora_dlmm_init_bin_array_from_pb(
    e: sol_parser_sdk::core::events::MeteoraDlmmInitializeBinArrayEvent,
    meta: EventMetadata,
) -> MeteoraDlmmInitializeBinArrayEvent {
    MeteoraDlmmInitializeBinArrayEvent {
        metadata: meta,
        pool: e.pool,
        bin_array: e.bin_array,
        index: e.index,
    }
}

pub(crate) fn meteora_dlmm_create_position_from_pb(
    e: sol_parser_sdk::core::events::MeteoraDlmmCreatePositionEvent,
    meta: EventMetadata,
) -> MeteoraDlmmCreatePositionEvent {
    MeteoraDlmmCreatePositionEvent {
        metadata: meta,
        pool: e.pool,
        position: e.position,
        owner: e.owner,
        lower_bin_id: e.lower_bin_id,
        width: e.width,
    }
}

pub(crate) fn meteora_dlmm_close_position_from_pb(
    e: sol_parser_sdk::core::events::MeteoraDlmmClosePositionEvent,
    meta: EventMetadata,
) -> MeteoraDlmmClosePositionEvent {
    MeteoraDlmmClosePositionEvent {
        metadata: meta,
        pool: e.pool,
        position: e.position,
        owner: e.owner,
    }
}

pub(crate) fn meteora_dlmm_claim_fee_from_pb(
    e: sol_parser_sdk::core::events::MeteoraDlmmClaimFeeEvent,
    meta: EventMetadata,
) -> MeteoraDlmmClaimFeeEvent {
    MeteoraDlmmClaimFeeEvent {
        metadata: meta,
        pool: e.pool,
        position: e.position,
        owner: e.owner,
        fee_x: e.fee_x,
        fee_y: e.fee_y,
    }
}
