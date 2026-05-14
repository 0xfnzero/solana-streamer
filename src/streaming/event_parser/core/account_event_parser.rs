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
        use crate::streaming::event_parser::core::dispatcher::EventDispatcher;

        // 1. 尝试从账户 discriminator 解析（协议特定账户）
        if account.data.len() >= 8 {
            let discriminator = &account.data[0..8];

            // 尝试识别协议类型
            if let Some(protocol) = EventDispatcher::match_protocol_by_program_id(&account.owner) {
                // 检查是否在请求的协议列表中
                if protocols.contains(&protocol) {
                    // 构建临时元数据（protocol会被dispatcher设置，event_type会在parser中设置）
                    let metadata = EventMetadata {
                        slot: account.slot,
                        signature: account.signature,
                        protocol: ProtocolType::Common, // 会被 EventDispatcher::dispatch_account 设置
                        event_type: EventType::default(), // 会被具体 parser 设置
                        program_id: account.owner,
                        recv_us: account.recv_us,
                        handle_us: elapsed_micros_since(account.recv_us),
                        ..Default::default()
                    };

                    // 使用 dispatcher 解析
                    if let Some(event) = EventDispatcher::dispatch_account(
                        protocol,
                        discriminator,
                        &account,
                        metadata,
                    ) {
                        // 应用事件类型过滤
                        if let Some(filter) = event_type_filter {
                            if filter.passes_event_type(&event.metadata().event_type) {
                                return Some(event);
                            }
                            // 不匹配过滤器，继续尝试其他解析方式
                        } else {
                            return Some(event);
                        }
                    }
                }
            }
        }

        // 2. 尝试解析特殊账户类型（Token、Nonce等）
        // 这些是通用的，不属于特定协议
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

        // 尝试解析 Nonce 账户
        if let Some(event) = Self::parse_nonce_account_event(&account, metadata.clone()) {
            if let Some(filter) = event_type_filter {
                if filter.passes_event_type(&event.metadata().event_type) {
                    return Some(event);
                }
            } else {
                return Some(event);
            }
        }

        // 尝试解析 Token 账户
        if let Some(event) = Self::parse_token_account_event(&account, metadata) {
            if let Some(filter) = event_type_filter {
                if filter.passes_event_type(&event.metadata().event_type) {
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
                let mut metadata = metadata.clone();
                metadata.event_type = EventType::TokenInfo;
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
                let mut metadata = metadata.clone();
                metadata.event_type = EventType::TokenInfo;
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
