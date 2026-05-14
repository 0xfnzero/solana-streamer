use solana_sdk::transaction::VersionedTransaction;
use std::collections::VecDeque;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};

use super::TransactionWithSlot;

/// TransactionWithSlot 对象池
pub struct TransactionWithSlotPool {
    pool: Arc<Mutex<VecDeque<Box<TransactionWithSlot>>>>,
    max_size: usize,
}

impl TransactionWithSlotPool {
    pub fn new(initial_size: usize, max_size: usize) -> Self {
        let mut pool = VecDeque::with_capacity(initial_size);

        // 预分配对象
        for _ in 0..initial_size {
            pool.push_back(Box::new(TransactionWithSlot::default()));
        }

        Self { pool: Arc::new(Mutex::new(pool)), max_size }
    }

    pub fn acquire(&self) -> PooledTransactionWithSlot {
        let mut pool = self.pool.lock().unwrap();
        let transaction = match pool.pop_front() {
            Some(reused) => reused,
            None => Box::new(TransactionWithSlot::default()),
        };

        PooledTransactionWithSlot {
            transaction,
            pool: Arc::clone(&self.pool),
            max_size: self.max_size,
        }
    }
}

/// 带自动归还的 TransactionWithSlot
pub struct PooledTransactionWithSlot {
    transaction: Box<TransactionWithSlot>,
    pool: Arc<Mutex<VecDeque<Box<TransactionWithSlot>>>>,
    max_size: usize,
}

impl PooledTransactionWithSlot {
    /// 从原始数据重置
    pub fn reset_from_data(
        &mut self,
        transaction: VersionedTransaction,
        slot: u64,
        recv_us: i64,
        tx_index: Option<u64>,
    ) {
        self.transaction.transaction = transaction;
        self.transaction.slot = slot;
        self.transaction.recv_us = recv_us;
        self.transaction.tx_index = tx_index;
    }

    /// 使用优化的工厂方法创建 TransactionWithSlot（移动数据而不是克隆）
    pub fn into_transaction_with_slot(mut self) -> TransactionWithSlot {
        // 移动数据而不是克隆，避免多余的内存分配
        std::mem::replace(self.deref_mut(), TransactionWithSlot::default())
    }
}

impl Drop for PooledTransactionWithSlot {
    fn drop(&mut self) {
        let mut pool = self.pool.lock().unwrap();
        if pool.len() < self.max_size {
            // 清理敏感数据
            self.transaction.slot = 0;
            self.transaction.recv_us = 0;
            self.transaction.tx_index = None;
            // 重置交易为默认值以清理敏感数据
            self.transaction.transaction = VersionedTransaction::default();
            pool.push_back(std::mem::take(&mut self.transaction));
        }
    }
}

impl std::ops::Deref for PooledTransactionWithSlot {
    type Target = TransactionWithSlot;

    fn deref(&self) -> &Self::Target {
        &self.transaction
    }
}

impl std::ops::DerefMut for PooledTransactionWithSlot {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transaction
    }
}

/// Shred 对象池管理器
pub struct ShredPoolManager {
    transaction_pool: TransactionWithSlotPool,
}

impl ShredPoolManager {
    pub fn new() -> Self {
        Self {
            transaction_pool: TransactionWithSlotPool::new(
                5000,  // 初始大小 - Shred 事件通常较多
                15000, // 最大大小
            ),
        }
    }

    pub fn get_transaction_pool(&self) -> &TransactionWithSlotPool {
        &self.transaction_pool
    }

    /// 创建优化的 TransactionWithSlot
    pub fn create_transaction_with_slot_optimized(
        &self,
        transaction: VersionedTransaction,
        slot: u64,
        recv_us: i64,
        tx_index: Option<u64>,
    ) -> TransactionWithSlot {
        let mut pooled_tx = self.transaction_pool.acquire();
        pooled_tx.reset_from_data(transaction, slot, recv_us, tx_index);
        pooled_tx.into_transaction_with_slot()
    }
}

impl Default for ShredPoolManager {
    fn default() -> Self {
        Self::new()
    }
}

// 全局 Shred 池管理器实例
pub static GLOBAL_SHRED_POOL_MANAGER: std::sync::LazyLock<ShredPoolManager> =
    std::sync::LazyLock::new(ShredPoolManager::new);

/// 便捷的全局工厂函数
pub mod factory {
    use super::*;

    /// 使用对象池创建 TransactionWithSlot（推荐用于高性能场景）
    pub fn create_transaction_with_slot_pooled(
        transaction: VersionedTransaction,
        slot: u64,
        recv_us: i64,
        tx_index: Option<u64>,
    ) -> TransactionWithSlot {
        GLOBAL_SHRED_POOL_MANAGER.create_transaction_with_slot_optimized(
            transaction,
            slot,
            recv_us,
            tx_index,
        )
    }
}
