use crate::streaming::event_parser::common::{
    types::EventType, ACCOUNT_EVENT_TYPES, BLOCK_EVENT_TYPES,
};
use crate::streaming::event_parser::{DexEvent, Protocol};
use sol_parser_sdk::grpc::types::EventType as SdkGrpcEventType;
use sol_parser_sdk::grpc::types::EventTypeFilter as SdkGrpcEventTypeFilter;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct EventTypeFilter {
    pub include: Vec<EventType>,
    pub exclude: Vec<EventType>,
}

impl EventTypeFilter {
    #[inline]
    pub fn all() -> Self {
        Self::default()
    }

    #[inline]
    pub fn include_only(include: impl Into<Vec<EventType>>) -> Self {
        Self { include: include.into(), exclude: Vec::new() }
    }

    #[inline]
    pub fn exclude_only(exclude: impl Into<Vec<EventType>>) -> Self {
        Self { include: Vec::new(), exclude: exclude.into() }
    }

    #[inline]
    pub fn include_exclude(
        include: impl Into<Vec<EventType>>,
        exclude: impl Into<Vec<EventType>>,
    ) -> Self {
        Self { include: include.into(), exclude: exclude.into() }
    }

    pub fn include_transaction_event(&self) -> bool {
        if self.include.is_empty() && self.exclude.is_empty() {
            return true;
        }
        if !self.include.is_empty() {
            return self.include.iter().any(|event| {
                !ACCOUNT_EVENT_TYPES.contains(event) && !BLOCK_EVENT_TYPES.contains(event)
            });
        }
        // With exclude-only filters, keep the stream open and drop matching events locally.
        !self.exclude.is_empty()
    }

    pub fn include_account_event(&self) -> bool {
        if self.include.is_empty() && self.exclude.is_empty() {
            return true;
        }
        if !self.include.is_empty() {
            return self.include.iter().any(|event| ACCOUNT_EVENT_TYPES.contains(event));
        }
        !self.exclude.is_empty()
    }

    pub fn include_block_event(&self) -> bool {
        if self.include.is_empty() && self.exclude.is_empty() {
            return true;
        }
        if !self.include.is_empty() {
            return self.include.iter().any(|event| BLOCK_EVENT_TYPES.contains(event));
        }
        !self.exclude.is_empty()
    }

    /// Apply `exclude` first, then `include`. Empty `include` means "allow all non-excluded types".
    #[inline]
    pub fn passes_event_type(&self, et: &EventType) -> bool {
        if self.exclude.iter().any(|excluded| event_type_matches(excluded, et)) {
            return false;
        }
        if self.include.is_empty() {
            return true;
        }
        self.include.iter().any(|included| event_type_matches(included, et))
    }

    #[inline]
    pub fn passes_for_event(&self, ev: &DexEvent) -> bool {
        self.passes_event_type(&ev.metadata().event_type)
    }
}

/// `None` means no filtering; `Some(f)` applies include/exclude event-type semantics.
#[inline]
pub(crate) fn passes_event_type_filter(filter: Option<&EventTypeFilter>, ev: &DexEvent) -> bool {
    match filter {
        None => true,
        Some(f) => f.passes_for_event(ev),
    }
}

/// Map the streamer filter to the SDK gRPC event-type filter used by
/// [`sol_parser_sdk::grpc::parse_subscribe_update_transaction_low_latency`].
///
/// - Empty include/exclude maps to `None`.
/// - Exclude-only maps to SDK `exclude_types` when at least one SDK type is known.
/// - Non-empty include maps to SDK `include_only`; streamer still applies exclude locally.
/// - If any included type cannot map to an SDK type, return `None` to avoid dropping it upstream.
pub(crate) fn build_sdk_parse_event_filter(
    filter: Option<&EventTypeFilter>,
) -> Option<SdkGrpcEventTypeFilter> {
    let f = filter?;

    if !f.exclude.is_empty() && f.include.is_empty() {
        let mut raw: Vec<SdkGrpcEventType> = Vec::with_capacity(f.exclude.len());
        for et in &f.exclude {
            push_streamer_event_sdk_grpc_types(et, &mut raw, FilterMapMode::Exclude);
        }
        dedup_sdk_grpc_event_types(&mut raw);
        return (!raw.is_empty()).then(|| SdkGrpcEventTypeFilter::exclude_types(raw));
    }

    if f.include.is_empty() {
        return None;
    }
    let mut raw: Vec<SdkGrpcEventType> = Vec::with_capacity(f.include.len());
    for et in &f.include {
        if !push_streamer_event_sdk_grpc_types(et, &mut raw, FilterMapMode::Include) {
            return None;
        }
    }
    dedup_sdk_grpc_event_types(&mut raw);
    Some(SdkGrpcEventTypeFilter::include_only(raw))
}

/// Build the SDK ShredStream hot-path filter. Unlike Yellowstone, ShredStream
/// subscription itself has no program filter, so protocol narrowing must be
/// pushed into the SDK parser as an event-type include list.
pub(crate) fn build_sdk_shred_parse_event_filter(
    protocols: &[Protocol],
    filter: Option<&EventTypeFilter>,
) -> Option<SdkGrpcEventTypeFilter> {
    if protocols.is_empty() {
        return build_sdk_parse_event_filter(filter);
    }

    if let Some(f) = filter.filter(|f| !f.include.is_empty()) {
        if let Some(exact) = build_sdk_parse_event_filter(Some(f)) {
            return Some(exact);
        }
        if !f.include_transaction_event() {
            return Some(SdkGrpcEventTypeFilter::include_only(Vec::new()));
        }
    }

    let mut raw = Vec::with_capacity(protocols.len() * 8);
    for protocol in protocols {
        push_protocol_sdk_grpc_event_types(protocol, &mut raw);
    }
    dedup_sdk_grpc_event_types(&mut raw);
    (!raw.is_empty()).then(|| SdkGrpcEventTypeFilter::include_only(raw))
}

fn dedup_sdk_grpc_event_types(v: &mut Vec<SdkGrpcEventType>) {
    let mut i = 0;
    while i < v.len() {
        if v[..i].contains(&v[i]) {
            v.remove(i);
        } else {
            i += 1;
        }
    }
}

#[derive(Clone, Copy)]
enum FilterMapMode {
    Include,
    Exclude,
}

fn push_streamer_event_sdk_grpc_types(
    t: &EventType,
    out: &mut Vec<SdkGrpcEventType>,
    mode: FilterMapMode,
) -> bool {
    use EventType as St;
    use SdkGrpcEventType as Sdk;
    match t {
        St::BlockMeta => out.push(Sdk::BlockMeta),
        St::PumpFunCreateToken => {
            out.push(Sdk::PumpFunCreate);
            out.push(Sdk::PumpFunCreateV2);
        }
        St::PumpFunCreateV2Token => {
            out.push(Sdk::PumpFunCreate);
            out.push(Sdk::PumpFunCreateV2);
        }
        St::PumpFunBuy => {
            if matches!(mode, FilterMapMode::Include) {
                out.push(Sdk::PumpFunTrade);
            }
            out.push(Sdk::PumpFunBuy);
            out.push(Sdk::PumpFunBuyExactSolIn);
        }
        St::PumpFunBuyExactSolIn => {
            if matches!(mode, FilterMapMode::Include) {
                out.push(Sdk::PumpFunTrade);
            }
            out.push(Sdk::PumpFunBuyExactSolIn);
        }
        St::PumpFunSell => {
            if matches!(mode, FilterMapMode::Include) {
                out.push(Sdk::PumpFunTrade);
            }
            out.push(Sdk::PumpFunSell);
        }
        St::PumpFunMigrate => out.push(Sdk::PumpFunMigrate),
        St::PumpFeesCreateFeeSharingConfig => out.push(Sdk::PumpFeesCreateFeeSharingConfig),
        St::PumpFeesInitializeFeeConfig => out.push(Sdk::PumpFeesInitializeFeeConfig),
        St::PumpFeesResetFeeSharingConfig => out.push(Sdk::PumpFeesResetFeeSharingConfig),
        St::PumpFeesRevokeFeeSharingAuthority => out.push(Sdk::PumpFeesRevokeFeeSharingAuthority),
        St::PumpFeesTransferFeeSharingAuthority => {
            out.push(Sdk::PumpFeesTransferFeeSharingAuthority)
        }
        St::PumpFeesUpdateAdmin => out.push(Sdk::PumpFeesUpdateAdmin),
        St::PumpFeesUpdateFeeConfig => out.push(Sdk::PumpFeesUpdateFeeConfig),
        St::PumpFeesUpdateFeeShares => out.push(Sdk::PumpFeesUpdateFeeShares),
        St::PumpFeesUpsertFeeTiers => out.push(Sdk::PumpFeesUpsertFeeTiers),
        St::PumpFunMigrateBondingCurveCreator => out.push(Sdk::PumpFunMigrateBondingCurveCreator),
        St::PumpSwapBuy => out.push(Sdk::PumpSwapBuy),
        St::PumpSwapSell => out.push(Sdk::PumpSwapSell),
        St::PumpSwapCreatePool => out.push(Sdk::PumpSwapCreatePool),
        St::PumpSwapDeposit => out.push(Sdk::PumpSwapLiquidityAdded),
        St::PumpSwapWithdraw => out.push(Sdk::PumpSwapLiquidityRemoved),
        St::BonkBuyExactIn | St::BonkBuyExactOut | St::BonkSellExactIn | St::BonkSellExactOut => {
            out.push(Sdk::BonkTrade)
        }
        St::BonkInitialize | St::BonkInitializeV2 | St::BonkInitializeWithToken2022 => {
            out.push(Sdk::BonkPoolCreate)
        }
        St::BonkMigrateToAmm => out.push(Sdk::BonkMigrateAmm),
        St::RaydiumCpmmSwapBaseInput | St::RaydiumCpmmSwapBaseOutput => {
            out.push(Sdk::RaydiumCpmmSwap)
        }
        St::RaydiumCpmmDeposit => out.push(Sdk::RaydiumCpmmDeposit),
        St::RaydiumCpmmInitialize => out.push(Sdk::RaydiumCpmmInitialize),
        St::RaydiumCpmmWithdraw => out.push(Sdk::RaydiumCpmmWithdraw),
        St::RaydiumClmmSwap | St::RaydiumClmmSwapV2 => out.push(Sdk::RaydiumClmmSwap),
        St::RaydiumClmmClosePosition => out.push(Sdk::RaydiumClmmClosePosition),
        St::RaydiumClmmIncreaseLiquidityV2 => out.push(Sdk::RaydiumClmmIncreaseLiquidity),
        St::RaydiumClmmDecreaseLiquidityV2 => out.push(Sdk::RaydiumClmmDecreaseLiquidity),
        St::RaydiumClmmLiquidityChange => out.push(Sdk::RaydiumClmmLiquidityChange),
        St::RaydiumClmmConfigChange => out.push(Sdk::RaydiumClmmConfigChange),
        St::RaydiumClmmCreatePersonalPosition => out.push(Sdk::RaydiumClmmCreatePersonalPosition),
        St::RaydiumClmmLiquidityCalculate => out.push(Sdk::RaydiumClmmLiquidityCalculate),
        St::RaydiumClmmOpenLimitOrder => out.push(Sdk::RaydiumClmmOpenLimitOrder),
        St::RaydiumClmmIncreaseLimitOrder => out.push(Sdk::RaydiumClmmIncreaseLimitOrder),
        St::RaydiumClmmDecreaseLimitOrder => out.push(Sdk::RaydiumClmmDecreaseLimitOrder),
        St::RaydiumClmmSettleLimitOrder => out.push(Sdk::RaydiumClmmSettleLimitOrder),
        St::RaydiumClmmUpdateRewardInfos => out.push(Sdk::RaydiumClmmUpdateRewardInfos),
        St::RaydiumClmmCreatePool => out.push(Sdk::RaydiumClmmCreatePool),
        St::RaydiumClmmOpenPositionWithToken22Nft => {
            out.push(Sdk::RaydiumClmmOpenPositionWithTokenExtNft)
        }
        St::RaydiumClmmOpenPositionV2 => out.push(Sdk::RaydiumClmmOpenPosition),
        St::RaydiumClmmCollectFee => out.push(Sdk::RaydiumClmmCollectFee),
        St::RaydiumAmmV4SwapBaseIn | St::RaydiumAmmV4SwapBaseOut => out.push(Sdk::RaydiumAmmV4Swap),
        St::RaydiumAmmV4Deposit => out.push(Sdk::RaydiumAmmV4Deposit),
        St::RaydiumAmmV4Initialize2 => out.push(Sdk::RaydiumAmmV4Initialize2),
        St::RaydiumAmmV4Withdraw => out.push(Sdk::RaydiumAmmV4Withdraw),
        St::RaydiumAmmV4WithdrawPnl => out.push(Sdk::RaydiumAmmV4WithdrawPnl),
        St::OrcaWhirlpoolSwap => out.push(Sdk::OrcaWhirlpoolSwap),
        St::OrcaWhirlpoolLiquidityIncreased => out.push(Sdk::OrcaWhirlpoolLiquidityIncreased),
        St::OrcaWhirlpoolLiquidityDecreased => out.push(Sdk::OrcaWhirlpoolLiquidityDecreased),
        St::OrcaWhirlpoolPoolInitialized => out.push(Sdk::OrcaWhirlpoolPoolInitialized),
        St::MeteoraPoolsSwap => out.push(Sdk::MeteoraPoolsSwap),
        St::MeteoraPoolsAddLiquidity => out.push(Sdk::MeteoraPoolsAddLiquidity),
        St::MeteoraPoolsRemoveLiquidity => out.push(Sdk::MeteoraPoolsRemoveLiquidity),
        St::MeteoraPoolsBootstrapLiquidity => out.push(Sdk::MeteoraPoolsBootstrapLiquidity),
        St::MeteoraPoolsPoolCreated => out.push(Sdk::MeteoraPoolsPoolCreated),
        St::MeteoraPoolsSetPoolFees => out.push(Sdk::MeteoraPoolsSetPoolFees),
        St::MeteoraDammV2Swap | St::MeteoraDammV2Swap2 => out.push(Sdk::MeteoraDammV2Swap),
        St::MeteoraDammV2AddLiquidity => out.push(Sdk::MeteoraDammV2AddLiquidity),
        St::MeteoraDammV2RemoveLiquidity => out.push(Sdk::MeteoraDammV2RemoveLiquidity),
        St::MeteoraDammV2CreatePosition => out.push(Sdk::MeteoraDammV2CreatePosition),
        St::MeteoraDammV2ClosePosition => out.push(Sdk::MeteoraDammV2ClosePosition),
        St::MeteoraDlmmSwap => out.push(Sdk::MeteoraDlmmSwap),
        St::MeteoraDlmmAddLiquidity => out.push(Sdk::MeteoraDlmmAddLiquidity),
        St::MeteoraDlmmRemoveLiquidity => out.push(Sdk::MeteoraDlmmRemoveLiquidity),
        St::MeteoraDlmmInitializePool => out.push(Sdk::MeteoraDlmmInitializePool),
        St::MeteoraDlmmInitializeBinArray => out.push(Sdk::MeteoraDlmmInitializeBinArray),
        St::MeteoraDlmmCreatePosition => out.push(Sdk::MeteoraDlmmCreatePosition),
        St::MeteoraDlmmClosePosition => out.push(Sdk::MeteoraDlmmClosePosition),
        St::MeteoraDlmmClaimFee => out.push(Sdk::MeteoraDlmmClaimFee),
        St::TokenAccount | St::TokenInfo => out.push(Sdk::TokenAccount),
        St::NonceAccount => out.push(Sdk::NonceAccount),
        St::AccountPumpFunGlobal => out.push(Sdk::AccountPumpFunGlobal),
        St::AccountPumpFunBondingCurve => out.push(Sdk::AccountPumpFunBondingCurve),
        St::AccountPumpFunFeeConfig => out.push(Sdk::AccountPumpFunFeeConfig),
        St::AccountPumpFunSharingConfig => out.push(Sdk::AccountPumpFunSharingConfig),
        St::AccountPumpFunGlobalVolumeAccumulator => {
            out.push(Sdk::AccountPumpFunGlobalVolumeAccumulator)
        }
        St::AccountPumpFunUserVolumeAccumulator => {
            out.push(Sdk::AccountPumpFunUserVolumeAccumulator)
        }
        St::AccountPumpSwapGlobalConfig => out.push(Sdk::AccountPumpSwapGlobalConfig),
        St::AccountPumpSwapPool => out.push(Sdk::AccountPumpSwapPool),
        _ => return false,
    }
    true
}

fn push_protocol_sdk_grpc_event_types(protocol: &Protocol, out: &mut Vec<SdkGrpcEventType>) {
    use Protocol as StProtocol;
    use SdkGrpcEventType as Sdk;

    match protocol {
        StProtocol::PumpFun => out.extend_from_slice(&[
            Sdk::PumpFunTrade,
            Sdk::PumpFunBuy,
            Sdk::PumpFunSell,
            Sdk::PumpFunBuyExactSolIn,
            Sdk::PumpFunCreate,
            Sdk::PumpFunCreateV2,
            Sdk::PumpFunMigrate,
            Sdk::PumpFunMigrateBondingCurveCreator,
            // Historical compatibility: streamer PumpFun also included PumpFees.
            Sdk::PumpFeesCreateFeeSharingConfig,
            Sdk::PumpFeesInitializeFeeConfig,
            Sdk::PumpFeesResetFeeSharingConfig,
            Sdk::PumpFeesRevokeFeeSharingAuthority,
            Sdk::PumpFeesTransferFeeSharingAuthority,
            Sdk::PumpFeesUpdateAdmin,
            Sdk::PumpFeesUpdateFeeConfig,
            Sdk::PumpFeesUpdateFeeShares,
            Sdk::PumpFeesUpsertFeeTiers,
        ]),
        StProtocol::PumpFees => out.extend_from_slice(&[
            Sdk::PumpFeesCreateFeeSharingConfig,
            Sdk::PumpFeesInitializeFeeConfig,
            Sdk::PumpFeesResetFeeSharingConfig,
            Sdk::PumpFeesRevokeFeeSharingAuthority,
            Sdk::PumpFeesTransferFeeSharingAuthority,
            Sdk::PumpFeesUpdateAdmin,
            Sdk::PumpFeesUpdateFeeConfig,
            Sdk::PumpFeesUpdateFeeShares,
            Sdk::PumpFeesUpsertFeeTiers,
        ]),
        StProtocol::PumpSwap => out.extend_from_slice(&[
            Sdk::PumpSwapTrade,
            Sdk::PumpSwapBuy,
            Sdk::PumpSwapSell,
            Sdk::PumpSwapCreatePool,
            Sdk::PumpSwapLiquidityAdded,
            Sdk::PumpSwapLiquidityRemoved,
        ]),
        StProtocol::Bonk | StProtocol::RaydiumLaunchpad => {
            out.extend_from_slice(&[Sdk::BonkTrade, Sdk::BonkPoolCreate, Sdk::BonkMigrateAmm])
        }
        StProtocol::RaydiumCpmm => out.extend_from_slice(&[
            Sdk::RaydiumCpmmSwap,
            Sdk::RaydiumCpmmDeposit,
            Sdk::RaydiumCpmmWithdraw,
            Sdk::RaydiumCpmmInitialize,
        ]),
        StProtocol::RaydiumClmm => out.extend_from_slice(&[
            Sdk::RaydiumClmmSwap,
            Sdk::RaydiumClmmCreatePool,
            Sdk::RaydiumClmmOpenPosition,
            Sdk::RaydiumClmmClosePosition,
            Sdk::RaydiumClmmIncreaseLiquidity,
            Sdk::RaydiumClmmDecreaseLiquidity,
            Sdk::RaydiumClmmLiquidityChange,
            Sdk::RaydiumClmmConfigChange,
            Sdk::RaydiumClmmCreatePersonalPosition,
            Sdk::RaydiumClmmLiquidityCalculate,
            Sdk::RaydiumClmmOpenLimitOrder,
            Sdk::RaydiumClmmIncreaseLimitOrder,
            Sdk::RaydiumClmmDecreaseLimitOrder,
            Sdk::RaydiumClmmSettleLimitOrder,
            Sdk::RaydiumClmmUpdateRewardInfos,
            Sdk::RaydiumClmmOpenPositionWithTokenExtNft,
            Sdk::RaydiumClmmCollectFee,
        ]),
        StProtocol::RaydiumAmmV4 => out.extend_from_slice(&[
            Sdk::RaydiumAmmV4Swap,
            Sdk::RaydiumAmmV4Deposit,
            Sdk::RaydiumAmmV4Withdraw,
            Sdk::RaydiumAmmV4Initialize2,
            Sdk::RaydiumAmmV4WithdrawPnl,
        ]),
        StProtocol::MeteoraDammV2 => out.extend_from_slice(&[
            Sdk::MeteoraDammV2Swap,
            Sdk::MeteoraDammV2AddLiquidity,
            Sdk::MeteoraDammV2RemoveLiquidity,
            Sdk::MeteoraDammV2CreatePosition,
            Sdk::MeteoraDammV2ClosePosition,
        ]),
        StProtocol::OrcaWhirlpool => out.extend_from_slice(&[
            Sdk::OrcaWhirlpoolSwap,
            Sdk::OrcaWhirlpoolLiquidityIncreased,
            Sdk::OrcaWhirlpoolLiquidityDecreased,
            Sdk::OrcaWhirlpoolPoolInitialized,
        ]),
        StProtocol::MeteoraPools => out.extend_from_slice(&[
            Sdk::MeteoraPoolsSwap,
            Sdk::MeteoraPoolsAddLiquidity,
            Sdk::MeteoraPoolsRemoveLiquidity,
            Sdk::MeteoraPoolsBootstrapLiquidity,
            Sdk::MeteoraPoolsPoolCreated,
            Sdk::MeteoraPoolsSetPoolFees,
        ]),
        StProtocol::MeteoraDlmm => out.extend_from_slice(&[
            Sdk::MeteoraDlmmSwap,
            Sdk::MeteoraDlmmAddLiquidity,
            Sdk::MeteoraDlmmRemoveLiquidity,
            Sdk::MeteoraDlmmInitializePool,
            Sdk::MeteoraDlmmInitializeBinArray,
            Sdk::MeteoraDlmmCreatePosition,
            Sdk::MeteoraDlmmClosePosition,
            Sdk::MeteoraDlmmClaimFee,
        ]),
    }
}

#[inline]
fn event_type_matches(filter_type: &EventType, event_type: &EventType) -> bool {
    filter_type == event_type
        || matches!(
            (filter_type, event_type),
            (EventType::PumpFunCreateToken, EventType::PumpFunCreateV2Token)
                | (EventType::PumpFunCreateV2Token, EventType::PumpFunCreateToken)
                | (EventType::PumpFunBuy, EventType::PumpFunBuyExactSolIn)
                | (EventType::PumpFunBuyExactSolIn, EventType::PumpFunBuy)
                | (EventType::TokenAccount, EventType::TokenInfo)
                | (EventType::RaydiumClmmSwap, EventType::RaydiumClmmSwapV2)
                | (EventType::RaydiumClmmSwapV2, EventType::RaydiumClmmSwap)
                | (EventType::MeteoraDammV2Swap, EventType::MeteoraDammV2Swap2)
                | (EventType::MeteoraDammV2Swap2, EventType::MeteoraDammV2Swap)
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming::event_parser::common::types::ProtocolType;
    use crate::streaming::event_parser::common::EventMetadata;
    use crate::streaming::event_parser::protocols::block::block_meta_event::BlockMetaEvent;
    use crate::streaming::event_parser::protocols::sol_parser_forward::events::ParserSdkErrorEvent;

    fn mk_meta(et: EventType) -> EventMetadata {
        EventMetadata::new(
            Default::default(),
            0,
            0,
            0,
            ProtocolType::PumpFun,
            et,
            Default::default(),
            0,
            None,
            0,
            None,
            None,
        )
    }

    #[test]
    fn passes_for_event_empty_include_is_all_true() {
        let f = EventTypeFilter { include: vec![], ..Default::default() };
        let ev = DexEvent::ParserSdkErrorEvent(ParserSdkErrorEvent {
            metadata: mk_meta(EventType::ParserSdkError),
            message: "x".into(),
        });
        assert!(f.passes_for_event(&ev));
    }

    #[test]
    fn constructors_set_expected_filter_sides() {
        assert_eq!(EventTypeFilter::all(), EventTypeFilter::default());
        assert_eq!(
            EventTypeFilter::include_only([EventType::PumpFunBuy]).include,
            vec![EventType::PumpFunBuy]
        );
        assert_eq!(
            EventTypeFilter::exclude_only([EventType::PumpFunSell]).exclude,
            vec![EventType::PumpFunSell]
        );
        let both =
            EventTypeFilter::include_exclude([EventType::PumpFunBuy], [EventType::PumpFunSell]);
        assert_eq!(both.include, vec![EventType::PumpFunBuy]);
        assert_eq!(both.exclude, vec![EventType::PumpFunSell]);
    }

    #[test]
    fn empty_filter_includes_all_subscription_kinds() {
        let f = EventTypeFilter::default();
        assert!(f.include_transaction_event());
        assert!(f.include_account_event());
        assert!(f.include_block_event());
    }

    #[test]
    fn exclude_blocks_even_when_include_empty() {
        let f = EventTypeFilter {
            include: vec![],
            exclude: vec![EventType::PumpFunSell],
            ..Default::default()
        };
        let ev = DexEvent::ParserSdkErrorEvent(ParserSdkErrorEvent {
            metadata: mk_meta(EventType::PumpFunSell),
            message: "x".into(),
        });
        assert!(!f.passes_for_event(&ev));
    }

    #[test]
    fn exclude_blocks_block_meta_when_stream_kept_open() {
        let f = EventTypeFilter::exclude_only([EventType::BlockMeta]);
        let ev = DexEvent::BlockMetaEvent(BlockMetaEvent {
            metadata: mk_meta(EventType::BlockMeta),
            slot: 0,
            block_hash: String::new(),
        });
        assert!(f.include_block_event());
        assert!(!f.passes_for_event(&ev));
    }

    #[test]
    fn exclude_applies_after_include_allow() {
        let f = EventTypeFilter {
            include: vec![EventType::PumpFunBuy, EventType::PumpFunSell],
            exclude: vec![EventType::PumpFunSell],
            ..Default::default()
        };
        let buy = DexEvent::ParserSdkErrorEvent(ParserSdkErrorEvent {
            metadata: mk_meta(EventType::PumpFunBuy),
            message: "x".into(),
        });
        let sell = DexEvent::ParserSdkErrorEvent(ParserSdkErrorEvent {
            metadata: mk_meta(EventType::PumpFunSell),
            message: "x".into(),
        });
        assert!(f.passes_for_event(&buy));
        assert!(!f.passes_for_event(&sell));
    }

    #[test]
    fn build_sdk_filter_exclude_only_pumpfun_sell() {
        let f = EventTypeFilter {
            include: vec![],
            exclude: vec![EventType::PumpFunSell],
            ..Default::default()
        };
        let sdk_f = build_sdk_parse_event_filter(Some(&f)).expect("mapped");
        assert!(sdk_f.should_include(SdkGrpcEventType::PumpFunBuy));
        assert!(!sdk_f.should_include(SdkGrpcEventType::PumpFunSell));
    }

    #[test]
    fn build_sdk_filter_pumpfun_buy_only() {
        let f = EventTypeFilter { include: vec![EventType::PumpFunBuy], ..Default::default() };
        let sdk_f = build_sdk_parse_event_filter(Some(&f)).expect("mapped");
        assert!(sdk_f.should_include(SdkGrpcEventType::PumpFunTrade));
        assert!(sdk_f.should_include(SdkGrpcEventType::PumpFunBuy));
        assert!(sdk_f.should_include(SdkGrpcEventType::PumpFunBuyExactSolIn));
        assert!(sdk_f.should_include(SdkGrpcEventType::PumpFunSell));
        assert!(f.passes_event_type(&EventType::PumpFunBuy));
        assert!(!f.passes_event_type(&EventType::PumpFunSell));
    }

    #[test]
    fn pumpfun_create_filter_matches_create_v2_for_backward_compat() {
        let f =
            EventTypeFilter { include: vec![EventType::PumpFunCreateToken], ..Default::default() };
        assert!(f.passes_event_type(&EventType::PumpFunCreateToken));
        assert!(f.passes_event_type(&EventType::PumpFunCreateV2Token));

        let sdk_f = build_sdk_parse_event_filter(Some(&f)).expect("mapped");
        assert!(sdk_f.should_include(SdkGrpcEventType::PumpFunCreate));
        assert!(sdk_f.should_include(SdkGrpcEventType::PumpFunCreateV2));
        assert!(!sdk_f.should_include(SdkGrpcEventType::PumpFunBuy));

        let f = EventTypeFilter {
            include: vec![EventType::PumpFunCreateV2Token],
            ..Default::default()
        };
        assert!(f.passes_event_type(&EventType::PumpFunCreateToken));
        assert!(f.passes_event_type(&EventType::PumpFunCreateV2Token));

        let sdk_f = build_sdk_parse_event_filter(Some(&f)).expect("mapped");
        assert!(sdk_f.should_include(SdkGrpcEventType::PumpFunCreate));
        assert!(sdk_f.should_include(SdkGrpcEventType::PumpFunCreateV2));
    }

    #[test]
    fn pumpfun_buy_filter_matches_exact_sol_in_for_backward_compat() {
        let f = EventTypeFilter { include: vec![EventType::PumpFunBuy], ..Default::default() };
        assert!(f.passes_event_type(&EventType::PumpFunBuy));
        assert!(f.passes_event_type(&EventType::PumpFunBuyExactSolIn));

        let f = EventTypeFilter {
            include: vec![EventType::PumpFunBuyExactSolIn],
            ..Default::default()
        };
        assert!(f.passes_event_type(&EventType::PumpFunBuy));
        assert!(f.passes_event_type(&EventType::PumpFunBuyExactSolIn));
    }

    #[test]
    fn token_account_filter_matches_token_info_for_backward_compat() {
        let f = EventTypeFilter { include: vec![EventType::TokenAccount], ..Default::default() };
        assert!(f.passes_event_type(&EventType::TokenAccount));
        assert!(f.passes_event_type(&EventType::TokenInfo));

        let f = EventTypeFilter { include: vec![EventType::TokenInfo], ..Default::default() };
        assert!(!f.passes_event_type(&EventType::TokenAccount));
        assert!(f.passes_event_type(&EventType::TokenInfo));
    }

    #[test]
    fn sdk_generic_swap_filters_match_streamer_specific_aliases() {
        let f =
            EventTypeFilter { include: vec![EventType::RaydiumClmmSwapV2], ..Default::default() };
        assert!(f.passes_event_type(&EventType::RaydiumClmmSwap));

        let f =
            EventTypeFilter { include: vec![EventType::MeteoraDammV2Swap2], ..Default::default() };
        assert!(f.passes_event_type(&EventType::MeteoraDammV2Swap));
    }

    #[test]
    fn build_sdk_filter_pumpfun_exact_sol_in_only() {
        let f = EventTypeFilter {
            include: vec![EventType::PumpFunBuyExactSolIn],
            ..Default::default()
        };
        let sdk_f = build_sdk_parse_event_filter(Some(&f)).expect("mapped");
        assert!(sdk_f.should_include(SdkGrpcEventType::PumpFunBuy));
        assert!(sdk_f.should_include(SdkGrpcEventType::PumpFunTrade));
        assert!(sdk_f.should_include(SdkGrpcEventType::PumpFunBuyExactSolIn));
        assert!(sdk_f.should_include(SdkGrpcEventType::PumpFunSell));
        assert!(f.passes_event_type(&EventType::PumpFunBuy));
        assert!(!f.passes_event_type(&EventType::PumpFunSell));
        assert!(f.passes_event_type(&EventType::PumpFunBuyExactSolIn));
    }

    #[test]
    fn build_sdk_filter_token_info_maps_to_sdk_token_account() {
        let f = EventTypeFilter { include: vec![EventType::TokenInfo], ..Default::default() };
        let sdk_f = build_sdk_parse_event_filter(Some(&f)).expect("mapped");
        assert!(sdk_f.should_include(SdkGrpcEventType::TokenAccount));
    }

    #[test]
    fn build_sdk_filter_maps_all_public_sdk_filter_events() {
        let sdk_filter_backed = [
            EventType::BlockMeta,
            EventType::PumpFunCreateToken,
            EventType::PumpFunCreateV2Token,
            EventType::PumpFunBuy,
            EventType::PumpFunBuyExactSolIn,
            EventType::PumpFunSell,
            EventType::PumpFunMigrate,
            EventType::PumpFunMigrateBondingCurveCreator,
            EventType::PumpFeesCreateFeeSharingConfig,
            EventType::PumpFeesInitializeFeeConfig,
            EventType::PumpFeesResetFeeSharingConfig,
            EventType::PumpFeesRevokeFeeSharingAuthority,
            EventType::PumpFeesTransferFeeSharingAuthority,
            EventType::PumpFeesUpdateAdmin,
            EventType::PumpFeesUpdateFeeConfig,
            EventType::PumpFeesUpdateFeeShares,
            EventType::PumpFeesUpsertFeeTiers,
            EventType::PumpSwapBuy,
            EventType::PumpSwapSell,
            EventType::PumpSwapCreatePool,
            EventType::PumpSwapDeposit,
            EventType::PumpSwapWithdraw,
            EventType::BonkBuyExactIn,
            EventType::BonkBuyExactOut,
            EventType::BonkSellExactIn,
            EventType::BonkSellExactOut,
            EventType::BonkInitialize,
            EventType::BonkInitializeV2,
            EventType::BonkInitializeWithToken2022,
            EventType::BonkMigrateToAmm,
            EventType::RaydiumCpmmSwapBaseInput,
            EventType::RaydiumCpmmSwapBaseOutput,
            EventType::RaydiumCpmmDeposit,
            EventType::RaydiumCpmmInitialize,
            EventType::RaydiumCpmmWithdraw,
            EventType::RaydiumClmmSwap,
            EventType::RaydiumClmmSwapV2,
            EventType::RaydiumClmmClosePosition,
            EventType::RaydiumClmmIncreaseLiquidityV2,
            EventType::RaydiumClmmDecreaseLiquidityV2,
            EventType::RaydiumClmmLiquidityChange,
            EventType::RaydiumClmmConfigChange,
            EventType::RaydiumClmmCreatePersonalPosition,
            EventType::RaydiumClmmLiquidityCalculate,
            EventType::RaydiumClmmOpenLimitOrder,
            EventType::RaydiumClmmIncreaseLimitOrder,
            EventType::RaydiumClmmDecreaseLimitOrder,
            EventType::RaydiumClmmSettleLimitOrder,
            EventType::RaydiumClmmUpdateRewardInfos,
            EventType::RaydiumClmmCreatePool,
            EventType::RaydiumClmmOpenPositionWithToken22Nft,
            EventType::RaydiumClmmOpenPositionV2,
            EventType::RaydiumClmmCollectFee,
            EventType::RaydiumAmmV4SwapBaseIn,
            EventType::RaydiumAmmV4SwapBaseOut,
            EventType::RaydiumAmmV4Deposit,
            EventType::RaydiumAmmV4Initialize2,
            EventType::RaydiumAmmV4Withdraw,
            EventType::RaydiumAmmV4WithdrawPnl,
            EventType::OrcaWhirlpoolSwap,
            EventType::OrcaWhirlpoolLiquidityIncreased,
            EventType::OrcaWhirlpoolLiquidityDecreased,
            EventType::OrcaWhirlpoolPoolInitialized,
            EventType::MeteoraPoolsSwap,
            EventType::MeteoraPoolsAddLiquidity,
            EventType::MeteoraPoolsRemoveLiquidity,
            EventType::MeteoraPoolsBootstrapLiquidity,
            EventType::MeteoraPoolsPoolCreated,
            EventType::MeteoraPoolsSetPoolFees,
            EventType::MeteoraDammV2Swap,
            EventType::MeteoraDammV2Swap2,
            EventType::MeteoraDammV2AddLiquidity,
            EventType::MeteoraDammV2RemoveLiquidity,
            EventType::MeteoraDammV2CreatePosition,
            EventType::MeteoraDammV2ClosePosition,
            EventType::MeteoraDlmmSwap,
            EventType::MeteoraDlmmAddLiquidity,
            EventType::MeteoraDlmmRemoveLiquidity,
            EventType::MeteoraDlmmInitializePool,
            EventType::MeteoraDlmmInitializeBinArray,
            EventType::MeteoraDlmmCreatePosition,
            EventType::MeteoraDlmmClosePosition,
            EventType::MeteoraDlmmClaimFee,
            EventType::TokenAccount,
            EventType::TokenInfo,
            EventType::NonceAccount,
            EventType::AccountPumpFunGlobal,
            EventType::AccountPumpFunBondingCurve,
            EventType::AccountPumpFunFeeConfig,
            EventType::AccountPumpFunSharingConfig,
            EventType::AccountPumpFunGlobalVolumeAccumulator,
            EventType::AccountPumpFunUserVolumeAccumulator,
            EventType::AccountPumpSwapGlobalConfig,
            EventType::AccountPumpSwapPool,
        ];

        for event_type in sdk_filter_backed {
            let f = EventTypeFilter { include: vec![event_type.clone()], ..Default::default() };
            assert!(
                build_sdk_parse_event_filter(Some(&f)).is_some(),
                "{event_type:?} should map to an SDK parse filter"
            );
        }
    }

    #[test]
    fn build_sdk_filter_none_when_public_sdk_filter_enum_cannot_express_requested_type() {
        let f = EventTypeFilter {
            include: vec![EventType::PumpFunBuy, EventType::MeteoraDammV2InitializePool],
            ..Default::default()
        };
        assert!(build_sdk_parse_event_filter(Some(&f)).is_none());
    }

    #[test]
    fn build_sdk_shred_filter_falls_back_to_protocol_group_for_sdk_enum_gap() {
        let f = EventTypeFilter {
            include: vec![EventType::MeteoraDammV2InitializePool],
            ..Default::default()
        };
        let sdk_f =
            build_sdk_shred_parse_event_filter(&[Protocol::MeteoraDammV2], Some(&f)).unwrap();

        assert!(sdk_f.should_include(SdkGrpcEventType::MeteoraDammV2Swap));
        assert!(!sdk_f.should_include(SdkGrpcEventType::PumpFunBuy));
    }

    #[test]
    fn build_sdk_shred_filter_skips_transaction_parse_for_account_only_gap() {
        let f = EventTypeFilter {
            include: vec![EventType::AccountRaydiumCpmmPoolState],
            ..Default::default()
        };
        let sdk_f = build_sdk_shred_parse_event_filter(&[Protocol::RaydiumCpmm], Some(&f)).unwrap();

        assert!(!sdk_f.should_include(SdkGrpcEventType::RaydiumCpmmSwap));
        assert!(!sdk_f.should_include(SdkGrpcEventType::PumpFunBuy));
    }

    #[test]
    fn build_sdk_filter_exclude_only_orca_maps_upstream() {
        let f = EventTypeFilter {
            include: vec![],
            exclude: vec![EventType::OrcaWhirlpoolSwap],
            ..Default::default()
        };
        let sdk_f = build_sdk_parse_event_filter(Some(&f)).expect("mapped");
        assert!(!sdk_f.should_include(SdkGrpcEventType::OrcaWhirlpoolSwap));
        assert!(sdk_f.should_include(SdkGrpcEventType::PumpFunBuy));
    }

    #[test]
    fn build_sdk_filter_exclude_only_none_when_only_sdk_filter_enum_gap() {
        let f = EventTypeFilter {
            include: vec![],
            exclude: vec![EventType::MeteoraDammV2InitializePool],
            ..Default::default()
        };
        assert!(build_sdk_parse_event_filter(Some(&f)).is_none());
    }

    #[test]
    fn build_sdk_filter_exclude_only_none_when_only_unmapped_types() {
        let f = EventTypeFilter {
            include: vec![],
            exclude: vec![EventType::SetComputeUnitLimit],
            ..Default::default()
        };
        assert!(build_sdk_parse_event_filter(Some(&f)).is_none());
    }
}
