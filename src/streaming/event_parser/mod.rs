//! Solana DEX 交易解析：**对外类型** [`DexEvent`]、[`Protocol`]；
//! **`common`**（元数据/过滤器）、**`core`**（SDK 调度与入口）、**`protocols`**（按协议的事件类型与兼容 facade）。
//!
//! gRPC/ShredStream 解析统一委托给 **sol-parser-sdk**，本 crate 只做订阅、过滤映射、
//! SDK 事件到 streamer 事件的适配，以及兼容旧公开路径的轻量转发。
//!
//! [`DexEvent`]: crate::streaming::event_parser::DexEvent
//! [`Protocol`]: crate::streaming::event_parser::Protocol

pub mod common;
pub mod core;
pub mod protocols;

pub use core::traits::DexEvent;
pub use protocols::types::Protocol;
