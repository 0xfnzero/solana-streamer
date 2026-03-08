<div align="center">
    <h1>🌊 Solana Streamer</h1>
    <h3><em>从 Solana DEX 交易程序实时流式传输事件。</em></h3>
</div>

<p align="center">
    <strong>一个轻量级的 Rust 库，为 PumpFun、PumpSwap、Bonk 和 Raydium 协议提供高效的事件解析和订阅功能。</strong>
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
- **Yellowstone gRPC 支持**: 使用 Yellowstone gRPC 进行高性能事件订阅
- **ShredStream 支持**: 使用 ShredStream 协议进行替代事件流传输
- **统一事件接口**: 在所有支持的协议中保持一致的事件处理

### 多协议支持
- **PumpFun**: 迷因币交易平台事件
- **PumpSwap**: PumpFun 的交换协议事件
- **Bonk**: 代币发布平台事件 (letsbonk.fun)
- **Raydium CPMM**: Raydium 集中池做市商事件
- **Raydium CLMM**: Raydium 集中流动性做市商事件
- **Raydium AMM V4**: Raydium 自动做市商 V4 事件

### 高级功能
- **事件解析系统**: 自动解析和分类协议特定事件
- **账户状态监控**: 实时监控协议账户状态和配置变更
- **交易与账户事件过滤**: 分别过滤交易事件和账户状态变化
- **动态订阅管理**: 运行时过滤器更新而无需重新连接，支持自适应监控策略
- **多重过滤器支持**: 在单个订阅中支持多个交易和账户过滤器
- **高级账户过滤**: 使用 memcmp 过滤器进行精确的账户数据匹配和监控
- **Token2022 支持**: 增强对 SPL Token 2022 的支持，包含扩展状态解析

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
solana-streamer-sdk = { path = "./solana-streamer", version = "1.3.0" }
```

### 使用 crates.io

```toml
# 添加到您的 Cargo.toml
solana-streamer-sdk = "1.3.0"
```

## 🔄 迁移指南

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
use solana_streamer_sdk::streaming::grpc::ClientConfig;

// 使用默认配置
let grpc = YellowstoneGrpc::new(endpoint, token)?;

// 或创建自定义配置
let mut config = ClientConfig::default();
config.enable_metrics = true;  // 启用性能监控
config.connection.connect_timeout = 30;  // 30 秒
config.connection.request_timeout = 120;  // 120 秒

let grpc = YellowstoneGrpc::new_with_config(endpoint, token, config)?;
```

**可用配置选项：**
- `enable_metrics`: 启用/禁用性能监控（默认：false）
- `connection.connect_timeout`: 连接超时（秒）（默认：10）
- `connection.request_timeout`: 请求超时（秒）（默认：60）
- `connection.max_decoding_message_size`: 最大消息大小（字节）（默认：10MB）

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
let event_type_filter = Some(EventTypeFilter { 
    include: vec![EventType::PumpSwapBuy, EventType::PumpSwapSell] 
});
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
let event_type_filter = Some(EventTypeFilter { 
    include: vec![
        EventType::PumpSwapBuy,
        EventType::PumpSwapSell,
        EventType::PumpFunTrade,
        EventType::RaydiumCpmmSwap,
        EventType::RaydiumClmmSwap,
        EventType::RaydiumAmmV4Swap,
        .....
    ] 
});
```

**池监控（专注流动性事件）**
```rust
let event_type_filter = Some(EventTypeFilter { 
    include: vec![
        EventType::PumpSwapCreatePool,
        EventType::PumpSwapDeposit,
        EventType::PumpSwapWithdraw,
        EventType::RaydiumCpmmInitialize,
        EventType::RaydiumCpmmDeposit,
        EventType::RaydiumCpmmWithdraw,
        EventType::RaydiumClmmCreatePool,
        ......
    ] 
});
```

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
- **PumpSwap**: PumpFun 的交换协议
- **Bonk**: 代币发布平台 (letsbonk.fun)
- **Raydium CPMM**: Raydium 集中池做市商协议
- **Raydium CLMM**: Raydium 集中流动性做市商协议
- **Raydium AMM V4**: Raydium 自动做市商 V4 协议

## 🌐 事件流服务

- **Yellowstone gRPC**: 高性能 Solana 事件流
- **ShredStream**: 替代事件流协议

## 🏗️ 架构特性

### 统一事件接口

- **DexEvent 枚举**: 包含所有协议事件的类型安全枚举
- **Protocol Enum**: 轻松识别事件来源
- **Event Factory**: 自动事件解析和分类

### 事件解析系统

- **协议特定解析器**: 每个支持协议的专用解析器
- **事件工厂**: 集中式事件创建和解析
- **可扩展设计**: 易于添加新协议和事件类型

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
│   ├── event_parser/ # 事件解析系统
│   │   ├── common/   # 通用事件解析工具
│   │   ├── core/     # 核心解析特征和接口
│   │   ├── protocols/# 协议特定解析器
│   │   │   ├── bonk/ # Bonk 事件解析
│   │   │   ├── pumpfun/ # PumpFun 事件解析
│   │   │   ├── pumpswap/ # PumpSwap 事件解析
│   │   │   ├── raydium_amm_v4/ # Raydium AMM V4 事件解析
│   │   │   ├── raydium_cpmm/ # Raydium CPMM 事件解析
│   │   │   └── raydium_clmm/ # Raydium CLMM 事件解析
│   │   └── factory.rs # 解析器工厂
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