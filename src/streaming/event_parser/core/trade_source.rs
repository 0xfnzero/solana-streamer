//! Consistent provenance access across streamer trade events.
use super::DexEvent;
use sol_parser_sdk::core::events::RawInstructionPayload;

impl DexEvent {
    pub fn third_program(&self) -> Option<&RawInstructionPayload> {
        match self {
            Self::PumpFunTradeEvent(t) => t.third_program.as_ref(),
            Self::PumpSwapBuyEvent(t) => t.third_program.as_ref(),
            Self::PumpSwapSellEvent(t) => t.third_program.as_ref(),
            Self::RaydiumCpmmSwapEvent(t) => t.third_program.as_ref(),
            Self::RaydiumAmmV4SwapEvent(t) => t.third_program.as_ref(),
            Self::RaydiumClmmSwapEvent(t) => t.third_program.as_ref(),
            Self::RaydiumClmmSwapV2Event(t) => t.third_program.as_ref(),
            Self::BonkTradeEvent(t) => t.third_program.as_ref(),
            Self::MeteoraDammV2SwapEvent(t) => t.third_program.as_ref(),
            Self::MeteoraDammV2Swap2Event(t) => t.third_program.as_ref(),
            Self::OrcaWhirlpoolSwapEvent(t) => t.third_program.as_ref(),
            Self::MeteoraDbcSwapEvent(t) => t.third_program.as_ref(),
            Self::MeteoraPoolsSwapEvent(t) => t.third_program.as_ref(),
            Self::MeteoraDlmmSwapEvent(t) => t.third_program.as_ref(),
            _ => None,
        }
    }
}
