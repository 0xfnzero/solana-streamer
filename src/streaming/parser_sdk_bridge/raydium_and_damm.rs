//! Raydium CPMM / CLMM / AMM V4 and Meteora DAMM v2 mapping.
use crate::streaming::event_parser::common::EventMetadata;
use crate::streaming::event_parser::protocols::meteora_damm_v2::events::{
    MeteoraDammV2AddLiquidityEvent, MeteoraDammV2ClosePositionEvent,
    MeteoraDammV2CreatePositionEvent, MeteoraDammV2RemoveLiquidityEvent, MeteoraDammV2SwapEvent,
};
use crate::streaming::event_parser::protocols::raydium_amm_v4::events::{
    RaydiumAmmV4DepositEvent, RaydiumAmmV4Initialize2Event, RaydiumAmmV4SwapEvent,
    RaydiumAmmV4WithdrawEvent, RaydiumAmmV4WithdrawPnlEvent,
};
use crate::streaming::event_parser::protocols::raydium_clmm::events::{
    RaydiumClmmAmmConfigAccountEvent, RaydiumClmmClosePositionEvent, RaydiumClmmCollectFeeEvent,
    RaydiumClmmConfigChangeEvent, RaydiumClmmCreatePersonalPositionEvent,
    RaydiumClmmCreatePoolEvent, RaydiumClmmDecreaseLimitOrderEvent,
    RaydiumClmmDecreaseLiquidityV2Event, RaydiumClmmIncreaseLimitOrderEvent,
    RaydiumClmmIncreaseLiquidityV2Event, RaydiumClmmLiquidityCalculateEvent,
    RaydiumClmmLiquidityChangeEvent, RaydiumClmmOpenLimitOrderEvent,
    RaydiumClmmOpenPositionV2Event, RaydiumClmmOpenPositionWithToken22NftEvent,
    RaydiumClmmPoolStateAccountEvent, RaydiumClmmSettleLimitOrderEvent, RaydiumClmmSwapEvent,
    RaydiumClmmTickArrayStateAccountEvent, RaydiumClmmUpdateRewardInfosEvent,
};
use crate::streaming::event_parser::protocols::raydium_clmm::types::{
    AmmConfig as ClmmAmmConfig, DynamicFeeInfo as ClmmDynamicFeeInfo, PoolState as ClmmPoolState,
    RewardInfo as ClmmRewardInfo, TickArrayState as ClmmTickArrayState, TickState as ClmmTickState,
};
use crate::streaming::event_parser::protocols::raydium_cpmm::events::{
    RaydiumCpmmDepositEvent, RaydiumCpmmInitializeEvent, RaydiumCpmmSwapEvent,
    RaydiumCpmmWithdrawEvent,
};
use solana_sdk::pubkey::Pubkey;
pub(crate) fn meteora_damm_v2_swap_from_parser(
    e: sol_parser_sdk::core::events::MeteoraDammV2SwapEvent,
    meta: EventMetadata,
) -> MeteoraDammV2SwapEvent {
    MeteoraDammV2SwapEvent {
        metadata: meta,
        pool: e.pool,
        trade_direction: e.trade_direction,
        collect_fee_mode: 0,
        has_referral: e.has_referral,
        amount_0: e.amount_in,
        amount_1: 0,
        swap_mode: 0,
        included_fee_input_amount: e.actual_amount_in,
        excluded_fee_input_amount: e.amount_in,
        amount_left: 0,
        output_amount: e.output_amount,
        next_sqrt_price: e.next_sqrt_price,
        trading_fee: e.lp_fee,
        protocol_fee: e.protocol_fee,
        partner_fee: e.partner_fee,
        referral_fee: e.referral_fee,
        included_transfer_fee_amount_in: 0,
        included_transfer_fee_amount_out: 0,
        excluded_transfer_fee_amount_out: 0,
        current_timestamp: e.current_timestamp,
        reserve_a_amount: 0,
        reserve_b_amount: 0,
        pool_authority: Pubkey::default(),
        input_token_account: Pubkey::default(),
        output_token_account: Pubkey::default(),
        token_a_vault: e.token_a_vault,
        token_b_vault: e.token_b_vault,
        token_a_mint: e.token_a_mint,
        token_b_mint: e.token_b_mint,
        payer: Pubkey::default(),
        token_a_program: e.token_a_program,
        token_b_program: e.token_b_program,
        referral_token_account: None,
        event_authority: Pubkey::default(),
        program: Pubkey::default(),
    }
}

pub(crate) fn raydium_cpmm_swap_from_parser(
    e: sol_parser_sdk::core::events::RaydiumCpmmSwapEvent,
    meta: EventMetadata,
) -> RaydiumCpmmSwapEvent {
    RaydiumCpmmSwapEvent {
        metadata: meta,
        amount_in: e.input_amount,
        minimum_amount_out: 0,
        max_amount_in: e.input_amount,
        amount_out: e.output_amount,
        payer: Pubkey::default(),
        authority: Pubkey::default(),
        amm_config: Pubkey::default(),
        pool_state: e.pool_id,
        input_token_account: Pubkey::default(),
        output_token_account: Pubkey::default(),
        input_vault: Pubkey::default(),
        output_vault: Pubkey::default(),
        input_token_program: Pubkey::default(),
        output_token_program: Pubkey::default(),
        input_token_mint: Pubkey::default(),
        output_token_mint: Pubkey::default(),
        observation_state: Pubkey::default(),
    }
}

pub(crate) fn raydium_cpmm_deposit_from_parser(
    e: sol_parser_sdk::core::events::RaydiumCpmmDepositEvent,
    meta: EventMetadata,
) -> RaydiumCpmmDepositEvent {
    RaydiumCpmmDepositEvent {
        metadata: meta,
        lp_token_amount: e.lp_token_amount,
        maximum_token0_amount: e.token0_amount,
        maximum_token1_amount: e.token1_amount,
        owner: e.user,
        authority: Pubkey::default(),
        pool_state: e.pool,
        owner_lp_token: Pubkey::default(),
        token_0_account: Pubkey::default(),
        token_1_account: Pubkey::default(),
        token_0_vault: Pubkey::default(),
        token_1_vault: Pubkey::default(),
        token_program: Pubkey::default(),
        token_program2022: Pubkey::default(),
        vault_0_mint: Pubkey::default(),
        vault_1_mint: Pubkey::default(),
        lp_mint: Pubkey::default(),
    }
}

pub(crate) fn raydium_cpmm_withdraw_from_parser(
    e: sol_parser_sdk::core::events::RaydiumCpmmWithdrawEvent,
    meta: EventMetadata,
) -> RaydiumCpmmWithdrawEvent {
    RaydiumCpmmWithdrawEvent {
        metadata: meta,
        lp_token_amount: e.lp_token_amount,
        minimum_token0_amount: e.token0_amount,
        minimum_token1_amount: e.token1_amount,
        owner: e.user,
        authority: Pubkey::default(),
        pool_state: e.pool,
        owner_lp_token: Pubkey::default(),
        token_0_account: Pubkey::default(),
        token_1_account: Pubkey::default(),
        token_0_vault: Pubkey::default(),
        token_1_vault: Pubkey::default(),
        token_program: Pubkey::default(),
        token_program2022: Pubkey::default(),
        vault_0_mint: Pubkey::default(),
        vault_1_mint: Pubkey::default(),
        lp_mint: Pubkey::default(),
        memo_program: Pubkey::default(),
    }
}

pub(crate) fn raydium_cpmm_initialize_from_parser(
    e: sol_parser_sdk::core::events::RaydiumCpmmInitializeEvent,
    meta: EventMetadata,
) -> RaydiumCpmmInitializeEvent {
    RaydiumCpmmInitializeEvent {
        metadata: meta,
        init_amount0: e.init_amount0,
        init_amount1: e.init_amount1,
        open_time: 0,
        creator: e.creator,
        pool_state: e.pool,
        ..Default::default()
    }
}

pub(crate) fn raydium_amm_v4_swap_from_parser(
    e: sol_parser_sdk::core::events::RaydiumAmmV4SwapEvent,
    meta: EventMetadata,
) -> RaydiumAmmV4SwapEvent {
    RaydiumAmmV4SwapEvent {
        metadata: meta,
        amount_in: e.amount_in,
        minimum_amount_out: e.minimum_amount_out,
        max_amount_in: e.max_amount_in,
        amount_out: e.amount_out,
        token_program: e.token_program,
        amm: e.amm,
        amm_authority: e.amm_authority,
        amm_open_orders: e.amm_open_orders,
        amm_target_orders: e.amm_target_orders,
        pool_coin_token_account: e.pool_coin_token_account,
        pool_pc_token_account: e.pool_pc_token_account,
        serum_program: e.serum_program,
        serum_market: e.serum_market,
        serum_bids: e.serum_bids,
        serum_asks: e.serum_asks,
        serum_event_queue: e.serum_event_queue,
        serum_coin_vault_account: e.serum_coin_vault_account,
        serum_pc_vault_account: e.serum_pc_vault_account,
        serum_vault_signer: e.serum_vault_signer,
        user_source_token_account: e.user_source_token_account,
        user_destination_token_account: e.user_destination_token_account,
        user_source_owner: e.user_source_owner,
    }
}

pub(crate) fn raydium_amm_v4_deposit_from_parser(
    e: sol_parser_sdk::core::events::RaydiumAmmV4DepositEvent,
    meta: EventMetadata,
) -> RaydiumAmmV4DepositEvent {
    RaydiumAmmV4DepositEvent {
        metadata: meta,
        max_coin_amount: e.max_coin_amount,
        max_pc_amount: e.max_pc_amount,
        base_side: e.base_side,
        token_program: e.token_program,
        amm: e.amm,
        amm_authority: e.amm_authority,
        amm_open_orders: e.amm_open_orders,
        amm_target_orders: e.amm_target_orders,
        lp_mint_address: e.lp_mint_address,
        pool_coin_token_account: e.pool_coin_token_account,
        pool_pc_token_account: e.pool_pc_token_account,
        serum_market: e.serum_market,
        user_coin_token_account: e.user_coin_token_account,
        user_pc_token_account: e.user_pc_token_account,
        user_lp_token_account: e.user_lp_token_account,
        user_owner: e.user_owner,
        serum_event_queue: e.serum_event_queue,
    }
}

pub(crate) fn raydium_amm_v4_withdraw_from_parser(
    e: sol_parser_sdk::core::events::RaydiumAmmV4WithdrawEvent,
    meta: EventMetadata,
) -> RaydiumAmmV4WithdrawEvent {
    RaydiumAmmV4WithdrawEvent {
        metadata: meta,
        amount: e.amount,
        token_program: e.token_program,
        amm: e.amm,
        amm_authority: e.amm_authority,
        amm_open_orders: e.amm_open_orders,
        amm_target_orders: e.amm_target_orders,
        lp_mint_address: e.lp_mint_address,
        pool_coin_token_account: e.pool_coin_token_account,
        pool_pc_token_account: e.pool_pc_token_account,
        pool_withdraw_queue: e.pool_withdraw_queue,
        pool_temp_lp_token_account: e.pool_temp_lp_token_account,
        serum_program: e.serum_program,
        serum_market: e.serum_market,
        serum_coin_vault_account: e.serum_coin_vault_account,
        serum_pc_vault_account: e.serum_pc_vault_account,
        serum_vault_signer: e.serum_vault_signer,
        user_lp_token_account: e.user_lp_token_account,
        user_coin_token_account: e.user_coin_token_account,
        user_pc_token_account: e.user_pc_token_account,
        user_owner: e.user_owner,
        serum_event_queue: e.serum_event_queue,
        serum_bids: e.serum_bids,
        serum_asks: e.serum_asks,
    }
}

pub(crate) fn raydium_amm_v4_withdraw_pnl_from_parser(
    e: sol_parser_sdk::core::events::RaydiumAmmV4WithdrawPnlEvent,
    meta: EventMetadata,
) -> RaydiumAmmV4WithdrawPnlEvent {
    RaydiumAmmV4WithdrawPnlEvent {
        metadata: meta,
        token_program: e.token_program,
        amm: e.amm,
        amm_config: e.amm_config,
        amm_authority: e.amm_authority,
        amm_open_orders: e.amm_open_orders,
        pool_coin_token_account: e.pool_coin_token_account,
        pool_pc_token_account: e.pool_pc_token_account,
        coin_pnl_token_account: e.coin_pnl_token_account,
        pc_pnl_token_account: e.pc_pnl_token_account,
        pnl_owner_account: e.pnl_owner,
        amm_target_orders: e.amm_target_orders,
        serum_program: e.serum_program,
        serum_market: e.serum_market,
        serum_event_queue: e.serum_event_queue,
        serum_coin_vault_account: e.serum_coin_vault_account,
        serum_pc_vault_account: e.serum_pc_vault_account,
        serum_vault_signer: e.serum_vault_signer,
    }
}

pub(crate) fn raydium_amm_v4_initialize2_from_parser(
    e: sol_parser_sdk::core::events::RaydiumAmmV4Initialize2Event,
    meta: EventMetadata,
) -> RaydiumAmmV4Initialize2Event {
    RaydiumAmmV4Initialize2Event {
        metadata: meta,
        nonce: e.nonce,
        open_time: e.open_time,
        init_pc_amount: e.init_pc_amount,
        init_coin_amount: e.init_coin_amount,
        token_program: e.token_program,
        spl_associated_token_account: e.spl_associated_token_account,
        system_program: e.system_program,
        rent: e.rent,
        amm: e.amm,
        amm_authority: e.amm_authority,
        amm_open_orders: e.amm_open_orders,
        lp_mint: e.lp_mint,
        coin_mint: e.coin_mint,
        pc_mint: e.pc_mint,
        pool_coin_token_account: e.pool_coin_token_account,
        pool_pc_token_account: e.pool_pc_token_account,
        pool_withdraw_queue: e.pool_withdraw_queue,
        amm_target_orders: e.amm_target_orders,
        pool_temp_lp: e.pool_temp_lp,
        serum_program: e.serum_program,
        serum_market: e.serum_market,
        user_wallet: e.user_wallet,
        user_token_coin: e.user_token_coin,
        user_token_pc: e.user_token_pc,
        user_lp_token_account: e.user_lp_token_account,
    }
}

pub(crate) fn raydium_clmm_swap_from_parser(
    e: sol_parser_sdk::core::events::RaydiumClmmSwapEvent,
    meta: EventMetadata,
) -> RaydiumClmmSwapEvent {
    let (amount, other_amount_threshold, input_token_account, output_token_account) =
        if e.zero_for_one {
            (e.amount_0, e.amount_1, e.token_account_0, e.token_account_1)
        } else {
            (e.amount_1, e.amount_0, e.token_account_1, e.token_account_0)
        };
    RaydiumClmmSwapEvent {
        metadata: meta,
        amount,
        other_amount_threshold,
        sqrt_price_limit_x64: e.sqrt_price_x64,
        is_base_input: e.zero_for_one,
        pool_state: e.pool_state,
        sender: e.sender,
        token_account_0: e.token_account_0,
        token_account_1: e.token_account_1,
        amount_0: e.amount_0,
        transfer_fee_0: e.transfer_fee_0,
        amount_1: e.amount_1,
        transfer_fee_1: e.transfer_fee_1,
        zero_for_one: e.zero_for_one,
        sqrt_price_x64: e.sqrt_price_x64,
        liquidity: e.liquidity,
        tick: e.tick,
        payer: e.sender,
        input_token_account,
        output_token_account,
        ..Default::default()
    }
}

pub(crate) fn raydium_clmm_create_pool_from_parser(
    e: sol_parser_sdk::core::events::RaydiumClmmCreatePoolEvent,
    meta: EventMetadata,
) -> RaydiumClmmCreatePoolEvent {
    RaydiumClmmCreatePoolEvent {
        metadata: meta,
        sqrt_price_x64: e.sqrt_price_x64,
        tick: e.tick,
        tick_spacing: e.tick_spacing,
        open_time: e.open_time,
        pool_creator: e.creator,
        pool_state: e.pool,
        token_mint0: e.token_0_mint,
        token_mint1: e.token_1_mint,
        token_vault0: e.token_vault_0,
        token_vault1: e.token_vault_1,
        ..Default::default()
    }
}

pub(crate) fn raydium_clmm_open_position_v2_from_parser(
    e: sol_parser_sdk::core::events::RaydiumClmmOpenPositionEvent,
    meta: EventMetadata,
) -> RaydiumClmmOpenPositionV2Event {
    RaydiumClmmOpenPositionV2Event {
        metadata: meta,
        tick_lower_index: e.tick_lower_index,
        tick_upper_index: e.tick_upper_index,
        liquidity: e.liquidity,
        payer: e.user,
        position_nft_owner: e.user,
        position_nft_mint: e.position_nft_mint,
        pool_state: e.pool,
        ..Default::default()
    }
}

pub(crate) fn raydium_clmm_open_position_token22_from_parser(
    e: sol_parser_sdk::core::events::RaydiumClmmOpenPositionWithTokenExtNftEvent,
    meta: EventMetadata,
) -> RaydiumClmmOpenPositionWithToken22NftEvent {
    RaydiumClmmOpenPositionWithToken22NftEvent {
        metadata: meta,
        tick_lower_index: e.tick_lower_index,
        tick_upper_index: e.tick_upper_index,
        liquidity: e.liquidity,
        payer: e.user,
        position_nft_owner: e.user,
        position_nft_mint: e.position_nft_mint,
        pool_state: e.pool,
        ..Default::default()
    }
}

pub(crate) fn raydium_clmm_close_position_from_parser(
    e: sol_parser_sdk::core::events::RaydiumClmmClosePositionEvent,
    meta: EventMetadata,
) -> RaydiumClmmClosePositionEvent {
    RaydiumClmmClosePositionEvent {
        metadata: meta,
        nft_owner: e.user,
        position_nft_mint: e.position_nft_mint,
        ..Default::default()
    }
}

pub(crate) fn raydium_clmm_increase_liquidity_v2_from_parser(
    e: sol_parser_sdk::core::events::RaydiumClmmIncreaseLiquidityEvent,
    meta: EventMetadata,
) -> RaydiumClmmIncreaseLiquidityV2Event {
    RaydiumClmmIncreaseLiquidityV2Event {
        metadata: meta,
        position_nft_mint: e.position_nft_mint,
        liquidity: e.liquidity,
        amount_0: e.amount_0,
        amount_1: e.amount_1,
        amount_0_transfer_fee: e.amount_0_transfer_fee,
        amount_1_transfer_fee: e.amount_1_transfer_fee,
        amount0_max: if e.amount0_max != 0 { e.amount0_max } else { e.amount_0 },
        amount1_max: if e.amount1_max != 0 { e.amount1_max } else { e.amount_1 },
        nft_owner: e.user,
        pool_state: e.pool,
        ..Default::default()
    }
}

pub(crate) fn raydium_clmm_decrease_liquidity_v2_from_parser(
    e: sol_parser_sdk::core::events::RaydiumClmmDecreaseLiquidityEvent,
    meta: EventMetadata,
) -> RaydiumClmmDecreaseLiquidityV2Event {
    RaydiumClmmDecreaseLiquidityV2Event {
        metadata: meta,
        position_nft_mint: e.position_nft_mint,
        liquidity: e.liquidity,
        decrease_amount_0: e.decrease_amount_0,
        decrease_amount_1: e.decrease_amount_1,
        fee_amount_0: e.fee_amount_0,
        fee_amount_1: e.fee_amount_1,
        reward_amounts: e.reward_amounts,
        transfer_fee_0: e.transfer_fee_0,
        transfer_fee_1: e.transfer_fee_1,
        amount0_min: if e.amount0_min != 0 { e.amount0_min } else { e.decrease_amount_0 },
        amount1_min: if e.amount1_min != 0 { e.amount1_min } else { e.decrease_amount_1 },
        nft_owner: e.user,
        pool_state: e.pool,
        ..Default::default()
    }
}

pub(crate) fn raydium_clmm_collect_fee_from_parser(
    e: sol_parser_sdk::core::events::RaydiumClmmCollectFeeEvent,
    meta: EventMetadata,
) -> RaydiumClmmCollectFeeEvent {
    RaydiumClmmCollectFeeEvent {
        metadata: meta,
        pool_state: e.pool_state,
        position_nft_mint: e.position_nft_mint,
        recipient_token_account_0: e.recipient_token_account_0,
        recipient_token_account_1: e.recipient_token_account_1,
        amount_0: e.amount_0,
        amount_1: e.amount_1,
    }
}

pub(crate) fn raydium_clmm_liquidity_change_from_parser(
    e: sol_parser_sdk::core::events::RaydiumClmmLiquidityChangeEvent,
    meta: EventMetadata,
) -> RaydiumClmmLiquidityChangeEvent {
    RaydiumClmmLiquidityChangeEvent {
        metadata: meta,
        pool_state: e.pool_state,
        tick: e.tick,
        tick_lower: e.tick_lower,
        tick_upper: e.tick_upper,
        liquidity_before: e.liquidity_before,
        liquidity_after: e.liquidity_after,
    }
}

pub(crate) fn raydium_clmm_config_change_from_parser(
    e: sol_parser_sdk::core::events::RaydiumClmmConfigChangeEvent,
    meta: EventMetadata,
) -> RaydiumClmmConfigChangeEvent {
    RaydiumClmmConfigChangeEvent {
        metadata: meta,
        index: e.index,
        owner: e.owner,
        protocol_fee_rate: e.protocol_fee_rate,
        trade_fee_rate: e.trade_fee_rate,
        tick_spacing: e.tick_spacing,
        fund_fee_rate: e.fund_fee_rate,
        fund_owner: e.fund_owner,
    }
}

pub(crate) fn raydium_clmm_create_personal_position_from_parser(
    e: sol_parser_sdk::core::events::RaydiumClmmCreatePersonalPositionEvent,
    meta: EventMetadata,
) -> RaydiumClmmCreatePersonalPositionEvent {
    RaydiumClmmCreatePersonalPositionEvent {
        metadata: meta,
        pool_state: e.pool_state,
        minter: e.minter,
        nft_owner: e.nft_owner,
        tick_lower_index: e.tick_lower_index,
        tick_upper_index: e.tick_upper_index,
        liquidity: e.liquidity,
        deposit_amount_0: e.deposit_amount_0,
        deposit_amount_1: e.deposit_amount_1,
        deposit_amount_0_transfer_fee: e.deposit_amount_0_transfer_fee,
        deposit_amount_1_transfer_fee: e.deposit_amount_1_transfer_fee,
    }
}

pub(crate) fn raydium_clmm_liquidity_calculate_from_parser(
    e: sol_parser_sdk::core::events::RaydiumClmmLiquidityCalculateEvent,
    meta: EventMetadata,
) -> RaydiumClmmLiquidityCalculateEvent {
    RaydiumClmmLiquidityCalculateEvent {
        metadata: meta,
        pool_liquidity: e.pool_liquidity,
        pool_sqrt_price_x64: e.pool_sqrt_price_x64,
        pool_tick: e.pool_tick,
        calc_amount_0: e.calc_amount_0,
        calc_amount_1: e.calc_amount_1,
        trade_fee_owed_0: e.trade_fee_owed_0,
        trade_fee_owed_1: e.trade_fee_owed_1,
        transfer_fee_0: e.transfer_fee_0,
        transfer_fee_1: e.transfer_fee_1,
    }
}

pub(crate) fn raydium_clmm_open_limit_order_from_parser(
    e: sol_parser_sdk::core::events::RaydiumClmmOpenLimitOrderEvent,
    meta: EventMetadata,
) -> RaydiumClmmOpenLimitOrderEvent {
    RaydiumClmmOpenLimitOrderEvent {
        metadata: meta,
        pool_id: e.pool_id,
        limit_order: e.limit_order,
        zero_for_one: e.zero_for_one,
        tick_index: e.tick_index,
        total_amount: e.total_amount,
        transfer_fee: e.transfer_fee,
    }
}

pub(crate) fn raydium_clmm_increase_limit_order_from_parser(
    e: sol_parser_sdk::core::events::RaydiumClmmIncreaseLimitOrderEvent,
    meta: EventMetadata,
) -> RaydiumClmmIncreaseLimitOrderEvent {
    RaydiumClmmIncreaseLimitOrderEvent {
        metadata: meta,
        pool_id: e.pool_id,
        limit_order: e.limit_order,
        zero_for_one: e.zero_for_one,
        tick_index: e.tick_index,
        total_amount: e.total_amount,
        increased_amount: e.increased_amount,
        transfer_fee: e.transfer_fee,
    }
}

pub(crate) fn raydium_clmm_decrease_limit_order_from_parser(
    e: sol_parser_sdk::core::events::RaydiumClmmDecreaseLimitOrderEvent,
    meta: EventMetadata,
) -> RaydiumClmmDecreaseLimitOrderEvent {
    RaydiumClmmDecreaseLimitOrderEvent {
        metadata: meta,
        pool_id: e.pool_id,
        limit_order: e.limit_order,
        zero_for_one: e.zero_for_one,
        tick_index: e.tick_index,
        total_amount: e.total_amount,
        filled_amount: e.filled_amount,
        settled_output_amount: e.settled_output_amount,
        decreased_amount: e.decreased_amount,
    }
}

pub(crate) fn raydium_clmm_settle_limit_order_from_parser(
    e: sol_parser_sdk::core::events::RaydiumClmmSettleLimitOrderEvent,
    meta: EventMetadata,
) -> RaydiumClmmSettleLimitOrderEvent {
    RaydiumClmmSettleLimitOrderEvent {
        metadata: meta,
        pool_id: e.pool_id,
        limit_order: e.limit_order,
        zero_for_one: e.zero_for_one,
        tick_index: e.tick_index,
        total_amount: e.total_amount,
        filled_amount: e.filled_amount,
        settled_amount_out: e.settled_amount_out,
    }
}

pub(crate) fn raydium_clmm_update_reward_infos_from_parser(
    e: sol_parser_sdk::core::events::RaydiumClmmUpdateRewardInfosEvent,
    meta: EventMetadata,
) -> RaydiumClmmUpdateRewardInfosEvent {
    RaydiumClmmUpdateRewardInfosEvent {
        metadata: meta,
        reward_growth_global_x64: e.reward_growth_global_x64,
    }
}

pub(crate) fn raydium_clmm_amm_config_account_from_parser(
    e: sol_parser_sdk::core::events::RaydiumClmmAmmConfigAccountEvent,
    meta: EventMetadata,
) -> RaydiumClmmAmmConfigAccountEvent {
    RaydiumClmmAmmConfigAccountEvent {
        metadata: meta,
        pubkey: e.pubkey,
        amm_config: raydium_clmm_amm_config_from_parser(e.amm_config),
        ..Default::default()
    }
}

pub(crate) fn raydium_clmm_pool_state_account_from_parser(
    e: sol_parser_sdk::core::events::RaydiumClmmPoolStateAccountEvent,
    meta: EventMetadata,
) -> RaydiumClmmPoolStateAccountEvent {
    RaydiumClmmPoolStateAccountEvent {
        metadata: meta,
        pubkey: e.pubkey,
        pool_state: raydium_clmm_pool_state_from_parser(e.pool_state),
        ..Default::default()
    }
}

pub(crate) fn raydium_clmm_tick_array_state_account_from_parser(
    e: sol_parser_sdk::core::events::RaydiumClmmTickArrayStateAccountEvent,
    meta: EventMetadata,
) -> RaydiumClmmTickArrayStateAccountEvent {
    RaydiumClmmTickArrayStateAccountEvent {
        metadata: meta,
        pubkey: e.pubkey,
        tick_array_state: raydium_clmm_tick_array_state_from_parser(e.tick_array_state),
        ..Default::default()
    }
}

fn raydium_clmm_amm_config_from_parser(
    e: sol_parser_sdk::core::events::RaydiumClmmAmmConfig,
) -> ClmmAmmConfig {
    ClmmAmmConfig {
        bump: e.bump,
        index: e.index,
        owner: e.owner,
        protocol_fee_rate: e.protocol_fee_rate,
        trade_fee_rate: e.trade_fee_rate,
        tick_spacing: e.tick_spacing,
        fund_fee_rate: e.fund_fee_rate,
        padding_u32: e.padding_u32,
        fund_owner: e.fund_owner,
        padding: e.padding,
    }
}

fn raydium_clmm_pool_state_from_parser(
    e: sol_parser_sdk::core::events::RaydiumClmmPoolState,
) -> ClmmPoolState {
    ClmmPoolState {
        bump: e.bump,
        amm_config: e.amm_config,
        owner: e.owner,
        token_mint_0: e.token_mint_0,
        token_mint_1: e.token_mint_1,
        token_vault_0: e.token_vault_0,
        token_vault_1: e.token_vault_1,
        observation_key: e.observation_key,
        mint_decimals_0: e.mint_decimals_0,
        mint_decimals_1: e.mint_decimals_1,
        tick_spacing: e.tick_spacing,
        liquidity: e.liquidity,
        sqrt_price_x64: e.sqrt_price_x64,
        tick_current: e.tick_current,
        padding3: e.padding3,
        padding4: e.padding4,
        fee_growth_global0_x64: e.fee_growth_global_0_x64,
        fee_growth_global1_x64: e.fee_growth_global_1_x64,
        protocol_fees_token0: e.protocol_fees_token_0,
        protocol_fees_token1: e.protocol_fees_token_1,
        padding5: e.padding5,
        status: e.status,
        fee_on: e.fee_on,
        padding: e.padding,
        reward_infos: [
            raydium_clmm_reward_info_from_parser(e.reward_infos[0].clone()),
            raydium_clmm_reward_info_from_parser(e.reward_infos[1].clone()),
            raydium_clmm_reward_info_from_parser(e.reward_infos[2].clone()),
        ],
        tick_array_bitmap: e.tick_array_bitmap,
        padding6: e.padding6,
        fund_fees_token0: e.fund_fees_token_0,
        fund_fees_token1: e.fund_fees_token_1,
        open_time: e.open_time,
        recent_epoch: e.recent_epoch,
        dynamic_fee_info: raydium_clmm_dynamic_fee_info_from_parser(e.dynamic_fee_info),
        padding1: e.padding1,
        padding2: e.padding2,
    }
}

fn raydium_clmm_reward_info_from_parser(
    e: sol_parser_sdk::core::events::RaydiumClmmRewardInfo,
) -> ClmmRewardInfo {
    ClmmRewardInfo {
        reward_state: e.reward_state,
        open_time: e.open_time,
        end_time: e.end_time,
        last_update_time: e.last_update_time,
        emissions_per_second_x64: e.emissions_per_second_x64,
        reward_total_emitted: e.reward_total_emitted,
        reward_claimed: e.reward_claimed,
        token_mint: e.token_mint,
        token_vault: e.token_vault,
        authority: e.authority,
        reward_growth_global_x64: e.reward_growth_global_x64,
    }
}

fn raydium_clmm_dynamic_fee_info_from_parser(
    e: sol_parser_sdk::core::events::RaydiumClmmDynamicFeeInfo,
) -> ClmmDynamicFeeInfo {
    ClmmDynamicFeeInfo {
        filter_period: e.filter_period,
        decay_period: e.decay_period,
        reduction_factor: e.reduction_factor,
        dynamic_fee_control: e.dynamic_fee_control,
        max_volatility_accumulator: e.max_volatility_accumulator,
        tick_spacing_index_reference: e.tick_spacing_index_reference,
        volatility_reference: e.volatility_reference,
        volatility_accumulator: e.volatility_accumulator,
        last_update_timestamp: e.last_update_timestamp,
        padding: e.padding,
    }
}

fn raydium_clmm_tick_array_state_from_parser(
    e: sol_parser_sdk::core::events::RaydiumClmmTickArrayState,
) -> ClmmTickArrayState {
    ClmmTickArrayState {
        pool_id: e.pool_id,
        start_tick_index: e.start_tick_index,
        ticks: core::array::from_fn(|i| {
            e.ticks.get(i).cloned().map(raydium_clmm_tick_state_from_parser).unwrap_or_default()
        }),
        initialized_tick_count: e.initialized_tick_count,
        recent_epoch: e.recent_epoch,
        padding: e.padding,
    }
}

fn raydium_clmm_tick_state_from_parser(e: sol_parser_sdk::core::events::Tick) -> ClmmTickState {
    ClmmTickState {
        tick: e.tick,
        liquidity_net: e.liquidity_net,
        liquidity_gross: e.liquidity_gross,
        fee_growth_outside0_x64: e.fee_growth_outside_0_x64,
        fee_growth_outside1_x64: e.fee_growth_outside_1_x64,
        reward_growths_outside_x64: e.reward_growths_outside_x64,
        order_phase: e.order_phase,
        orders_amount: e.orders_amount,
        part_filled_orders_remaining: e.part_filled_orders_remaining,
        unfilled_ratio_x64: e.unfilled_ratio_x64,
        padding: e.padding,
    }
}

pub(crate) fn meteora_damm_v2_add_liquidity_from_pb(
    e: sol_parser_sdk::core::events::MeteoraDammV2AddLiquidityEvent,
    meta: EventMetadata,
) -> MeteoraDammV2AddLiquidityEvent {
    MeteoraDammV2AddLiquidityEvent {
        metadata: meta,
        pool: e.pool,
        position: e.position,
        owner: e.owner,
        token_a_amount: e.token_a_amount,
        token_b_amount: e.token_b_amount,
        liquidity_delta: e.liquidity_delta,
        token_a_amount_threshold: e.token_a_amount_threshold,
        token_b_amount_threshold: e.token_b_amount_threshold,
        total_amount_a: e.total_amount_a,
        total_amount_b: e.total_amount_b,
    }
}

pub(crate) fn meteora_damm_v2_remove_liquidity_from_pb(
    e: sol_parser_sdk::core::events::MeteoraDammV2RemoveLiquidityEvent,
    meta: EventMetadata,
) -> MeteoraDammV2RemoveLiquidityEvent {
    MeteoraDammV2RemoveLiquidityEvent {
        metadata: meta,
        pool: e.pool,
        position: e.position,
        owner: e.owner,
        token_a_amount: e.token_a_amount,
        token_b_amount: e.token_b_amount,
        liquidity_delta: e.liquidity_delta,
        token_a_amount_threshold: e.token_a_amount_threshold,
        token_b_amount_threshold: e.token_b_amount_threshold,
    }
}

pub(crate) fn meteora_damm_v2_create_position_from_pb(
    e: sol_parser_sdk::core::events::MeteoraDammV2CreatePositionEvent,
    meta: EventMetadata,
) -> MeteoraDammV2CreatePositionEvent {
    MeteoraDammV2CreatePositionEvent {
        metadata: meta,
        pool: e.pool,
        owner: e.owner,
        position: e.position,
        position_nft_mint: e.position_nft_mint,
    }
}

pub(crate) fn meteora_damm_v2_close_position_from_pb(
    e: sol_parser_sdk::core::events::MeteoraDammV2ClosePositionEvent,
    meta: EventMetadata,
) -> MeteoraDammV2ClosePositionEvent {
    MeteoraDammV2ClosePositionEvent {
        metadata: meta,
        pool: e.pool,
        owner: e.owner,
        position: e.position,
        position_nft_mint: e.position_nft_mint,
    }
}
