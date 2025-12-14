use crate::streaming::event_parser::common::EventMetadata;
use crate::streaming::event_parser::protocols::whirlpool::types::Whirlpool;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// Whirlpool 账户事件
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhirlpoolAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub whirlpool: Whirlpool,
}

/// 事件鉴别器常量
pub mod discriminators {
    // 账户鉴别器 - Anchor discriminator for "Whirlpool" account
    // 这是通过 Anchor 的账户名称 "account:Whirlpool" 计算得出的 8 字节哈希
    pub const WHIRLPOOL: &[u8] = &[63, 149, 209, 12, 225, 128, 99, 9];
}
