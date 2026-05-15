use crate::streaming::event_parser::common::filter::EventTypeFilter;
use crate::streaming::event_parser::common::{EventMetadata, EventType};
use crate::streaming::event_parser::core::traits::DexEvent;
use crate::streaming::event_parser::Protocol;
use crate::streaming::grpc::AccountPretty;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// 通用账户事件
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub amount: Option<u64>,
    pub token_owner: Pubkey,
}

/// Nonce account event
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NonceAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub nonce: String,
    pub authority: String,
}

/// Nonce account event
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenInfoEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub supply: u64,
    pub decimals: u8,
}

pub struct AccountEventParser {}

impl AccountEventParser {
    pub fn parse_account_event(
        protocols: &[Protocol],
        account: AccountPretty,
        event_type_filter: Option<&EventTypeFilter>,
    ) -> Option<DexEvent> {
        crate::streaming::parser_sdk_bridge::parse_sdk_account_event(
            &account,
            protocols,
            event_type_filter,
        )
    }

    pub fn parse_token_account_event(
        account: &AccountPretty,
        metadata: EventMetadata,
    ) -> Option<DexEvent> {
        let mut account = account.clone();
        overlay_metadata(&mut account, &metadata);
        let filter = EventTypeFilter::include_only([EventType::TokenAccount]);
        crate::streaming::parser_sdk_bridge::parse_sdk_account_event(&account, &[], Some(&filter))
    }

    pub fn parse_nonce_account_event(
        account: &AccountPretty,
        metadata: EventMetadata,
    ) -> Option<DexEvent> {
        let mut account = account.clone();
        overlay_metadata(&mut account, &metadata);
        let filter = EventTypeFilter::include_only([EventType::NonceAccount]);
        crate::streaming::parser_sdk_bridge::parse_sdk_account_event(&account, &[], Some(&filter))
    }
}

fn overlay_metadata(account: &mut AccountPretty, metadata: &EventMetadata) {
    account.slot = metadata.slot;
    account.signature = metadata.signature;
    if metadata.recv_us != 0 {
        account.recv_us = metadata.recv_us;
    }
}
