<div align="center">
    <h1>🌊 Solana Streamer</h1>
    <h3><em>从 Solana DEX 交易程序实时流式传输事件。</em></h3>
</div>

<p align="center">
    <strong>一个基于 sol-parser-sdk 的轻量级 Rust 流式封装库，提供低延迟订阅能力，并保持面向 Bot 用户的稳定 API。</strong>
</p>

<p align="center">
    <a href="https://crates.io/crates/solana-streamer-sdk">
        <img src="https://img.shields.io/crates/v/solana-streamer-sdk.svg" alt="Crates.io">
    </a>
    <a href="https://docs.rs/solana-streamer-sdk">
        <img src="https://docs.rs/solana-streamer-sdk/badge.svg" alt="Documentation">
    </a>
    <a href="https://github.com/0xfnzero/solana-streamer/blob/main/LICENSE">
        <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License">
    </a>
    <a href="https://github.com/0xfnzero/solana-streamer">
        <img src="https://img.shields.io/github/stars/0xfnzero/solana-streamer?style=social" alt="GitHub stars">
    </a>
    <a href="https://github.com/0xfnzero/solana-streamer/network">
        <img src="https://img.shields.io/github/forks/0xfnzero/solana-streamer?style=social" alt="GitHub forks">
    </a>
</p>

<p align="center">
    <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust">
    <img src="https://img.shields.io/badge/Solana-9945FF?style=for-the-badge&logo=solana&logoColor=white" alt="Solana">
    <img src="https://img.shields.io/badge/Streaming-FF6B6B?style=for-the-badge&logo=livestream&logoColor=white" alt="Real-time Streaming">
    <img src="https://img.shields.io/badge/gRPC-4285F4?style=for-the-badge&logo=grpc&logoColor=white" alt="gRPC">
</p>

<p align="center">
    <a href="README_CN.md">中文</a> | 
    <a href="README.md">English</a> | 
    <a href="https://fnzero.dev/">Website</a> |
    <a href="https://t.me/fnzero_group">Telegram</a>
</p>

> ☕ **支持本项目**
>
> 本 SDK 完全免费且开源。但维护和持续更新需要消耗大量 AI 算力与 Token。如果这个 SDK 对您的开发有帮助，欢迎每月捐赠任意数量的 SOL，您的支持将帮助这个项目持续运行！
>
> **捐赠钱包：** `6oW7AXz1yRb57pYSxysuXnMs2aR1ha5rzGzReZ1MjPV8`

---

## 目录

- [🚀 项目特性](#-项目特性)
- [⚡ 安装](#-安装)
- [🔄 迁移指南](#-迁移指南)
- [⚙️ 配置系统](#️-配置系统)
- [📚 使用示例](#-使用示例)
- [🔧 支持的协议](#-支持的协议)
- [🌐 事件流服务](#-事件流服务)
- [🏗️ 架构特性](#️-架构特性)
- [📁 项目结构](#-项目结构)
- [⚡ 性能考虑](#-性能考虑)
- [📄 许可证](#-许可证)
- [📞 联系方式](#-联系方式)
- [⚠️ 重要注意事项](#️-重要注意事项)

## 🚀 项目特性

### 核心功能
- **实时事件流**: 订阅多个 Solana DEX 协议的实时交易事件
- **SDK 底层解析核心**: 交易、RPC、账户和 ShredStream 解析都优先复用 `sol-parser-sdk`
- **Yellowstone gRPC 支持**: 使用 Yellowstone gRPC 进行高性能事件订阅
- **ShredStream 支持**: 使用 ShredStream 协议进行替代事件流传输；ALT-loaded 账户会以默认账户占位并尽量解析外层事件
- **统一事件接口**: 在所有支持的协议中保持一致的事件处理

### 多协议支持
- **PumpFun**: 迷因币交易平台事件
- **Pump Fees**: Pump 费用分成配置事件
- **PumpSwap**: PumpFun 的交换协议事件
- **Raydium LaunchLab**: 代币发射平台事件；`Protocol::Bonk` 和 `Protocol::RaydiumLaunchpad` 仍作为兼容别名保留
- **Raydium CPMM**: Raydium 集中池做市商事件
- **Raydium CLMM**: Raydium 集中流动性做市商事件
- **Raydium AMM V4**: Raydium 自动做市商 V4 事件
- **Meteora DAMM v2**: Meteora DAMM v2 交易和流动性事件
- **Orca Whirlpool**: Orca Whirlpool 交易和流动性事件
- **Meteora Pools**: Meteora Pools 交易、流动性、启动流动性和费用事件
- **Meteora DBC**: Meteora Dynamic Bonding Curve 的 log-side swap、initialize-pool 和 curve-complete 事件
- **Meteora DLMM**: Meteora DLMM 交易、流动性、池、bin array 和费用事件

### 高级功能
- **事件解析系统**: 自动解析和分类协议特定事件
- **账户状态监控**: 实时监控协议账户状态和配置变更
- **交易与账户事件过滤**: 分别过滤交易事件和账户状态变化
- **动态订阅管理**: 运行时过滤器更新而无需重新连接，支持自适应监控策略
- **多重过滤器支持**: 在单个订阅中支持多个交易和账户过滤器
- **高级账户过滤**: 使用 memcmp 过滤器进行精确的账户数据匹配和监控
- **Token2022 支持**: 增强对 SPL Token 2022 的支持，包含扩展状态解析
- **RPC 交易解析**: 可解析已获取的 RPC 交易，也可按签名拉取并转换为 streamer 事件
- **高级 SDK 互操作**: 通过 `parser_sdk` 或 `sdk_bridge::raw` 直接访问底层 `sol-parser-sdk`

### 性能与优化
- **高性能**: 针对低延迟事件处理进行优化
- **批处理优化**: 批量处理事件以减少回调开销
- **性能监控**: 内置性能指标监控，包括事件处理速度
- **内存优化**: 对象池和缓存机制减少内存分配
- **灵活配置系统**: 支持自定义批处理大小、背压策略、通道大小等参数
- **预设配置**: 提供高吞吐量、低延迟等预设配置，针对不同使用场景优化
- **背压处理**: 支持阻塞、丢弃等背压策略
- **运行时配置更新**: 支持在运行时动态更新配置参数
- **优雅关闭**: 支持编程式 stop() 方法进行干净的关闭

## ⚡ 安装

### 直接克隆

将项目克隆到您的项目目录：

```bash
cd your_project_root_directory
git clone https://github.com/0xfnzero/solana-streamer
```

在您的 `Cargo.toml` 中添加依赖：

```toml
# 添加到您的 Cargo.toml
solana-streamer-sdk = { path = "./solana-streamer", version = "1.5.11" }
```

### 使用 crates.io

```toml
# 添加到您的 Cargo.toml
solana-streamer-sdk = "1.5.11"
```

解析后端 feature：

```toml
# 默认：sol-parser-sdk parse-borsh 后端
solana-streamer-sdk = "1.5.11"

# 面向低延迟 Bot 的 zero-copy 解析后端
solana-streamer-sdk = { version = "1.5.11", default-features = false, features = ["sdk-parse-zero-copy"] }
```

如果同时启用 `sdk-parse-borsh` 和 `sdk-parse-zero-copy`，`sol-parser-sdk 0.5.11+` 会优先使用 zero-copy 后端。

## 🔄 迁移指南

### 升级到 v1.5.11

v1.5.11 使用 crates.io 上的 `sol-parser-sdk 0.5.11`。PumpSwap `PumpSwapCreatePoolEvent` 现在会在 `create_pool` instruction args 可用时携带 `is_cashback_coin`，包括 ShredStream 外层指令解析。log-only 的 `CreatePoolEvent` payload 因为链上 log event IDL 不包含该字段，仍会保持默认 `false`。`AccountPumpSwapPool` 仍是读取池子账户状态里该标记的权威来源。

### 升级到 v1.5.10

v1.5.10 使用 crates.io 上的 `sol-parser-sdk 0.5.10`。PumpSwap `CreatePoolEvent` 现在与链上 IDL 对齐：暴露 `is_mayhem_mode`，但不暴露 `is_cashback_coin`。如果需要 cashback 标记，请订阅 `AccountPumpSwapPool`，并读取 `PumpSwapPoolAccountEvent.pool.is_cashback_coin`。PumpSwap CreatePool log payload 长度检查现在包含最后的 `is_mayhem_mode` 字节。

### 升级到 v1.5.9

v1.5.9 使用 crates.io 上的 `sol-parser-sdk 0.5.9`。该版本继承 Yellowstone gRPC stop 生命周期修复：`stop()` 现在会通知、abort 并等待当前订阅任务结束，订阅生命周期切换会串行化，并且 gRPC 流错误日志会标记为 `Grpc Stream error`，便于和 ShredStream 日志区分。

### 升级到 v1.5.8

v1.5.8 使用 crates.io 上的 `sol-parser-sdk 0.5.8`。该版本继承 parser SDK 的 Pump.fun ShredStream 过滤大类语义：`PumpFunBuy` 覆盖 `buy`、`buy_v2`、`buy_exact_sol_in`、`buy_exact_quote_in_v2`；`PumpFunSell` 覆盖 `sell`、`sell_v2`；`PumpFunTrade` 覆盖所有 buy/sell 指令，并且当只请求通用 trade filter 时，parser 可统一输出 trade 事件。

### 升级到 v1.5.5

v1.5.5 使用 crates.io 上的 `sol-parser-sdk 0.5.5`。底层 SDK 已将 Raydium LaunchLab 事件统一暴露为 `RaydiumLaunchlab*`；streamer 仍保留现有 `Bonk*` 事件结构以及 `Protocol::Bonk` / `Protocol::RaydiumLaunchpad` 兼容别名，同时把解析调用和上游 gRPC 事件过滤映射到新的 LaunchLab SDK variant。该版本也同步了 CLMM/CPMM/Orca account bridge、Meteora DAMM v2 initialize-pool、Meteora DBC 事件，以及客户端创建时的 parser warmup。

### 升级到 v1.5.4

v1.5.4 使用 crates.io 上的 `sol-parser-sdk 0.5.4`。Pump.fun `create` 和 `create_v2` 会统一投递为 canonical `PumpFunCreateTokenEvent`；订阅 `PumpFunCreateToken` 或 `PumpFunCreateV2Token` 任意一个，都能收到同一份完整 create-family 数据。该版本避免 gRPC log + instruction 双路径解析导致新 mint 回调重复，同时在 canonical create 事件上保留 create_v2 账户字段。

### 升级到 v1.5.3

v1.5.3 使用 crates.io 上的 `sol-parser-sdk 0.5.3`。该版本会通过 streamer bridge 保留真实的 Pump.fun v2 `ix_name`，改进 ShredStream 对 Pump.fun v2 短账户列表的 best-effort 解析，并让订阅 `PumpFunBuy` 或 `PumpFunBuyExactSolIn` 都能匹配兼容的 buy-family 事件。ShredStream 仍使用直接读取 Entry 的自动重连低延迟路径；回调应避免阻塞，`tx_index` 仍是 Entry 内 best-effort 索引。

### 升级到 v1.5.2

v1.5.2 使用 crates.io 上的 `sol-parser-sdk 0.5.2`。ShredStream 投递使用直接读取 Entry 并调用 SDK parser 的低延迟路径，支持自动重连，避免额外的队列消费任务并复用解析事件缓冲。ShredStream 的 `tx_index` 是 Entry 内 best-effort 索引，不是 Yellowstone 的 slot 级交易索引。direct-callback 路径中的用户回调运行在读流任务中，应避免阻塞操作。

### 升级到 v1.5.1

v1.5.1 使用 crates.io 上的 `sol-parser-sdk 0.5.1`。该版本改进 ShredStream 的 ALT 处理：ALT-loaded 账户会以默认账户占位，外层指令按 data/discriminator 尽量解析；同时增加队列满时的 dropped event 可观测性，并明确 CPI/inner-only 事件仍是 ShredStream 的固有限制。

### 升级到 v1.5.0

v1.5.0 使用 `sol-parser-sdk 0.5.0`，打通 Raydium CLMM 账户解析的 SDK bridge，并降低有序缓冲低延迟投递路径中的分配和搬移开销。`sol-parser-sdk 0.4.19` 和 `solana-streamer-sdk 1.4.14` 已由 v1.5.0 取代，因为新增公开 parser event variant 应放到 0.5 版本线。

### 升级到 v1.4.14

v1.4.14 已由 v1.5.0 取代。

### 升级到 v1.4.13

v1.4.13 使用 `sol-parser-sdk 0.4.18`，并按 Raydium CLMM 官方升级后 IDL 更新集成：当前 log-side event discriminator、官方 Swap/Liquidity/Create/Collect 事件布局、限价单事件、dynamic fee 相关事件，以及重塑后的 PoolState/TickState 账户结构。

### 升级到 v1.4.12

v1.4.12 使用 `sol-parser-sdk 0.4.17`，会把 legacy PumpFun SOL 的 `quote_mint` 归一为 Solscan SOL sentinel，同时保留真实 USDC quote mint 和 quote reserve 字段。Streamer bridge 与 merger 路径继续保持 Yellowstone gRPC 和 ShredStream 输出与 parser SDK 语义一致。

新增可选能力：

- `solana_streamer_sdk::parser_sdk` 重新导出原始 `sol-parser-sdk` crate。
- `solana_streamer_sdk::sdk_bridge` 可将原始 SDK 事件适配回 streamer `DexEvent`。
- `fetch_rpc_transaction_as_streamer_events` 和 `parse_encoded_rpc_transaction_as_streamer_events` 可将 RPC 交易解析为 streamer 事件。
- `grpc::ClientConfig::order_mode` 支持 `Unordered`、`Ordered`、`StreamingOrdered` 和 `MicroBatch`。
- `sdk-parse-zero-copy` 可启用 SDK zero-copy 解析后端。

### 从 v0.5.x 迁移到 v1.x.x

版本 1.0.0 引入了从基于 trait 的事件处理到基于 enum 的事件的重大架构变更。这提供了更好的类型安全性、改进的性能和更简单的代码模式。

**主要变更：**

1. **事件类型变更** - `Box<dyn UnifiedEvent>` → `DexEvent` 枚举
2. **回调签名** - 回调现在接收具体的 `DexEvent` 而不是 trait 对象
3. **事件匹配** - 使用标准 Rust `match` 而不是 `match_event!` 宏
4. **元数据访问** - 事件属性现在通过 `.metadata()` 方法访问

详细的迁移步骤和代码示例，请参阅 [MIGRATION.md](MIGRATION.md) 或 [MIGRATION_CN.md](MIGRATION_CN.md)（中文版本）。

**快速迁移示例：**

```rust
// 旧版 (v0.5.x)
let callback = |event: Box<dyn UnifiedEvent>| {
    println!("Event: {:?}", event.event_type());
};

// 新版 (v1.x.x)
let callback = |event: DexEvent| {
    println!("Event: {:?}", event.metadata().event_type);
};
```

## ⚙️ 配置系统

您可以自定义客户端配置：

```rust
use solana_streamer_sdk::streaming::{
    grpc::{ClientConfig, OrderMode},
    YellowstoneGrpc,
};

// 使用默认配置
let grpc = YellowstoneGrpc::new(endpoint, token)?;

// 或创建自定义配置
let mut config = ClientConfig::default();
config.enable_metrics = true;  // 启用性能监控
config.connection.connect_timeout = 30;  // 30 秒
config.connection.request_timeout = 120;  // 120 秒
config.order_mode = OrderMode::MicroBatch;  // Unordered / Ordered / StreamingOrdered / MicroBatch
config.order_timeout_ms = 100;
config.micro_batch_us = 100;

let grpc = YellowstoneGrpc::new_with_config(endpoint, token, config)?;
```

**可用配置选项：**
- `enable_metrics`: 启用/禁用性能监控（默认：false）
- `connection.connect_timeout`: 连接超时（秒）（默认：10）
- `connection.request_timeout`: 请求超时（秒）（默认：60）
- `connection.max_decoding_message_size`: 最大消息大小（字节）（默认：10MB）
- `order_mode`: 交易事件输出顺序模式（默认：`Unordered`）
- `order_timeout_ms`: `Ordered` 和 `StreamingOrdered` 模式的刷新超时（默认：100）
- `micro_batch_us`: `MicroBatch` 模式的微批窗口（默认：100）

### 最小 gRPC 订阅

```rust
use solana_streamer_sdk::streaming::{
    event_parser::{
        common::{filter::EventTypeFilter, EventType},
        core::EventDispatcher,
        DexEvent, Protocol,
    },
    yellowstone_grpc::{AccountFilter, TransactionFilter},
    YellowstoneGrpc,
};

let grpc = YellowstoneGrpc::new(endpoint, token)?;

let protocols = vec![
    Protocol::PumpFun,
    Protocol::PumpFees,
    Protocol::PumpSwap,
    Protocol::RaydiumLaunchpad,
    Protocol::RaydiumCpmm,
    Protocol::RaydiumClmm,
    Protocol::RaydiumAmmV4,
    Protocol::OrcaWhirlpool,
    Protocol::MeteoraPools,
    Protocol::MeteoraDammV2,
    Protocol::MeteoraDlmm,
];

let program_ids = EventDispatcher::get_program_ids(&protocols)
    .into_iter()
    .map(|pubkey| pubkey.to_string())
    .collect::<Vec<_>>();

let transaction_filter = TransactionFilter {
    account_include: program_ids.clone(),
    account_exclude: vec![],
    account_required: vec![],
};
let account_filter = AccountFilter { account: vec![], owner: program_ids, filters: vec![] };

let event_type_filter = Some(EventTypeFilter::include_only(vec![
    EventType::PumpFunBuy,
    EventType::PumpSwapBuy,
    EventType::BonkBuyExactIn,
    EventType::RaydiumCpmmSwapBaseInput,
    EventType::MeteoraDlmmSwap,
]));

grpc.subscribe_events_immediate(
    protocols,
    None,
    vec![transaction_filter],
    vec![account_filter],
    event_type_filter,
    None,
    |event: DexEvent| {
        println!("{:?}", event.metadata().event_type);
    },
)
.await?;
```

## 📚 使用示例

### 使用示例概览表

| 描述 | 运行命令 | 源码路径 |
|------|---------|----------|
| 使用 Yellowstone gRPC 监控交易事件 | `cargo run --example grpc_example` | [examples/grpc_example.rs](examples/grpc_example.rs) |
| 使用 ShredStream 监控交易事件 | `cargo run --example shred_example` | [examples/shred_example.rs](examples/shred_example.rs) |
| 解析 Solana 主网交易数据 | `cargo run --example parse_tx_events` | [examples/parse_tx_events.rs](examples/parse_tx_events.rs) |
| 从 RPC 解析 PumpFun 交易（签名：环境变量 `TX_SIGNATURE` 或 CLI 参数） | `cargo run --example parse_pump_tx --release` | [examples/parse_pump_tx.rs](examples/parse_pump_tx.rs) |
| 从 RPC 解析 PumpSwap 交易 | `cargo run --example parse_pumpswap_tx --release` | [examples/parse_pumpswap_tx.rs](examples/parse_pumpswap_tx.rs) |
| 从 RPC 解析 Meteora DAMM v2 交易 | `TX_SIGNATURE=<sig> cargo run --example parse_meteora_damm_tx --release` | [examples/parse_meteora_damm_tx.rs](examples/parse_meteora_damm_tx.rs) |
| 调试 PumpFun 交易（拉取、打印 meta/logs、解析） | `TX_SIGNATURE=<sig> cargo run --example debug_pump_tx --release` | [examples/debug_pump_tx.rs](examples/debug_pump_tx.rs) |
| 调试 PumpSwap 交易（拉取、打印 meta、解析） | `TX_SIGNATURE=<sig> cargo run --example debug_pumpswap_tx --release` | [examples/debug_pumpswap_tx.rs](examples/debug_pumpswap_tx.rs) |
| 运行时更新过滤器 | `cargo run --example dynamic_subscription` | [examples/dynamic_subscription.rs](examples/dynamic_subscription.rs) |
| 快速测试：订阅 PumpFun，打印前 10 条或运行 60 秒 | `cargo run --example pumpfun_quick_test --release` | [examples/pumpfun_quick_test.rs](examples/pumpfun_quick_test.rs) |
| PumpFun 交易过滤：买入/卖出/创建及延迟统计 | `cargo run --example pumpfun_trade_filter --release` | [examples/pumpfun_trade_filter.rs](examples/pumpfun_trade_filter.rs) |
| PumpFun gRPC 订阅（含指标） | `cargo run --example pumpfun_with_metrics --release` | [examples/pumpfun_with_metrics.rs](examples/pumpfun_with_metrics.rs) |
| PumpSwap gRPC 订阅（含指标） | `cargo run --example pumpswap_with_metrics --release` | [examples/pumpswap_with_metrics.rs](examples/pumpswap_with_metrics.rs) |
| Meteora DAMM v2 gRPC 订阅 | `cargo run --example meteora_damm_grpc --release` | [examples/meteora_damm_grpc.rs](examples/meteora_damm_grpc.rs) |
| 监控特定代币账户余额变化 | `cargo run --example token_balance_listen_example` | [examples/token_balance_listen_example.rs](examples/token_balance_listen_example.rs) |
| 通过账户订阅监控代币精度 | `cargo run --example token_decimals_listen_example` | [examples/token_decimals_listen_example.rs](examples/token_decimals_listen_example.rs) |
| 跟踪 nonce 账户状态变化 | `cargo run --example nonce_listen_example` | [examples/nonce_listen_example.rs](examples/nonce_listen_example.rs) |
| 使用 memcmp 过滤器监控 PumpSwap 池账户 | `cargo run --example pumpswap_pool_account_listen_example` | [examples/pumpswap_pool_account_listen_example.rs](examples/pumpswap_pool_account_listen_example.rs) |
| 使用 memcmp 过滤器监控特定代币的所有关联代币账户 | `cargo run --example mint_all_ata_account_listen_example` | [examples/mint_all_ata_account_listen_example.rs](examples/mint_all_ata_account_listen_example.rs) |

### 事件过滤

库支持灵活的事件过滤以减少处理开销并提升性能：

#### 基础过滤

```rust
use solana_streamer_sdk::streaming::event_parser::common::{filter::EventTypeFilter, EventType};

// 无过滤 - 接收所有事件
let event_type_filter = None;

// 过滤特定事件类型 - 只接收 PumpSwap 买入/卖出事件
let event_type_filter = Some(EventTypeFilter::include_only(vec![
    EventType::PumpSwapBuy,
    EventType::PumpSwapSell,
]));

// 排除高频噪声事件，保留其他所有事件
let event_type_filter = Some(EventTypeFilter::exclude_only(vec![EventType::BlockMeta]));
```

#### 性能影响

事件过滤可以带来显著的性能提升：
- **减少 60-80%** 的不必要事件处理
- **降低内存使用** 通过过滤掉无关事件
- **减少网络带宽** 在分布式环境中
- **更好的专注性** 只处理对应用有意义的事件

#### 按使用场景的过滤示例

**交易机器人（专注交易事件）**
```rust
let event_type_filter = Some(EventTypeFilter::include_only(vec![
    EventType::PumpFunBuy,
    EventType::PumpFunBuyExactSolIn,
    EventType::PumpFunSell,
    EventType::PumpSwapBuy,
    EventType::PumpSwapSell,
    EventType::BonkBuyExactIn,
    EventType::BonkSellExactIn,
    EventType::RaydiumCpmmSwapBaseInput,
    EventType::RaydiumCpmmSwapBaseOutput,
    EventType::RaydiumClmmSwap,
    EventType::RaydiumAmmV4SwapBaseIn,
    EventType::RaydiumAmmV4SwapBaseOut,
    EventType::OrcaWhirlpoolSwap,
    EventType::MeteoraPoolsSwap,
    EventType::MeteoraDammV2Swap,
    EventType::MeteoraDlmmSwap,
]));
```

**池监控（专注流动性事件）**
```rust
let event_type_filter = Some(EventTypeFilter::include_only(vec![
    EventType::PumpFeesUpdateFeeShares,
    EventType::PumpSwapCreatePool,
    EventType::AccountPumpSwapPool,
    EventType::PumpSwapDeposit,
    EventType::PumpSwapWithdraw,
    EventType::RaydiumCpmmInitialize,
    EventType::RaydiumCpmmDeposit,
    EventType::RaydiumCpmmWithdraw,
    EventType::RaydiumClmmCreatePool,
    EventType::OrcaWhirlpoolPoolInitialized,
    EventType::MeteoraPoolsPoolCreated,
    EventType::MeteoraDammV2AddLiquidity,
    EventType::MeteoraPoolsAddLiquidity,
    EventType::MeteoraDlmmAddLiquidity,
]));
```

`PumpSwapCreatePool` 包含 `is_mayhem_mode`。对于 `is_cashback_coin`，
ShredStream/外层指令解析会从 `create_pool` instruction args 读取；
log-only 的 `CreatePoolEvent` payload 因为 IDL 不包含该字段，会保持默认
`false`。账户里的权威值也可通过
`PumpSwapPoolAccountEvent.pool.is_cashback_coin` 读取。

## 动态订阅管理

在运行时更新订阅过滤器而无需重新连接到流。

```rust
// 在现有订阅上更新过滤器
grpc.update_subscription(
    vec![TransactionFilter {
        account_include: vec!["new_program_id".to_string()],
        account_exclude: vec![],
        account_required: vec![],
    }],
    vec![AccountFilter {
        account: vec![],
        owner: vec![],
        filters: vec![],
    }],
).await?;
```

- **无需重新连接**: 过滤器变更立即生效，无需关闭流
- **原子更新**: 交易和账户过滤器同时更新
- **单一订阅**: 每个客户端实例只有一个活跃订阅
- **兼容性**: 与立即订阅和高级订阅方法兼容

注意：在同一客户端上多次尝试订阅会返回错误。

## 🔧 支持的协议

- **PumpFun**: 主要迷因币交易平台
- **Pump Fees**: Pump 费用分成配置事件
- **PumpSwap**: PumpFun 的交换协议
- **Raydium LaunchLab**: 代币发射平台；`Bonk` 和 `RaydiumLaunchpad` 作为兼容别名保留
- **Raydium CPMM**: Raydium 集中池做市商协议
- **Raydium CLMM**: Raydium 集中流动性做市商协议
- **Raydium AMM V4**: Raydium 自动做市商 V4 协议
- **Meteora DAMM v2**: Meteora DAMM v2 协议
- **Orca Whirlpool**: Orca Whirlpool 协议
- **Meteora Pools**: Meteora Pools 协议
- **Meteora DBC**: Meteora Dynamic Bonding Curve 协议
- **Meteora DLMM**: Meteora 动态流动性做市商协议
- **通用/账户事件**: Token 账户、Token 元信息、Nonce 账户、区块元数据、ComputeBudget 事件，以及 Raydium CLMM/CPMM、Pump/PumpSwap、Orca Whirlpool 等已支持协议账户状态

## 🌐 事件流服务

- **Yellowstone gRPC**: 高性能 Solana 事件流
- **ShredStream**: 替代事件流协议

## 🏗️ 架构特性

### 统一事件接口

- **DexEvent 枚举**: 包含所有协议事件的类型安全枚举
- **Protocol Enum**: 轻松识别事件来源
- **SDK 桥接层**: 将 `sol-parser-sdk::DexEvent` 适配为 streamer `DexEvent`

### 事件解析系统

- **sol-parser-sdk facade**: Yellowstone gRPC、ShredStream、RPC 交易解析和账户解析中的协议解析都委托给 `sol-parser-sdk`
- **本地非 DEX 补充**: 本地逻辑仅保留 streamer 基础设施和 ComputeBudget 元数据等非 DEX 兼容场景
- **可扩展桥接层**: `streaming::sdk_bridge` 暴露原始 SDK 能力，但不强迫已有 Bot 改回调 API

### 流基础设施

- **Yellowstone gRPC 客户端**: 针对 Solana 事件流优化
- **ShredStream 客户端**: 替代流实现
- **高性能处理**: 优化的事件处理机制

## 📁 项目结构

```
src/
├── common/           # 通用功能和类型
├── protos/           # Protocol buffer 定义
├── streaming/        # 事件流系统
│   ├── event_parser/ # 基于 sol-parser-sdk 的 streamer 兼容事件 facade
│   │   ├── common/   # 公开事件元数据和过滤类型
│   │   ├── core/     # SDK 分发入口和兼容封装
│   │   ├── protocols/# Streamer 事件类型和旧模块路径
│   │   │   └── sol_parser_forward/ # SDK 转发协议事件封装
│   ├── parser_sdk_bridge/ # sol-parser-sdk 事件适配层
│   ├── rpc_parse.rs # RPC 交易解析 helper
│   ├── sdk_bridge.rs # 公开的高级 SDK 互操作模块
│   ├── shred_stream.rs # ShredStream 客户端
│   ├── yellowstone_grpc.rs # Yellowstone gRPC 客户端
│   └── yellowstone_sub_system.rs # Yellowstone 子系统
└── lib.rs            # 主库文件
```

## ⚡ 性能考虑

1. **连接管理**: 正确处理连接生命周期和重连
2. **事件过滤**: 使用协议过滤减少不必要的事件处理
3. **内存管理**: 为长时间运行的流实现适当的清理
4. **错误处理**: 对网络问题和服务中断进行健壮的错误处理
5. **批处理优化**: 使用批处理减少回调开销，提高吞吐量
6. **性能监控**: 启用性能监控以识别瓶颈和优化机会
7. **优雅关闭**: 使用 stop() 方法进行干净关闭，并实现信号处理器以正确清理资源

---

## 📄 许可证

MIT 许可证

## 📞 联系方式

- **网站**: https://fnzero.dev/
- **项目仓库**: https://github.com/0xfnzero/solana-streamer
- **Telegram 群组**: https://t.me/fnzero_group

## ⚠️ 重要注意事项

1. **网络稳定性**: 确保稳定的网络连接以进行连续的事件流传输
2. **速率限制**: 注意公共 gRPC 端点的速率限制
3. **错误恢复**: 实现适当的错误处理和重连逻辑
5. **合规性**: 确保遵守相关法律法规

## 语言版本

- [English](README.md)
- [中文](README_CN.md)
