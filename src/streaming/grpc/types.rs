use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::{collections::HashMap, fmt};
use yellowstone_grpc_proto::{
    geyser::{
        SubscribeRequestFilterAccounts, SubscribeRequestFilterTransactions,
        SubscribeUpdateTransactionInfo,
    },
    prost_types::Timestamp,
};

pub type TransactionsFilterMap = HashMap<String, SubscribeRequestFilterTransactions>;
pub type AccountsFilterMap = HashMap<String, SubscribeRequestFilterAccounts>;

#[derive(Clone, Debug)]
pub enum EventPretty {
    BlockMeta(BlockMetaPretty),
    Transaction(TransactionPretty),
    Account(AccountPretty),
}

#[derive(Clone, Default)]
pub struct AccountPretty {
    pub slot: u64,
    pub signature: Signature,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub data: Vec<u8>,
    pub recv_us: i64,
}

impl fmt::Debug for AccountPretty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AccountPretty")
            .field("slot", &self.slot)
            .field("signature", &self.signature)
            .field("pubkey", &self.pubkey)
            .field("executable", &self.executable)
            .field("lamports", &self.lamports)
            .field("owner", &self.owner)
            .field("rent_epoch", &self.rent_epoch)
            .field("data", &self.data)
            .finish()
    }
}

#[derive(Clone, Default)]
pub struct BlockMetaPretty {
    pub slot: u64,
    pub block_hash: String,
    pub block_time: Option<Timestamp>,
    pub recv_us: i64,
}

impl fmt::Debug for BlockMetaPretty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlockMetaPretty")
            .field("slot", &self.slot)
            .field("block_hash", &self.block_hash)
            .field("block_time", &self.block_time)
            .field("recv_us", &self.recv_us)
            .finish()
    }
}

#[derive(Clone)]
pub struct TransactionPretty {
    pub slot: u64,
    pub tx_index: Option<u64>, // 新增：交易在slot中的索引
    pub block_hash: String,
    pub block_time: Option<Timestamp>,
    pub signature: Signature,
    pub is_vote: bool,
    pub recv_us: i64,
    pub grpc_tx: SubscribeUpdateTransactionInfo,
}

impl fmt::Debug for TransactionPretty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TransactionPretty")
            .field("slot", &self.slot)
            .field("tx_index", &self.tx_index)
            .field("signature", &self.signature)
            .field("is_vote", &self.is_vote)
            .field("recv_us", &self.recv_us)
            .finish()
    }
}

impl Default for TransactionPretty {
    fn default() -> Self {
        Self {
            slot: 0,
            tx_index: None,
            block_hash: String::new(),
            block_time: None,
            signature: Signature::default(),
            is_vote: false,
            grpc_tx: SubscribeUpdateTransactionInfo::default(),
            recv_us: 0,
        }
    }
}
