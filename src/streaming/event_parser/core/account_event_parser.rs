use crate::streaming::common::SimdUtils;
use crate::streaming::event_parser::common::filter::EventTypeFilter;
use crate::streaming::event_parser::common::high_performance_clock::elapsed_micros_since;
use crate::streaming::event_parser::common::EventMetadata;
use crate::streaming::event_parser::core::parser_cache::get_account_configs;
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
        // 直接从 parser_cache 获取配置
        let configs = get_account_configs(
            protocols,
            event_type_filter,
            Self::parse_nonce_account_event,
            Self::parse_token_account_event,
        );
        for config in configs {
            if config.program_id == Pubkey::default()
                || (account.owner == config.program_id
                    && SimdUtils::fast_discriminator_match(
                        &account.data,
                        config.account_discriminator,
                    ))
            {
                let event = (config.account_parser)(
                    &account,
                    EventMetadata {
                        slot: account.slot,
                        signature: account.signature,
                        protocol: config.protocol_type,
                        event_type: config.event_type,
                        program_id: config.program_id,
                        recv_us: account.recv_us,
                        handle_us: elapsed_micros_since(account.recv_us),
                        ..Default::default()
                    },
                );
                if event.is_some() {
                    return event;
                }
            }
        }
        None
    }

    pub fn parse_token_account_event(
        account: &AccountPretty,
        metadata: EventMetadata,
    ) -> Option<DexEvent> {
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
        metadata: EventMetadata,
    ) -> Option<DexEvent> {
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
