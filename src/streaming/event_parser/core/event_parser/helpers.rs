//! Event enrichment for PumpFun / PumpSwap / Bonk / bot flags.
use crate::streaming::event_parser::{
    core::global_state::{
        add_bonk_dev_address, add_dev_address, is_bonk_dev_address_in_signature,
        is_dev_address_in_signature,
    },
    protocols::pumpfun::events::PumpFunTradeEvent,
    DexEvent,
};
use solana_sdk::pubkey::Pubkey;

/// Quote-side amount for PumpFun trades. Non-SOL quote mints populate `quote_amount` while
/// legacy SOL-only trades keep amounts in `sol_amount`.
#[inline]
fn pumpfun_trade_quote_leg_amount(trade: &PumpFunTradeEvent) -> u64 {
    if trade.quote_amount > 0 {
        trade.quote_amount
    } else if trade.spendable_quote_in > 0 {
        trade.spendable_quote_in
    } else {
        trade.sol_amount
    }
}

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
                let quote_leg = pumpfun_trade_quote_leg_amount(&trade_info);
                swap_data.from_amount =
                    if trade_info.is_buy { quote_leg } else { trade_info.token_amount };
                swap_data.to_amount =
                    if trade_info.is_buy { trade_info.token_amount } else { quote_leg };
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming::event_parser::common::types::{EventType, ProtocolType, SwapData};
    use crate::streaming::event_parser::common::EventMetadata;

    fn trade_with_swap_data(sol_amount: u64, quote_amount: u64, is_buy: bool) -> DexEvent {
        let mut metadata = EventMetadata::new(
            Default::default(),
            0,
            0,
            0,
            ProtocolType::PumpFun,
            EventType::PumpFunBuy,
            Default::default(),
            0,
            None,
            0,
            None,
            None,
        );
        metadata.swap_data = Some(SwapData::default());
        DexEvent::PumpFunTradeEvent(PumpFunTradeEvent {
            metadata,
            sol_amount,
            quote_amount,
            token_amount: 999,
            is_buy,
            ..Default::default()
        })
    }

    #[test]
    fn pumpfun_swap_data_prefers_quote_amount_for_non_sol_quotes() {
        let event = process_event(trade_with_swap_data(0, 500, true), None);
        let swap_data = event.metadata().swap_data.as_ref().expect("swap_data");
        assert_eq!(swap_data.from_amount, 500);
        assert_eq!(swap_data.to_amount, 999);
    }

    #[test]
    fn pumpfun_swap_data_falls_back_to_sol_amount_for_legacy_trades() {
        let event = process_event(trade_with_swap_data(400, 0, false), None);
        let swap_data = event.metadata().swap_data.as_ref().expect("swap_data");
        assert_eq!(swap_data.from_amount, 999);
        assert_eq!(swap_data.to_amount, 400);
    }
}
