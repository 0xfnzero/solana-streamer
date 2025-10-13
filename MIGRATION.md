# Migration Guide: v0.5.x to v1.0.0

## Overview

Version 1.0.0 introduces a significant architectural improvement by transitioning from a trait-based event system to an enum-based event system. This change brings:

- **Better Type Safety**: Compile-time guarantees for event types
- **Improved Performance**: Eliminates dynamic dispatch overhead (no `Box<dyn Trait>`)
- **Simpler Code**: Standard Rust patterns instead of custom macros
- **Better IDE Support**: Full autocomplete and type inference

## Breaking Changes Summary

| Component          | v0.5.x                      | v1.0.0                      |
| ------------------ | --------------------------- | --------------------------- |
| Event Type         | `Box<dyn UnifiedEvent>`     | `DexEvent` (enum)           |
| Callback Signature | `Fn(Box<dyn UnifiedEvent>)` | `Fn(DexEvent)`              |
| Event Matching     | `match_event!` macro        | Standard `match` expression |
| Metadata Access    | `.event_type()`             | `.metadata().event_type`    |
| Event Properties   | `.signature()`              | `.metadata().signature`     |

## Migration Steps

### Step 1: Update Callback Signatures

**Before (v0.5.x):**

```rust
use solana_streamer_sdk::streaming::event_parser::UnifiedEvent;

let callback = |event: Box<dyn UnifiedEvent>| {
    println!("Received event: {:?}", event);
};
```

**After (v1.0.0):**

```rust
use solana_streamer_sdk::streaming::event_parser::DexEvent;

let callback = |event: DexEvent| {
    println!("Received event: {:?}", event);
};
```

### Step 2: Update Event Matching

**Before (v0.5.x):**

```rust
use solana_streamer_sdk::match_event;

match_event!(event, {
    PumpFunTradeEvent => |e: PumpFunTradeEvent| {
        println!("PumpFun trade: {:?}", e);
    },
    RaydiumCpmmSwapEvent => |e: RaydiumCpmmSwapEvent| {
        println!("Raydium swap: {:?}", e);
    },
});
```

**After (v1.0.0):**

```rust
match event {
    DexEvent::PumpFunTradeEvent(e) => {
        println!("PumpFun trade: {:?}", e);
    }
    DexEvent::RaydiumCpmmSwapEvent(e) => {
        println!("Raydium swap: {:?}", e);
    }
    _ => {}
}
```

### Step 3: Update Metadata Access

**Before (v0.5.x):**

```rust
let event_type = event.event_type();
let signature = event.signature();
let slot = event.slot();
let protocol = event.protocol();
```

**After (v1.0.0):**

```rust
let event_type = event.metadata().event_type;
let signature = event.metadata().signature;
let slot = event.metadata().slot;
let protocol = event.metadata().protocol;
```

### Step 4: Update Import Statements

**Before (v0.5.x):**

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

**After (v1.0.0):**

```rust
use solana_streamer_sdk::streaming::event_parser::{
    DexEvent,
    protocols::{
        pumpfun::{PumpFunTradeEvent, PumpFunCreateTokenEvent},
        raydium_cpmm::{RaydiumCpmmSwapEvent},
    },
};
```

Note: The `match_event!` macro is no longer needed or available.

## Complete Example Migration

### Before (v0.5.x)

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
            "Event type: {:?}, Signature: {}",
            event.event_type(),
            event.signature()
        );

        match_event!(event, {
            PumpFunTradeEvent => |e: PumpFunTradeEvent| {
                println!("PumpFun trade: {} SOL", e.sol_amount);
            },
            PumpFunCreateTokenEvent => |e: PumpFunCreateTokenEvent| {
                println!("New token: {}", e.name);
            },
            RaydiumCpmmSwapEvent => |e: RaydiumCpmmSwapEvent| {
                println!("Raydium swap");
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

### After (v1.0.0)

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
            "Event type: {:?}, Signature: {}",
            event.metadata().event_type,
            event.metadata().signature
        );

        match event {
            DexEvent::PumpFunTradeEvent(e) => {
                println!("PumpFun trade: {} SOL", e.sol_amount);
            }
            DexEvent::PumpFunCreateTokenEvent(e) => {
                println!("New token: {}", e.name);
            }
            DexEvent::RaydiumCpmmSwapEvent(e) => {
                println!("Raydium swap");
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

## Advanced Patterns

### Pattern 1: Event Filtering with Match

**v1.0.0:**

```rust
let callback = |event: DexEvent| {
    // Only process specific event types
    match event {
        DexEvent::PumpFunTradeEvent(e) if e.is_buy => {
            println!("Buy: {} tokens", e.token_amount);
        }
        DexEvent::PumpFunTradeEvent(e) if !e.is_buy => {
            println!("Sell: {} tokens", e.token_amount);
        }
        _ => {} // Ignore other events
    }
};
```

### Pattern 2: Generic Event Processing

**v1.0.0:**

```rust
fn process_event(event: DexEvent) {
    let metadata = event.metadata();

    println!("Protocol: {:?}", metadata.protocol);
    println!("Event Type: {:?}", metadata.event_type);
    println!("Signature: {}", metadata.signature);
    println!("Slot: {}", metadata.slot);

    // Process specific event types
    match event {
        DexEvent::PumpFunTradeEvent(e) => handle_pumpfun_trade(e),
        DexEvent::RaydiumCpmmSwapEvent(e) => handle_raydium_swap(e),
        _ => {}
    }
}
```

### Pattern 3: Event Type Categorization

**v1.0.0:**

```rust
fn categorize_event(event: &DexEvent) -> &'static str {
    match event {
        DexEvent::PumpFunTradeEvent(_)
        | DexEvent::PumpSwapBuyEvent(_)
        | DexEvent::PumpSwapSellEvent(_) => "Trade",

        DexEvent::PumpFunCreateTokenEvent(_)
        | DexEvent::PumpSwapCreatePoolEvent(_) => "Creation",

        DexEvent::RaydiumCpmmDepositEvent(_)
        | DexEvent::RaydiumCpmmWithdrawEvent(_) => "Liquidity",

        _ => "Other",
    }
}
```

## EventParser API Changes

### Parsing Transactions

**Before (v0.5.x):**

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

**After (v1.0.0):**

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

The `EventParser` is now stateless with static methods, eliminating the need to create an instance.

## Common Pitfalls

### Pitfall 1: Forgetting to Handle All Variants

❌ **Incorrect:**

```rust
match event {
    DexEvent::PumpFunTradeEvent(e) => { /* ... */ }
    // Missing other variants!
}
```

✅ **Correct:**

```rust
match event {
    DexEvent::PumpFunTradeEvent(e) => { /* ... */ }
    _ => {} // Handle or ignore other events
}
```

### Pitfall 2: Using Old Metadata Access Pattern

❌ **Incorrect:**

```rust
let sig = event.signature(); // Method doesn't exist anymore
```

✅ **Correct:**

```rust
let sig = event.metadata().signature;
```

### Pitfall 3: Attempting to Use `match_event!` Macro

❌ **Incorrect:**

```rust
match_event!(event, { /* ... */ }); // Macro no longer exists
```

✅ **Correct:**

```rust
match event {
    DexEvent::PumpFunTradeEvent(e) => { /* ... */ }
    _ => {}
}
```

## Benefits of the New System

1. **Type Safety**: The compiler catches more errors at compile time
2. **Performance**: No dynamic dispatch overhead
3. **Simplicity**: Standard Rust patterns, no custom macros
4. **Better Tooling**: Full IDE support with autocomplete
5. **Easier Debugging**: Clearer stack traces and error messages
6. **Serialization**: Built-in `Serialize`/`Deserialize` support for all events
