use crate::streaming::event_parser::common::EventMetadata;
use crate::streaming::event_parser::protocols::meteora_dlmm::types::LbPair;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// LbPair 账户事件
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeteoraDlmmLbPairAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub lb_pair: LbPair,
}

/// 事件鉴别器常量
pub mod discriminators {
    // 账户鉴别器
    pub const LB_PAIR: &[u8] = &[33, 11, 49, 98, 181, 101, 177, 13];
}
