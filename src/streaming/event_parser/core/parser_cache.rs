//! # 事件解析器配置缓存模块
//!
//! 本模块管理事件解析器的缓存，包括：
//! - 程序ID缓存
//! - 账户事件解析器（Account Event Parser）
//! - 高性能缓存工具
//!
//! ## 设计目标
//! - **高性能缓存**：避免重复初始化和内存分配
//! - **易于扩展**：通过 dispatcher 动态派发

use crate::streaming::{
    event_parser::{
        common::{filter::EventTypeFilter, EventMetadata, EventType, ProtocolType},
        core::dispatcher::EventDispatcher,
        DexEvent, Protocol,
    },
    grpc::AccountPretty,
};
use solana_sdk::pubkey::Pubkey;
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

// ============================================================================
// 第一部分：程序ID缓存
// ============================================================================

/// 缓存键：用于标识不同的协议和过滤器组合
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    /// 协议列表（已排序）
    pub protocols: Vec<Protocol>,
    /// 事件类型过滤器（可选，已排序）
    pub event_types: Option<Vec<EventType>>,
}

impl CacheKey {
    /// 创建新的缓存键
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
static GLOBAL_PROGRAM_IDS_CACHE: LazyLock<std::sync::RwLock<HashMap<CacheKey, Arc<Vec<Pubkey>>>>> =
    LazyLock::new(|| std::sync::RwLock::new(HashMap::new()));

/// 获取指定协议的程序ID列表
///
/// 使用 EventDispatcher 获取 program_ids，并缓存结果
pub fn get_global_program_ids(
    protocols: &[Protocol],
    filter: Option<&EventTypeFilter>,
) -> Arc<Vec<Pubkey>> {
    let cache_key = CacheKey::new(protocols.to_vec(), filter);

    // 快速路径：尝试读取缓存
    {
        let cache = GLOBAL_PROGRAM_IDS_CACHE.read().unwrap();
        if let Some(program_ids) = cache.get(&cache_key) {
            return program_ids.clone();
        }
    }

    // 慢速路径：通过 EventDispatcher 获取并缓存
    let program_ids = Arc::new(EventDispatcher::get_program_ids(protocols));

    // 缓存结果（写锁）
    GLOBAL_PROGRAM_IDS_CACHE.write().unwrap().insert(cache_key, program_ids.clone());

    program_ids
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
        Self { cache: Vec::with_capacity(32) }
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
