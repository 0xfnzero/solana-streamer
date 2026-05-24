//! Account parser bridge: keep SDK account parsing details out of streamer core paths.
use crate::streaming::event_parser::common::filter::{passes_event_type_filter, EventTypeFilter};
use crate::streaming::event_parser::common::EventType;
use crate::streaming::event_parser::{DexEvent, Protocol};
use crate::streaming::grpc::AccountPretty;
use sol_parser_sdk::grpc::types::{
    EventType as SdkGrpcEventType, EventTypeFilter as SdkGrpcEventTypeFilter,
};

use super::convert_parser_event;
use super::filter::event_matches_protocol;

pub(crate) enum AccountParseResult {
    Event(DexEvent),
    Filtered,
    Unsupported,
}

pub(crate) fn parse_account_event(
    account: &AccountPretty,
    protocols: &[Protocol],
    event_type_filter: Option<&EventTypeFilter>,
) -> Option<DexEvent> {
    match parse_account_event_for_streamer(account, protocols, event_type_filter) {
        AccountParseResult::Event(event) => Some(event),
        AccountParseResult::Filtered | AccountParseResult::Unsupported => None,
    }
}

pub(crate) fn parse_account_event_for_streamer(
    account: &AccountPretty,
    protocols: &[Protocol],
    event_type_filter: Option<&EventTypeFilter>,
) -> AccountParseResult {
    let sdk_account = sol_parser_sdk::accounts::AccountData {
        pubkey: account.pubkey,
        executable: account.executable,
        lamports: account.lamports,
        owner: account.owner,
        rent_epoch: account.rent_epoch,
        data: account.data.clone(),
    };
    let sdk_metadata = sol_parser_sdk::core::events::EventMetadata {
        signature: account.signature,
        slot: account.slot,
        tx_index: 0,
        block_time_us: 0,
        grpc_recv_us: account.recv_us,
        recent_blockhash: None,
    };
    let sdk_parse_filter = build_sdk_account_event_filter(event_type_filter);
    let sdk_event = sol_parser_sdk::accounts::parse_account_unified(
        &sdk_account,
        sdk_metadata,
        Some(&sdk_parse_filter),
    );

    let Some(sdk_event) = sdk_event else {
        return AccountParseResult::Unsupported;
    };

    let Some(mut event) = convert_parser_event(sdk_event, None, account.recv_us) else {
        return AccountParseResult::Unsupported;
    };
    if !event_matches_protocol(protocols, &event)
        || !passes_event_type_filter(event_type_filter, &event)
    {
        return AccountParseResult::Filtered;
    }
    normalize_account_event(&mut event, account);
    AccountParseResult::Event(event)
}

fn build_sdk_account_event_filter(filter: Option<&EventTypeFilter>) -> SdkGrpcEventTypeFilter {
    let Some(f) = filter else {
        return SdkGrpcEventTypeFilter::exclude_types(Vec::new());
    };

    if f.include.is_empty() {
        let mut raw = Vec::new();
        for et in &f.exclude {
            raw.extend(streamer_account_event_to_sdk_types(et));
        }
        dedup_sdk_grpc_event_types(&mut raw);
        return SdkGrpcEventTypeFilter::exclude_types(raw);
    }

    let mut raw = Vec::new();
    for et in &f.include {
        raw.extend(streamer_account_event_to_sdk_types(et));
    }
    dedup_sdk_grpc_event_types(&mut raw);
    SdkGrpcEventTypeFilter::include_only(raw)
}

fn streamer_account_event_to_sdk_types(t: &EventType) -> Vec<SdkGrpcEventType> {
    match t {
        EventType::TokenAccount | EventType::TokenInfo => vec![SdkGrpcEventType::TokenAccount],
        EventType::NonceAccount => vec![SdkGrpcEventType::NonceAccount],
        EventType::AccountPumpFunGlobal => vec![SdkGrpcEventType::AccountPumpFunGlobal],
        EventType::AccountPumpFunBondingCurve => {
            vec![SdkGrpcEventType::AccountPumpFunBondingCurve]
        }
        EventType::AccountPumpFunFeeConfig => vec![SdkGrpcEventType::AccountPumpFunFeeConfig],
        EventType::AccountPumpFunSharingConfig => {
            vec![SdkGrpcEventType::AccountPumpFunSharingConfig]
        }
        EventType::AccountPumpFunGlobalVolumeAccumulator => {
            vec![SdkGrpcEventType::AccountPumpFunGlobalVolumeAccumulator]
        }
        EventType::AccountPumpFunUserVolumeAccumulator => {
            vec![SdkGrpcEventType::AccountPumpFunUserVolumeAccumulator]
        }
        EventType::AccountPumpSwapGlobalConfig => {
            vec![SdkGrpcEventType::AccountPumpSwapGlobalConfig]
        }
        EventType::AccountPumpSwapPool => vec![SdkGrpcEventType::AccountPumpSwapPool],
        _ => Vec::new(),
    }
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

fn normalize_account_event(event: &mut DexEvent, account: &AccountPretty) {
    match event {
        DexEvent::PumpFunGlobalAccountEvent(e) => {
            e.executable = account.executable;
            e.lamports = account.lamports;
            e.owner = account.owner;
            e.rent_epoch = account.rent_epoch;
        }
        DexEvent::PumpFunBondingCurveAccountEvent(e) => {
            e.executable = account.executable;
            e.lamports = account.lamports;
            e.owner = account.owner;
            e.rent_epoch = account.rent_epoch;
        }
        DexEvent::PumpFunFeeConfigAccountEvent(e) => {
            e.executable = account.executable;
            e.lamports = account.lamports;
            e.owner = account.owner;
            e.rent_epoch = account.rent_epoch;
        }
        DexEvent::PumpFunSharingConfigAccountEvent(e) => {
            e.executable = account.executable;
            e.lamports = account.lamports;
            e.owner = account.owner;
            e.rent_epoch = account.rent_epoch;
        }
        DexEvent::PumpFunGlobalVolumeAccumulatorAccountEvent(e) => {
            e.executable = account.executable;
            e.lamports = account.lamports;
            e.owner = account.owner;
            e.rent_epoch = account.rent_epoch;
        }
        DexEvent::PumpFunUserVolumeAccumulatorAccountEvent(e) => {
            e.executable = account.executable;
            e.lamports = account.lamports;
            e.owner = account.owner;
            e.rent_epoch = account.rent_epoch;
        }
        _ => {}
    }
}
