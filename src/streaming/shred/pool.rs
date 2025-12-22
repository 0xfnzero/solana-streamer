use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::ops::DerefMut;
use solana_sdk::transaction::VersionedTransaction;

use super::TransactionWithSlot;


/// TransactionWithSlot object pool
pub struct TransactionWithSlotPool {
    pool: Arc<Mutex<VecDeque<Box<TransactionWithSlot>>>>,
    max_size: usize,
}

impl TransactionWithSlotPool {
    pub fn new(initial_size: usize, max_size: usize) -> Self {
        let mut pool = VecDeque::with_capacity(initial_size);

        // Pre-allocate objects
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
            max_size: self.max_size 
        }
    }
}

/// TransactionWithSlot with automatic return
pub struct PooledTransactionWithSlot {
    transaction: Box<TransactionWithSlot>,
    pool: Arc<Mutex<VecDeque<Box<TransactionWithSlot>>>>,
    max_size: usize,
}

impl PooledTransactionWithSlot {
    /// Reset from raw data
    pub fn reset_from_data(
        &mut self, 
        transaction: VersionedTransaction, 
        slot: u64, 
        recv_us: i64
    ) {
        self.transaction.transaction = transaction;
        self.transaction.slot = slot;
        self.transaction.recv_us = recv_us;
    }

    /// Create TransactionWithSlot using optimized factory method (move data instead of cloning)
    pub fn into_transaction_with_slot(mut self) -> TransactionWithSlot {
        // Move data instead of cloning to avoid unnecessary memory allocation
        std::mem::replace(self.deref_mut(), TransactionWithSlot::default())
    }
}

impl Drop for PooledTransactionWithSlot {
    fn drop(&mut self) {
        let mut pool = self.pool.lock().unwrap();
        if pool.len() < self.max_size {
            // Clear sensitive data
            self.transaction.slot = 0;
            self.transaction.recv_us = 0;
            // Reset transaction to default to clear sensitive data
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

/// Shred object pool manager
pub struct ShredPoolManager {
    transaction_pool: TransactionWithSlotPool,
}

impl ShredPoolManager {
    pub fn new() -> Self {
        Self {
            transaction_pool: TransactionWithSlotPool::new(
                5000,  // Initial size - Shred events are usually numerous
                15000, // Max size
            ),
        }
    }

    pub fn get_transaction_pool(&self) -> &TransactionWithSlotPool {
        &self.transaction_pool
    }

    /// Create optimized TransactionWithSlot
    pub fn create_transaction_with_slot_optimized(
        &self,
        transaction: VersionedTransaction,
        slot: u64,
        recv_us: i64,
    ) -> TransactionWithSlot {
        let mut pooled_tx = self.transaction_pool.acquire();
        pooled_tx.reset_from_data(transaction, slot, recv_us);
        pooled_tx.into_transaction_with_slot()
    }
}

impl Default for ShredPoolManager {
    fn default() -> Self {
        Self::new()
    }
}

// Global Shred pool manager instance
lazy_static::lazy_static! {
    pub static ref GLOBAL_SHRED_POOL_MANAGER: ShredPoolManager = ShredPoolManager::new();
}

/// Convenient global factory functions
pub mod factory {
    use super::*;

    /// Create TransactionWithSlot using object pool (recommended for high-performance scenarios)
    pub fn create_transaction_with_slot_pooled(
        transaction: VersionedTransaction,
        slot: u64,
        recv_us: i64,
    ) -> TransactionWithSlot {
        GLOBAL_SHRED_POOL_MANAGER.create_transaction_with_slot_optimized(
            transaction, 
            slot, 
            recv_us
        )
    }
}
