use crate::streaming::event_parser::common::filter::EventTypeFilter;
use crate::streaming::event_parser::common::high_performance_clock::elapsed_micros_since;
use crate::streaming::event_parser::common::{EventMetadata, EventType, ProtocolType};
use crate::streaming::event_parser::core::traits::DexEvent;
use crate::streaming::event_parser::Protocol;
use crate::streaming::grpc::AccountPretty;
use serde::{Deserialize, Serialize};
use solana_account_decoder::parse_nonce::parse_nonce;
use solana_sdk::pubkey::Pubkey;
use spl_token::solana_program::program_pack::Pack;
use spl_token::state::{Account, Mint};
use spl_token_2022::{
    extension::StateWithExtensions,
    state::{Account as Account2022, Mint as Mint2022},
};

/// Generic account event
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
        use crate::streaming::event_parser::core::dispatcher::EventDispatcher;

        // 1. Try to parse from account discriminator (protocol-specific accounts)
        if account.data.len() >= 8 {
            let discriminator = &account.data[0..8];

            // Try to identify protocol type
            if let Some(protocol) = EventDispatcher::match_protocol_by_program_id(&account.owner) {
                // Check if in the requested protocol list
                if protocols.contains(&protocol) {
                    // Build temporary metadata (protocol will be set by dispatcher, event_type will be set by parser)
                    let metadata = EventMetadata {
                        slot: account.slot,
                        signature: account.signature,
                        protocol: ProtocolType::Common, // Will be set by EventDispatcher::dispatch_account
                        event_type: EventType::default(), // Will be set by specific parser
                        program_id: account.owner,
                        recv_us: account.recv_us,
                        handle_us: elapsed_micros_since(account.recv_us),
                        ..Default::default()
                    };

                    // Use dispatcher to parse
                    if let Some(event) = EventDispatcher::dispatch_account(
                        protocol,
                        discriminator,
                        &account,
                        metadata,
                    ) {
                        // Apply event type filter
                        if let Some(filter) = event_type_filter {
                            if filter.include.contains(&event.metadata().event_type) {
                                return Some(event);
                            }
                            // Doesn't match filter, continue trying other parsing methods
                        } else {
                            return Some(event);
                        }
                    }
                }
            }
        }

        // 2. Try to parse special account types (Token, Nonce, etc.)
        // These are generic and don't belong to specific protocols
        let metadata = EventMetadata {
            slot: account.slot,
            signature: account.signature,
            protocol: ProtocolType::Common,
            event_type: EventType::default(),
            program_id: account.owner,
            recv_us: account.recv_us,
            handle_us: elapsed_micros_since(account.recv_us),
            ..Default::default()
        };

        // Try to parse Nonce account
        if let Some(event) = Self::parse_nonce_account_event(&account, metadata.clone()) {
            if let Some(filter) = event_type_filter {
                if filter.include.contains(&event.metadata().event_type) {
                    return Some(event);
                }
            } else {
                return Some(event);
            }
        }

        // Try to parse Token account
        if let Some(event) = Self::parse_token_account_event(&account, metadata) {
            if let Some(filter) = event_type_filter {
                if filter.include.contains(&event.metadata().event_type) {
                    return Some(event);
                }
            } else {
                return Some(event);
            }
        }

        None
    }

    pub fn parse_token_account_event(
        account: &AccountPretty,
        mut metadata: EventMetadata,
    ) -> Option<DexEvent> {
        metadata.event_type = EventType::TokenAccount;

        let pubkey = account.pubkey;
        let executable = account.executable;
        let lamports = account.lamports;
        let owner = account.owner;
        let rent_epoch = account.rent_epoch;
        // Spl Token Mint
        if account.data.len() >= Mint::LEN {
            if let Ok(mint) = Mint::unpack_from_slice(&account.data) {
                let mut event = TokenInfoEvent {
                    metadata,
                    pubkey,
                    executable,
                    lamports,
                    owner,
                    rent_epoch,
                    supply: mint.supply,
                    decimals: mint.decimals,
                };
                let recv_delta = elapsed_micros_since(account.recv_us);
                event.metadata.handle_us = recv_delta;
                return Some(DexEvent::TokenInfoEvent(event));
            }
        }
        // Spl Token2022 Mint
        if account.data.len() >= Account2022::LEN {
            if let Ok(mint) = StateWithExtensions::<Mint2022>::unpack(&account.data) {
                let mut event = TokenInfoEvent {
                    metadata,
                    pubkey,
                    executable,
                    lamports,
                    owner,
                    rent_epoch,
                    supply: mint.base.supply,
                    decimals: mint.base.decimals,
                };
                let recv_delta = elapsed_micros_since(account.recv_us);
                event.metadata.handle_us = recv_delta;
                return Some(DexEvent::TokenInfoEvent(event));
            }
        }
        let amount = if account.owner.to_bytes() == spl_token_2022::ID.to_bytes() {
            StateWithExtensions::<Account2022>::unpack(&account.data)
                .ok()
                .map(|info| info.base.amount)
        } else {
            Account::unpack(&account.data).ok().map(|info| info.amount)
        };

        let mut event = TokenAccountEvent {
            metadata,
            pubkey,
            executable,
            lamports,
            owner,
            rent_epoch,
            amount,
            token_owner: account.owner,
        };
        let recv_delta = elapsed_micros_since(account.recv_us);
        event.metadata.handle_us = recv_delta;
        Some(DexEvent::TokenAccountEvent(event))
    }

    pub fn parse_nonce_account_event(
        account: &AccountPretty,
        mut metadata: EventMetadata,
    ) -> Option<DexEvent> {
        metadata.event_type = EventType::NonceAccount;

        if let Ok(info) = parse_nonce(&account.data) {
            match info {
                solana_account_decoder::parse_nonce::UiNonceState::Initialized(details) => {
                    let mut event = NonceAccountEvent {
                        metadata,
                        pubkey: account.pubkey,
                        executable: account.executable,
                        lamports: account.lamports,
                        owner: account.owner,
                        rent_epoch: account.rent_epoch,
                        nonce: details.blockhash,
                        authority: details.authority,
                    };
                    event.metadata.handle_us = elapsed_micros_since(account.recv_us);
                    return Some(DexEvent::NonceAccountEvent(event));
                }
                solana_account_decoder::parse_nonce::UiNonceState::Uninitialized => {}
            }
        }
        None
    }
}
