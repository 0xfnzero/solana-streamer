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
    <a href="https://github.com/0xfnzero/solana-streamer/blob/main/README_CN.md">中文</a> |
    <a href="https://github.com/0xfnzero/solana-streamer/blob/main/README.md">English</a> |
    <a href="https://fnzero.dev/">Website</a> |
    <a href="https://t.me/fnzero_group">Telegram</a> |
    <a href="https://discord.gg/vuazbGkqQE">Discord</a>
</p>

当前版本：**5.0.1**。升级前请阅读 [迁移说明](RELEASE_NOTES.md)；历史变更见 [版本记录](CHANGELOG_CN.md)。

## 目录

- [相关 SDK](#related-sdks)
- [安装](#安装)
- [这个 SDK 适合什么场景](#这个-sdk-适合什么场景)
- [项目特性](#项目特性)
- [使用示例](#使用示例)
- [交易来源与解析边界](#交易来源与解析边界)
- [支持的协议](#支持的协议)
- [文档导航](#文档导航)
- [配置系统](#配置系统)
- [动态订阅管理](#动态订阅管理)

<a id="related-sdks"></a>

## 相关 SDK

各仓库版本独立，依赖兼容要求以对应 SDK 的安装说明为准。

| 语言 | 仓库 | 用途 | 版本 |
|----------|------------|-------------|---------|
| Rust | [sol-parser-sdk](https://github.com/0xfnzero/sol-parser-sdk) | DEX 交易与账户事件解析 | `v0.10.0` |
| Node.js | [sol-parser-sdk-nodejs](https://github.com/0xfnzero/sol-parser-sdk-nodejs) | DEX 交易与账户事件解析 | `v0.10.0` |
| Python | [sol-parser-sdk-python](https://github.com/0xfnzero/sol-parser-sdk-python) | DEX 交易与账户事件解析 | `v0.10.0` |
| Go | [sol-parser-sdk-golang](https://github.com/0xfnzero/sol-parser-sdk-golang) | DEX 交易与账户事件解析 | `v0.10.0` |
| Rust | [sol-trade-sdk](https://github.com/0xfnzero/sol-trade-sdk) | 交易构建、签名与多通道提交 | `v10.0.0` |
| Node.js | [sol-trade-sdk-nodejs](https://github.com/0xfnzero/sol-trade-sdk-nodejs) | 交易构建、签名与多通道提交 | `v10.0.0` |
| Python | [sol-trade-sdk-python](https://github.com/0xfnzero/sol-trade-sdk-python) | 交易构建、签名与多通道提交 | `v10.0.0` |
| Go | [sol-trade-sdk-golang](https://github.com/0xfnzero/sol-trade-sdk-golang) | 交易构建、签名与多通道提交 | `v10.0.0` |
| Rust | [sol-shred-sdk](https://github.com/0xfnzero/sol-shred-sdk) | 原始 Shred 解码与 ShredStream 指令事件 | `v6.0.2` |
| Rust | [solana-streamer](https://github.com/0xfnzero/solana-streamer) | gRPC / ShredStream 订阅与事件流封装 | `v5.0.1` |
| Rust | [sol-trade-router-sdk](https://github.com/0xfnzero/sol-trade-router-sdk) | 链上多跳 Router 与客户端 SDK | `v0.4.0` |

<a id="安装"></a>

## 安装

```toml
# 添加到您的 Cargo.toml
solana-streamer-sdk = "5.0.1"
```

解析后端 feature：

```toml
# 默认：sol-parser-sdk parse-borsh 后端
solana-streamer-sdk = "5.0.1"

# 面向低延迟 Bot 的 zero-copy 解析后端
solana-streamer-sdk = { version = "5.0.1", default-features = false, features = ["sdk-parse-zero-copy"] }
```

如果同时启用 `sdk-parse-borsh` 和 `sdk-parse-zero-copy`，`sol-parser-sdk 0.6.1+` 会优先使用 zero-copy 后端。

### 从源码使用

将项目克隆到您的项目目录：

```bash
cd your_project_root_directory
git clone https://github.com/0xfnzero/solana-streamer
```

在您的 `Cargo.toml` 中添加依赖：

```toml
# 添加到您的 Cargo.toml
solana-streamer-sdk = { path = "./solana-streamer", version = "5.0.1" }
```

<a id="这个-sdk-适合什么场景"></a>

## 这个 SDK 适合什么场景

`solana-streamer-sdk` 是面向 Solana Bot、索引服务、跟单系统、狙击机器人和 DEX 监控工具的 Rust 事件流层，在 Yellowstone gRPC、ShredStream、RPC 交易解析和底层 `sol-parser-sdk` 解析核心之上提供稳定 API。

| 方向 | 覆盖范围 |
|------|----------|
| 输入来源 | Yellowstone gRPC、Jito ShredStream、已获取的 RPC 交易、编码交易数据 |
| DEX 协议 | PumpFun、PumpSwap、Pump Fees、LaunchLab、StonkFun、Raydium CPMM、Raydium CLMM、Raydium AMM V4、Meteora DAMM v2、Meteora DLMM、Meteora DBC、Orca Whirlpool |
| 使用场景 | 实时 DEX 事件流、代币发射监控、跟单交易、账户状态订阅、Bot 信号管道 |
| 解析后端 | `sol-parser-sdk`，默认 Borsh 解析，并可为低延迟场景启用 zero-copy 后端 |

<a id="项目特性"></a>

## 🚀 项目特性

### 核心功能
- **实时事件流**: 订阅多个 Solana DEX 协议的实时交易事件
- **SDK 底层解析核心**: 交易、RPC、账户和 ShredStream 解析都优先复用 `sol-parser-sdk`
- **Yellowstone gRPC 支持**: 使用 Yellowstone gRPC 进行高性能事件订阅
- **ShredStream 支持**: 使用 ShredStream 协议进行替代事件流传输；ALT 账户需由传入的加载地址或本地缓存解析，不猜测未解析地址
- **统一事件接口**: 在所有支持的协议中保持一致的事件处理

### 多协议支持
- **PumpFun**: 迷因币交易平台事件
- **Pump Fees**: Pump 费用分成配置事件
- **PumpSwap**: PumpFun 的交换协议事件
- **LaunchLab**: 通过 `Protocol::LaunchLab` 订阅共享代币发射程序事件；`Protocol::Bonk` 和 `Protocol::RaydiumLaunchpad` 仍作为兼容别名保留
- **StonkFun**: 通过 `Protocol::StonkFun` 订阅归属于 StonkFun 的 LaunchLab 事件；事件 metadata 使用 `ProtocolType::StonkFun`
- **Raydium CPMM**: Raydium 恒定乘积做市商事件
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

<a id="使用示例"></a>

## 📚 使用示例

### 使用示例概览表

| 描述 | 运行命令 | 源码路径 |
|------|---------|----------|
| 使用 Yellowstone gRPC 监控交易事件 | `cargo run --example grpc_example` | [examples/grpc_example.rs](examples/grpc_example.rs) |
| 使用 ShredStream 监控交易事件 | `cargo run --example shred_example` | [examples/shred_example.rs](examples/shred_example.rs) |
| 解析 Solana 主网交易数据 | `cargo run --example parse_tx_events` | [examples/parse_tx_events.rs](examples/parse_tx_events.rs) |
| 从 RPC 解析 PumpFun 交易（签名：环境变量 `TX_SIGNATURE` 或 CLI 参数） | `cargo run --example parse_pump_tx --release` | [examples/parse_pump_tx.rs](examples/parse_pump_tx.rs) |
| 从 RPC 解析 PumpFun quote_mint 边界案例 | `TX_SIGNATURES=<sig1,sig2> cargo run --example parse_pumpfun_quote_cases --release` | [examples/parse_pumpfun_quote_cases.rs](examples/parse_pumpfun_quote_cases.rs) |
| 从 RPC 解析 PumpSwap 交易 | `cargo run --example parse_pumpswap_tx --release` | [examples/parse_pumpswap_tx.rs](examples/parse_pumpswap_tx.rs) |
| 从 RPC 解析 Meteora DAMM v2 交易 | `TX_SIGNATURE=<sig> cargo run --example parse_meteora_damm_tx --release` | [examples/parse_meteora_damm_tx.rs](examples/parse_meteora_damm_tx.rs) |
| 抓取任意交易并离线回放（保存原始 meta/logs/账户） | `cargo run --example rpc_transaction_replay -- --fetch <SIGNATURE> /tmp/transaction.json` | [examples/rpc_transaction_replay.rs](examples/rpc_transaction_replay.rs) |
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

`PumpFunCreateTokenEvent.transaction_version` 在 gRPC/RPC 路径下表示交易消息版本：`Some(YellowstoneMessageVersion::Legacy)`、`V0` 或 `V1`，JSON 分别为 `"legacy"`、`"v0"`、`"v1"`。缺少交易上下文时（包括独立 Shred 解析）为 `None`。该字段与 `ix_name` 表示的 Pump 指令版本相互独立。

<a id="交易来源与解析边界"></a>

## 交易来源与解析边界

gRPC/RPC 使用提供的执行元数据和实际 CPI 指令；Shred 只能看到外层指令与本地已解析的账户。未知第三方指令在自身账户中引用 Pump.fun 时，可回调 `PumpFunTrade` 候选，携带 `third_program` 原始输入；这不证明交易已执行，也不确定方向或金额。买入／卖出专用过滤器不会把未知方向当作成交。

解析热路径不调用 RPC。缺失字段的默认值不代表实际零成交。账户顺序与原始指令数据保留在 `third_program`；交易序号和接收时间仍在事件元数据中，本地 Shred 序号不等于全 slot 执行顺序。详见 [第三方包装交易](docs/wrapped-pumpfun.md)。

已知 Pump.fun 创建／交易指令在 ALT 未展开时仍保留回调，未知账户保持未知。适配层保留 `create_v2`、V2/V3 方法名、创建账户和官方 `multi_hop_swap` 来源。运行 `cargo run --example rpc_transaction_replay`，可用已报告的真实签名交易离线比较纯 Shred 与执行解析；Shred 路由预算不是各跳成交金额，实际报价币／代币金额来自执行元数据。

<a id="配置系统"></a>

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

<a id="动态订阅管理"></a>

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

<a id="支持的协议"></a>

## 🔧 支持的协议

- **PumpFun**: 主要迷因币交易平台
- **Pump Fees**: Pump 费用分成配置事件
- **PumpSwap**: PumpFun 的交换协议
- **LaunchLab**: 代币发射平台；`Bonk` 和 `RaydiumLaunchpad` 作为兼容别名保留
- **StonkFun**: 归属于 StonkFun 的 LaunchLab 交易和建池事件
- **Raydium CPMM**: Raydium 恒定乘积做市商协议
- **Raydium CLMM**: Raydium 集中流动性做市商协议
- **Raydium AMM V4**: Raydium 自动做市商 V4 协议
- **Meteora DAMM v2**: Meteora DAMM v2 协议
- **Orca Whirlpool**: Orca Whirlpool 协议
- **Meteora Pools**: Meteora Pools 协议
- **Meteora DBC**: Meteora Dynamic Bonding Curve 协议
- **Meteora DLMM**: Meteora 动态流动性做市商协议
- **通用/账户事件**: Token 账户、Token 元信息、Nonce 账户、区块元数据、ComputeBudget 事件，以及 Raydium CLMM/CPMM、Pump/PumpSwap、Orca Whirlpool 等已支持协议账户状态

<a id="事件流服务"></a>

## 🌐 事件流服务

- **Yellowstone gRPC**: 高性能 Solana 事件流
- **ShredStream**: 替代事件流协议

<a id="架构特性"></a>

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

<a id="项目结构"></a>

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

<a id="性能考虑"></a>

## ⚡ 性能考虑

1. **连接管理**: 正确处理连接生命周期和重连
2. **事件过滤**: 使用协议过滤减少不必要的事件处理
3. **内存管理**: 为长时间运行的流实现适当的清理
4. **错误处理**: 对网络问题和服务中断进行健壮的错误处理
5. **批处理优化**: 使用批处理减少回调开销，提高吞吐量
6. **性能监控**: 启用性能监控以识别瓶颈和优化机会
7. **优雅关闭**: 使用 stop() 方法进行干净关闭，并实现信号处理器以正确清理资源

<a id="重要注意事项"></a>

## ⚠️ 重要注意事项

1. **网络稳定性**: 确保稳定的网络连接以进行连续的事件流传输
2. **速率限制**: 注意公共 gRPC 端点的速率限制
3. **错误恢复**: 实现适当的错误处理和重连逻辑
5. **合规性**: 确保遵守相关法律法规

<a id="文档导航"></a>

## 文档导航

- [发布与迁移](RELEASE_NOTES.md)
- [使用指南](docs/wrapped-pumpfun.md)

<a id="语言版本"></a>

## 语言版本

- [English](README.md)
- [中文](README_CN.md)


<a id="许可证"></a>

## 📄 许可证

MIT 许可证

<a id="联系方式"></a>

## 📞 联系方式

- **网站**: https://fnzero.dev/
- **项目仓库**: https://github.com/0xfnzero/solana-streamer
- **Telegram 群组**: https://t.me/fnzero_group


股票池同步：33 种官方股票/ETF、58 个股票池；配置扩展、USDC 起点、缓存选路与使用边界见 [11 SDK 同步说明](docs/STOCK_BRIDGE_SYNC.md)。
