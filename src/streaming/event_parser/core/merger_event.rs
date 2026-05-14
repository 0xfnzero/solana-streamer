use crate::streaming::event_parser::DexEvent;
use solana_sdk::pubkey::Pubkey;

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
                e.track_volume = cpie.track_volume;
                e.total_unclaimed_tokens = cpie.total_unclaimed_tokens;
                e.total_claimed_tokens = cpie.total_claimed_tokens;
                e.current_sol_volume = cpie.current_sol_volume;
                e.last_update_timestamp = cpie.last_update_timestamp;
                e.ix_name = cpie.ix_name.clone();
                e.mayhem_mode = cpie.mayhem_mode;
                e.cashback_fee_basis_points = cpie.cashback_fee_basis_points;
                e.cashback = cpie.cashback;
                e.is_cashback_coin = cpie.is_cashback_coin;
                if cpie.account.is_some() {
                    e.account = cpie.account;
                }
            }
            _ => {}
        },
        DexEvent::PumpFunCreateTokenEvent(e) => match cpi_log_event {
            DexEvent::PumpFunCreateV2TokenEvent(cpie) => {
                if cpie.mint != solana_sdk::pubkey::Pubkey::default() {
                    e.mint = cpie.mint;
                }
                if cpie.bonding_curve != solana_sdk::pubkey::Pubkey::default() {
                    e.bonding_curve = cpie.bonding_curve;
                }
                if cpie.user != solana_sdk::pubkey::Pubkey::default() {
                    e.user = cpie.user;
                }
                if cpie.creator != solana_sdk::pubkey::Pubkey::default() {
                    e.creator = cpie.creator;
                }
                if cpie.timestamp != 0 {
                    e.timestamp = cpie.timestamp;
                }
                if cpie.virtual_token_reserves != 0 {
                    e.virtual_token_reserves = cpie.virtual_token_reserves;
                }
                if cpie.virtual_sol_reserves != 0 {
                    e.virtual_sol_reserves = cpie.virtual_sol_reserves;
                }
                if cpie.real_token_reserves != 0 {
                    e.real_token_reserves = cpie.real_token_reserves;
                }
                if cpie.token_total_supply != 0 {
                    e.token_total_supply = cpie.token_total_supply;
                }
                if cpie.token_program != solana_sdk::pubkey::Pubkey::default() {
                    e.token_program = cpie.token_program;
                }
                e.is_mayhem_mode = cpie.is_mayhem_mode;
                e.is_cashback_enabled = cpie.is_cashback_enabled;
            }
            _ => {}
        },
        DexEvent::PumpFunCreateV2TokenEvent(e) => match cpi_log_event {
            DexEvent::PumpFunCreateV2TokenEvent(cpie) => {
                if cpie.mint != solana_sdk::pubkey::Pubkey::default() {
                    e.mint = cpie.mint;
                }
                if cpie.bonding_curve != solana_sdk::pubkey::Pubkey::default() {
                    e.bonding_curve = cpie.bonding_curve;
                }
                if cpie.user != solana_sdk::pubkey::Pubkey::default() {
                    e.user = cpie.user;
                }
                if cpie.creator != solana_sdk::pubkey::Pubkey::default() {
                    e.creator = cpie.creator;
                }
                if cpie.timestamp != 0 {
                    e.timestamp = cpie.timestamp;
                }
                if cpie.virtual_token_reserves != 0 {
                    e.virtual_token_reserves = cpie.virtual_token_reserves;
                }
                if cpie.virtual_sol_reserves != 0 {
                    e.virtual_sol_reserves = cpie.virtual_sol_reserves;
                }
                if cpie.real_token_reserves != 0 {
                    e.real_token_reserves = cpie.real_token_reserves;
                }
                if cpie.token_total_supply != 0 {
                    e.token_total_supply = cpie.token_total_supply;
                }
                if cpie.token_program != solana_sdk::pubkey::Pubkey::default() {
                    e.token_program = cpie.token_program;
                }
                e.is_mayhem_mode = cpie.is_mayhem_mode;
                e.is_cashback_enabled = cpie.is_cashback_enabled;
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
                e.track_volume = cpie.track_volume;
                e.total_unclaimed_tokens = cpie.total_unclaimed_tokens;
                e.total_claimed_tokens = cpie.total_claimed_tokens;
                e.current_sol_volume = cpie.current_sol_volume;
                e.last_update_timestamp = cpie.last_update_timestamp;
                e.min_base_amount_out = cpie.min_base_amount_out;
                e.ix_name = cpie.ix_name.clone();
                e.cashback_fee_basis_points = cpie.cashback_fee_basis_points;
                e.cashback = cpie.cashback;
                e.is_pump_pool = cpie.is_pump_pool;
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
                e.cashback_fee_basis_points = cpie.cashback_fee_basis_points;
                e.cashback = cpie.cashback;
                e.is_pump_pool = cpie.is_pump_pool;
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
        DexEvent::MeteoraDammV2SwapEvent(e) => match cpi_log_event {
            DexEvent::MeteoraDammV2SwapEvent(cpie) => {
                e.pool = cpie.pool;
                e.trade_direction = cpie.trade_direction;
                e.collect_fee_mode = cpie.collect_fee_mode;
                e.has_referral = cpie.has_referral;
                e.amount_0 = cpie.amount_0;
                e.amount_1 = cpie.amount_1;
                e.swap_mode = cpie.swap_mode;
                e.included_fee_input_amount = cpie.included_fee_input_amount;
                e.excluded_fee_input_amount = cpie.excluded_fee_input_amount;
                e.amount_left = cpie.amount_left;
                e.output_amount = cpie.output_amount;
                e.next_sqrt_price = cpie.next_sqrt_price;
                e.trading_fee = cpie.trading_fee;
                e.partner_fee = cpie.partner_fee;
                e.referral_fee = cpie.referral_fee;
                e.included_transfer_fee_amount_in = cpie.included_transfer_fee_amount_in;
                e.included_transfer_fee_amount_out = cpie.included_transfer_fee_amount_out;
                e.excluded_transfer_fee_amount_out = cpie.excluded_transfer_fee_amount_out;
                e.current_timestamp = cpie.current_timestamp;
                e.reserve_a_amount = cpie.reserve_a_amount;
                e.reserve_b_amount = cpie.reserve_b_amount;
            }
            _ => {}
        },
        DexEvent::MeteoraDammV2Swap2Event(e) => match cpi_log_event {
            DexEvent::MeteoraDammV2SwapEvent(cpie) => {
                e.pool = cpie.pool;
                e.trade_direction = cpie.trade_direction;
                e.collect_fee_mode = cpie.collect_fee_mode;
                e.has_referral = cpie.has_referral;
                e.amount_0 = cpie.amount_0;
                e.amount_1 = cpie.amount_1;
                e.swap_mode = cpie.swap_mode;
                e.included_fee_input_amount = cpie.included_fee_input_amount;
                e.excluded_fee_input_amount = cpie.excluded_fee_input_amount;
                e.amount_left = cpie.amount_left;
                e.output_amount = cpie.output_amount;
                e.next_sqrt_price = cpie.next_sqrt_price;
                e.trading_fee = cpie.trading_fee;
                e.partner_fee = cpie.partner_fee;
                e.referral_fee = cpie.referral_fee;
                e.included_transfer_fee_amount_in = cpie.included_transfer_fee_amount_in;
                e.included_transfer_fee_amount_out = cpie.included_transfer_fee_amount_out;
                e.excluded_transfer_fee_amount_out = cpie.excluded_transfer_fee_amount_out;
                e.current_timestamp = cpie.current_timestamp;
                e.reserve_a_amount = cpie.reserve_a_amount;
                e.reserve_b_amount = cpie.reserve_b_amount;
            }
            _ => {}
        },
        DexEvent::MeteoraDammV2InitializePoolEvent(e) => match cpi_log_event {
            DexEvent::MeteoraDammV2InitializePoolEvent(cpie) => {
                e.pool = cpie.pool;
                e.token_a_mint = cpie.token_a_mint;
                e.token_b_mint = cpie.token_b_mint;
                e.creator = cpie.creator;
                e.payer = cpie.payer;
                e.alpha_vault = cpie.alpha_vault;
                e.pool_fees = cpie.pool_fees;
                e.sqrt_min_price = cpie.sqrt_min_price;
                e.sqrt_max_price = cpie.sqrt_max_price;
                e.activation_type = cpie.activation_type;
                e.collect_fee_mode = cpie.collect_fee_mode;
                e.liquidity = cpie.liquidity;
                e.sqrt_price = cpie.sqrt_price;
                e.activation_point = cpie.activation_point;
                e.token_a_flag = cpie.token_a_flag;
                e.token_b_flag = cpie.token_b_flag;
                e.token_a_amount = cpie.token_a_amount;
                e.token_b_amount = cpie.token_b_amount;
                e.total_amount_a = cpie.total_amount_a;
                e.total_amount_b = cpie.total_amount_b;
                e.pool_type = cpie.pool_type;
            }
            _ => {}
        },
        DexEvent::MeteoraDammV2InitializeCustomizablePoolEvent(e) => match cpi_log_event {
            DexEvent::MeteoraDammV2InitializePoolEvent(cpie) => {
                e.pool = cpie.pool;
                e.token_a_mint = cpie.token_a_mint;
                e.token_b_mint = cpie.token_b_mint;
                e.creator = cpie.creator;
                e.payer = cpie.payer;
                e.alpha_vault = cpie.alpha_vault;
                e.pool_fees = cpie.pool_fees;
                e.sqrt_min_price = cpie.sqrt_min_price;
                e.sqrt_max_price = cpie.sqrt_max_price;
                e.activation_type = cpie.activation_type;
                e.collect_fee_mode = cpie.collect_fee_mode;
                e.liquidity = cpie.liquidity;
                e.sqrt_price = cpie.sqrt_price;
                e.activation_point = cpie.activation_point;
                e.token_a_flag = cpie.token_a_flag;
                e.token_b_flag = cpie.token_b_flag;
                e.token_a_amount = cpie.token_a_amount;
                e.token_b_amount = cpie.token_b_amount;
                e.total_amount_a = cpie.total_amount_a;
                e.total_amount_b = cpie.total_amount_b;
                e.pool_type = cpie.pool_type;
            }
            _ => {}
        },
        DexEvent::MeteoraDammV2InitializePoolWithDynamicConfigEvent(e) => match cpi_log_event {
            DexEvent::MeteoraDammV2InitializePoolEvent(cpie) => {
                e.pool = cpie.pool;
                e.token_a_mint = cpie.token_a_mint;
                e.token_b_mint = cpie.token_b_mint;
                e.creator = cpie.creator;
                e.payer = cpie.payer;
                e.alpha_vault = cpie.alpha_vault;
                e.pool_fees = cpie.pool_fees;
                e.sqrt_min_price = cpie.sqrt_min_price;
                e.sqrt_max_price = cpie.sqrt_max_price;
                e.activation_type = cpie.activation_type;
                e.collect_fee_mode = cpie.collect_fee_mode;
                e.liquidity = cpie.liquidity;
                e.sqrt_price = cpie.sqrt_price;
                e.activation_point = cpie.activation_point;
                e.token_a_flag = cpie.token_a_flag;
                e.token_b_flag = cpie.token_b_flag;
                e.token_a_amount = cpie.token_a_amount;
                e.token_b_amount = cpie.token_b_amount;
                e.total_amount_a = cpie.total_amount_a;
                e.total_amount_b = cpie.total_amount_b;
                e.pool_type = cpie.pool_type;
            }
            _ => {}
        },

        // Orca Whirlpool：外层指令粗字段 + CPI 日志精修
        DexEvent::OrcaWhirlpoolSwapEvent(e) => match cpi_log_event {
            DexEvent::OrcaWhirlpoolSwapEvent(cpie) => {
                if cpie.whirlpool != Pubkey::default() {
                    e.whirlpool = cpie.whirlpool;
                }
                if cpie.input_amount != 0 {
                    e.input_amount = cpie.input_amount;
                }
                if cpie.output_amount != 0 {
                    e.output_amount = cpie.output_amount;
                }
                e.a_to_b = cpie.a_to_b;
                if cpie.pre_sqrt_price != 0 {
                    e.pre_sqrt_price = cpie.pre_sqrt_price;
                }
                if cpie.post_sqrt_price != 0 {
                    e.post_sqrt_price = cpie.post_sqrt_price;
                }
                if cpie.input_transfer_fee != 0 {
                    e.input_transfer_fee = cpie.input_transfer_fee;
                }
                if cpie.output_transfer_fee != 0 {
                    e.output_transfer_fee = cpie.output_transfer_fee;
                }
                if cpie.lp_fee != 0 {
                    e.lp_fee = cpie.lp_fee;
                }
                if cpie.protocol_fee != 0 {
                    e.protocol_fee = cpie.protocol_fee;
                }
            }
            _ => {}
        },
        DexEvent::OrcaWhirlpoolLiquidityIncreasedEvent(e) => match cpi_log_event {
            DexEvent::OrcaWhirlpoolLiquidityIncreasedEvent(cpie) => {
                if cpie.position != Pubkey::default() {
                    e.position = cpie.position;
                }
                if cpie.tick_lower_index != 0 || cpie.tick_upper_index != 0 {
                    e.tick_lower_index = cpie.tick_lower_index;
                    e.tick_upper_index = cpie.tick_upper_index;
                }
                if cpie.token_a_amount != 0 {
                    e.token_a_amount = cpie.token_a_amount;
                }
                if cpie.token_b_amount != 0 {
                    e.token_b_amount = cpie.token_b_amount;
                }
                if cpie.liquidity != 0 {
                    e.liquidity = cpie.liquidity;
                }
                if cpie.token_a_transfer_fee != 0 {
                    e.token_a_transfer_fee = cpie.token_a_transfer_fee;
                }
                if cpie.token_b_transfer_fee != 0 {
                    e.token_b_transfer_fee = cpie.token_b_transfer_fee;
                }
            }
            _ => {}
        },
        DexEvent::OrcaWhirlpoolLiquidityDecreasedEvent(e) => match cpi_log_event {
            DexEvent::OrcaWhirlpoolLiquidityDecreasedEvent(cpie) => {
                if cpie.position != Pubkey::default() {
                    e.position = cpie.position;
                }
                if cpie.tick_lower_index != 0 || cpie.tick_upper_index != 0 {
                    e.tick_lower_index = cpie.tick_lower_index;
                    e.tick_upper_index = cpie.tick_upper_index;
                }
                if cpie.token_a_amount != 0 {
                    e.token_a_amount = cpie.token_a_amount;
                }
                if cpie.token_b_amount != 0 {
                    e.token_b_amount = cpie.token_b_amount;
                }
                if cpie.liquidity != 0 {
                    e.liquidity = cpie.liquidity;
                }
                if cpie.token_a_transfer_fee != 0 {
                    e.token_a_transfer_fee = cpie.token_a_transfer_fee;
                }
                if cpie.token_b_transfer_fee != 0 {
                    e.token_b_transfer_fee = cpie.token_b_transfer_fee;
                }
            }
            _ => {}
        },

        // Meteora Pools swap：外层 min_out 等与 CPI 实际结算合并
        DexEvent::MeteoraPoolsSwapEvent(e) => match cpi_log_event {
            DexEvent::MeteoraPoolsSwapEvent(cpie) => {
                if cpie.in_amount != 0 {
                    e.in_amount = cpie.in_amount;
                }
                if cpie.out_amount != 0 {
                    e.out_amount = cpie.out_amount;
                }
                if cpie.trade_fee != 0 {
                    e.trade_fee = cpie.trade_fee;
                }
                if cpie.admin_fee != 0 {
                    e.admin_fee = cpie.admin_fee;
                }
                if cpie.host_fee != 0 {
                    e.host_fee = cpie.host_fee;
                }
            }
            _ => {}
        },
        DexEvent::MeteoraPoolsAddLiquidityEvent(e) => match cpi_log_event {
            DexEvent::MeteoraPoolsAddLiquidityEvent(cpie) => {
                if cpie.lp_mint_amount != 0 {
                    e.lp_mint_amount = cpie.lp_mint_amount;
                }
                if cpie.token_a_amount != 0 {
                    e.token_a_amount = cpie.token_a_amount;
                }
                if cpie.token_b_amount != 0 {
                    e.token_b_amount = cpie.token_b_amount;
                }
            }
            _ => {}
        },
        DexEvent::MeteoraPoolsRemoveLiquidityEvent(e) => match cpi_log_event {
            DexEvent::MeteoraPoolsRemoveLiquidityEvent(cpie) => {
                if cpie.lp_unmint_amount != 0 {
                    e.lp_unmint_amount = cpie.lp_unmint_amount;
                }
                if cpie.token_a_out_amount != 0 {
                    e.token_a_out_amount = cpie.token_a_out_amount;
                }
                if cpie.token_b_out_amount != 0 {
                    e.token_b_out_amount = cpie.token_b_out_amount;
                }
            }
            _ => {}
        },

        // Meteora DLMM
        DexEvent::MeteoraDlmmSwapEvent(e) => match cpi_log_event {
            DexEvent::MeteoraDlmmSwapEvent(cpie) => {
                if cpie.pool != Pubkey::default() {
                    e.pool = cpie.pool;
                }
                if cpie.from != Pubkey::default() {
                    e.from = cpie.from;
                }
                if cpie.start_bin_id != 0 || cpie.end_bin_id != 0 {
                    e.start_bin_id = cpie.start_bin_id;
                    e.end_bin_id = cpie.end_bin_id;
                }
                if cpie.amount_out != 0 {
                    e.amount_out = cpie.amount_out;
                }
                if cpie.amount_in != 0 {
                    e.amount_in = cpie.amount_in;
                }
                e.swap_for_y = cpie.swap_for_y;
                if cpie.fee != 0 {
                    e.fee = cpie.fee;
                }
                if cpie.protocol_fee != 0 {
                    e.protocol_fee = cpie.protocol_fee;
                }
                if cpie.fee_bps != 0 {
                    e.fee_bps = cpie.fee_bps;
                }
                if cpie.host_fee != 0 {
                    e.host_fee = cpie.host_fee;
                }
            }
            _ => {}
        },
        DexEvent::MeteoraDlmmAddLiquidityEvent(e) => match cpi_log_event {
            DexEvent::MeteoraDlmmAddLiquidityEvent(cpie) => {
                if cpie.active_bin_id != 0 {
                    e.active_bin_id = cpie.active_bin_id;
                }
                e.amounts = cpie.amounts;
            }
            _ => {}
        },
        DexEvent::MeteoraDlmmRemoveLiquidityEvent(e) => match cpi_log_event {
            DexEvent::MeteoraDlmmRemoveLiquidityEvent(cpie) => {
                if cpie.active_bin_id != 0 {
                    e.active_bin_id = cpie.active_bin_id;
                }
                e.amounts = cpie.amounts;
            }
            _ => {}
        },

        DexEvent::MeteoraPoolsBootstrapLiquidityEvent(e) => match cpi_log_event {
            DexEvent::MeteoraPoolsBootstrapLiquidityEvent(cpie) => {
                if cpie.pool != Pubkey::default() {
                    e.pool = cpie.pool;
                }
                if cpie.lp_mint_amount != 0 {
                    e.lp_mint_amount = cpie.lp_mint_amount;
                }
                if cpie.token_a_amount != 0 {
                    e.token_a_amount = cpie.token_a_amount;
                }
                if cpie.token_b_amount != 0 {
                    e.token_b_amount = cpie.token_b_amount;
                }
            }
            _ => {}
        },
        DexEvent::MeteoraPoolsPoolCreatedEvent(e) => match cpi_log_event {
            DexEvent::MeteoraPoolsPoolCreatedEvent(cpie) => {
                if cpie.pool != Pubkey::default() {
                    e.pool = cpie.pool;
                }
                if cpie.lp_mint != Pubkey::default() {
                    e.lp_mint = cpie.lp_mint;
                }
                if cpie.token_a_mint != Pubkey::default() {
                    e.token_a_mint = cpie.token_a_mint;
                }
                if cpie.token_b_mint != Pubkey::default() {
                    e.token_b_mint = cpie.token_b_mint;
                }
                if cpie.pool_type != 0 {
                    e.pool_type = cpie.pool_type;
                }
            }
            _ => {}
        },
        DexEvent::MeteoraPoolsSetPoolFeesEvent(e) => match cpi_log_event {
            DexEvent::MeteoraPoolsSetPoolFeesEvent(cpie) => {
                if cpie.pool != Pubkey::default() {
                    e.pool = cpie.pool;
                }
                e.trade_fee_numerator = cpie.trade_fee_numerator;
                e.trade_fee_denominator = cpie.trade_fee_denominator;
                e.owner_trade_fee_numerator = cpie.owner_trade_fee_numerator;
                e.owner_trade_fee_denominator = cpie.owner_trade_fee_denominator;
            }
            _ => {}
        },

        _ => {}
    }
}
