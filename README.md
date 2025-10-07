<div align="center">
    <h1>🌊 Solana Streamer</h1>
    <h3><em>Real-time event streaming from Solana DEX trading programs.</em></h3>
</div>

<p align="center">
    <strong>A lightweight Rust library providing efficient event parsing and subscription capabilities for PumpFun, PumpSwap, Bonk, and Raydium protocols.</strong>
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

---

## Table of Contents

- [🚀 Project Features](#-project-features)
- [⚡ Installation](#-installation)
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
- **Yellowstone gRPC Support**: High-performance event subscription using Yellowstone gRPC
- **ShredStream Support**: Alternative event streaming using ShredStream protocol
- **Unified Event Interface**: Consistent event handling across all supported protocols

### Multi-Protocol Support
- **PumpFun**: Meme coin trading platform events
- **PumpSwap**: PumpFun's swap protocol events
- **Bonk**: Token launch platform events (letsbonk.fun)
- **Raydium CPMM**: Raydium's Concentrated Pool Market Maker events
- **Raydium CLMM**: Raydium's Concentrated Liquidity Market Maker events
- **Raydium AMM V4**: Raydium's Automated Market Maker V4 events

### Advanced Features
- **Event Parsing System**: Automatic parsing and categorization of protocol-specific events
- **Account State Monitoring**: Real-time monitoring of protocol account states and configuration changes
- **Transaction & Account Event Filtering**: Separate filtering for transaction events and account state changes
- **Dynamic Subscription Management**: Runtime filter updates without reconnection, enabling adaptive monitoring strategies
- **Multi-Filter Support**: Support for multiple transaction and account filters in a single subscription
- **Advanced Account Filtering**: Memcmp filters for precise account data matching and monitoring
- **Token2022 Support**: Enhanced support for SPL Token 2022 with extended state parsing

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
solana-streamer-sdk = { path = "./solana-streamer", version = "0.5.0" }
```

### Use crates.io

```toml
# Add to your Cargo.toml
solana-streamer-sdk = "0.5.0"
```

## ⚙️ Configuration System

### Preset Configurations

The library provides three preset configurations optimized for different use cases:

#### 1. High Throughput Configuration (`high_throughput()`)

Optimized for high-concurrency scenarios, prioritizing throughput over latency:

```rust
let config = StreamClientConfig::high_throughput();
// Or use convenience methods
let grpc = YellowstoneGrpc::new_high_throughput(endpoint, token)?;
let shred = ShredStreamGrpc::new_high_throughput(endpoint).await?;
```

**Features:**
- **Backpressure Strategy**: Drop - drops messages during high load to avoid blocking
- **Buffer Size**: 5,000 permits to handle burst traffic
- **Use Case**: Scenarios where you need to process large volumes of data and can tolerate occasional message drops during peak loads

#### 2. Low Latency Configuration (`low_latency()`)

Optimized for real-time scenarios, prioritizing latency over throughput:

```rust
let config = StreamClientConfig::low_latency();
// Or use convenience methods
let grpc = YellowstoneGrpc::new_low_latency(endpoint, token)?;
let shred = ShredStreamGrpc::new_low_latency(endpoint).await?;
```

**Features:**
- **Backpressure Strategy**: Block - ensures no data loss
- **Buffer Size**: 4000 permits for balanced throughput and latency
- **Immediate Processing**: No buffering, processes events immediately
- **Use Case**: Scenarios where every millisecond counts and you cannot afford to lose any events, such as trading applications or real-time monitoring


### Custom Configuration

You can also create custom configurations:

```rust
let config = StreamClientConfig {
    connection: ConnectionConfig {
        connect_timeout: 30,
        request_timeout: 120,
        max_decoding_message_size: 20 * 1024 * 1024, // 20MB
    },
    backpressure: BackpressureConfig {
        permits: 2000,
        strategy: BackpressureStrategy::Block,
    },
    enable_metrics: true,
};
```

## 📚 Usage Examples

### Usage Examples Summary Table

| Description | Run Command | Source Path |
|------|---------|----------|
| Monitor transaction events using Yellowstone gRPC | `cargo run --example grpc_example` | [examples/grpc_example.rs](examples/grpc_example.rs) |
| Monitor transaction events using ShredStream | `cargo run --example shred_example` | [examples/shred_example.rs](examples/shred_example.rs) |
| Parse Solana mainnet transaction data | `cargo run --example parse_tx_events` | [examples/parse_tx_events.rs](examples/parse_tx_events.rs) |
| Update filters at runtime | `cargo run --example dynamic_subscription` | [examples/dynamic_subscription.rs](examples/dynamic_subscription.rs) |
| Monitor specific token account balance changes | `cargo run --example token_balance_listen_example` | [examples/token_balance_listen_example.rs](examples/token_balance_listen_example.rs) |
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
let event_type_filter = Some(EventTypeFilter { 
    include: vec![EventType::PumpSwapBuy, EventType::PumpSwapSell] 
});
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
let event_type_filter = Some(EventTypeFilter { 
    include: vec![
        EventType::PumpSwapBuy,
        EventType::PumpSwapSell,
        EventType::PumpFunTrade,
        EventType::RaydiumCpmmSwap,
        EventType::RaydiumClmmSwap,
        EventType::RaydiumAmmV4Swap,
        ......
    ] 
});
```

**Pool Monitoring (Focus on Liquidity Events)**
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
- **PumpSwap**: PumpFun's swap protocol
- **Bonk**: Token launch platform (letsbonk.fun)
- **Raydium CPMM**: Raydium's Concentrated Pool Market Maker protocol
- **Raydium CLMM**: Raydium's Concentrated Liquidity Market Maker protocol
- **Raydium AMM V4**: Raydium's Automated Market Maker V4 protocol

## 🌐 Event Streaming Services

- **Yellowstone gRPC**: High-performance Solana event streaming
- **ShredStream**: Alternative event streaming protocol

## 🏗️ Architecture Features

### Unified Event Interface

- **UnifiedEvent Trait**: All protocol events implement a common interface
- **Protocol Enum**: Easy identification of event sources
- **Event Factory**: Automatic event parsing and categorization

### Event Parsing System

- **Protocol-specific Parsers**: Dedicated parsers for each supported protocol
- **Event Factory**: Centralized event creation and parsing
- **Extensible Design**: Easy to add new protocols and event types

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
│   ├── event_parser/ # Event parsing system
│   │   ├── common/   # Common event parsing tools
│   │   ├── core/     # Core parsing traits and interfaces
│   │   ├── protocols/# Protocol-specific parsers
│   │   │   ├── bonk/ # Bonk event parsing
│   │   │   ├── pumpfun/ # PumpFun event parsing
│   │   │   ├── pumpswap/ # PumpSwap event parsing
│   │   │   ├── raydium_amm_v4/ # Raydium AMM V4 event parsing
│   │   │   ├── raydium_cpmm/ # Raydium CPMM event parsing
│   │   │   └── raydium_clmm/ # Raydium CLMM event parsing
│   │   └── factory.rs # Parser factory
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
