# 迁移指南：v0.5.x 到 v1.0.0

## 概述

版本 0.6.0 引入了一个重要的架构改进，从基于 trait 的事件系统过渡到基于 enum 的事件系统。此变更带来：

- **更好的类型安全**: 编译时保证事件类型
- **改进的性能**: 消除动态分发开销（不再使用 `Box<dyn Trait>`）
- **更简单的代码**: 标准 Rust 模式而不是自定义宏
- **更好的 IDE 支持**: 完整的自动补全和类型推断

## 破坏性变更摘要

| 组件 | v0.5.x | v1.0.0 |
|-----------|--------|--------|
| 事件类型 | `Box<dyn UnifiedEvent>` | `DexEvent`（枚举） |
| 回调签名 | `Fn(Box<dyn UnifiedEvent>)` | `Fn(DexEvent)` |
| 事件匹配 | `match_event!` 宏 | 标准 `match` 表达式 |
| 元数据访问 | `.event_type()` | `.metadata().event_type` |
| 事件属性 | `.signature()` | `.metadata().signature` |

## 迁移步骤

### 步骤 1：更新回调签名

**之前 (v0.5.x):**
```rust
use solana_streamer_sdk::streaming::event_parser::UnifiedEvent;

let callback = |event: Box<dyn UnifiedEvent>| {
    println!("接收到事件: {:?}", event);
};
```

**之后 (v1.0.0):**
```rust
use solana_streamer_sdk::streaming::event_parser::DexEvent;

let callback = |event: DexEvent| {
    println!("接收到事件: {:?}", event);
};
```

### 步骤 2：更新事件匹配

**之前 (v0.5.x):**
```rust
use solana_streamer_sdk::match_event;

match_event!(event, {
    PumpFunTradeEvent => |e: PumpFunTradeEvent| {
        println!("PumpFun 交易: {:?}", e);
    },
    RaydiumCpmmSwapEvent => |e: RaydiumCpmmSwapEvent| {
        println!("Raydium 交换: {:?}", e);
    },
});
```

**之后 (v1.0.0):**
```rust
match event {
    DexEvent::PumpFunTradeEvent(e) => {
        println!("PumpFun 交易: {:?}", e);
    }
    DexEvent::RaydiumCpmmSwapEvent(e) => {
        println!("Raydium 交换: {:?}", e);
    }
    _ => {}
}
```

### 步骤 3：更新元数据访问

**之前 (v0.5.x):**
```rust
let event_type = event.event_type();
let signature = event.signature();
let slot = event.slot();
let protocol = event.protocol();
```

**之后 (v1.0.0):**
```rust
let event_type = event.metadata().event_type;
let signature = event.metadata().signature;
let slot = event.metadata().slot;
let protocol = event.metadata().protocol;
```

### 步骤 4：更新导入语句

**之前 (v0.5.x):**
```rust
use solana_streamer_sdk::{
    match_event,
    streaming::event_parser::{
        UnifiedEvent,
        protocols::{
            pumpfun::{PumpFunTradeEvent, PumpFunCreateTokenEvent},
            raydium_cpmm::{RaydiumCpmmSwapEvent},
        },
    },
};
```

**之后 (v1.0.0):**
```rust
use solana_streamer_sdk::streaming::event_parser::{
    DexEvent,
    protocols::{
        pumpfun::{PumpFunTradeEvent, PumpFunCreateTokenEvent},
        raydium_cpmm::{RaydiumCpmmSwapEvent},
    },
};
```

注意：`match_event!` 宏不再需要或可用。

## 完整示例迁移

### 之前 (v0.5.x)

```rust
use solana_streamer_sdk::{
    match_event,
    streaming::{
        event_parser::{
            UnifiedEvent,
            protocols::{
                pumpfun::{PumpFunTradeEvent, PumpFunCreateTokenEvent},
                raydium_cpmm::RaydiumCpmmSwapEvent,
            },
        },
        YellowstoneGrpc,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let grpc = YellowstoneGrpc::new(
        "grpc-endpoint".to_string(),
        Some("api-key".to_string()),
    )?;

    let callback = |event: Box<dyn UnifiedEvent>| {
        println!(
            "事件类型: {:?}, 签名: {}",
            event.event_type(),
            event.signature()
        );

        match_event!(event, {
            PumpFunTradeEvent => |e: PumpFunTradeEvent| {
                println!("PumpFun 交易: {} SOL", e.sol_amount);
            },
            PumpFunCreateTokenEvent => |e: PumpFunCreateTokenEvent| {
                println!("新代币: {}", e.name);
            },
            RaydiumCpmmSwapEvent => |e: RaydiumCpmmSwapEvent| {
                println!("Raydium 交换");
            },
        });
    };

    grpc.subscribe_events(
        protocols,
        event_filter,
        tx_filter,
        account_filter,
        callback,
    ).await?;

    Ok(())
}
```

### 之后 (v1.0.0)

```rust
use solana_streamer_sdk::streaming::{
    event_parser::{
        DexEvent,
        protocols::{
            pumpfun::{PumpFunTradeEvent, PumpFunCreateTokenEvent},
            raydium_cpmm::RaydiumCpmmSwapEvent,
        },
    },
    YellowstoneGrpc,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let grpc = YellowstoneGrpc::new(
        "grpc-endpoint".to_string(),
        Some("api-key".to_string()),
    )?;

    let callback = |event: DexEvent| {
        println!(
            "事件类型: {:?}, 签名: {}",
            event.metadata().event_type,
            event.metadata().signature
        );

        match event {
            DexEvent::PumpFunTradeEvent(e) => {
                println!("PumpFun 交易: {} SOL", e.sol_amount);
            }
            DexEvent::PumpFunCreateTokenEvent(e) => {
                println!("新代币: {}", e.name);
            }
            DexEvent::RaydiumCpmmSwapEvent(e) => {
                println!("Raydium 交换");
            }
            _ => {}
        }
    };

    grpc.subscribe_events(
        protocols,
        event_filter,
        tx_filter,
        account_filter,
        callback,
    ).await?;

    Ok(())
}
```

## 高级模式

### 模式 1：使用 Match 进行事件过滤

**v1.0.0:**
```rust
let callback = |event: DexEvent| {
    // 只处理特定事件类型
    match event {
        DexEvent::PumpFunTradeEvent(e) if e.is_buy => {
            println!("买入: {} 代币", e.token_amount);
        }
        DexEvent::PumpFunTradeEvent(e) if !e.is_buy => {
            println!("卖出: {} 代币", e.token_amount);
        }
        _ => {} // 忽略其他事件
    }
};
```

### 模式 2：通用事件处理

**v1.0.0:**
```rust
fn process_event(event: DexEvent) {
    let metadata = event.metadata();

    println!("协议: {:?}", metadata.protocol);
    println!("事件类型: {:?}", metadata.event_type);
    println!("签名: {}", metadata.signature);
    println!("插槽: {}", metadata.slot);

    // 处理特定事件类型
    match event {
        DexEvent::PumpFunTradeEvent(e) => handle_pumpfun_trade(e),
        DexEvent::RaydiumCpmmSwapEvent(e) => handle_raydium_swap(e),
        _ => {}
    }
}
```

### 模式 3：事件类型分类

**v1.0.0:**
```rust
fn categorize_event(event: &DexEvent) -> &'static str {
    match event {
        DexEvent::PumpFunTradeEvent(_)
        | DexEvent::PumpSwapBuyEvent(_)
        | DexEvent::PumpSwapSellEvent(_) => "交易",

        DexEvent::PumpFunCreateTokenEvent(_)
        | DexEvent::PumpSwapCreatePoolEvent(_) => "创建",

        DexEvent::RaydiumCpmmDepositEvent(_)
        | DexEvent::RaydiumCpmmWithdrawEvent(_) => "流动性",

        _ => "其他",
    }
}
```

## EventParser API 变更

### 解析交易

**之前 (v0.5.x):**
```rust
let parser = Arc::new(EventParser::new(protocols, event_filter));
parser.parse_encoded_confirmed_transaction_with_status_meta(
    signature,
    transaction,
    Arc::new(|event: &Box<dyn UnifiedEvent>| {
        println!("{:?}", event);
    }),
).await?;
```

**之后 (v1.0.0):**
```rust
EventParser::parse_encoded_confirmed_transaction_with_status_meta(
    &protocols,
    event_filter.as_ref(),
    signature,
    transaction,
    Arc::new(|event: &DexEvent| {
        println!("{:?}", event);
    }),
).await?;
```

`EventParser` 现在是无状态的，使用静态方法，无需创建实例。

## 常见陷阱

### 陷阱 1：忘记处理所有变体

❌ **错误:**
```rust
match event {
    DexEvent::PumpFunTradeEvent(e) => { /* ... */ }
    // 缺少其他变体！
}
```

✅ **正确:**
```rust
match event {
    DexEvent::PumpFunTradeEvent(e) => { /* ... */ }
    _ => {} // 处理或忽略其他事件
}
```

### 陷阱 2：使用旧的元数据访问模式

❌ **错误:**
```rust
let sig = event.signature(); // 此方法不再存在
```

✅ **正确:**
```rust
let sig = event.metadata().signature;
```

### 陷阱 3：尝试使用 `match_event!` 宏

❌ **错误:**
```rust
match_event!(event, { /* ... */ }); // 宏不再存在
```

✅ **正确:**
```rust
match event {
    DexEvent::PumpFunTradeEvent(e) => { /* ... */ }
    _ => {}
}
```

## 新系统的优势

1. **类型安全**: 编译器在编译时捕获更多错误
2. **性能**: 无动态分发开销
3. **简洁性**: 标准 Rust 模式，无自定义宏
4. **更好的工具支持**: 完整的 IDE 自动补全支持
5. **更易调试**: 更清晰的堆栈跟踪和错误消息
6. **序列化**: 所有事件内置 `Serialize`/`Deserialize` 支持
