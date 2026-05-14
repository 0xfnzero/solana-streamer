use crate::streaming::event_parser::common::{
    types::EventType, ACCOUNT_EVENT_TYPES, BLOCK_EVENT_TYPES,
};
use crate::streaming::event_parser::DexEvent;
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

/// Whether the local pass should parse ComputeBudget instructions.
#[inline]
pub(crate) fn filter_includes_compute_budget_types(filter: Option<&EventTypeFilter>) -> bool {
    match filter {
        None => true,
        Some(f) => {
            let limit_excluded = f.exclude.contains(&EventType::SetComputeUnitLimit);
            let price_excluded = f.exclude.contains(&EventType::SetComputeUnitPrice);
            if limit_excluded && price_excluded {
                return false;
            }
            if f.include.is_empty() {
                return true;
            }
            f.include.iter().any(|t| {
                matches!(t, EventType::SetComputeUnitLimit | EventType::SetComputeUnitPrice)
            })
        }
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
        let mut raw: Vec<SdkGrpcEventType> = Vec::new();
        for et in &f.exclude {
            raw.extend(streamer_event_to_sdk_grpc_types(et));
        }
        dedup_sdk_grpc_event_types(&mut raw);
        return (!raw.is_empty()).then(|| SdkGrpcEventTypeFilter::exclude_types(raw));
    }

    if f.include.is_empty() {
        return None;
    }
    let mut raw: Vec<SdkGrpcEventType> = Vec::new();
    for et in &f.include {
        let mapped = streamer_event_to_sdk_grpc_types(et);
        if mapped.is_empty() {
            return None;
        }
        raw.extend(mapped);
    }
    dedup_sdk_grpc_event_types(&mut raw);
    Some(SdkGrpcEventTypeFilter::include_only(raw))
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

fn streamer_event_to_sdk_grpc_types(t: &EventType) -> Vec<SdkGrpcEventType> {
    use EventType as St;
    use SdkGrpcEventType as Sdk;
    match t {
        St::BlockMeta => vec![Sdk::BlockMeta],
        St::PumpFunCreateToken => vec![Sdk::PumpFunCreate],
        St::PumpFunCreateV2Token => vec![Sdk::PumpFunCreateV2],
        St::PumpFunBuy => vec![Sdk::PumpFunBuy, Sdk::PumpFunBuyExactSolIn],
        St::PumpFunBuyExactSolIn => vec![Sdk::PumpFunBuyExactSolIn],
        St::PumpFunSell => vec![Sdk::PumpFunSell],
        St::PumpFunMigrate => vec![Sdk::PumpFunMigrate],
        St::PumpFeesCreateFeeSharingConfig => vec![Sdk::PumpFeesCreateFeeSharingConfig],
        St::PumpFeesInitializeFeeConfig => vec![Sdk::PumpFeesInitializeFeeConfig],
        St::PumpFeesResetFeeSharingConfig => vec![Sdk::PumpFeesResetFeeSharingConfig],
        St::PumpFeesRevokeFeeSharingAuthority => vec![Sdk::PumpFeesRevokeFeeSharingAuthority],
        St::PumpFeesTransferFeeSharingAuthority => vec![Sdk::PumpFeesTransferFeeSharingAuthority],
        St::PumpFeesUpdateAdmin => vec![Sdk::PumpFeesUpdateAdmin],
        St::PumpFeesUpdateFeeConfig => vec![Sdk::PumpFeesUpdateFeeConfig],
        St::PumpFeesUpdateFeeShares => vec![Sdk::PumpFeesUpdateFeeShares],
        St::PumpFeesUpsertFeeTiers => vec![Sdk::PumpFeesUpsertFeeTiers],
        St::PumpFunMigrateBondingCurveCreator => vec![Sdk::PumpFunMigrateBondingCurveCreator],
        St::PumpSwapBuy => vec![Sdk::PumpSwapBuy],
        St::PumpSwapSell => vec![Sdk::PumpSwapSell],
        St::PumpSwapCreatePool => vec![Sdk::PumpSwapCreatePool],
        St::PumpSwapDeposit => vec![Sdk::PumpSwapLiquidityAdded],
        St::PumpSwapWithdraw => vec![Sdk::PumpSwapLiquidityRemoved],
        St::BonkBuyExactIn | St::BonkBuyExactOut | St::BonkSellExactIn | St::BonkSellExactOut => {
            vec![Sdk::BonkTrade]
        }
        St::BonkInitialize | St::BonkInitializeV2 | St::BonkInitializeWithToken2022 => {
            vec![Sdk::BonkPoolCreate]
        }
        St::BonkMigrateToAmm => vec![Sdk::BonkMigrateAmm],
        St::MeteoraDammV2Swap | St::MeteoraDammV2Swap2 => vec![Sdk::MeteoraDammV2Swap],
        St::MeteoraDammV2AddLiquidity => vec![Sdk::MeteoraDammV2AddLiquidity],
        St::MeteoraDammV2RemoveLiquidity => vec![Sdk::MeteoraDammV2RemoveLiquidity],
        St::MeteoraDammV2CreatePosition => vec![Sdk::MeteoraDammV2CreatePosition],
        St::MeteoraDammV2ClosePosition => vec![Sdk::MeteoraDammV2ClosePosition],
        St::TokenAccount => vec![Sdk::TokenAccount],
        St::TokenInfo => vec![Sdk::TokenAccount],
        St::NonceAccount => vec![Sdk::NonceAccount],
        St::AccountPumpFunGlobal => vec![Sdk::AccountPumpFunGlobal],
        St::AccountPumpSwapGlobalConfig => vec![Sdk::AccountPumpSwapGlobalConfig],
        St::AccountPumpSwapPool => vec![Sdk::AccountPumpSwapPool],
        _ => vec![],
    }
}

#[inline]
fn event_type_matches(filter_type: &EventType, event_type: &EventType) -> bool {
    filter_type == event_type
        || matches!(
            (filter_type, event_type),
            (EventType::PumpFunBuy, EventType::PumpFunBuyExactSolIn)
                | (EventType::TokenAccount, EventType::TokenInfo)
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
        assert!(sdk_f.should_include(SdkGrpcEventType::PumpFunBuy));
        assert!(sdk_f.should_include(SdkGrpcEventType::PumpFunBuyExactSolIn));
        assert!(!sdk_f.should_include(SdkGrpcEventType::PumpFunSell));
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
        assert!(!f.passes_event_type(&EventType::PumpFunBuy));
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
    fn build_sdk_filter_pumpfun_exact_sol_in_only() {
        let f = EventTypeFilter {
            include: vec![EventType::PumpFunBuyExactSolIn],
            ..Default::default()
        };
        let sdk_f = build_sdk_parse_event_filter(Some(&f)).expect("mapped");
        assert!(!sdk_f.should_include(SdkGrpcEventType::PumpFunBuy));
        assert!(sdk_f.should_include(SdkGrpcEventType::PumpFunBuyExactSolIn));
        assert!(!sdk_f.should_include(SdkGrpcEventType::PumpFunSell));
    }

    #[test]
    fn build_sdk_filter_token_info_maps_to_sdk_token_account() {
        let f = EventTypeFilter { include: vec![EventType::TokenInfo], ..Default::default() };
        let sdk_f = build_sdk_parse_event_filter(Some(&f)).expect("mapped");
        assert!(sdk_f.should_include(SdkGrpcEventType::TokenAccount));
    }

    #[test]
    fn build_sdk_filter_none_when_orca_in_mix() {
        let f = EventTypeFilter {
            include: vec![EventType::PumpFunBuy, EventType::OrcaWhirlpoolSwap],
            ..Default::default()
        };
        assert!(build_sdk_parse_event_filter(Some(&f)).is_none());
    }

    #[test]
    fn build_sdk_filter_exclude_only_none_when_only_unmapped_types() {
        let f = EventTypeFilter {
            include: vec![],
            exclude: vec![EventType::OrcaWhirlpoolSwap],
            ..Default::default()
        };
        assert!(build_sdk_parse_event_filter(Some(&f)).is_none());
    }
}
