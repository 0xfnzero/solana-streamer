use solana_sdk::transaction::VersionedTransaction;

/// 携带槽位信息的交易
#[derive(Debug, Clone, Default)]
pub struct TransactionWithSlot {
    pub transaction: VersionedTransaction,
    pub slot: u64,
    pub recv_us: i64,
    /// 交易在 entry 内的索引（shredstream 无 slot 级 index 时用作 best-effort）
    pub tx_index: Option<u64>,
}

impl TransactionWithSlot {
    /// 创建新的带槽位的交易
    pub fn new(
        transaction: VersionedTransaction,
        slot: u64,
        recv_us: i64,
        tx_index: Option<u64>,
    ) -> Self {
        Self { transaction, slot, recv_us, tx_index }
    }
}
