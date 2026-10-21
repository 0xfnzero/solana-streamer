<div align="center">
    <h1>Solana Streamer</h1>
    <p>Rust event subscriptions built on sol-parser-sdk</p>
    <a href="https://crates.io/crates/solana-streamer-sdk"><img src="https://img.shields.io/crates/v/solana-streamer-sdk.svg" alt="solana-streamer-sdk"></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</div>

<p align="center">
    <a href="README.md">English</a> · <a href="README_CN.md">中文</a> ·
    <a href="#usage-examples">Examples</a> ·
    <a href="docs/wrapped-pumpfun.md">Documentation</a> ·
    <a href="RELEASE_NOTES.md">Release and migration</a> ·
    <a href="#related-sdks">Related SDKs</a>
</p>

Manage gRPC and ShredStream subscriptions, event filters and dynamic subscriptions while preserving parser provenance and metadata.

Current version: **5.0.0**. Read the [migration notes](RELEASE_NOTES.md) before upgrading; see the [changelog](CHANGELOG.md) for historical changes.

## Contents

- [Related SDKs](#related-sdks)
- [Installation](#installation)
- [What This SDK Is For](#what-this-sdk-is-for)
- [Project Features](#project-features)
- [Usage Examples](#usage-examples)
- [Trade provenance and parsing boundaries](#trade-provenance-and-parsing-boundaries)
- [Supported Protocols](#supported-protocols)
- [Documentation](#documentation)
- [Configuration](#configuration-system)
- [Dynamic subscriptions](#dynamic-subscription-management)

<a id="related-sdks"></a>

## Related SDKs

Repositories are versioned independently; follow each SDK’s installation guide for compatible dependencies.

| Language | Repository | Purpose | Version |
|----------|------------|-------------|---------|
| Rust | [sol-parser-sdk](https://github.com/0xfnzero/sol-parser-sdk) | DEX transaction and account event parsing | `v0.9.1` |
| Node.js | [sol-parser-sdk-nodejs](https://github.com/0xfnzero/sol-parser-sdk-nodejs) | DEX transaction and account event parsing | `v0.9.1` |
| Python | [sol-parser-sdk-python](https://github.com/0xfnzero/sol-parser-sdk-python) | DEX transaction and account event parsing | `v0.9.1` |
| Go | [sol-parser-sdk-golang](https://github.com/0xfnzero/sol-parser-sdk-golang) | DEX transaction and account event parsing | `v0.9.1` |
| Rust | [sol-trade-sdk](https://github.com/0xfnzero/sol-trade-sdk) | Transaction building, signing and multi-channel submission | `v9.0.1` |
| Node.js | [sol-trade-sdk-nodejs](https://github.com/0xfnzero/sol-trade-sdk-nodejs) | Transaction building, signing and multi-channel submission | `v9.0.1` |
| Python | [sol-trade-sdk-python](https://github.com/0xfnzero/sol-trade-sdk-python) | Transaction building, signing and multi-channel submission | `v9.0.1` |
| Go | [sol-trade-sdk-golang](https://github.com/0xfnzero/sol-trade-sdk-golang) | Transaction building, signing and multi-channel submission | `v9.0.1` |
| Rust | [sol-shred-sdk](https://github.com/0xfnzero/sol-shred-sdk) | Raw shred decoding and ShredStream instruction events | `v6.0.1` |
| Rust | [solana-streamer](https://github.com/0xfnzero/solana-streamer) | gRPC / ShredStream subscriptions and event streaming | `v5.0.0` |
| Rust | [sol-trade-router-sdk](https://github.com/0xfnzero/sol-trade-router-sdk) | On-chain multi-hop router and client SDK | `v0.3.0` |

<a id="installation"></a>

## Installation

```toml
# Add to your Cargo.toml
solana-streamer-sdk = "5.0.0"
```

Parser backend features:

```toml
# Default: sol-parser-sdk parse-borsh backend
solana-streamer-sdk = "5.0.0"

# Zero-copy parser backend for latency-sensitive bots
solana-streamer-sdk = { version = "5.0.0", default-features = false, features = ["sdk-parse-zero-copy"] }
```

If both `sdk-parse-borsh` and `sdk-parse-zero-copy` are enabled, `sol-parser-sdk 0.6.1+` uses the zero-copy backend.

### Use from source

Clone this project to your project directory:

```bash
cd your_project_root_directory
git clone https://github.com/0xfnzero/solana-streamer
```

Add the dependency to your `Cargo.toml`:

```toml
# Add to your Cargo.toml
solana-streamer-sdk = { path = "./solana-streamer", version = "5.0.0" }
```

<a id="what-this-sdk-is-for"></a>

## What This SDK Is For

`solana-streamer-sdk` is a Rust event streaming layer for Solana bots, indexers, copy-trading systems, sniper bots, and DEX monitoring tools. It provides a stable API over Yellowstone gRPC, ShredStream, RPC transaction parsing, and the underlying `sol-parser-sdk` parser core.

| Area | Coverage |
|------|----------|
| Input sources | Yellowstone gRPC, Jito ShredStream, fetched RPC transactions, encoded transaction data |
| DEX protocols | PumpFun, PumpSwap, Pump Fees, LaunchLab, StonkFun, Raydium CPMM, Raydium CLMM, Raydium AMM V4, Meteora DAMM v2, Meteora DLMM, Meteora DBC, Orca Whirlpool |
| Use cases | Real-time DEX event streaming, token launch monitoring, copy trading, account-state subscriptions, bot signal pipelines |
| Parser backend | `sol-parser-sdk` with default Borsh parsing and optional zero-copy backend for latency-sensitive workloads |

<a id="project-features"></a>

## 🚀 Project Features

### Core Capabilities
- **Real-time Event Streaming**: Subscribe to live trading events from multiple Solana DEX protocols
- **SDK-backed Parser Core**: Transaction, RPC, account, and ShredStream parsing are backed by `sol-parser-sdk`
- **Yellowstone gRPC Support**: High-performance event subscription using Yellowstone gRPC
- **ShredStream Support**: Alternative event streaming using ShredStream protocol; ALT addresses must come from supplied loaded addresses or a local cache; unresolved addresses are not guessed
- **Unified Event Interface**: Consistent event handling across all supported protocols

### Multi-Protocol Support
- **PumpFun**: Meme coin trading platform events
- **Pump Fees**: Pump fee-sharing configuration events
- **PumpSwap**: PumpFun's swap protocol events
- **LaunchLab**: Shared token-launch program events through `Protocol::LaunchLab`; `Protocol::Bonk` and `Protocol::RaydiumLaunchpad` remain compatible aliases
- **StonkFun**: StonkFun-attributed LaunchLab events through `Protocol::StonkFun`; event metadata reports `ProtocolType::StonkFun`
- **Raydium CPMM**: Raydium's Constant Product Market Maker events
- **Raydium CLMM**: Raydium's Concentrated Liquidity Market Maker events
- **Raydium AMM V4**: Raydium's Automated Market Maker V4 events
- **Meteora DAMM v2**: Meteora DAMM v2 swap and liquidity events
- **Orca Whirlpool**: Orca Whirlpool swap and liquidity events
- **Meteora Pools**: Meteora Pools swap, liquidity, bootstrap, and fee events
- **Meteora DBC**: Meteora Dynamic Bonding Curve log-side swap, initialize-pool, and curve-complete events
- **Meteora DLMM**: Meteora DLMM swap, liquidity, pool, bin-array, and fee events

### Advanced Features
- **Event Parsing System**: Automatic parsing and categorization of protocol-specific events
- **Account State Monitoring**: Real-time monitoring of protocol account states and configuration changes
- **Transaction & Account Event Filtering**: Separate filtering for transaction events and account state changes
- **Dynamic Subscription Management**: Runtime filter updates without reconnection, enabling adaptive monitoring strategies
- **Multi-Filter Support**: Support for multiple transaction and account filters in a single subscription
- **Advanced Account Filtering**: Memcmp filters for precise account data matching and monitoring
- **Token2022 Support**: Enhanced support for SPL Token 2022 with extended state parsing
- **RPC Transaction Parsing**: Parse already-fetched RPC transactions or fetch by signature through streamer-compatible helpers
- **Advanced SDK Interop**: Access the raw `sol-parser-sdk` crate through `parser_sdk` or `sdk_bridge::raw`

### Performance & Optimization
- **High Performance**: Optimized for low-latency event processing
- **Batch Processing Optimization**: Batch processing events to reduce callback overhead
- **Performance Monitoring**: Built-in performance metrics monitoring, including event processing speed
- **Memory Optimization**: Object pooling and caching mechanisms to reduce memory allocations
- **Flexible Configuration System**: Support for custom batch sizes, backpressure strategies, channel sizes
- **Preset Configurations**: High-throughput and low-latency preset configurations optimized for different use cases
- **Backpressure Handling**: Supports blocking and dropping backpressure strategies
- **Runtime Configuration Updates**: Dynamic configuration parameter updates at runtime
- **Graceful Shutdown**: Support for programmatic stop() method for clean shutdown

<a id="usage-examples"></a>

## 📚 Usage Examples

### Usage Examples Summary Table

| Description | Run Command | Source Path |
|------|---------|----------|
| Monitor transaction events using Yellowstone gRPC | `cargo run --example grpc_example` | [examples/grpc_example.rs](examples/grpc_example.rs) |
| Monitor transaction events using ShredStream | `cargo run --example shred_example` | [examples/shred_example.rs](examples/shred_example.rs) |
| Parse Solana mainnet transaction data | `cargo run --example parse_tx_events` | [examples/parse_tx_events.rs](examples/parse_tx_events.rs) |
| Parse PumpFun transaction from RPC (signature: `TX_SIGNATURE` or CLI arg) | `cargo run --example parse_pump_tx --release` | [examples/parse_pump_tx.rs](examples/parse_pump_tx.rs) |
| Parse PumpFun quote-mint cases from RPC | `TX_SIGNATURES=<sig1,sig2> cargo run --example parse_pumpfun_quote_cases --release` | [examples/parse_pumpfun_quote_cases.rs](examples/parse_pumpfun_quote_cases.rs) |
| Parse PumpSwap transaction from RPC | `cargo run --example parse_pumpswap_tx --release` | [examples/parse_pumpswap_tx.rs](examples/parse_pumpswap_tx.rs) |
| Parse Meteora DAMM v2 transaction from RPC | `TX_SIGNATURE=<sig> cargo run --example parse_meteora_damm_tx --release` | [examples/parse_meteora_damm_tx.rs](examples/parse_meteora_damm_tx.rs) |
| Fetch any transaction and replay offline (preserves raw meta/logs/accounts) | `cargo run --example rpc_transaction_replay -- --fetch <SIGNATURE> /tmp/transaction.json` | [examples/rpc_transaction_replay.rs](examples/rpc_transaction_replay.rs) |
| Update filters at runtime | `cargo run --example dynamic_subscription` | [examples/dynamic_subscription.rs](examples/dynamic_subscription.rs) |
| Quick test: subscribe to PumpFun, print first 10 or run 60s | `cargo run --example pumpfun_quick_test --release` | [examples/pumpfun_quick_test.rs](examples/pumpfun_quick_test.rs) |
| PumpFun trade filter: Buy/Sell/Create with latency | `cargo run --example pumpfun_trade_filter --release` | [examples/pumpfun_trade_filter.rs](examples/pumpfun_trade_filter.rs) |
| PumpFun gRPC subscription with metrics | `cargo run --example pumpfun_with_metrics --release` | [examples/pumpfun_with_metrics.rs](examples/pumpfun_with_metrics.rs) |
| PumpSwap gRPC subscription with metrics | `cargo run --example pumpswap_with_metrics --release` | [examples/pumpswap_with_metrics.rs](examples/pumpswap_with_metrics.rs) |
| Meteora DAMM v2 gRPC subscription | `cargo run --example meteora_damm_grpc --release` | [examples/meteora_damm_grpc.rs](examples/meteora_damm_grpc.rs) |
| Monitor specific token account balance changes | `cargo run --example token_balance_listen_example` | [examples/token_balance_listen_example.rs](examples/token_balance_listen_example.rs) |
| Monitor token decimals via account subscription | `cargo run --example token_decimals_listen_example` | [examples/token_decimals_listen_example.rs](examples/token_decimals_listen_example.rs) |
| Track nonce account state changes | `cargo run --example nonce_listen_example` | [examples/nonce_listen_example.rs](examples/nonce_listen_example.rs) |
| Monitor PumpSwap pool accounts using memcmp filters | `cargo run --example pumpswap_pool_account_listen_example` | [examples/pumpswap_pool_account_listen_example.rs](examples/pumpswap_pool_account_listen_example.rs) |
| Monitor all associated token accounts for specific mints using memcmp filters | `cargo run --example mint_all_ata_account_listen_example` | [examples/mint_all_ata_account_listen_example.rs](examples/mint_all_ata_account_listen_example.rs) |

### Event Filtering

The library supports flexible event filtering to reduce processing overhead and improve performance:

#### Basic Filtering

```rust
use solana_streamer_sdk::streaming::event_parser::common::{filter::EventTypeFilter, EventType};

// No filtering - receive all events
let event_type_filter = None;

// Filter specific event types - only receive PumpSwap buy/sell events
let event_type_filter = Some(EventTypeFilter::include_only(vec![
    EventType::PumpSwapBuy,
    EventType::PumpSwapSell,
]));

// Exclude noisy events while keeping everything else
let event_type_filter = Some(EventTypeFilter::exclude_only(vec![EventType::BlockMeta]));
```

Transaction fee, priority-fee, and SWQoS tip parsing is strictly opt-in:

```rust
let event_type_filter = Some(EventTypeFilter::include_only(vec![
    EventType::PumpFunBuy,
    EventType::TransactionCost,
]));

let callback = |event: DexEvent| {
    if let DexEvent::TransactionCostEvent(cost) = event {
        println!("tips: {:?}", cost.tip_payments);
    }
};
```

`None`, `EventTypeFilter::default()`, and `EventTypeFilter::all()` do not enable transaction-cost
scanning. Each tip payment includes the matching SWQoS provider from the complete address registry
shared with `sol-trade-sdk`.

#### Performance Impact

Event filtering can provide significant performance improvements:
- **60-80% reduction** in unnecessary event processing
- **Lower memory usage** by filtering out irrelevant events
- **Reduced network bandwidth** in distributed setups
- **Better focus** on events that matter to your application

#### Filtering Examples by Use Case

**Trading Bot (Focus on Trade Events)**
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

**Pool Monitoring (Focus on Liquidity Events)**
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

`PumpSwapCreatePool` includes `is_mayhem_mode`. For `is_cashback_coin`,
ShredStream/outer-instruction parsing reads the flag from the `create_pool`
instruction args, while log-only `CreatePoolEvent` payloads keep the default
`false` because the log event IDL does not carry this field. The authoritative
account value is also available from
`PumpSwapPoolAccountEvent.pool.is_cashback_coin`.

<a id="trade-provenance-and-parsing-boundaries"></a>

## Trade provenance and parsing boundaries

gRPC/RPC use supplied execution metadata and actual CPI instructions; Shred sees outer instructions and locally resolved accounts. An unknown third-party instruction referencing Pump.fun in its own accounts can produce a `PumpFunTrade` candidate with the original input in `third_program` and `Unresolved` status. This does not establish execution, direction or amounts. Buy/sell-only filters do not treat an unknown direction as a fill.

The parsing hot path performs no RPC. Check `parse_status` before interpreting defaults as amounts. Original account order and instruction bytes remain in `third_program`; transaction position and receive time remain in event metadata. A local Shred index is not confirmed slot-wide execution order. See [wrapped transactions](docs/wrapped-pumpfun.md).

<a id="configuration-system"></a>

## ⚙️ Configuration System

You can customize client configuration:

```rust
use solana_streamer_sdk::streaming::{
    grpc::{ClientConfig, OrderMode},
    YellowstoneGrpc,
};

// Use default configuration
let grpc = YellowstoneGrpc::new(endpoint, token)?;

// Or create custom configuration
let mut config = ClientConfig::default();
config.enable_metrics = true;  // Enable performance monitoring
config.connection.connect_timeout = 30;  // 30 seconds
config.connection.request_timeout = 120;  // 120 seconds
config.order_mode = OrderMode::MicroBatch;  // Unordered / Ordered / StreamingOrdered / MicroBatch
config.order_timeout_ms = 100;
config.micro_batch_us = 100;

let grpc = YellowstoneGrpc::new_with_config(endpoint, token, config)?;
```

**Available Configuration Options:**
- `enable_metrics`: Enable/disable performance monitoring (default: false)
- `connection.connect_timeout`: Connection timeout in seconds (default: 10)
- `connection.request_timeout`: Request timeout in seconds (default: 60)
- `connection.max_decoding_message_size`: Maximum message size in bytes (default: 10MB)
- `order_mode`: Transaction event output ordering mode (default: `Unordered`)
- `order_timeout_ms`: Flush timeout for `Ordered` and `StreamingOrdered` modes (default: 100)
- `micro_batch_us`: Micro-batch window for `MicroBatch` mode (default: 100)

### Minimal gRPC Subscription

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

<a id="dynamic-subscription-management"></a>

## Dynamic Subscription Management

Update subscription filters at runtime without reconnecting to the stream.

```rust
// Update filters on existing subscription
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

- **No Reconnection**: Filter changes apply immediately without closing the stream
- **Atomic Updates**: Both transaction and account filters updated together
- **Single Subscription**: One active subscription per client instance
- **Compatible**: Works with both immediate and advanced subscription methods

Note: Multiple subscription attempts on the same client return an error.

<a id="supported-protocols"></a>

## 🔧 Supported Protocols

- **PumpFun**: Primary meme coin trading platform
- **Pump Fees**: Pump fee-sharing configuration events
- **PumpSwap**: PumpFun's swap protocol
- **LaunchLab**: Token launch platform; `Bonk` and `RaydiumLaunchpad` are kept as compatibility aliases
- **StonkFun**: StonkFun-attributed LaunchLab trade and pool-create events
- **Raydium CPMM**: Raydium's Constant Product Market Maker protocol
- **Raydium CLMM**: Raydium's Concentrated Liquidity Market Maker protocol
- **Raydium AMM V4**: Raydium's Automated Market Maker V4 protocol
- **Meteora DAMM v2**: Meteora DAMM v2 protocol
- **Orca Whirlpool**: Orca Whirlpool protocol
- **Meteora Pools**: Meteora Pools protocol
- **Meteora DBC**: Meteora Dynamic Bonding Curve protocol
- **Meteora DLMM**: Meteora Dynamic Liquidity Market Maker protocol
- **Common/account events**: Token accounts, token metadata, nonce accounts, block metadata, ComputeBudget events, and supported protocol account states such as Raydium CLMM/CPMM, Pump/PumpSwap, and Orca Whirlpool accounts

<a id="event-streaming-services"></a>

## 🌐 Event Streaming Services

- **Yellowstone gRPC**: High-performance Solana event streaming
- **ShredStream**: Alternative event streaming protocol

<a id="architecture-features"></a>

## 🏗️ Architecture Features

### Unified Event Interface

- **DexEvent Enum**: Type-safe enum containing all protocol events
- **Protocol Enum**: Easy identification of event sources
- **SDK Bridge**: Adapts `sol-parser-sdk::DexEvent` into streamer `DexEvent`

### Event Parsing System

- **sol-parser-sdk Facade**: Yellowstone gRPC, ShredStream, RPC transaction parsing, and account parsing delegate protocol parsing to `sol-parser-sdk`
- **Local Non-DEX Pass**: Local handling is limited to streamer infrastructure and non-DEX compatibility cases such as ComputeBudget metadata
- **Extensible Bridge**: `streaming::sdk_bridge` exposes raw SDK access without forcing existing bots to change callbacks

### Streaming Infrastructure

- **Yellowstone gRPC Client**: Optimized for Solana event streaming
- **ShredStream Client**: Alternative streaming implementation
- **Async Processing**: Non-blocking event handling

<a id="project-structure"></a>

## 📁 Project Structure

```
src/
├── common/           # Common functionality and types
├── protos/           # Protocol buffer definitions
├── streaming/        # Event streaming system
│   ├── event_parser/ # Streamer-compatible event facade over sol-parser-sdk
│   │   ├── common/   # Public event metadata and filter types
│   │   ├── core/     # SDK dispatch entry points and compatibility wrappers
│   │   ├── protocols/# Streamer event types and legacy module paths
│   │   │   └── sol_parser_forward/ # SDK-forwarded protocol event wrappers
│   ├── parser_sdk_bridge/ # sol-parser-sdk event adapter
│   ├── rpc_parse.rs # RPC transaction parsing helpers
│   ├── sdk_bridge.rs # Public advanced SDK interop module
│   ├── shred_stream.rs # ShredStream client
│   ├── yellowstone_grpc.rs # Yellowstone gRPC client
│   └── yellowstone_sub_system.rs # Yellowstone subsystem
├── lib.rs            # Main library file
└── main.rs           # Example program
```

<a id="performance-considerations"></a>

## ⚡ Performance Considerations

1. **Connection Management**: Properly handle connection lifecycle and reconnection
2. **Event Filtering**: Use protocol filtering to reduce unnecessary event processing
3. **Memory Management**: Implement appropriate cleanup for long-running streams
4. **Error Handling**: Robust error handling for network issues and service interruptions
5. **Batch Processing Optimization**: Use batch processing to reduce callback overhead and improve throughput
6. **Performance Monitoring**: Enable performance monitoring to identify bottlenecks and optimization opportunities
7. **Graceful Shutdown**: Use the stop() method for clean shutdown and implement signal handlers for proper resource cleanup

<a id="important-notes"></a>

## ⚠️ Important Notes

1. **Network Stability**: Ensure stable network connection for continuous event streaming
2. **Rate Limiting**: Be aware of rate limits on public gRPC endpoints
3. **Error Recovery**: Implement proper error handling and reconnection logic
5. **Compliance**: Ensure compliance with relevant laws and regulations

<a id="pump-upgrade-october-2026"></a>

## Pump upgrade (October 2026)

Compact Pump v3 and PumpSwap v2 trades use their new 17-account layouts. New typed events cover `PumpFunPostCompleteBuy`, `PumpFunComplete`, `PumpFunSweepBondingCurveFee` and `PumpSwapSweepPoolFee`, with program-scoped log and CPI parsing. Historical SOL CompleteEvent payloads remain supported. Curve/pool retained fees and synthetic counters are exposed; optional historical tails default to zero.

For completing buys, aggregate TradeEvent and PostCompleteBuyEvent amounts within the same invocation. CompleteEvent is a notification. Multi-hop execution retains each venue’s trade events in order; separate venues are not merged into one fill. There is no route-intent decoder or raw-event callback. Shred candidates carry original wrapper input in third_program and remain unresolved; gRPC/RPC uses actual execution metadata.

Reference: [pump-public-docs](https://github.com/pump-fun/pump-public-docs/tree/8cda1fa30ea658b20909d8aedf002047119388d2). Validation uses offline official IDL fixtures; no live trade is sent by the tests.

<a id="reproducible-local-hot-path-latency"></a>

## Reproducible local hot-path latency

Run the native Rust example with a configuration file (no RPC):

```sh
cargo run --release --example hotpath_latency -- examples/hotpath_latency.json
```

The JSON configuration controls `warmup`, `samples`, and `batch`. Output includes
p50/p95/p99 nanoseconds per operation and the build profile. Use `--release` for
performance comparisons and run on an otherwise idle machine. Re-run with the
same configuration and fixtures before comparing changes.

The workload measures the public Pump trade event bridge, with and without a
recent blockhash. Input cloning happens outside the measured interval; output
destruction is included. This is CPU conversion latency, not network latency.

<a id="documentation"></a>

## Documentation

- [Release and migration](RELEASE_NOTES.md)
- [Usage guide](docs/wrapped-pumpfun.md)

<a id="language-versions"></a>

## Language Versions

- [English](README.md)
- [中文](README_CN.md)


<a id="license"></a>

## 📄 License

MIT License

<a id="contact"></a>

## 📞 Contact

- **Website**: https://fnzero.dev/
- **Project Repository**: https://github.com/0xfnzero/solana-streamer
- **Telegram Group**: https://t.me/fnzero_group
- **Discord**: https://discord.gg/vuazbGkqQE


Shared stock bridges: 33 official stock/ETF mints and 58 pools. See [eleven-SDK catalog and routing alignment](docs/STOCK_BRIDGE_SYNC.md) for configuration, USDC routing and cache-only boundaries.
