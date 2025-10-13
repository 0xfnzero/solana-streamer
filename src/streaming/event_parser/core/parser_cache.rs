//! # 事件解析器配置缓存模块
//!
//! 本模块统一管理所有事件解析器的配置和缓存，包括：
//! - 指令事件解析器（Instruction Event Parser）
//! - 账户事件解析器（Account Event Parser）
//! - 高性能缓存工具
//!
//! ## 设计目标
//! - **统一配置管理**：所有协议的解析器配置集中管理
//! - **高性能缓存**：避免重复初始化和内存分配
//! - **易于扩展**：添加新协议只需修改配置映射

use crate::streaming::{
    event_parser::{
        common::{filter::EventTypeFilter, EventMetadata, EventType, ProtocolType},
        protocols::{
            bonk::parser::BONK_PROGRAM_ID,
            pumpfun::parser::PUMPFUN_PROGRAM_ID,
            pumpswap::parser::PUMPSWAP_PROGRAM_ID,
            raydium_amm_v4::parser::RAYDIUM_AMM_V4_PROGRAM_ID,
            raydium_clmm::parser::RAYDIUM_CLMM_PROGRAM_ID,
            raydium_cpmm::parser::RAYDIUM_CPMM_PROGRAM_ID,
        },
        Protocol, DexEvent,
    },
    grpc::AccountPretty,
};
use solana_sdk::pubkey::Pubkey;
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, OnceLock},
};

// ============================================================================
// 第一部分：指令事件解析器配置（Instruction Event Parser）
// ============================================================================

/// 内联指令事件解析器函数类型
///
/// 用于解析内联指令（Inner Instruction）生成的事件
pub type InnerInstructionEventParser =
    fn(data: &[u8], metadata: EventMetadata) -> Option<DexEvent>;

/// 指令事件解析器函数类型
///
/// 用于解析普通指令（Instruction）生成的事件，需要账户信息
pub type InstructionEventParser =
    fn(data: &[u8], accounts: &[Pubkey], metadata: EventMetadata) -> Option<DexEvent>;

/// 指令事件解析器配置
///
/// 定义如何解析特定协议的指令事件
#[derive(Debug, Clone)]
pub struct GenericEventParseConfig {
    /// 程序ID（Program ID）
    pub program_id: Pubkey,
    /// 协议类型
    pub protocol_type: ProtocolType,
    /// 内联指令判别器（8字节）
    pub inner_instruction_discriminator: &'static [u8],
    /// 普通指令判别器（8字节）
    pub instruction_discriminator: &'static [u8],
    /// 事件类型
    pub event_type: EventType,
    /// 内联指令解析器
    pub inner_instruction_parser: Option<InnerInstructionEventParser>,
    /// 普通指令解析器
    pub instruction_parser: Option<InstructionEventParser>,
    /// 是否需要内联指令数据
    pub requires_inner_instruction: bool,
}

/// 全局指令事件解析器注册表
///
/// 存储所有协议的指令解析器配置，启动时初始化一次
pub static EVENT_PARSERS: LazyLock<HashMap<Protocol, (Pubkey, &[GenericEventParseConfig])>> =
    LazyLock::new(|| {
        let mut parsers = HashMap::with_capacity(6);

        // 注册各协议的解析器配置
        parsers.insert(
            Protocol::PumpSwap,
            (
                PUMPSWAP_PROGRAM_ID,
                crate::streaming::event_parser::protocols::pumpswap::parser::CONFIGS,
            ),
        );
        parsers.insert(
            Protocol::PumpFun,
            (
                PUMPFUN_PROGRAM_ID,
                crate::streaming::event_parser::protocols::pumpfun::parser::CONFIGS,
            ),
        );
        parsers.insert(
            Protocol::Bonk,
            (BONK_PROGRAM_ID, crate::streaming::event_parser::protocols::bonk::parser::CONFIGS),
        );
        parsers.insert(
            Protocol::RaydiumCpmm,
            (
                RAYDIUM_CPMM_PROGRAM_ID,
                crate::streaming::event_parser::protocols::raydium_cpmm::parser::CONFIGS,
            ),
        );
        parsers.insert(
            Protocol::RaydiumClmm,
            (
                RAYDIUM_CLMM_PROGRAM_ID,
                crate::streaming::event_parser::protocols::raydium_clmm::parser::CONFIGS,
            ),
        );
        parsers.insert(
            Protocol::RaydiumAmmV4,
            (
                RAYDIUM_AMM_V4_PROGRAM_ID,
                crate::streaming::event_parser::protocols::raydium_amm_v4::parser::CONFIGS,
            ),
        );

        parsers
    });

// ----------------------------------------------------------------------------
// 指令解析器缓存工具
// ----------------------------------------------------------------------------

/// 缓存键：用于标识不同的协议和过滤器组合
///
/// 通过对协议和事件类型排序，确保相同组合生成相同的哈希键
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    /// 协议列表（已排序）
    pub protocols: Vec<Protocol>,
    /// 事件类型过滤器（可选，已排序）
    pub event_types: Option<Vec<EventType>>,
}

impl CacheKey {
    /// 创建新的缓存键
    ///
    /// # 参数
    /// - `protocols`: 协议列表
    /// - `filter`: 事件类型过滤器
    ///
    /// # 优化
    /// 使用 `sort_by_cached_key` 避免重复调用 `format!`
    pub fn new(mut protocols: Vec<Protocol>, filter: Option<&EventTypeFilter>) -> Self {
        // 排序协议列表，确保相同协议组合生成相同的key
        protocols.sort_by_cached_key(|p| format!("{:?}", p));

        let event_types = filter.map(|f| {
            let mut types = f.include.clone();
            types.sort_by_cached_key(|t| format!("{:?}", t));
            types
        });

        Self { protocols, event_types }
    }
}

/// 全局程序ID缓存（使用读写锁保护）
static GLOBAL_PROGRAM_IDS_CACHE: LazyLock<
    parking_lot::RwLock<HashMap<CacheKey, Arc<Vec<Pubkey>>>>,
> = LazyLock::new(|| parking_lot::RwLock::new(HashMap::new()));

/// 全局指令配置缓存（使用读写锁保护）
static GLOBAL_INSTRUCTION_CONFIGS_CACHE: LazyLock<
    parking_lot::RwLock<HashMap<CacheKey, Arc<HashMap<Vec<u8>, Vec<GenericEventParseConfig>>>>>,
> = LazyLock::new(|| parking_lot::RwLock::new(HashMap::new()));

/// 获取指定协议的程序ID列表
///
/// # 参数
/// - `protocols`: 协议列表
/// - `filter`: 事件类型过滤器（可选）
///
/// # 返回
/// Arc包装的程序ID向量，支持零成本共享
///
/// # 缓存策略
/// - 快速路径：从缓存读取（读锁）
/// - 慢速路径：构建并缓存（写锁）
pub fn get_global_program_ids(
    protocols: &[Protocol],
    filter: Option<&EventTypeFilter>,
) -> Arc<Vec<Pubkey>> {
    let cache_key = CacheKey::new(protocols.to_vec(), filter);

    // 快速路径：尝试读取缓存
    {
        let cache = GLOBAL_PROGRAM_IDS_CACHE.read();
        if let Some(program_ids) = cache.get(&cache_key) {
            return program_ids.clone();
        }
    }

    // 慢速路径：构建并缓存
    let mut program_ids = Vec::with_capacity(protocols.len());
    for protocol in protocols {
        if let Some(parse) = EVENT_PARSERS.get(protocol) {
            program_ids.push(parse.0);
        }
    }
    let program_ids = Arc::new(program_ids);

    // 缓存结果（写锁）
    GLOBAL_PROGRAM_IDS_CACHE.write().insert(cache_key, program_ids.clone());

    program_ids
}

/// 获取指定协议的指令配置
///
/// # 参数
/// - `protocols`: 协议列表
/// - `filter`: 事件类型过滤器（可选）
///
/// # 返回
/// Arc包装的指令配置映射表，按判别器（Discriminator）索引
///
/// # 缓存策略
/// - 快速路径：从缓存读取（读锁）
/// - 慢速路径：构建并缓存（写锁）
pub fn get_global_instruction_configs(
    protocols: &[Protocol],
    filter: Option<&EventTypeFilter>,
) -> Arc<HashMap<Vec<u8>, Vec<GenericEventParseConfig>>> {
    let cache_key = CacheKey::new(protocols.to_vec(), filter);

    // 快速路径：尝试读取缓存
    {
        let cache = GLOBAL_INSTRUCTION_CONFIGS_CACHE.read();
        if let Some(configs) = cache.get(&cache_key) {
            return configs.clone();
        }
    }

    // 慢速路径：构建并缓存
    let mut instruction_configs = HashMap::with_capacity(protocols.len() * 4);
    for protocol in protocols {
        if let Some(parse) = EVENT_PARSERS.get(protocol) {
            parse
                .1
                .iter()
                .filter(|config| {
                    filter.as_ref().map(|f| f.include.contains(&config.event_type)).unwrap_or(true)
                })
                .for_each(|config| {
                    instruction_configs
                        .entry(config.instruction_discriminator.to_vec())
                        .or_insert_with(Vec::new)
                        .push(config.clone());
                });
        }
    }
    let instruction_configs = Arc::new(instruction_configs);

    // 缓存结果（写锁）
    GLOBAL_INSTRUCTION_CONFIGS_CACHE.write().insert(cache_key, instruction_configs.clone());

    instruction_configs
}

// ============================================================================
// 第二部分：账户公钥缓存工具（Account Pubkey Cache）
// ============================================================================

/// 高性能账户公钥缓存
///
/// 通过重用内存避免重复Vec分配，提升性能
#[derive(Debug)]
pub struct AccountPubkeyCache {
    /// 预分配的账户公钥向量
    cache: Vec<Pubkey>,
}

impl AccountPubkeyCache {
    /// 创建新的账户公钥缓存
    ///
    /// 预分配32个位置，覆盖大多数交易场景
    pub fn new() -> Self {
        Self {
            cache: Vec::with_capacity(32),
        }
    }

    /// 从指令账户索引构建账户公钥向量
    ///
    /// # 参数
    /// - `instruction_accounts`: 指令账户索引列表
    /// - `all_accounts`: 所有账户公钥列表
    ///
    /// # 返回
    /// 账户公钥切片引用
    ///
    /// # 性能优化
    /// - 重用内部缓存，避免重新分配
    /// - 仅在必要时扩容
    #[inline]
    pub fn build_account_pubkeys(
        &mut self,
        instruction_accounts: &[u8],
        all_accounts: &[Pubkey],
    ) -> &[Pubkey] {
        self.cache.clear();

        // 确保容量足够，避免动态扩容
        if self.cache.capacity() < instruction_accounts.len() {
            self.cache.reserve(instruction_accounts.len() - self.cache.capacity());
        }

        // 快速填充账户公钥（带边界检查）
        for &idx in instruction_accounts.iter() {
            if (idx as usize) < all_accounts.len() {
                self.cache.push(all_accounts[idx as usize]);
            }
        }

        &self.cache
    }
}

impl Default for AccountPubkeyCache {
    fn default() -> Self {
        Self::new()
    }
}

thread_local! {
    static THREAD_LOCAL_ACCOUNT_CACHE: std::cell::RefCell<AccountPubkeyCache> =
        std::cell::RefCell::new(AccountPubkeyCache::new());
}

/// 从线程局部缓存构建账户公钥列表
///
/// # 参数
/// - `instruction_accounts`: 指令账户索引列表
/// - `all_accounts`: 所有账户公钥列表
///
/// # 返回
/// 账户公钥向量
///
/// # 线程安全
/// 使用线程局部存储，每个线程独立缓存
#[inline]
pub fn build_account_pubkeys_with_cache(
    instruction_accounts: &[u8],
    all_accounts: &[Pubkey],
) -> Vec<Pubkey> {
    THREAD_LOCAL_ACCOUNT_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache.build_account_pubkeys(instruction_accounts, all_accounts).to_vec()
    })
}

// ============================================================================
// 第三部分：账户事件解析器配置（Account Event Parser）
// ============================================================================

/// 账户事件解析器函数类型
///
/// 用于解析账户状态变更生成的事件
pub type AccountEventParserFn =
    fn(account: &AccountPretty, metadata: EventMetadata) -> Option<DexEvent>;

/// 账户事件解析器配置
///
/// 定义如何解析特定协议的账户事件
#[derive(Debug, Clone)]
pub struct AccountEventParseConfig {
    /// 程序ID（Program ID）
    pub program_id: Pubkey,
    /// 协议类型
    pub protocol_type: ProtocolType,
    /// 事件类型
    pub event_type: EventType,
    /// 账户判别器（Account Discriminator）
    pub account_discriminator: &'static [u8],
    /// 账户解析器函数
    pub account_parser: AccountEventParserFn,
}

/// 协议账户配置缓存
///
/// 存储各协议的账户解析器配置
static PROTOCOL_CONFIGS_CACHE: OnceLock<HashMap<Protocol, Vec<AccountEventParseConfig>>> =
    OnceLock::new();

/// Nonce 账户配置缓存
static NONCE_CONFIG: OnceLock<AccountEventParseConfig> = OnceLock::new();

/// 通用 Token 账户配置缓存
static COMMON_CONFIG: OnceLock<AccountEventParseConfig> = OnceLock::new();

/// 获取账户解析器配置列表
///
/// # 参数
/// - `protocols`: 协议列表
/// - `event_type_filter`: 事件类型过滤器（可选）
/// - `nonce_parser`: Nonce账户解析器
/// - `token_parser`: Token账户解析器
///
/// # 返回
/// 账户解析器配置向量
///
/// # 配置组成
/// 1. 协议特定配置（根据protocols参数）
/// 2. Nonce账户配置（通用）
/// 3. Token账户配置（通用，作为兜底）
///
/// # 示例
/// ```ignore
/// let configs = get_account_configs(
///     &[Protocol::PumpFun, Protocol::Bonk],
///     None,
///     parse_nonce_account,
///     parse_token_account,
/// );
/// ```
pub fn get_account_configs(
    protocols: &[Protocol],
    event_type_filter: Option<&EventTypeFilter>,
    nonce_parser: AccountEventParserFn,
    token_parser: AccountEventParserFn,
) -> Vec<AccountEventParseConfig> {
    // 初始化协议配置缓存（仅在首次调用时执行）
    let protocols_map = PROTOCOL_CONFIGS_CACHE.get_or_init(|| {
        let mut map = HashMap::new();

        // PumpSwap 协议
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

        // PumpFun 协议
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

        // Bonk 协议
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

        // Raydium CPMM 协议
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

        // Raydium CLMM 协议
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

        // Raydium AMM V4 协议
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

    // 构建配置列表
    let mut configs = Vec::new();
    let empty_vec = Vec::new();

    // 预估容量（大多数协议有2-3个配置）
    let estimated_capacity = protocols.len() * 3;
    configs.reserve(estimated_capacity);

    // 1. 添加协议特定配置
    for protocol in protocols {
        let protocol_configs = protocols_map.get(protocol).unwrap_or(&empty_vec);

        if event_type_filter.is_none() {
            // 无过滤器：添加所有配置
            configs.extend(protocol_configs.iter().cloned());
        } else {
            // 有过滤器：仅添加匹配的配置
            let filter = event_type_filter.unwrap();
            configs.extend(
                protocol_configs
                    .iter()
                    .filter(|config| filter.include.contains(&config.event_type))
                    .cloned(),
            );
        }
    }

    // 2. 添加 Nonce 账户配置（通用配置）
    if event_type_filter.is_none()
        || event_type_filter.unwrap().include.contains(&EventType::NonceAccount)
    {
        let nonce_config = NONCE_CONFIG.get_or_init(|| AccountEventParseConfig {
            program_id: Pubkey::default(),
            protocol_type: ProtocolType::Common,
            event_type: EventType::NonceAccount,
            account_discriminator: &[1, 0, 0, 0, 1, 0, 0, 0],
            account_parser: nonce_parser,
        });
        configs.push(nonce_config.clone());
    }

    // 3. 添加通用 Token 账户配置（兜底配置，始终添加）
    let common_config = COMMON_CONFIG.get_or_init(|| AccountEventParseConfig {
        program_id: Pubkey::default(),
        protocol_type: ProtocolType::Common,
        event_type: EventType::TokenAccount,
        account_discriminator: &[],
        account_parser: token_parser,
    });
    configs.push(common_config.clone());

    configs
}
