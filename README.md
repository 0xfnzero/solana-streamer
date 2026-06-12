<div align="center">
    <h1>🌊 Solana Streamer</h1>
    <h3><em>Real-time event streaming from Solana DEX trading programs.</em></h3>
</div>

<p align="center">
    <strong>A lightweight Rust streaming facade over sol-parser-sdk, with low-latency subscriptions and a stable bot-facing API.</strong>
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
    <a href="https://t.me/fnzero_group">Telegram</a> |
    <a href="https://discord.gg/vuazbGkqQE">Discord</a>
</p>

> ☕ **Support This Project**
>
> This SDK is completely free and open source. However, maintaining and continuously updating it requires significant AI computing resources and token consumption. If this SDK helps with your development, consider making a monthly SOL donation — any amount is appreciated and helps keep this project alive!
>
> **Donation Wallet:** `6oW7AXz1yRb57pYSxysuXnMs2aR1ha5rzGzReZ1MjPV8`

---

## Table of Contents

- [🚀 Project Features](#-project-features)
- [⚡ Installation](#-installation)
- [🔄 Migration Guide](#-migration-guide)
- [⚙️ Configuration System](#️-configuration-system)
- [📚 Usage Examples](#-usage-examples)
- [🔧 Supported Protocols](#-supported-protocols)
- [🌐 Event Streaming Services](#-event-streaming-services)
- [🏗️ Architecture Features](#️-architecture-features)
- [📁 Project Structure](#-project-structure)
- [⚡ Performance Considerations](#-performance-considerations)
- [📄 License](#-license)
- [📞 Contact](#-contact)
- [⚠️ Important Notes](#️-important-notes)

## 🚀 Project Features

### Core Capabilities
- **Real-time Event Streaming**: Subscribe to live trading events from multiple Solana DEX protocols
- **SDK-backed Parser Core**: Transaction, RPC, account, and ShredStream parsing are backed by `sol-parser-sdk`
- **Yellowstone gRPC Support**: High-performance event subscription using Yellowstone gRPC
- **ShredStream Support**: Alternative event streaming using ShredStream protocol; ALT-loaded accounts are parsed best-effort with default account placeholders
- **Unified Event Interface**: Consistent event handling across all supported protocols

### Multi-Protocol Support
- **PumpFun**: Meme coin trading platform events
- **Pump Fees**: Pump fee-sharing configuration events
- **PumpSwap**: PumpFun's swap protocol events
- **Raydium LaunchLab**: Token launch platform events; `Protocol::Bonk` and `Protocol::RaydiumLaunchpad` remain compatible aliases
- **Raydium CPMM**: Raydium's Concentrated Pool Market Maker events
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

## ⚡ Installation

### Direct Clone

Clone this project to your project directory:

```bash
cd your_project_root_directory
git clone https://github.com/0xfnzero/solana-streamer
```

Add the dependency to your `Cargo.toml`:

```toml
# Add to your Cargo.toml
solana-streamer-sdk = { path = "./solana-streamer", version = "1.5.15" }
```

### Use crates.io

```toml
# Add to your Cargo.toml
solana-streamer-sdk = "1.5.15"
```

Parser backend features:

```toml
# Default: sol-parser-sdk parse-borsh backend
solana-streamer-sdk = "1.5.15"

# Zero-copy parser backend for latency-sensitive bots
solana-streamer-sdk = { version = "1.5.15", default-features = false, features = ["sdk-parse-zero-copy"] }
```

If both `sdk-parse-borsh` and `sdk-parse-zero-copy` are enabled, `sol-parser-sdk 0.5.15+` uses the zero-copy backend.

## 🔄 Migration Guide

### Upgrading to v1.5.15

Version 1.5.15 tracks `sol-parser-sdk 0.5.15` at GitHub rev `4464880`. Pump.fun `create_v2` now distinguishes 16-account SOL-sentinel creates from 19-account quote-pool creates, and account filling selects the actual create/create_v2 instruction before reading quote fields.

### Upgrading to v1.5.14

Version 1.5.14 uses `sol-parser-sdk 0.5.14` from crates.io. Pump.fun canonical create events now expose `quote_mint`, `quote_vault`, and `quote_token_program` for `create_v2` quote pools, including USDC pools, across gRPC/RPC parser bridge and ShredStream-backed SDK output.

### Upgrading to v1.5.13

Version 1.5.13 uses `sol-parser-sdk 0.5.13` from crates.io. Pump.fun gRPC and ShredStream create/trade outputs now preserve real WSOL quote mints (`So11111111111111111111111111111111111111112`). The Solscan SOL sentinel (`So11111111111111111111111111111111111111111`) is used only when legacy data omits a quote mint.

### Upgrading to v1.5.11

Version 1.5.11 uses `sol-parser-sdk 0.5.11` from crates.io. PumpSwap `PumpSwapCreatePoolEvent` now carries `is_cashback_coin` when it is available from the `create_pool` instruction args, including ShredStream outer-instruction parsing. Log-only `CreatePoolEvent` payloads still default this field to `false` because the on-chain log event IDL does not carry it. `AccountPumpSwapPool` remains the authoritative account-state source for the pool flag.

### Upgrading to v1.5.10

Version 1.5.10 uses `sol-parser-sdk 0.5.10` from crates.io. PumpSwap `CreatePoolEvent` now matches the on-chain IDL: it exposes `is_mayhem_mode` but does not expose `is_cashback_coin`. To read the cashback flag, subscribe to `AccountPumpSwapPool` and use `PumpSwapPoolAccountEvent.pool.is_cashback_coin`. The PumpSwap CreatePool log payload length check now includes the final `is_mayhem_mode` byte.

### Upgrading to v1.5.9

Version 1.5.9 uses `sol-parser-sdk 0.5.9` from crates.io. It inherits the Yellowstone gRPC stop lifecycle fix: `stop()` now signals, aborts, and awaits the active subscription task, subscription lifecycle transitions are serialized, and gRPC stream errors are labeled as `Grpc Stream error` for easier log separation from ShredStream.

### Upgrading to v1.5.8

Version 1.5.8 uses `sol-parser-sdk 0.5.8` from crates.io. It inherits the Pump.fun ShredStream filter-family semantics from the parser SDK: `PumpFunBuy` covers `buy`, `buy_v2`, `buy_exact_sol_in`, and `buy_exact_quote_in_v2`; `PumpFunSell` covers `sell` and `sell_v2`; and `PumpFunTrade` covers all buy/sell instructions while the parser can emit unified trade events when only the generic trade filter is requested.

### Upgrading to v1.5.5

Version 1.5.5 uses `sol-parser-sdk 0.5.5` from crates.io. The SDK now exposes Raydium LaunchLab as `RaydiumLaunchlab*`; streamer keeps the existing `Bonk*` event structs and `Protocol::Bonk` / `Protocol::RaydiumLaunchpad` aliases for source compatibility, while routing parser calls and upstream gRPC event filters to the new LaunchLab SDK variants. This release also syncs the CLMM/CPMM/Orca account bridges, Meteora DAMM v2 initialize-pool events, Meteora DBC events, and parser warmup on client creation.

### Upgrading to v1.5.4

Version 1.5.4 uses `sol-parser-sdk 0.5.4` from crates.io. Pump.fun `create` and `create_v2` are delivered as one canonical `PumpFunCreateTokenEvent`; subscribing to either `PumpFunCreateToken` or `PumpFunCreateV2Token` receives the same complete create-family data. This prevents duplicate new-mint callbacks from gRPC log + instruction parsing while preserving create_v2 account fields on the canonical create event.

### Upgrading to v1.5.3

Version 1.5.3 uses `sol-parser-sdk 0.5.3` from crates.io. It preserves real Pump.fun v2 `ix_name` values through the streamer bridge, improves ShredStream Pump.fun v2 best-effort parsing for short account lists, and treats `PumpFunBuy` and `PumpFunBuyExactSolIn` subscriptions as compatible buy-family filters. ShredStream still uses the direct entry-reading path with automatic reconnect; callbacks should remain non-blocking and `tx_index` remains entry-local best-effort.

### Upgrading to v1.5.2

Version 1.5.2 uses `sol-parser-sdk 0.5.2` from crates.io. ShredStream delivery uses the SDK parser on a direct entry-reading path with automatic reconnect, avoiding the extra queue consumer task while keeping parser event buffers reused. ShredStream `tx_index` is entry-local best-effort, not the Yellowstone slot-level transaction index. User callbacks on the direct path run on the read task and should avoid blocking work.

### Upgrading to v1.5.1

Version 1.5.1 uses `sol-parser-sdk 0.5.1` from crates.io. It improves ShredStream ALT handling by parsing outer instructions best-effort with default placeholders for ALT-loaded accounts, adds dropped-event queue observability, and keeps CPI/inner-only events documented as a ShredStream limitation.

### Upgrading to v1.5.0

Version 1.5.0 uses `sol-parser-sdk 0.5.0`, wires Raydium CLMM account parsing through the SDK bridge, and reduces ordered-buffer allocation/move overhead in low-latency delivery paths. `sol-parser-sdk 0.4.19` and `solana-streamer-sdk 1.4.14` were superseded because adding public parser event variants belongs in the 0.5 line.

### Upgrading to v1.4.14

Version 1.4.14 was superseded by v1.5.0.

### Upgrading to v1.4.13

Version 1.4.13 uses `sol-parser-sdk 0.4.18` and updates Raydium CLMM integration to the official upgraded IDL: current log-side event discriminators, official Swap/Liquidity/Create/Collect layouts, limit-order events, dynamic-fee related events, and the reshaped PoolState/TickState account structs.

### Upgrading to v1.4.12

Version 1.4.12 uses `sol-parser-sdk 0.4.17` and normalizes legacy PumpFun SOL quote mints to the Solscan SOL sentinel while preserving real USDC quote mints and quote-reserve fields. Streamer bridge and merger paths keep the parser SDK semantics aligned for Yellowstone gRPC and ShredStream output.

New optional capabilities:

- `solana_streamer_sdk::parser_sdk` re-exports the raw `sol-parser-sdk` crate.
- `solana_streamer_sdk::sdk_bridge` adapts raw SDK events back into streamer `DexEvent`.
- `fetch_rpc_transaction_as_streamer_events` and `parse_encoded_rpc_transaction_as_streamer_events` parse RPC transactions into streamer events.
- `grpc::ClientConfig::order_mode` supports `Unordered`, `Ordered`, `StreamingOrdered`, and `MicroBatch`.
- `sdk-parse-zero-copy` enables the SDK zero-copy parser backend.

### Migrating from v0.5.x to v1.x.x

Version 1.0.0 introduces a major architectural change from trait-based event handling to enum-based events. This provides better type safety, improved performance, and simpler code patterns.

**Key Changes:**

1. **Event Type Changed** - `Box<dyn UnifiedEvent>` → `DexEvent` enum
2. **Callback Signature** - Callbacks now receive concrete `DexEvent` instead of trait objects
3. **Event Matching** - Use standard Rust `match` instead of `match_event!` macro
4. **Metadata Access** - Event properties now accessed through `.metadata()` method

For detailed migration steps and code examples, see [MIGRATION.md](MIGRATION.md) or [MIGRATION_CN.md](MIGRATION_CN.md) (Chinese version).

**Quick Migration Example:**

```rust
// Old (v0.5.x)
let callback = |event: Box<dyn UnifiedEvent>| {
    println!("Event: {:?}", event.event_type());
};

// New (v1.x.x)
let callback = |event: DexEvent| {
    println!("Event: {:?}", event.metadata().event_type);
};
```

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
| Debug PumpFun transaction (fetch, print meta/logs, parse) | `TX_SIGNATURE=<sig> cargo run --example debug_pump_tx --release` | [examples/debug_pump_tx.rs](examples/debug_pump_tx.rs) |
| Debug PumpSwap transaction (fetch, print meta, parse) | `TX_SIGNATURE=<sig> cargo run --example debug_pumpswap_tx --release` | [examples/debug_pumpswap_tx.rs](examples/debug_pumpswap_tx.rs) |
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

## 🔧 Supported Protocols

- **PumpFun**: Primary meme coin trading platform
- **Pump Fees**: Pump fee-sharing configuration events
- **PumpSwap**: PumpFun's swap protocol
- **Raydium LaunchLab**: Token launch platform; `Bonk` and `RaydiumLaunchpad` are kept as compatibility aliases
- **Raydium CPMM**: Raydium's Concentrated Pool Market Maker protocol
- **Raydium CLMM**: Raydium's Concentrated Liquidity Market Maker protocol
- **Raydium AMM V4**: Raydium's Automated Market Maker V4 protocol
- **Meteora DAMM v2**: Meteora DAMM v2 protocol
- **Orca Whirlpool**: Orca Whirlpool protocol
- **Meteora Pools**: Meteora Pools protocol
- **Meteora DBC**: Meteora Dynamic Bonding Curve protocol
- **Meteora DLMM**: Meteora Dynamic Liquidity Market Maker protocol
- **Common/account events**: Token accounts, token metadata, nonce accounts, block metadata, ComputeBudget events, and supported protocol account states such as Raydium CLMM/CPMM, Pump/PumpSwap, and Orca Whirlpool accounts

## 🌐 Event Streaming Services

- **Yellowstone gRPC**: High-performance Solana event streaming
- **ShredStream**: Alternative event streaming protocol

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

## ⚡ Performance Considerations

1. **Connection Management**: Properly handle connection lifecycle and reconnection
2. **Event Filtering**: Use protocol filtering to reduce unnecessary event processing
3. **Memory Management**: Implement appropriate cleanup for long-running streams
4. **Error Handling**: Robust error handling for network issues and service interruptions
5. **Batch Processing Optimization**: Use batch processing to reduce callback overhead and improve throughput
6. **Performance Monitoring**: Enable performance monitoring to identify bottlenecks and optimization opportunities
7. **Graceful Shutdown**: Use the stop() method for clean shutdown and implement signal handlers for proper resource cleanup

---

## 📄 License

MIT License

## 📞 Contact

- **Website**: https://fnzero.dev/
- **Project Repository**: https://github.com/0xfnzero/solana-streamer
- **Telegram Group**: https://t.me/fnzero_group
- **Discord**: https://discord.gg/vuazbGkqQE

## ⚠️ Important Notes

1. **Network Stability**: Ensure stable network connection for continuous event streaming
2. **Rate Limiting**: Be aware of rate limits on public gRPC endpoints
3. **Error Recovery**: Implement proper error handling and reconnection logic
5. **Compliance**: Ensure compliance with relevant laws and regulations

## Language Versions

- [English](README.md)
- [中文](README_CN.md)
