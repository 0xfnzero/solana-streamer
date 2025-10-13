use crate::streaming::event_parser::DexEvent;

pub fn merge(instruction_event: &mut DexEvent, cpi_log_event: DexEvent) {
    match instruction_event {
        // PumpFun events
        DexEvent::PumpFunTradeEvent(e) => match cpi_log_event {
            DexEvent::PumpFunTradeEvent(cpie) => {
                e.mint = cpie.mint;
                e.sol_amount = cpie.sol_amount;
                e.token_amount = cpie.token_amount;
                e.is_buy = cpie.is_buy;
                e.user = cpie.user;
                e.timestamp = cpie.timestamp;
                e.virtual_sol_reserves = cpie.virtual_sol_reserves;
                e.virtual_token_reserves = cpie.virtual_token_reserves;
                e.real_sol_reserves = cpie.real_sol_reserves;
                e.real_token_reserves = cpie.real_token_reserves;
                e.fee_recipient = cpie.fee_recipient;
                e.fee_basis_points = cpie.fee_basis_points;
                e.fee = cpie.fee;
                e.creator = cpie.creator;
                e.creator_fee_basis_points = cpie.creator_fee_basis_points;
                e.creator_fee = cpie.creator_fee;
            }
            _ => {}
        },
        DexEvent::PumpFunCreateTokenEvent(e) => match cpi_log_event {
            DexEvent::PumpFunCreateTokenEvent(cpie) => {
                e.mint = cpie.mint;
                e.bonding_curve = cpie.bonding_curve;
                e.user = cpie.user;
                e.creator = cpie.creator;
                e.timestamp = cpie.timestamp;
                e.virtual_token_reserves = cpie.virtual_token_reserves;
                e.virtual_sol_reserves = cpie.virtual_sol_reserves;
                e.real_token_reserves = cpie.real_token_reserves;
                e.token_total_supply = cpie.token_total_supply;
            }
            _ => {}
        },
        DexEvent::PumpFunMigrateEvent(e) => match cpi_log_event {
            DexEvent::PumpFunMigrateEvent(cpie) => {
                e.user = cpie.user;
                e.mint = cpie.mint;
                e.mint_amount = cpie.mint_amount;
                e.sol_amount = cpie.sol_amount;
                e.pool_migration_fee = cpie.pool_migration_fee;
                e.bonding_curve = cpie.bonding_curve;
                e.timestamp = cpie.timestamp;
                e.pool = cpie.pool;
            }
            _ => {}
        },

        // Bonk events
        DexEvent::BonkTradeEvent(e) => match cpi_log_event {
            DexEvent::BonkTradeEvent(cpie) => {
                e.pool_state = cpie.pool_state;
                e.total_base_sell = cpie.total_base_sell;
                e.virtual_base = cpie.virtual_base;
                e.virtual_quote = cpie.virtual_quote;
                e.real_base_before = cpie.real_base_before;
                e.real_quote_before = cpie.real_quote_before;
                e.real_base_after = cpie.real_base_after;
                e.real_quote_after = cpie.real_quote_after;
                e.amount_in = cpie.amount_in;
                e.amount_out = cpie.amount_out;
                e.protocol_fee = cpie.protocol_fee;
                e.platform_fee = cpie.platform_fee;
                e.creator_fee = cpie.creator_fee;
                e.share_fee = cpie.share_fee;
                e.trade_direction = cpie.trade_direction;
                e.pool_status = cpie.pool_status;
                e.exact_in = cpie.exact_in;
            }
            _ => {}
        },
        DexEvent::BonkPoolCreateEvent(e) => match cpi_log_event {
            DexEvent::BonkPoolCreateEvent(cpie) => {
                e.pool_state = cpie.pool_state;
                e.creator = cpie.creator;
                e.config = cpie.config;
                e.base_mint_param = cpie.base_mint_param;
                e.curve_param = cpie.curve_param;
                e.vesting_param = cpie.vesting_param;
                e.amm_fee_on = cpie.amm_fee_on;
            }
            _ => {}
        },
        DexEvent::BonkMigrateToAmmEvent(e) => match cpi_log_event {
            DexEvent::BonkMigrateToAmmEvent(cpie) => {
                e.base_lot_size = cpie.base_lot_size;
                e.quote_lot_size = cpie.quote_lot_size;
                e.market_vault_signer_nonce = cpie.market_vault_signer_nonce;
            }
            _ => {}
        },

        // PumpSwap events
        DexEvent::PumpSwapBuyEvent(e) => match cpi_log_event {
            DexEvent::PumpSwapBuyEvent(cpie) => {
                e.timestamp = cpie.timestamp;
                e.base_amount_out = cpie.base_amount_out;
                e.max_quote_amount_in = cpie.max_quote_amount_in;
                e.user_base_token_reserves = cpie.user_base_token_reserves;
                e.user_quote_token_reserves = cpie.user_quote_token_reserves;
                e.pool_base_token_reserves = cpie.pool_base_token_reserves;
                e.pool_quote_token_reserves = cpie.pool_quote_token_reserves;
                e.quote_amount_in = cpie.quote_amount_in;
                e.lp_fee_basis_points = cpie.lp_fee_basis_points;
                e.lp_fee = cpie.lp_fee;
                e.protocol_fee_basis_points = cpie.protocol_fee_basis_points;
                e.protocol_fee = cpie.protocol_fee;
                e.quote_amount_in_with_lp_fee = cpie.quote_amount_in_with_lp_fee;
                e.user_quote_amount_in = cpie.user_quote_amount_in;
                e.pool = cpie.pool;
                e.user = cpie.user;
                e.user_base_token_account = cpie.user_base_token_account;
                e.user_quote_token_account = cpie.user_quote_token_account;
                e.protocol_fee_recipient = cpie.protocol_fee_recipient;
                e.protocol_fee_recipient_token_account = cpie.protocol_fee_recipient_token_account;
                e.coin_creator = cpie.coin_creator;
                e.coin_creator_fee_basis_points = cpie.coin_creator_fee_basis_points;
                e.coin_creator_fee = cpie.coin_creator_fee;
            }
            _ => {}
        },
        DexEvent::PumpSwapSellEvent(e) => match cpi_log_event {
            DexEvent::PumpSwapSellEvent(cpie) => {
                e.timestamp = cpie.timestamp;
                e.base_amount_in = cpie.base_amount_in;
                e.min_quote_amount_out = cpie.min_quote_amount_out;
                e.user_base_token_reserves = cpie.user_base_token_reserves;
                e.user_quote_token_reserves = cpie.user_quote_token_reserves;
                e.pool_base_token_reserves = cpie.pool_base_token_reserves;
                e.pool_quote_token_reserves = cpie.pool_quote_token_reserves;
                e.quote_amount_out = cpie.quote_amount_out;
                e.lp_fee_basis_points = cpie.lp_fee_basis_points;
                e.lp_fee = cpie.lp_fee;
                e.protocol_fee_basis_points = cpie.protocol_fee_basis_points;
                e.protocol_fee = cpie.protocol_fee;
                e.quote_amount_out_without_lp_fee = cpie.quote_amount_out_without_lp_fee;
                e.user_quote_amount_out = cpie.user_quote_amount_out;
                e.pool = cpie.pool;
                e.user = cpie.user;
                e.user_base_token_account = cpie.user_base_token_account;
                e.user_quote_token_account = cpie.user_quote_token_account;
                e.protocol_fee_recipient = cpie.protocol_fee_recipient;
                e.protocol_fee_recipient_token_account = cpie.protocol_fee_recipient_token_account;
                e.coin_creator = cpie.coin_creator;
                e.coin_creator_fee_basis_points = cpie.coin_creator_fee_basis_points;
                e.coin_creator_fee = cpie.coin_creator_fee;
            }
            _ => {}
        },
        DexEvent::PumpSwapCreatePoolEvent(e) => match cpi_log_event {
            DexEvent::PumpSwapCreatePoolEvent(cpie) => {
                e.timestamp = cpie.timestamp;
                e.index = cpie.index;
                e.creator = cpie.creator;
                e.base_mint = cpie.base_mint;
                e.quote_mint = cpie.quote_mint;
                e.base_mint_decimals = cpie.base_mint_decimals;
                e.quote_mint_decimals = cpie.quote_mint_decimals;
                e.base_amount_in = cpie.base_amount_in;
                e.quote_amount_in = cpie.quote_amount_in;
                e.pool_base_amount = cpie.pool_base_amount;
                e.pool_quote_amount = cpie.pool_quote_amount;
                e.minimum_liquidity = cpie.minimum_liquidity;
                e.initial_liquidity = cpie.initial_liquidity;
                e.lp_token_amount_out = cpie.lp_token_amount_out;
                e.pool_bump = cpie.pool_bump;
                e.pool = cpie.pool;
                e.lp_mint = cpie.lp_mint;
                e.user_base_token_account = cpie.user_base_token_account;
                e.user_quote_token_account = cpie.user_quote_token_account;
                e.coin_creator = cpie.coin_creator;
            }
            _ => {}
        },
        DexEvent::PumpSwapDepositEvent(e) => match cpi_log_event {
            DexEvent::PumpSwapDepositEvent(cpie) => {
                e.timestamp = cpie.timestamp;
                e.lp_token_amount_out = cpie.lp_token_amount_out;
                e.max_base_amount_in = cpie.max_base_amount_in;
                e.max_quote_amount_in = cpie.max_quote_amount_in;
                e.user_base_token_reserves = cpie.user_base_token_reserves;
                e.user_quote_token_reserves = cpie.user_quote_token_reserves;
                e.pool_base_token_reserves = cpie.pool_base_token_reserves;
                e.pool_quote_token_reserves = cpie.pool_quote_token_reserves;
                e.base_amount_in = cpie.base_amount_in;
                e.quote_amount_in = cpie.quote_amount_in;
                e.lp_mint_supply = cpie.lp_mint_supply;
                e.pool = cpie.pool;
                e.user = cpie.user;
                e.user_base_token_account = cpie.user_base_token_account;
                e.user_quote_token_account = cpie.user_quote_token_account;
                e.user_pool_token_account = cpie.user_pool_token_account;
            }
            _ => {}
        },
        DexEvent::PumpSwapWithdrawEvent(e) => match cpi_log_event {
            DexEvent::PumpSwapWithdrawEvent(cpie) => {
                e.timestamp = cpie.timestamp;
                e.lp_token_amount_in = cpie.lp_token_amount_in;
                e.min_base_amount_out = cpie.min_base_amount_out;
                e.min_quote_amount_out = cpie.min_quote_amount_out;
                e.user_base_token_reserves = cpie.user_base_token_reserves;
                e.user_quote_token_reserves = cpie.user_quote_token_reserves;
                e.pool_base_token_reserves = cpie.pool_base_token_reserves;
                e.pool_quote_token_reserves = cpie.pool_quote_token_reserves;
                e.base_amount_out = cpie.base_amount_out;
                e.quote_amount_out = cpie.quote_amount_out;
                e.lp_mint_supply = cpie.lp_mint_supply;
                e.pool = cpie.pool;
                e.user = cpie.user;
                e.user_base_token_account = cpie.user_base_token_account;
                e.user_quote_token_account = cpie.user_quote_token_account;
                e.user_pool_token_account = cpie.user_pool_token_account;
            }
            _ => {}
        },

        _ => {}
    }
}
