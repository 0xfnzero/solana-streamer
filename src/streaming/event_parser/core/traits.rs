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
use crate::streaming::event_parser::protocols::sol_parser_forward::events::{
    MeteoraDlmmAddLiquidityEvent, MeteoraDlmmClaimFeeEvent, MeteoraDlmmClosePositionEvent,
    MeteoraDlmmCreatePositionEvent, MeteoraDlmmInitializeBinArrayEvent,
    MeteoraDlmmInitializePoolEvent, MeteoraDlmmRemoveLiquidityEvent, MeteoraDlmmSwapEvent,
    MeteoraPoolsAddLiquidityEvent, MeteoraPoolsBootstrapLiquidityEvent,
    MeteoraPoolsPoolCreatedEvent, MeteoraPoolsRemoveLiquidityEvent, MeteoraPoolsSetPoolFeesEvent,
    MeteoraPoolsSwapEvent, OrcaWhirlpoolLiquidityDecreasedEvent,
    OrcaWhirlpoolLiquidityIncreasedEvent, OrcaWhirlpoolPoolInitializedEvent,
    OrcaWhirlpoolSwapEvent, ParserSdkErrorEvent,
};
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
    PumpFeesCreateFeeSharingConfigEvent(PumpFeesCreateFeeSharingConfigEvent),
    PumpFeesInitializeFeeConfigEvent(PumpFeesInitializeFeeConfigEvent),
    PumpFeesResetFeeSharingConfigEvent(PumpFeesResetFeeSharingConfigEvent),
    PumpFeesRevokeFeeSharingAuthorityEvent(PumpFeesRevokeFeeSharingAuthorityEvent),
    PumpFeesTransferFeeSharingAuthorityEvent(PumpFeesTransferFeeSharingAuthorityEvent),
    PumpFeesUpdateAdminEvent(PumpFeesUpdateAdminEvent),
    PumpFeesUpdateFeeConfigEvent(PumpFeesUpdateFeeConfigEvent),
    PumpFeesUpdateFeeSharesEvent(PumpFeesUpdateFeeSharesEvent),
    PumpFeesUpsertFeeTiersEvent(PumpFeesUpsertFeeTiersEvent),
    PumpFunMigrateBondingCurveCreatorEvent(PumpFunMigrateBondingCurveCreatorEvent),
    PumpFunBondingCurveAccountEvent(PumpFunBondingCurveAccountEvent),
    PumpFunGlobalAccountEvent(PumpFunGlobalAccountEvent),
    PumpFunFeeConfigAccountEvent(PumpFunFeeConfigAccountEvent),
    PumpFunSharingConfigAccountEvent(PumpFunSharingConfigAccountEvent),
    PumpFunGlobalVolumeAccumulatorAccountEvent(PumpFunGlobalVolumeAccumulatorAccountEvent),
    PumpFunUserVolumeAccumulatorAccountEvent(PumpFunUserVolumeAccumulatorAccountEvent),

    // PumpSwap events
    PumpSwapBuyEvent(PumpSwapBuyEvent),
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
    RaydiumClmmCollectFeeEvent(RaydiumClmmCollectFeeEvent),
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
    MeteoraDammV2InitializePoolWithDynamicConfigEvent(
        MeteoraDammV2InitializePoolWithDynamicConfigEvent,
    ),

    MeteoraDammV2AddLiquidityEvent(MeteoraDammV2AddLiquidityEvent),
    MeteoraDammV2RemoveLiquidityEvent(MeteoraDammV2RemoveLiquidityEvent),
    MeteoraDammV2CreatePositionEvent(MeteoraDammV2CreatePositionEvent),
    MeteoraDammV2ClosePositionEvent(MeteoraDammV2ClosePositionEvent),

    OrcaWhirlpoolSwapEvent(OrcaWhirlpoolSwapEvent),
    OrcaWhirlpoolLiquidityIncreasedEvent(OrcaWhirlpoolLiquidityIncreasedEvent),
    OrcaWhirlpoolLiquidityDecreasedEvent(OrcaWhirlpoolLiquidityDecreasedEvent),
    OrcaWhirlpoolPoolInitializedEvent(OrcaWhirlpoolPoolInitializedEvent),

    MeteoraPoolsSwapEvent(MeteoraPoolsSwapEvent),
    MeteoraPoolsAddLiquidityEvent(MeteoraPoolsAddLiquidityEvent),
    MeteoraPoolsRemoveLiquidityEvent(MeteoraPoolsRemoveLiquidityEvent),
    MeteoraPoolsBootstrapLiquidityEvent(MeteoraPoolsBootstrapLiquidityEvent),
    MeteoraPoolsPoolCreatedEvent(MeteoraPoolsPoolCreatedEvent),
    MeteoraPoolsSetPoolFeesEvent(MeteoraPoolsSetPoolFeesEvent),

    MeteoraDlmmSwapEvent(MeteoraDlmmSwapEvent),
    MeteoraDlmmAddLiquidityEvent(MeteoraDlmmAddLiquidityEvent),
    MeteoraDlmmRemoveLiquidityEvent(MeteoraDlmmRemoveLiquidityEvent),
    MeteoraDlmmInitializePoolEvent(MeteoraDlmmInitializePoolEvent),
    MeteoraDlmmInitializeBinArrayEvent(MeteoraDlmmInitializeBinArrayEvent),
    MeteoraDlmmCreatePositionEvent(MeteoraDlmmCreatePositionEvent),
    MeteoraDlmmClosePositionEvent(MeteoraDlmmClosePositionEvent),
    MeteoraDlmmClaimFeeEvent(MeteoraDlmmClaimFeeEvent),

    // Common events
    TokenAccountEvent(TokenAccountEvent),
    NonceAccountEvent(NonceAccountEvent),
    TokenInfoEvent(TokenInfoEvent),
    BlockMetaEvent(BlockMetaEvent),
    SetComputeUnitLimitEvent(SetComputeUnitLimitEvent),
    SetComputeUnitPriceEvent(SetComputeUnitPriceEvent),
    ParserSdkErrorEvent(ParserSdkErrorEvent),
}

/// Macro to generate metadata accessors for all DexEvent variants
macro_rules! impl_dex_event_metadata {
    ($($variant:ident),* $(,)?) => {
        impl DexEvent {
            pub fn metadata(&self) -> &EventMetadata {
                match self {
                    $(DexEvent::$variant(e) => &e.metadata,)*
                }
            }

            pub fn metadata_mut(&mut self) -> &mut EventMetadata {
                match self {
                    $(DexEvent::$variant(e) => &mut e.metadata,)*
                }
            }
        }
    };
}

impl_dex_event_metadata!(
    // Bonk events
    BonkTradeEvent,
    BonkPoolCreateEvent,
    BonkMigrateToAmmEvent,
    BonkMigrateToCpswapEvent,
    BonkPoolStateAccountEvent,
    BonkGlobalConfigAccountEvent,
    BonkPlatformConfigAccountEvent,
    // PumpFun events
    PumpFunCreateTokenEvent,
    PumpFunCreateV2TokenEvent,
    PumpFunTradeEvent,
    PumpFunMigrateEvent,
    PumpFeesCreateFeeSharingConfigEvent,
    PumpFeesInitializeFeeConfigEvent,
    PumpFeesResetFeeSharingConfigEvent,
    PumpFeesRevokeFeeSharingAuthorityEvent,
    PumpFeesTransferFeeSharingAuthorityEvent,
    PumpFeesUpdateAdminEvent,
    PumpFeesUpdateFeeConfigEvent,
    PumpFeesUpdateFeeSharesEvent,
    PumpFeesUpsertFeeTiersEvent,
    PumpFunMigrateBondingCurveCreatorEvent,
    PumpFunBondingCurveAccountEvent,
    PumpFunGlobalAccountEvent,
    PumpFunFeeConfigAccountEvent,
    PumpFunSharingConfigAccountEvent,
    PumpFunGlobalVolumeAccumulatorAccountEvent,
    PumpFunUserVolumeAccumulatorAccountEvent,
    // PumpSwap events
    PumpSwapBuyEvent,
    PumpSwapSellEvent,
    PumpSwapCreatePoolEvent,
    PumpSwapDepositEvent,
    PumpSwapWithdrawEvent,
    PumpSwapGlobalConfigAccountEvent,
    PumpSwapPoolAccountEvent,
    // Raydium AMM V4 events
    RaydiumAmmV4SwapEvent,
    RaydiumAmmV4DepositEvent,
    RaydiumAmmV4WithdrawEvent,
    RaydiumAmmV4WithdrawPnlEvent,
    RaydiumAmmV4Initialize2Event,
    RaydiumAmmV4AmmInfoAccountEvent,
    // Raydium CLMM events
    RaydiumClmmSwapEvent,
    RaydiumClmmSwapV2Event,
    RaydiumClmmClosePositionEvent,
    RaydiumClmmIncreaseLiquidityV2Event,
    RaydiumClmmDecreaseLiquidityV2Event,
    RaydiumClmmCollectFeeEvent,
    RaydiumClmmCreatePoolEvent,
    RaydiumClmmOpenPositionWithToken22NftEvent,
    RaydiumClmmOpenPositionV2Event,
    RaydiumClmmAmmConfigAccountEvent,
    RaydiumClmmPoolStateAccountEvent,
    RaydiumClmmTickArrayStateAccountEvent,
    // Raydium CPMM events
    RaydiumCpmmSwapEvent,
    RaydiumCpmmDepositEvent,
    RaydiumCpmmWithdrawEvent,
    RaydiumCpmmInitializeEvent,
    RaydiumCpmmAmmConfigAccountEvent,
    RaydiumCpmmPoolStateAccountEvent,
    // Meteora DAMM v2 events
    MeteoraDammV2SwapEvent,
    MeteoraDammV2Swap2Event,
    MeteoraDammV2InitializePoolEvent,
    MeteoraDammV2InitializeCustomizablePoolEvent,
    MeteoraDammV2InitializePoolWithDynamicConfigEvent,
    MeteoraDammV2AddLiquidityEvent,
    MeteoraDammV2RemoveLiquidityEvent,
    MeteoraDammV2CreatePositionEvent,
    MeteoraDammV2ClosePositionEvent,
    OrcaWhirlpoolSwapEvent,
    OrcaWhirlpoolLiquidityIncreasedEvent,
    OrcaWhirlpoolLiquidityDecreasedEvent,
    OrcaWhirlpoolPoolInitializedEvent,
    MeteoraPoolsSwapEvent,
    MeteoraPoolsAddLiquidityEvent,
    MeteoraPoolsRemoveLiquidityEvent,
    MeteoraPoolsBootstrapLiquidityEvent,
    MeteoraPoolsPoolCreatedEvent,
    MeteoraPoolsSetPoolFeesEvent,
    MeteoraDlmmSwapEvent,
    MeteoraDlmmAddLiquidityEvent,
    MeteoraDlmmRemoveLiquidityEvent,
    MeteoraDlmmInitializePoolEvent,
    MeteoraDlmmInitializeBinArrayEvent,
    MeteoraDlmmCreatePositionEvent,
    MeteoraDlmmClosePositionEvent,
    MeteoraDlmmClaimFeeEvent,
    // Common events
    TokenAccountEvent,
    NonceAccountEvent,
    TokenInfoEvent,
    BlockMetaEvent,
    SetComputeUnitLimitEvent,
    SetComputeUnitPriceEvent,
    ParserSdkErrorEvent,
);
