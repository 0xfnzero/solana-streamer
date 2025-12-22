use solana_sdk::transaction::VersionedTransaction;

/// Transaction with slot information
#[derive(Debug, Clone, Default)]
pub struct TransactionWithSlot {
    pub transaction: VersionedTransaction,
    pub slot: u64,
    pub recv_us: i64,
}

impl TransactionWithSlot {
    /// Create new transaction with slot
    pub fn new(
        transaction: VersionedTransaction,
        slot: u64,
        recv_us: i64,
    ) -> Self {
        Self { transaction, slot, recv_us }
    }
}
