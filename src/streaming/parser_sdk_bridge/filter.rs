//! Matching between subscribed [`Protocol`] values and streamer [`DexEvent`] variants.
use crate::streaming::event_parser::{DexEvent, Protocol};

pub(crate) fn event_matches_protocol(protocols: &[Protocol], ev: &DexEvent) -> bool {
    if is_protocol_independent_event(ev) {
        return true;
    }
    if protocols.is_empty() {
        return true;
    }
    protocols.iter().any(|p| protocol_matches_event(p, ev))
}

#[inline]
fn is_protocol_independent_event(ev: &DexEvent) -> bool {
    matches!(
        ev,
        DexEvent::TokenAccountEvent(_)
            | DexEvent::TokenInfoEvent(_)
            | DexEvent::NonceAccountEvent(_)
            | DexEvent::BlockMetaEvent(_)
            | DexEvent::SetComputeUnitLimitEvent(_)
            | DexEvent::SetComputeUnitPriceEvent(_)
            | DexEvent::ParserSdkErrorEvent(_)
    )
}

fn protocol_matches_event(p: &Protocol, ev: &DexEvent) -> bool {
    match (p, ev) {
        (Protocol::PumpFun, DexEvent::PumpFunCreateTokenEvent(_))
        | (Protocol::PumpFun, DexEvent::PumpFunCreateV2TokenEvent(_))
        | (Protocol::PumpFun, DexEvent::PumpFunTradeEvent(_))
        | (Protocol::PumpFun, DexEvent::PumpFunMigrateEvent(_))
        | (Protocol::PumpFun, DexEvent::PumpFeesCreateFeeSharingConfigEvent(_))
        | (Protocol::PumpFun, DexEvent::PumpFeesInitializeFeeConfigEvent(_))
        | (Protocol::PumpFun, DexEvent::PumpFeesResetFeeSharingConfigEvent(_))
        | (Protocol::PumpFun, DexEvent::PumpFeesRevokeFeeSharingAuthorityEvent(_))
        | (Protocol::PumpFun, DexEvent::PumpFeesTransferFeeSharingAuthorityEvent(_))
        | (Protocol::PumpFun, DexEvent::PumpFeesUpdateAdminEvent(_))
        | (Protocol::PumpFun, DexEvent::PumpFeesUpdateFeeConfigEvent(_))
        | (Protocol::PumpFun, DexEvent::PumpFeesUpdateFeeSharesEvent(_))
        | (Protocol::PumpFun, DexEvent::PumpFeesUpsertFeeTiersEvent(_))
        | (Protocol::PumpFun, DexEvent::PumpFunMigrateBondingCurveCreatorEvent(_))
        | (Protocol::PumpFun, DexEvent::PumpFunBondingCurveAccountEvent(_))
        | (Protocol::PumpFun, DexEvent::PumpFunGlobalAccountEvent(_)) => true,
        (Protocol::PumpFees, DexEvent::PumpFeesCreateFeeSharingConfigEvent(_))
        | (Protocol::PumpFees, DexEvent::PumpFeesInitializeFeeConfigEvent(_))
        | (Protocol::PumpFees, DexEvent::PumpFeesResetFeeSharingConfigEvent(_))
        | (Protocol::PumpFees, DexEvent::PumpFeesRevokeFeeSharingAuthorityEvent(_))
        | (Protocol::PumpFees, DexEvent::PumpFeesTransferFeeSharingAuthorityEvent(_))
        | (Protocol::PumpFees, DexEvent::PumpFeesUpdateAdminEvent(_))
        | (Protocol::PumpFees, DexEvent::PumpFeesUpdateFeeConfigEvent(_))
        | (Protocol::PumpFees, DexEvent::PumpFeesUpdateFeeSharesEvent(_))
        | (Protocol::PumpFees, DexEvent::PumpFeesUpsertFeeTiersEvent(_)) => true,
        (Protocol::PumpSwap, DexEvent::PumpSwapBuyEvent(_))
        | (Protocol::PumpSwap, DexEvent::PumpSwapSellEvent(_))
        | (Protocol::PumpSwap, DexEvent::PumpSwapCreatePoolEvent(_))
        | (Protocol::PumpSwap, DexEvent::PumpSwapDepositEvent(_))
        | (Protocol::PumpSwap, DexEvent::PumpSwapWithdrawEvent(_))
        | (Protocol::PumpSwap, DexEvent::PumpSwapGlobalConfigAccountEvent(_))
        | (Protocol::PumpSwap, DexEvent::PumpSwapPoolAccountEvent(_)) => true,
        (Protocol::Bonk, DexEvent::BonkTradeEvent(_))
        | (Protocol::Bonk, DexEvent::BonkPoolCreateEvent(_))
        | (Protocol::Bonk, DexEvent::BonkMigrateToAmmEvent(_))
        | (Protocol::Bonk, DexEvent::BonkMigrateToCpswapEvent(_))
        | (Protocol::Bonk, DexEvent::BonkPoolStateAccountEvent(_))
        | (Protocol::Bonk, DexEvent::BonkGlobalConfigAccountEvent(_))
        | (Protocol::Bonk, DexEvent::BonkPlatformConfigAccountEvent(_))
        | (Protocol::RaydiumLaunchpad, DexEvent::BonkTradeEvent(_))
        | (Protocol::RaydiumLaunchpad, DexEvent::BonkPoolCreateEvent(_))
        | (Protocol::RaydiumLaunchpad, DexEvent::BonkMigrateToAmmEvent(_))
        | (Protocol::RaydiumLaunchpad, DexEvent::BonkMigrateToCpswapEvent(_))
        | (Protocol::RaydiumLaunchpad, DexEvent::BonkPoolStateAccountEvent(_))
        | (Protocol::RaydiumLaunchpad, DexEvent::BonkGlobalConfigAccountEvent(_))
        | (Protocol::RaydiumLaunchpad, DexEvent::BonkPlatformConfigAccountEvent(_)) => true,
        (Protocol::RaydiumCpmm, DexEvent::RaydiumCpmmSwapEvent(_))
        | (Protocol::RaydiumCpmm, DexEvent::RaydiumCpmmDepositEvent(_))
        | (Protocol::RaydiumCpmm, DexEvent::RaydiumCpmmWithdrawEvent(_))
        | (Protocol::RaydiumCpmm, DexEvent::RaydiumCpmmInitializeEvent(_))
        | (Protocol::RaydiumCpmm, DexEvent::RaydiumCpmmAmmConfigAccountEvent(_))
        | (Protocol::RaydiumCpmm, DexEvent::RaydiumCpmmPoolStateAccountEvent(_)) => true,
        (Protocol::RaydiumClmm, DexEvent::RaydiumClmmSwapEvent(_))
        | (Protocol::RaydiumClmm, DexEvent::RaydiumClmmSwapV2Event(_))
        | (Protocol::RaydiumClmm, DexEvent::RaydiumClmmClosePositionEvent(_))
        | (Protocol::RaydiumClmm, DexEvent::RaydiumClmmIncreaseLiquidityV2Event(_))
        | (Protocol::RaydiumClmm, DexEvent::RaydiumClmmDecreaseLiquidityV2Event(_))
        | (Protocol::RaydiumClmm, DexEvent::RaydiumClmmCollectFeeEvent(_))
        | (Protocol::RaydiumClmm, DexEvent::RaydiumClmmCreatePoolEvent(_))
        | (Protocol::RaydiumClmm, DexEvent::RaydiumClmmOpenPositionWithToken22NftEvent(_))
        | (Protocol::RaydiumClmm, DexEvent::RaydiumClmmOpenPositionV2Event(_))
        | (Protocol::RaydiumClmm, DexEvent::RaydiumClmmAmmConfigAccountEvent(_))
        | (Protocol::RaydiumClmm, DexEvent::RaydiumClmmPoolStateAccountEvent(_))
        | (Protocol::RaydiumClmm, DexEvent::RaydiumClmmTickArrayStateAccountEvent(_)) => true,
        (Protocol::RaydiumAmmV4, DexEvent::RaydiumAmmV4SwapEvent(_))
        | (Protocol::RaydiumAmmV4, DexEvent::RaydiumAmmV4DepositEvent(_))
        | (Protocol::RaydiumAmmV4, DexEvent::RaydiumAmmV4WithdrawEvent(_))
        | (Protocol::RaydiumAmmV4, DexEvent::RaydiumAmmV4WithdrawPnlEvent(_))
        | (Protocol::RaydiumAmmV4, DexEvent::RaydiumAmmV4Initialize2Event(_))
        | (Protocol::RaydiumAmmV4, DexEvent::RaydiumAmmV4AmmInfoAccountEvent(_)) => true,
        (Protocol::MeteoraDammV2, DexEvent::MeteoraDammV2SwapEvent(_))
        | (Protocol::MeteoraDammV2, DexEvent::MeteoraDammV2Swap2Event(_))
        | (Protocol::MeteoraDammV2, DexEvent::MeteoraDammV2InitializePoolEvent(_))
        | (Protocol::MeteoraDammV2, DexEvent::MeteoraDammV2InitializeCustomizablePoolEvent(_))
        | (
            Protocol::MeteoraDammV2,
            DexEvent::MeteoraDammV2InitializePoolWithDynamicConfigEvent(_),
        )
        | (Protocol::MeteoraDammV2, DexEvent::MeteoraDammV2AddLiquidityEvent(_))
        | (Protocol::MeteoraDammV2, DexEvent::MeteoraDammV2RemoveLiquidityEvent(_))
        | (Protocol::MeteoraDammV2, DexEvent::MeteoraDammV2CreatePositionEvent(_))
        | (Protocol::MeteoraDammV2, DexEvent::MeteoraDammV2ClosePositionEvent(_)) => true,
        (Protocol::OrcaWhirlpool, DexEvent::OrcaWhirlpoolSwapEvent(_))
        | (Protocol::OrcaWhirlpool, DexEvent::OrcaWhirlpoolLiquidityIncreasedEvent(_))
        | (Protocol::OrcaWhirlpool, DexEvent::OrcaWhirlpoolLiquidityDecreasedEvent(_))
        | (Protocol::OrcaWhirlpool, DexEvent::OrcaWhirlpoolPoolInitializedEvent(_)) => true,
        (Protocol::MeteoraPools, DexEvent::MeteoraPoolsSwapEvent(_))
        | (Protocol::MeteoraPools, DexEvent::MeteoraPoolsAddLiquidityEvent(_))
        | (Protocol::MeteoraPools, DexEvent::MeteoraPoolsRemoveLiquidityEvent(_))
        | (Protocol::MeteoraPools, DexEvent::MeteoraPoolsBootstrapLiquidityEvent(_))
        | (Protocol::MeteoraPools, DexEvent::MeteoraPoolsPoolCreatedEvent(_))
        | (Protocol::MeteoraPools, DexEvent::MeteoraPoolsSetPoolFeesEvent(_)) => true,
        (Protocol::MeteoraDlmm, DexEvent::MeteoraDlmmSwapEvent(_))
        | (Protocol::MeteoraDlmm, DexEvent::MeteoraDlmmAddLiquidityEvent(_))
        | (Protocol::MeteoraDlmm, DexEvent::MeteoraDlmmRemoveLiquidityEvent(_))
        | (Protocol::MeteoraDlmm, DexEvent::MeteoraDlmmInitializePoolEvent(_))
        | (Protocol::MeteoraDlmm, DexEvent::MeteoraDlmmInitializeBinArrayEvent(_))
        | (Protocol::MeteoraDlmm, DexEvent::MeteoraDlmmCreatePositionEvent(_))
        | (Protocol::MeteoraDlmm, DexEvent::MeteoraDlmmClosePositionEvent(_))
        | (Protocol::MeteoraDlmm, DexEvent::MeteoraDlmmClaimFeeEvent(_)) => true,
        _ => false,
    }
}
