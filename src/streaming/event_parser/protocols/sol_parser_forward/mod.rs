//! 由 `sol-parser-sdk` 产出、经 `parser_sdk_bridge` 映射的协议事件类型；`native` 将 sdk 指令解析接到 Yellowstone/shred 路径。
pub mod events;
pub mod native;

use solana_sdk::pubkey::Pubkey;

pub const ORCA_WHIRLPOOL_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc");
pub const METEORA_POOLS_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB");
pub const METEORA_DLMM_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");
