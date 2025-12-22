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

/// Generic object pool trait
pub trait ObjectPool<T> {
    fn acquire(&self) -> PooledObject<T>;
    fn return_object(&self, obj: Box<T>);
}

/// Smart pointer with automatic return
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
            // Discard when exceeding max capacity
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

/// AccountPretty object pool
pub struct AccountPrettyPool {
    pool: Arc<Mutex<VecDeque<Box<AccountPretty>>>>,
    max_size: usize,
}

impl AccountPrettyPool {
    pub fn new(initial_size: usize, max_size: usize) -> Self {
        let mut pool = VecDeque::with_capacity(initial_size);

        // Pre-allocate objects
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

/// AccountPretty with automatic return
pub struct PooledAccountPretty {
    account: Box<AccountPretty>,
    pool: Arc<Mutex<VecDeque<Box<AccountPretty>>>>,
    max_size: usize,
}

impl PooledAccountPretty {
    /// Reset data from gRPC update
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

        // Optimize data field reuse
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
            // Clear sensitive data
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

/// BlockMetaPretty object pool
pub struct BlockMetaPrettyPool {
    pool: Arc<Mutex<VecDeque<Box<BlockMetaPretty>>>>,
    max_size: usize,
}

impl BlockMetaPrettyPool {
    pub fn new(initial_size: usize, max_size: usize) -> Self {
        let mut pool = VecDeque::with_capacity(initial_size);

        // Pre-allocate objects
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

/// BlockMetaPretty with automatic return
pub struct PooledBlockMetaPretty {
    block_meta: Box<BlockMetaPretty>,
    pool: Arc<Mutex<VecDeque<Box<BlockMetaPretty>>>>,
    max_size: usize,
}

impl PooledBlockMetaPretty {
    /// Reset data from gRPC update
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
            // Clear data
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

/// TransactionPretty object pool
pub struct TransactionPrettyPool {
    pool: Arc<Mutex<VecDeque<Box<TransactionPretty>>>>,
    max_size: usize,
}

impl TransactionPrettyPool {
    pub fn new(initial_size: usize, max_size: usize) -> Self {
        let mut pool = VecDeque::with_capacity(initial_size);

        // Pre-allocate objects
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

/// TransactionPretty with automatic return
pub struct PooledTransactionPretty {
    transaction: Box<TransactionPretty>,
    pool: Arc<Mutex<VecDeque<Box<TransactionPretty>>>>,
    max_size: usize,
}

impl PooledTransactionPretty {
    /// Reset data from gRPC update
    pub fn reset_from_update(
        &mut self,
        tx_update: SubscribeUpdateTransaction,
        block_time: Option<Timestamp>,
    ) {
        let tx = tx_update.transaction.expect("should be defined");

        self.transaction.slot = tx_update.slot;
        self.transaction.transaction_index = Some(tx.index);
        self.transaction.block_time = block_time;
        self.transaction.block_hash.clear(); // Reset block_hash
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
            // Clear data
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

/// EventPretty object pool (composite pool)
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

    /// Get account event object
    pub fn acquire_account(&self) -> PooledAccountPretty {
        self.account_pool.acquire()
    }

    /// Get block event object
    pub fn acquire_block(&self) -> PooledBlockMetaPretty {
        self.block_pool.acquire()
    }

    /// Get transaction event object
    pub fn acquire_transaction(&self) -> PooledTransactionPretty {
        self.transaction_pool.acquire()
    }
}

/// Object pool manager (singleton)
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

/// Factory functions for creating optimized EventPretty
impl EventPrettyPool {
    /// Create account event - optimized with object pool
    pub fn create_account_event_optimized(&self, update: SubscribeUpdateAccount) -> AccountPretty {
        let mut pooled_account = self.acquire_account();
        pooled_account.reset_from_update(update);
        // Move data instead of cloning to avoid unnecessary memory allocation
        let result = std::mem::replace(pooled_account.deref_mut(), AccountPretty::default());
        result
    }

    /// Create block event - optimized with object pool
    pub fn create_block_event_optimized(
        &self,
        update: SubscribeUpdateBlockMeta,
        block_time: Option<Timestamp>,
    ) -> BlockMetaPretty {
        let mut pooled_block = self.acquire_block();
        pooled_block.reset_from_update(update, block_time);
        // Move data instead of cloning
        let result = std::mem::replace(pooled_block.deref_mut(), BlockMetaPretty::default());
        result
    }

    /// Create transaction event - optimized with object pool
    pub fn create_transaction_event_optimized(
        &self,
        update: SubscribeUpdateTransaction,
        block_time: Option<Timestamp>,
    ) -> TransactionPretty {
        let mut pooled_tx = self.acquire_transaction();
        pooled_tx.reset_from_update(update, block_time);
        // Move data instead of cloning
        let result = std::mem::replace(pooled_tx.deref_mut(), TransactionPretty::default());
        result
    }
}

// Global pool manager instance
lazy_static::lazy_static! {
    pub static ref GLOBAL_POOL_MANAGER: PoolManager = PoolManager::new();
}

/// Convenient global factory functions
pub mod factory {
    use super::*;

    /// Create account event using object pool (recommended for high-performance scenarios)
    pub fn create_account_pretty_pooled(update: SubscribeUpdateAccount) -> AccountPretty {
        GLOBAL_POOL_MANAGER.get_event_pool().create_account_event_optimized(update)
    }

    /// Create block event using object pool (recommended for high-performance scenarios)
    pub fn create_block_meta_pretty_pooled(
        update: SubscribeUpdateBlockMeta,
        block_time: Option<Timestamp>,
    ) -> BlockMetaPretty {
        GLOBAL_POOL_MANAGER.get_event_pool().create_block_event_optimized(update, block_time)
    }

    /// Create transaction event using object pool (recommended for high-performance scenarios)
    pub fn create_transaction_pretty_pooled(
        update: SubscribeUpdateTransaction,
        block_time: Option<Timestamp>,
    ) -> TransactionPretty {
        GLOBAL_POOL_MANAGER.get_event_pool().create_transaction_event_optimized(update, block_time)
    }
}
