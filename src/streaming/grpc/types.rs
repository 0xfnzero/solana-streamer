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
    pub transaction_index: Option<u64>, // 新增：交易在slot中的索引
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
            .field("transaction_index", &self.transaction_index)
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
            transaction_index: None,
            block_hash: String::new(),
            block_time: None,
            signature: Signature::default(),
            is_vote: false,
            grpc_tx: SubscribeUpdateTransactionInfo::default(),
            recv_us: 0,
        }
    }
}

// impl From<SubscribeUpdateAccount> for AccountPretty {
//     fn from(account: SubscribeUpdateAccount) -> Self {
//         let account_info = account.account.unwrap();
//         Self {
//             slot: account.slot,
//             signature: if let Some(txn_signature) = account_info.txn_signature {
//                 Signature::try_from(txn_signature.as_slice()).expect("valid signature")
//             } else {
//                 Signature::default()
//             },
//             pubkey: Pubkey::try_from(account_info.pubkey.as_slice()).expect("valid pubkey"),
//             executable: account_info.executable,
//             lamports: account_info.lamports,
//             owner: Pubkey::try_from(account_info.owner.as_slice()).expect("valid pubkey"),
//             rent_epoch: account_info.rent_epoch,
//             data: account_info.data,
//             recv_us: get_high_perf_clock(),
//         }
//     }
// }

// impl From<(SubscribeUpdateBlockMeta, Option<Timestamp>)> for BlockMetaPretty {
//     fn from(
//         (SubscribeUpdateBlockMeta { slot, blockhash, .. }, block_time): (
//             SubscribeUpdateBlockMeta,
//             Option<Timestamp>,
//         ),
//     ) -> Self {
//         Self {
//             block_hash: blockhash,
//             block_time,
//             slot,
//             recv_us: get_high_perf_clock(),
//         }
//     }
// }

// impl From<(SubscribeUpdateTransaction, Option<Timestamp>)> for TransactionPretty {
//     fn from(
//         (SubscribeUpdateTransaction { transaction, slot }, block_time): (
//             SubscribeUpdateTransaction,
//             Option<Timestamp>,
//         ),
//     ) -> Self {
//         let tx = transaction.expect("should be defined");
//         // 根据用户说明，交易索引在 transaction.index 中
//         let transaction_index = tx.index;
//         Self {
//             slot,
//             transaction_index: Some(transaction_index), // 提取交易索引
//             block_time,
//             block_hash: String::new(),
//             signature: Signature::try_from(tx.signature.as_slice()).expect("valid signature"),
//             is_vote: tx.is_vote,
//             tx: yellowstone_grpc_proto::convert_from::create_tx_with_meta(tx)
//                 .expect("valid tx with meta"),
//             recv_us: get_high_perf_clock(),
//         }
//     }
// }
