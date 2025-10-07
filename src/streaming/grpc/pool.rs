use super::types::{AccountPretty, BlockMetaPretty, TransactionPretty};
use crate::streaming::event_parser::common::high_performance_clock::get_high_perf_clock;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::collections::VecDeque;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use yellowstone_grpc_proto::{
    geyser::{SubscribeUpdateAccount, SubscribeUpdateBlockMeta, SubscribeUpdateTransaction},
    prost_types::Timestamp,
};

/// 通用对象池特征
pub trait ObjectPool<T> {
    fn acquire(&self) -> PooledObject<T>;
    fn return_object(&self, obj: Box<T>);
}

/// 带自动归还的智能指针
pub struct PooledObject<T> {
    object: Option<Box<T>>,
    pool: Arc<Mutex<VecDeque<Box<T>>>>,
    max_size: usize,
}

impl<T> PooledObject<T> {
    #[allow(dead_code)]
    fn new(object: Box<T>, pool: Arc<Mutex<VecDeque<Box<T>>>>, max_size: usize) -> Self {
        Self { object: Some(object), pool, max_size }
    }
}

impl<T> Drop for PooledObject<T> {
    fn drop(&mut self) {
        if let Some(obj) = self.object.take() {
            let mut pool = self.pool.lock().unwrap();
            if pool.len() < self.max_size {
                pool.push_back(obj);
            }
            // 超过最大容量时直接丢弃
        }
    }
}

impl<T> std::ops::Deref for PooledObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.object.as_ref().unwrap()
    }
}

impl<T> std::ops::DerefMut for PooledObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.object.as_mut().unwrap()
    }
}

/// AccountPretty 对象池
pub struct AccountPrettyPool {
    pool: Arc<Mutex<VecDeque<Box<AccountPretty>>>>,
    max_size: usize,
}

impl AccountPrettyPool {
    pub fn new(initial_size: usize, max_size: usize) -> Self {
        let mut pool = VecDeque::with_capacity(initial_size);

        // 预分配对象
        for _ in 0..initial_size {
            pool.push_back(Box::new(AccountPretty::default()));
        }

        Self { pool: Arc::new(Mutex::new(pool)), max_size }
    }

    pub fn acquire(&self) -> PooledAccountPretty {
        let mut pool = self.pool.lock().unwrap();
        let account = match pool.pop_front() {
            Some(reused) => reused,
            None => Box::new(AccountPretty::default()),
        };

        PooledAccountPretty { account, pool: Arc::clone(&self.pool), max_size: self.max_size }
    }
}

/// 带自动归还的 AccountPretty
pub struct PooledAccountPretty {
    account: Box<AccountPretty>,
    pool: Arc<Mutex<VecDeque<Box<AccountPretty>>>>,
    max_size: usize,
}

impl PooledAccountPretty {
    /// 从 gRPC 更新重置数据
    pub fn reset_from_update(&mut self, account_update: SubscribeUpdateAccount) {
        let account_info = account_update.account.unwrap();

        self.account.slot = account_update.slot;
        self.account.signature = if let Some(txn_signature) = account_info.txn_signature {
            Signature::try_from(txn_signature.as_slice()).expect("valid signature")
        } else {
            Signature::default()
        };
        self.account.pubkey =
            Pubkey::try_from(account_info.pubkey.as_slice()).expect("valid pubkey");
        self.account.executable = account_info.executable;
        self.account.lamports = account_info.lamports;
        self.account.owner = Pubkey::try_from(account_info.owner.as_slice()).expect("valid pubkey");
        self.account.rent_epoch = account_info.rent_epoch;

        // 优化数据字段的重用
        let new_data = account_info.data;
        if self.account.data.capacity() >= new_data.len() {
            self.account.data.clear();
            self.account.data.extend_from_slice(&new_data);
        } else {
            self.account.data = new_data;
        }

        self.account.recv_us = get_high_perf_clock();
    }
}

impl Drop for PooledAccountPretty {
    fn drop(&mut self) {
        let mut pool = self.pool.lock().unwrap();
        if pool.len() < self.max_size {
            // 清理敏感数据
            self.account.data.clear();
            self.account.signature = Signature::default();
            self.account.pubkey = Pubkey::default();
            self.account.owner = Pubkey::default();
            pool.push_back(std::mem::take(&mut self.account));
        }
    }
}

impl std::ops::Deref for PooledAccountPretty {
    type Target = AccountPretty;

    fn deref(&self) -> &Self::Target {
        &self.account
    }
}

impl std::ops::DerefMut for PooledAccountPretty {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.account
    }
}

/// BlockMetaPretty 对象池
pub struct BlockMetaPrettyPool {
    pool: Arc<Mutex<VecDeque<Box<BlockMetaPretty>>>>,
    max_size: usize,
}

impl BlockMetaPrettyPool {
    pub fn new(initial_size: usize, max_size: usize) -> Self {
        let mut pool = VecDeque::with_capacity(initial_size);

        // 预分配对象
        for _ in 0..initial_size {
            pool.push_back(Box::new(BlockMetaPretty::default()));
        }

        Self { pool: Arc::new(Mutex::new(pool)), max_size }
    }

    pub fn acquire(&self) -> PooledBlockMetaPretty {
        let mut pool = self.pool.lock().unwrap();
        let block_meta = match pool.pop_front() {
            Some(reused) => reused,
            None => Box::new(BlockMetaPretty::default()),
        };

        PooledBlockMetaPretty { block_meta, pool: Arc::clone(&self.pool), max_size: self.max_size }
    }
}

/// 带自动归还的 BlockMetaPretty
pub struct PooledBlockMetaPretty {
    block_meta: Box<BlockMetaPretty>,
    pool: Arc<Mutex<VecDeque<Box<BlockMetaPretty>>>>,
    max_size: usize,
}

impl PooledBlockMetaPretty {
    /// 从 gRPC 更新重置数据
    pub fn reset_from_update(
        &mut self,
        block_update: SubscribeUpdateBlockMeta,
        block_time: Option<Timestamp>,
    ) {
        self.block_meta.slot = block_update.slot;
        self.block_meta.block_hash = block_update.blockhash;
        self.block_meta.block_time = block_time;
        self.block_meta.recv_us = get_high_perf_clock();
    }
}

impl Drop for PooledBlockMetaPretty {
    fn drop(&mut self) {
        let mut pool = self.pool.lock().unwrap();
        if pool.len() < self.max_size {
            // 清理数据
            self.block_meta.block_hash.clear();
            self.block_meta.block_time = None;
            pool.push_back(std::mem::take(&mut self.block_meta));
        }
    }
}

impl std::ops::Deref for PooledBlockMetaPretty {
    type Target = BlockMetaPretty;

    fn deref(&self) -> &Self::Target {
        &self.block_meta
    }
}

impl std::ops::DerefMut for PooledBlockMetaPretty {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.block_meta
    }
}

/// TransactionPretty 对象池
pub struct TransactionPrettyPool {
    pool: Arc<Mutex<VecDeque<Box<TransactionPretty>>>>,
    max_size: usize,
}

impl TransactionPrettyPool {
    pub fn new(initial_size: usize, max_size: usize) -> Self {
        let mut pool = VecDeque::with_capacity(initial_size);

        // 预分配对象
        for _ in 0..initial_size {
            pool.push_back(Box::new(TransactionPretty::default()));
        }

        Self { pool: Arc::new(Mutex::new(pool)), max_size }
    }

    pub fn acquire(&self) -> PooledTransactionPretty {
        let mut pool = self.pool.lock().unwrap();
        let transaction = match pool.pop_front() {
            Some(reused) => reused,
            None => Box::new(TransactionPretty::default()),
        };

        PooledTransactionPretty {
            transaction,
            pool: Arc::clone(&self.pool),
            max_size: self.max_size,
        }
    }
}

/// 带自动归还的 TransactionPretty
pub struct PooledTransactionPretty {
    transaction: Box<TransactionPretty>,
    pool: Arc<Mutex<VecDeque<Box<TransactionPretty>>>>,
    max_size: usize,
}

impl PooledTransactionPretty {
    /// 从 gRPC 更新重置数据
    pub fn reset_from_update(
        &mut self,
        tx_update: SubscribeUpdateTransaction,
        block_time: Option<Timestamp>,
    ) {
        let tx = tx_update.transaction.expect("should be defined");

        self.transaction.slot = tx_update.slot;
        self.transaction.transaction_index = Some(tx.index);
        self.transaction.block_time = block_time;
        self.transaction.block_hash.clear(); // 重置 block_hash
        self.transaction.signature =
            Signature::try_from(tx.signature.as_slice()).expect("valid signature");
        self.transaction.is_vote = tx.is_vote;
        self.transaction.recv_us = get_high_perf_clock();
        self.transaction.grpc_tx = tx;
    }
}

impl Drop for PooledTransactionPretty {
    fn drop(&mut self) {
        let mut pool = self.pool.lock().unwrap();
        if pool.len() < self.max_size {
            // 清理数据
            self.transaction.block_hash.clear();
            self.transaction.block_time = None;
            self.transaction.signature = Signature::default();
            pool.push_back(std::mem::take(&mut self.transaction));
        }
    }
}

impl std::ops::Deref for PooledTransactionPretty {
    type Target = TransactionPretty;

    fn deref(&self) -> &Self::Target {
        &self.transaction
    }
}

impl std::ops::DerefMut for PooledTransactionPretty {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transaction
    }
}

/// EventPretty 对象池（组合池）
pub struct EventPrettyPool {
    account_pool: AccountPrettyPool,
    block_pool: BlockMetaPrettyPool,
    transaction_pool: TransactionPrettyPool,
}

impl EventPrettyPool {
    pub fn new() -> Self {
        Self {
            account_pool: AccountPrettyPool::new(10000, 20000),
            block_pool: BlockMetaPrettyPool::new(500, 1000),
            transaction_pool: TransactionPrettyPool::new(10000, 20000),
        }
    }

    /// 获取账户事件对象
    pub fn acquire_account(&self) -> PooledAccountPretty {
        self.account_pool.acquire()
    }

    /// 获取区块事件对象
    pub fn acquire_block(&self) -> PooledBlockMetaPretty {
        self.block_pool.acquire()
    }

    /// 获取交易事件对象
    pub fn acquire_transaction(&self) -> PooledTransactionPretty {
        self.transaction_pool.acquire()
    }
}

/// 对象池管理器（单例）
pub struct PoolManager {
    event_pool: EventPrettyPool,
}

impl PoolManager {
    pub fn new() -> Self {
        Self { event_pool: EventPrettyPool::new() }
    }

    pub fn get_event_pool(&self) -> &EventPrettyPool {
        &self.event_pool
    }
}

impl Default for PoolManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 工厂函数用于创建优化的 EventPretty
impl EventPrettyPool {
    /// 创建账户事件 - 使用对象池优化
    pub fn create_account_event_optimized(&self, update: SubscribeUpdateAccount) -> AccountPretty {
        let mut pooled_account = self.acquire_account();
        pooled_account.reset_from_update(update);
        // 移动数据而不是克隆，避免多余的内存分配
        let result = std::mem::replace(pooled_account.deref_mut(), AccountPretty::default());
        result
    }

    /// 创建区块事件 - 使用对象池优化
    pub fn create_block_event_optimized(
        &self,
        update: SubscribeUpdateBlockMeta,
        block_time: Option<Timestamp>,
    ) -> BlockMetaPretty {
        let mut pooled_block = self.acquire_block();
        pooled_block.reset_from_update(update, block_time);
        // 移动数据而不是克隆
        let result = std::mem::replace(pooled_block.deref_mut(), BlockMetaPretty::default());
        result
    }

    /// 创建交易事件 - 使用对象池优化
    pub fn create_transaction_event_optimized(
        &self,
        update: SubscribeUpdateTransaction,
        block_time: Option<Timestamp>,
    ) -> TransactionPretty {
        let mut pooled_tx = self.acquire_transaction();
        pooled_tx.reset_from_update(update, block_time);
        // 移动数据而不是克隆
        let result = std::mem::replace(pooled_tx.deref_mut(), TransactionPretty::default());
        result
    }
}

// 全局池管理器实例
lazy_static::lazy_static! {
    pub static ref GLOBAL_POOL_MANAGER: PoolManager = PoolManager::new();
}

/// 便捷的全局工厂函数
pub mod factory {
    use super::*;

    /// 使用对象池创建账户事件（推荐用于高性能场景）
    pub fn create_account_pretty_pooled(update: SubscribeUpdateAccount) -> AccountPretty {
        GLOBAL_POOL_MANAGER.get_event_pool().create_account_event_optimized(update)
    }

    /// 使用对象池创建区块事件（推荐用于高性能场景）
    pub fn create_block_meta_pretty_pooled(
        update: SubscribeUpdateBlockMeta,
        block_time: Option<Timestamp>,
    ) -> BlockMetaPretty {
        GLOBAL_POOL_MANAGER.get_event_pool().create_block_event_optimized(update, block_time)
    }

    /// 使用对象池创建交易事件（推荐用于高性能场景）
    pub fn create_transaction_pretty_pooled(
        update: SubscribeUpdateTransaction,
        block_time: Option<Timestamp>,
    ) -> TransactionPretty {
        GLOBAL_POOL_MANAGER.get_event_pool().create_transaction_event_optimized(update, block_time)
    }
}
