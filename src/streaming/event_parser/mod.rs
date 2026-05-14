//! Solana DEX 交易解析：**对外类型** [`DexEvent`]、[`Protocol`]；
//! **`common`**（元数据/过滤器）、**`core`**（调度、合并、gRPC 解析入口）、**`protocols`**（按协议的 parser/events）。
//!
//! 与 **sol-parser-sdk** 对照：`protocols/*/parser` ≈ sdk `instr/*`，`parser_sdk_bridge` ≈ sdk 事件枚举与各实现的胶水层。
//!
//! [`DexEvent`]: crate::streaming::event_parser::DexEvent
//! [`Protocol`]: crate::streaming::event_parser::Protocol

pub mod common;
pub mod core;
pub mod protocols;

pub use core::traits::DexEvent;
pub use protocols::types::Protocol;
