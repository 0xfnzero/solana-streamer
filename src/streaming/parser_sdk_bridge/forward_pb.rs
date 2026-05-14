//! Orca Whirlpool and Meteora Pools / DLMM events with SDK-shaped payloads.
use crate::streaming::event_parser::common::EventMetadata;
use crate::streaming::event_parser::protocols::sol_parser_forward::events::{
    MeteoraDlmmAddLiquidityEvent, MeteoraDlmmClaimFeeEvent, MeteoraDlmmClosePositionEvent,
    MeteoraDlmmCreatePositionEvent, MeteoraDlmmInitializeBinArrayEvent,
    MeteoraDlmmInitializePoolEvent, MeteoraDlmmRemoveLiquidityEvent, MeteoraDlmmSwapEvent,
    MeteoraPoolsAddLiquidityEvent, MeteoraPoolsBootstrapLiquidityEvent,
    MeteoraPoolsPoolCreatedEvent, MeteoraPoolsRemoveLiquidityEvent, MeteoraPoolsSetPoolFeesEvent,
    MeteoraPoolsSwapEvent, OrcaWhirlpoolLiquidityDecreasedEvent,
    OrcaWhirlpoolLiquidityIncreasedEvent, OrcaWhirlpoolPoolInitializedEvent,
    OrcaWhirlpoolSwapEvent,
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
