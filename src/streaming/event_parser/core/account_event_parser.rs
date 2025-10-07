use crate::impl_unified_event;
use crate::streaming::common::SimdUtils;
use crate::streaming::event_parser::common::filter::EventTypeFilter;
use crate::streaming::event_parser::common::high_performance_clock::elapsed_micros_since;
use crate::streaming::event_parser::common::{EventMetadata, EventType, ProtocolType};
use crate::streaming::event_parser::core::traits::UnifiedEvent;
use crate::streaming::event_parser::protocols::bonk::parser::BONK_PROGRAM_ID;
use crate::streaming::event_parser::protocols::pumpfun::parser::PUMPFUN_PROGRAM_ID;
use crate::streaming::event_parser::protocols::pumpswap::parser::PUMPSWAP_PROGRAM_ID;
use crate::streaming::event_parser::protocols::raydium_amm_v4::parser::RAYDIUM_AMM_V4_PROGRAM_ID;
use crate::streaming::event_parser::protocols::raydium_clmm::parser::RAYDIUM_CLMM_PROGRAM_ID;
use crate::streaming::event_parser::protocols::raydium_cpmm::parser::RAYDIUM_CPMM_PROGRAM_ID;
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
use std::collections::HashMap;
use std::sync::OnceLock;

/// 通用事件解析器配置
#[derive(Debug, Clone)]
pub struct AccountEventParseConfig {
    pub program_id: Pubkey,
    pub protocol_type: ProtocolType,
    pub event_type: EventType,
    pub account_discriminator: &'static [u8],
    pub account_parser: AccountEventParserFn,
}

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
impl_unified_event!(TokenAccountEvent,);

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
impl_unified_event!(NonceAccountEvent,);

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
impl_unified_event!(TokenInfoEvent,);

/// 账户事件解析器
pub type AccountEventParserFn =
    fn(account: &AccountPretty, metadata: EventMetadata) -> Option<Box<dyn UnifiedEvent>>;

static PROTOCOL_CONFIGS_CACHE: OnceLock<HashMap<Protocol, Vec<AccountEventParseConfig>>> =
    OnceLock::new();

// 通用账户解析配置的静态缓存
static COMMON_CONFIG: OnceLock<AccountEventParseConfig> = OnceLock::new();
// Nonce account config
static NONCE_CONFIG: OnceLock<AccountEventParseConfig> = OnceLock::new();

pub struct AccountEventParser {}

impl AccountEventParser {
    pub fn configs(
        protocols: &[Protocol],
        event_type_filter: Option<&EventTypeFilter>,
    ) -> Vec<AccountEventParseConfig> {
        let protocols_map = PROTOCOL_CONFIGS_CACHE.get_or_init(|| {
            let mut map: HashMap<Protocol, Vec<AccountEventParseConfig>> = HashMap::new();
            map.insert(Protocol::PumpSwap, vec![
                AccountEventParseConfig {
                    program_id: PUMPSWAP_PROGRAM_ID,
                    protocol_type: ProtocolType::PumpSwap,
                    event_type: EventType::AccountPumpSwapGlobalConfig,
                    account_discriminator: crate::streaming::event_parser::protocols::pumpswap::discriminators::GLOBAL_CONFIG_ACCOUNT,
                    account_parser: crate::streaming::event_parser::protocols::pumpswap::types::global_config_parser,
                },
                AccountEventParseConfig {
                    program_id: PUMPSWAP_PROGRAM_ID,
                    protocol_type: ProtocolType::PumpSwap,
                    event_type: EventType::AccountPumpSwapPool,
                    account_discriminator: crate::streaming::event_parser::protocols::pumpswap::discriminators::POOL_ACCOUNT,
                    account_parser: crate::streaming::event_parser::protocols::pumpswap::types::pool_parser,
                },
            ]);
            map.insert(Protocol::PumpFun, vec![
                AccountEventParseConfig {
                    program_id: PUMPFUN_PROGRAM_ID,
                    protocol_type: ProtocolType::PumpFun,
                    event_type: EventType::AccountPumpFunBondingCurve,
                    account_discriminator: crate::streaming::event_parser::protocols::pumpfun::discriminators::BONDING_CURVE_ACCOUNT,
                    account_parser: crate::streaming::event_parser::protocols::pumpfun::types::bonding_curve_parser,
                },
                AccountEventParseConfig {
                    program_id: PUMPFUN_PROGRAM_ID,
                    protocol_type: ProtocolType::PumpFun,
                    event_type: EventType::AccountPumpFunGlobal,
                    account_discriminator: crate::streaming::event_parser::protocols::pumpfun::discriminators::GLOBAL_ACCOUNT,
                    account_parser: crate::streaming::event_parser::protocols::pumpfun::types::global_parser,
                },
            ]);
            map.insert(Protocol::Bonk, vec![
                AccountEventParseConfig {
                    program_id: BONK_PROGRAM_ID,
                    protocol_type: ProtocolType::Bonk,
                    event_type: EventType::AccountBonkPoolState,
                    account_discriminator: crate::streaming::event_parser::protocols::bonk::discriminators::POOL_STATE_ACCOUNT,
                    account_parser: crate::streaming::event_parser::protocols::bonk::types::pool_state_parser,
                },
                AccountEventParseConfig {
                    program_id: BONK_PROGRAM_ID,
                    protocol_type: ProtocolType::Bonk,
                    event_type: EventType::AccountBonkGlobalConfig,
                    account_discriminator: crate::streaming::event_parser::protocols::bonk::discriminators::GLOBAL_CONFIG_ACCOUNT,
                    account_parser: crate::streaming::event_parser::protocols::bonk::types::global_config_parser,
                },
                AccountEventParseConfig {
                    program_id: BONK_PROGRAM_ID,
                    protocol_type: ProtocolType::Bonk,
                    event_type: EventType::AccountBonkPlatformConfig,
                    account_discriminator: crate::streaming::event_parser::protocols::bonk::discriminators::PLATFORM_CONFIG_ACCOUNT,
                    account_parser: crate::streaming::event_parser::protocols::bonk::types::platform_config_parser,
                },
            ]);
            map.insert(Protocol::RaydiumCpmm, vec![
                AccountEventParseConfig {
                    program_id: RAYDIUM_CPMM_PROGRAM_ID,
                    protocol_type: ProtocolType::RaydiumCpmm,
                    event_type: EventType::AccountRaydiumCpmmAmmConfig,
                    account_discriminator: crate::streaming::event_parser::protocols::raydium_cpmm::discriminators::AMM_CONFIG,
                    account_parser: crate::streaming::event_parser::protocols::raydium_cpmm::types::amm_config_parser,
                },
                AccountEventParseConfig {
                    program_id: RAYDIUM_CPMM_PROGRAM_ID,
                    protocol_type: ProtocolType::RaydiumCpmm,
                    event_type: EventType::AccountRaydiumCpmmPoolState,
                    account_discriminator: crate::streaming::event_parser::protocols::raydium_cpmm::discriminators::POOL_STATE,
                    account_parser: crate::streaming::event_parser::protocols::raydium_cpmm::types::pool_state_parser,
                },
            ]);
            map.insert(Protocol::RaydiumClmm, vec![
                AccountEventParseConfig {
                    program_id: RAYDIUM_CLMM_PROGRAM_ID,
                    protocol_type: ProtocolType::RaydiumClmm,
                    event_type: EventType::AccountRaydiumClmmAmmConfig,
                    account_discriminator: crate::streaming::event_parser::protocols::raydium_clmm::discriminators::AMM_CONFIG,
                    account_parser: crate::streaming::event_parser::protocols::raydium_clmm::types::amm_config_parser,
                },
                AccountEventParseConfig {
                    program_id: RAYDIUM_CLMM_PROGRAM_ID,
                    protocol_type: ProtocolType::RaydiumClmm,
                    event_type: EventType::AccountRaydiumClmmPoolState,
                    account_discriminator: crate::streaming::event_parser::protocols::raydium_clmm::discriminators::POOL_STATE,
                    account_parser: crate::streaming::event_parser::protocols::raydium_clmm::types::pool_state_parser,
                },
                AccountEventParseConfig {
                    program_id: RAYDIUM_CLMM_PROGRAM_ID,
                    protocol_type: ProtocolType::RaydiumClmm,
                    event_type: EventType::AccountRaydiumClmmTickArrayState,
                    account_discriminator: crate::streaming::event_parser::protocols::raydium_clmm::discriminators::TICK_ARRAY_STATE,
                    account_parser: crate::streaming::event_parser::protocols::raydium_clmm::types::tick_array_state_parser,
                },
            ]);
            map.insert(Protocol::RaydiumAmmV4, vec![
                AccountEventParseConfig {
                    program_id: RAYDIUM_AMM_V4_PROGRAM_ID,
                    protocol_type: ProtocolType::RaydiumAmmV4,
                    event_type: EventType::AccountRaydiumAmmV4AmmInfo,
                    account_discriminator: crate::streaming::event_parser::protocols::raydium_amm_v4::discriminators::AMM_INFO,
                    account_parser: crate::streaming::event_parser::protocols::raydium_amm_v4::types::amm_info_parser,
                },
            ]);
            map
        });

        let mut configs = Vec::new();
        let empty_vec = Vec::new();

        // 预估容量以减少重新分配
        let estimated_capacity = protocols.len() * 3; // 大多数协议有2-3个配置
        configs.reserve(estimated_capacity);

        for protocol in protocols {
            let protocol_configs = protocols_map.get(protocol).unwrap_or(&empty_vec);
            // 如果没有过滤器，直接扩展所有配置
            if event_type_filter.is_none() {
                configs.extend(protocol_configs.iter().cloned());
            } else {
                // 有过滤器时才进行过滤
                let filter = event_type_filter.unwrap();
                configs.extend(
                    protocol_configs
                        .iter()
                        .filter(|config| filter.include.contains(&config.event_type))
                        .cloned(),
                );
            }
        }

        if event_type_filter.is_none()
            || event_type_filter.unwrap().include.contains(&EventType::NonceAccount)
        {
            let nonce_config = NONCE_CONFIG.get_or_init(|| AccountEventParseConfig {
                program_id: Pubkey::default(),
                protocol_type: ProtocolType::Common,
                event_type: EventType::NonceAccount,
                account_discriminator: &[1, 0, 0, 0, 1, 0, 0, 0],
                account_parser: Self::parse_nonce_account_event,
            });
            configs.push(nonce_config.clone());
        }

        let common_config = COMMON_CONFIG.get_or_init(|| AccountEventParseConfig {
            program_id: Pubkey::default(),
            protocol_type: ProtocolType::Common,
            event_type: EventType::TokenAccount,
            account_discriminator: &[],
            account_parser: Self::parse_token_account_event,
        });
        configs.push(common_config.clone());

        configs
    }

    pub fn parse_account_event(
        protocols: &[Protocol],
        account: AccountPretty,
        event_type_filter: Option<&EventTypeFilter>,
    ) -> Option<Box<dyn UnifiedEvent>> {
        let configs = Self::configs(protocols, event_type_filter);
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
                        ..Default::default()
                    },
                );
                if let Some(mut event) = event {
                    event.set_handle_us(elapsed_micros_since(account.recv_us));
                    return Some(event);
                }
            }
        }
        None
    }

    pub fn parse_token_account_event(
        account: &AccountPretty,
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
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
                event.set_handle_us(recv_delta);
                return Some(Box::new(event));
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
                event.set_handle_us(recv_delta);
                return Some(Box::new(event));
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
        event.set_handle_us(recv_delta);
        Some(Box::new(event))
    }

    pub fn parse_nonce_account_event(
        account: &AccountPretty,
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
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
                    event.set_handle_us(elapsed_micros_since(account.recv_us));
                    return Some(Box::new(event));
                }
                solana_account_decoder::parse_nonce::UiNonceState::Uninitialized => {}
            }
        }
        None
    }
}
