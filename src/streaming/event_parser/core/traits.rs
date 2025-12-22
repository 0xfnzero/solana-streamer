use crate::streaming::event_parser::common::EventMetadata;
use crate::streaming::event_parser::core::account_event_parser::{
    NonceAccountEvent, TokenAccountEvent, TokenInfoEvent,
};
use crate::streaming::event_parser::core::common_event_parser::{
    SetComputeUnitLimitEvent, SetComputeUnitPriceEvent,
};
use crate::streaming::event_parser::protocols::block::block_meta_event::BlockMetaEvent;
use crate::streaming::event_parser::protocols::bonk::events::*;
use crate::streaming::event_parser::protocols::meteora_damm_v2::events::*;
use crate::streaming::event_parser::protocols::pumpfun::events::*;
use crate::streaming::event_parser::protocols::pumpswap::events::*;
use crate::streaming::event_parser::protocols::raydium_amm_v4::events::*;
use crate::streaming::event_parser::protocols::raydium_clmm::events::*;
use crate::streaming::event_parser::protocols::raydium_cpmm::events::*;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Unified Event Enum - Replaces the trait-based approach with a type-safe enum
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DexEvent {
    // Bonk events
    BonkTradeEvent(BonkTradeEvent),
    BonkPoolCreateEvent(BonkPoolCreateEvent),
    BonkMigrateToAmmEvent(BonkMigrateToAmmEvent),
    BonkMigrateToCpswapEvent(BonkMigrateToCpswapEvent),
    BonkPoolStateAccountEvent(BonkPoolStateAccountEvent),
    BonkGlobalConfigAccountEvent(BonkGlobalConfigAccountEvent),
    BonkPlatformConfigAccountEvent(BonkPlatformConfigAccountEvent),

    // PumpFun events
    PumpFunCreateTokenEvent(PumpFunCreateTokenEvent),
    PumpFunCreateV2TokenEvent(PumpFunCreateV2TokenEvent),
    PumpFunTradeEvent(PumpFunTradeEvent),
    PumpFunMigrateEvent(PumpFunMigrateEvent),
    PumpFunBondingCurveAccountEvent(PumpFunBondingCurveAccountEvent),
    PumpFunGlobalAccountEvent(PumpFunGlobalAccountEvent),

    // PumpSwap events
    PumpSwapBuyEvent(PumpSwapBuyEvent),
    PumpSwapBuyExactQuoteInEvent(PumpSwapBuyExactQuoteInEvent),
    PumpSwapSellEvent(PumpSwapSellEvent),
    PumpSwapCreatePoolEvent(PumpSwapCreatePoolEvent),
    PumpSwapDepositEvent(PumpSwapDepositEvent),
    PumpSwapWithdrawEvent(PumpSwapWithdrawEvent),
    PumpSwapGlobalConfigAccountEvent(PumpSwapGlobalConfigAccountEvent),
    PumpSwapPoolAccountEvent(PumpSwapPoolAccountEvent),

    // Raydium AMM V4 events
    RaydiumAmmV4SwapEvent(RaydiumAmmV4SwapEvent),
    RaydiumAmmV4DepositEvent(RaydiumAmmV4DepositEvent),
    RaydiumAmmV4WithdrawEvent(RaydiumAmmV4WithdrawEvent),
    RaydiumAmmV4WithdrawPnlEvent(RaydiumAmmV4WithdrawPnlEvent),
    RaydiumAmmV4Initialize2Event(RaydiumAmmV4Initialize2Event),
    RaydiumAmmV4AmmInfoAccountEvent(RaydiumAmmV4AmmInfoAccountEvent),

    // Raydium CLMM events
    RaydiumClmmSwapEvent(RaydiumClmmSwapEvent),
    RaydiumClmmSwapV2Event(RaydiumClmmSwapV2Event),
    RaydiumClmmClosePositionEvent(RaydiumClmmClosePositionEvent),
    RaydiumClmmIncreaseLiquidityV2Event(RaydiumClmmIncreaseLiquidityV2Event),
    RaydiumClmmDecreaseLiquidityV2Event(RaydiumClmmDecreaseLiquidityV2Event),
    RaydiumClmmCreatePoolEvent(RaydiumClmmCreatePoolEvent),
    RaydiumClmmOpenPositionWithToken22NftEvent(RaydiumClmmOpenPositionWithToken22NftEvent),
    RaydiumClmmOpenPositionV2Event(RaydiumClmmOpenPositionV2Event),
    RaydiumClmmAmmConfigAccountEvent(RaydiumClmmAmmConfigAccountEvent),
    RaydiumClmmPoolStateAccountEvent(RaydiumClmmPoolStateAccountEvent),
    RaydiumClmmTickArrayStateAccountEvent(RaydiumClmmTickArrayStateAccountEvent),

    // Raydium CPMM events
    RaydiumCpmmSwapEvent(RaydiumCpmmSwapEvent),
    RaydiumCpmmDepositEvent(RaydiumCpmmDepositEvent),
    RaydiumCpmmWithdrawEvent(RaydiumCpmmWithdrawEvent),
    RaydiumCpmmInitializeEvent(RaydiumCpmmInitializeEvent),
    RaydiumCpmmAmmConfigAccountEvent(RaydiumCpmmAmmConfigAccountEvent),
    RaydiumCpmmPoolStateAccountEvent(RaydiumCpmmPoolStateAccountEvent),

    // Meteora DAMM v2 events
    MeteoraDammV2SwapEvent(MeteoraDammV2SwapEvent),
    MeteoraDammV2Swap2Event(MeteoraDammV2Swap2Event),
    MeteoraDammV2InitializePoolEvent(MeteoraDammV2InitializePoolEvent),
    MeteoraDammV2InitializeCustomizablePoolEvent(MeteoraDammV2InitializeCustomizablePoolEvent),
    MeteoraDammV2InitializePoolWithDynamicConfigEvent(MeteoraDammV2InitializePoolWithDynamicConfigEvent),

    // Common events
    TokenAccountEvent(TokenAccountEvent),
    NonceAccountEvent(NonceAccountEvent),
    TokenInfoEvent(TokenInfoEvent),
    BlockMetaEvent(BlockMetaEvent),
    SetComputeUnitLimitEvent(SetComputeUnitLimitEvent),
    SetComputeUnitPriceEvent(SetComputeUnitPriceEvent),
}

impl DexEvent {
    pub fn metadata(&self) -> &EventMetadata {
        match self {
            DexEvent::BonkTradeEvent(e) => &e.metadata,
            DexEvent::BonkPoolCreateEvent(e) => &e.metadata,
            DexEvent::BonkMigrateToAmmEvent(e) => &e.metadata,
            DexEvent::BonkMigrateToCpswapEvent(e) => &e.metadata,
            DexEvent::BonkPoolStateAccountEvent(e) => &e.metadata,
            DexEvent::BonkGlobalConfigAccountEvent(e) => &e.metadata,
            DexEvent::BonkPlatformConfigAccountEvent(e) => &e.metadata,
            DexEvent::PumpFunCreateTokenEvent(e) => &e.metadata,
            DexEvent::PumpFunCreateV2TokenEvent(e) => &e.metadata,
            DexEvent::PumpFunTradeEvent(e) => &e.metadata,
            DexEvent::PumpFunMigrateEvent(e) => &e.metadata,
            DexEvent::PumpFunBondingCurveAccountEvent(e) => &e.metadata,
            DexEvent::PumpFunGlobalAccountEvent(e) => &e.metadata,
            DexEvent::PumpSwapBuyEvent(e) => &e.metadata,
            DexEvent::PumpSwapBuyExactQuoteInEvent(e) => &e.metadata,
            DexEvent::PumpSwapSellEvent(e) => &e.metadata,
            DexEvent::PumpSwapCreatePoolEvent(e) => &e.metadata,
            DexEvent::PumpSwapDepositEvent(e) => &e.metadata,
            DexEvent::PumpSwapWithdrawEvent(e) => &e.metadata,
            DexEvent::PumpSwapGlobalConfigAccountEvent(e) => &e.metadata,
            DexEvent::PumpSwapPoolAccountEvent(e) => &e.metadata,
            DexEvent::RaydiumAmmV4SwapEvent(e) => &e.metadata,
            DexEvent::RaydiumAmmV4DepositEvent(e) => &e.metadata,
            DexEvent::RaydiumAmmV4WithdrawEvent(e) => &e.metadata,
            DexEvent::RaydiumAmmV4WithdrawPnlEvent(e) => &e.metadata,
            DexEvent::RaydiumAmmV4Initialize2Event(e) => &e.metadata,
            DexEvent::RaydiumAmmV4AmmInfoAccountEvent(e) => &e.metadata,
            DexEvent::RaydiumClmmSwapEvent(e) => &e.metadata,
            DexEvent::RaydiumClmmSwapV2Event(e) => &e.metadata,
            DexEvent::RaydiumClmmClosePositionEvent(e) => &e.metadata,
            DexEvent::RaydiumClmmIncreaseLiquidityV2Event(e) => &e.metadata,
            DexEvent::RaydiumClmmDecreaseLiquidityV2Event(e) => &e.metadata,
            DexEvent::RaydiumClmmCreatePoolEvent(e) => &e.metadata,
            DexEvent::RaydiumClmmOpenPositionWithToken22NftEvent(e) => &e.metadata,
            DexEvent::RaydiumClmmOpenPositionV2Event(e) => &e.metadata,
            DexEvent::RaydiumClmmAmmConfigAccountEvent(e) => &e.metadata,
            DexEvent::RaydiumClmmPoolStateAccountEvent(e) => &e.metadata,
            DexEvent::RaydiumClmmTickArrayStateAccountEvent(e) => &e.metadata,
            DexEvent::RaydiumCpmmSwapEvent(e) => &e.metadata,
            DexEvent::RaydiumCpmmDepositEvent(e) => &e.metadata,
            DexEvent::RaydiumCpmmWithdrawEvent(e) => &e.metadata,
            DexEvent::RaydiumCpmmInitializeEvent(e) => &e.metadata,
            DexEvent::RaydiumCpmmAmmConfigAccountEvent(e) => &e.metadata,
            DexEvent::RaydiumCpmmPoolStateAccountEvent(e) => &e.metadata,
            DexEvent::MeteoraDammV2SwapEvent(e) => &e.metadata,
            DexEvent::MeteoraDammV2Swap2Event(e) => &e.metadata,
            DexEvent::MeteoraDammV2InitializePoolEvent(e) => &e.metadata,
            DexEvent::MeteoraDammV2InitializeCustomizablePoolEvent(e) => &e.metadata,
            DexEvent::MeteoraDammV2InitializePoolWithDynamicConfigEvent(e) => &e.metadata,
            DexEvent::TokenAccountEvent(e) => &e.metadata,
            DexEvent::NonceAccountEvent(e) => &e.metadata,
            DexEvent::TokenInfoEvent(e) => &e.metadata,
            DexEvent::BlockMetaEvent(e) => &e.metadata,
            DexEvent::SetComputeUnitLimitEvent(e) => &e.metadata,
            DexEvent::SetComputeUnitPriceEvent(e) => &e.metadata,
        }
    }

    pub fn metadata_mut(&mut self) -> &mut EventMetadata {
        match self {
            DexEvent::BonkTradeEvent(e) => &mut e.metadata,
            DexEvent::BonkPoolCreateEvent(e) => &mut e.metadata,
            DexEvent::BonkMigrateToAmmEvent(e) => &mut e.metadata,
            DexEvent::BonkMigrateToCpswapEvent(e) => &mut e.metadata,
            DexEvent::BonkPoolStateAccountEvent(e) => &mut e.metadata,
            DexEvent::BonkGlobalConfigAccountEvent(e) => &mut e.metadata,
            DexEvent::BonkPlatformConfigAccountEvent(e) => &mut e.metadata,
            DexEvent::PumpFunCreateTokenEvent(e) => &mut e.metadata,
            DexEvent::PumpFunCreateV2TokenEvent(e) => &mut e.metadata,
            DexEvent::PumpFunTradeEvent(e) => &mut e.metadata,
            DexEvent::PumpFunMigrateEvent(e) => &mut e.metadata,
            DexEvent::PumpFunBondingCurveAccountEvent(e) => &mut e.metadata,
            DexEvent::PumpFunGlobalAccountEvent(e) => &mut e.metadata,
            DexEvent::PumpSwapBuyEvent(e) => &mut e.metadata,
            DexEvent::PumpSwapBuyExactQuoteInEvent(e) => &mut e.metadata,
            DexEvent::PumpSwapSellEvent(e) => &mut e.metadata,
            DexEvent::PumpSwapCreatePoolEvent(e) => &mut e.metadata,
            DexEvent::PumpSwapDepositEvent(e) => &mut e.metadata,
            DexEvent::PumpSwapWithdrawEvent(e) => &mut e.metadata,
            DexEvent::PumpSwapGlobalConfigAccountEvent(e) => &mut e.metadata,
            DexEvent::PumpSwapPoolAccountEvent(e) => &mut e.metadata,
            DexEvent::RaydiumAmmV4SwapEvent(e) => &mut e.metadata,
            DexEvent::RaydiumAmmV4DepositEvent(e) => &mut e.metadata,
            DexEvent::RaydiumAmmV4WithdrawEvent(e) => &mut e.metadata,
            DexEvent::RaydiumAmmV4WithdrawPnlEvent(e) => &mut e.metadata,
            DexEvent::RaydiumAmmV4Initialize2Event(e) => &mut e.metadata,
            DexEvent::RaydiumAmmV4AmmInfoAccountEvent(e) => &mut e.metadata,
            DexEvent::RaydiumClmmSwapEvent(e) => &mut e.metadata,
            DexEvent::RaydiumClmmSwapV2Event(e) => &mut e.metadata,
            DexEvent::RaydiumClmmClosePositionEvent(e) => &mut e.metadata,
            DexEvent::RaydiumClmmIncreaseLiquidityV2Event(e) => &mut e.metadata,
            DexEvent::RaydiumClmmDecreaseLiquidityV2Event(e) => &mut e.metadata,
            DexEvent::RaydiumClmmCreatePoolEvent(e) => &mut e.metadata,
            DexEvent::RaydiumClmmOpenPositionWithToken22NftEvent(e) => &mut e.metadata,
            DexEvent::RaydiumClmmOpenPositionV2Event(e) => &mut e.metadata,
            DexEvent::RaydiumClmmAmmConfigAccountEvent(e) => &mut e.metadata,
            DexEvent::RaydiumClmmPoolStateAccountEvent(e) => &mut e.metadata,
            DexEvent::RaydiumClmmTickArrayStateAccountEvent(e) => &mut e.metadata,
            DexEvent::RaydiumCpmmSwapEvent(e) => &mut e.metadata,
            DexEvent::RaydiumCpmmDepositEvent(e) => &mut e.metadata,
            DexEvent::RaydiumCpmmWithdrawEvent(e) => &mut e.metadata,
            DexEvent::RaydiumCpmmInitializeEvent(e) => &mut e.metadata,
            DexEvent::RaydiumCpmmAmmConfigAccountEvent(e) => &mut e.metadata,
            DexEvent::RaydiumCpmmPoolStateAccountEvent(e) => &mut e.metadata,
            DexEvent::MeteoraDammV2SwapEvent(e) => &mut e.metadata,
            DexEvent::MeteoraDammV2Swap2Event(e) => &mut e.metadata,
            DexEvent::MeteoraDammV2InitializePoolEvent(e) => &mut e.metadata,
            DexEvent::MeteoraDammV2InitializeCustomizablePoolEvent(e) => &mut e.metadata,
            DexEvent::MeteoraDammV2InitializePoolWithDynamicConfigEvent(e) => &mut e.metadata,
            DexEvent::TokenAccountEvent(e) => &mut e.metadata,
            DexEvent::NonceAccountEvent(e) => &mut e.metadata,
            DexEvent::TokenInfoEvent(e) => &mut e.metadata,
            DexEvent::BlockMetaEvent(e) => &mut e.metadata,
            DexEvent::SetComputeUnitLimitEvent(e) => &mut e.metadata,
            DexEvent::SetComputeUnitPriceEvent(e) => &mut e.metadata,
        }
    }
}
