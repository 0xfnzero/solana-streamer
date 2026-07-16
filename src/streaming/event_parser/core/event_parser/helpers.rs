//! Event enrichment for PumpFun / PumpSwap / Bonk / bot flags.
use crate::streaming::event_parser::{
    core::global_state::{
        add_bonk_dev_address, add_dev_address, is_bonk_dev_address_in_signature,
        is_dev_address_in_signature,
    },
    DexEvent,
};
use solana_sdk::pubkey::Pubkey;

// ================================================================================================
// Event Post-Processing
// ================================================================================================

/// Process and enrich parsed event with additional context
///
/// Handles protocol-specific post-processing:
/// - PumpFun: Tracks dev addresses and marks dev trades
/// - PumpSwap: Fills swap data amounts
/// - Bonk: Tracks pool creators and marks dev trades
/// - General: Marks bot wallet trades
pub(crate) fn process_event(event: DexEvent, bot_wallet: Option<Pubkey>) -> DexEvent {
    let signature = event.metadata().signature; // Copy the signature to avoid borrowing issues
    match event {
        DexEvent::PumpFunCreateTokenEvent(token_info) => {
            add_dev_address(&signature, token_info.user);
            if token_info.creator != Pubkey::default() && token_info.creator != token_info.user {
                add_dev_address(&signature, token_info.creator);
            }
            DexEvent::PumpFunCreateTokenEvent(token_info)
        }
        DexEvent::PumpFunCreateV2TokenEvent(token_info) => {
            add_dev_address(&signature, token_info.user);
            if token_info.creator != Pubkey::default() && token_info.creator != token_info.user {
                add_dev_address(&signature, token_info.creator);
            }
            DexEvent::PumpFunCreateV2TokenEvent(token_info)
        }
        DexEvent::PumpFunTradeEvent(mut trade_info) => {
            trade_info.is_dev_create_token_trade = trade_info.is_dev_create_token_trade
                || trade_info.is_created_buy
                || is_dev_address_in_signature(&signature, &trade_info.user)
                || is_dev_address_in_signature(&signature, &trade_info.creator);
            trade_info.is_bot = Some(trade_info.user) == bot_wallet;

            if let Some(swap_data) = trade_info.metadata.swap_data.as_mut() {
                swap_data.from_amount =
                    if trade_info.is_buy { trade_info.sol_amount } else { trade_info.token_amount };
                swap_data.to_amount =
                    if trade_info.is_buy { trade_info.token_amount } else { trade_info.sol_amount };
            }
            DexEvent::PumpFunTradeEvent(trade_info)
        }
        DexEvent::PumpSwapBuyEvent(mut trade_info) => {
            if let Some(swap_data) = trade_info.metadata.swap_data.as_mut() {
                swap_data.from_amount = trade_info.user_quote_amount_in;
                swap_data.to_amount = trade_info.base_amount_out;
            }
            DexEvent::PumpSwapBuyEvent(trade_info)
        }
        DexEvent::PumpSwapSellEvent(mut trade_info) => {
            if let Some(swap_data) = trade_info.metadata.swap_data.as_mut() {
                swap_data.from_amount = trade_info.base_amount_in;
                swap_data.to_amount = trade_info.user_quote_amount_out;
            }
            DexEvent::PumpSwapSellEvent(trade_info)
        }
        DexEvent::BonkPoolCreateEvent(pool_info) => {
            add_bonk_dev_address(&signature, pool_info.creator);
            DexEvent::BonkPoolCreateEvent(pool_info)
        }
        DexEvent::BonkTradeEvent(mut trade_info) => {
            trade_info.is_dev_create_token_trade =
                is_bonk_dev_address_in_signature(&signature, &trade_info.payer);
            trade_info.is_bot = Some(trade_info.payer) == bot_wallet;
            DexEvent::BonkTradeEvent(trade_info)
        }
        _ => event,
    }
}
