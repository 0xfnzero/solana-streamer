# Shredstream 解析限制与差异说明

与 gRPC 订阅相比，shredstream 路径存在以下限制和解析差异，使用时请注意。

## gRPC（Yellowstone）说明

当订阅返回的交易带有完整 `meta`（日志、loaded addresses、inner instructions）时，DEX 相关事件在内部由 **sol-parser-sdk** 解析（与 upstream 相同的 logs + instructions 与 log/ix 去重；streamer 使用 **顺序**解析路径以降低单笔延迟），再映射为本 crate 的 `DexEvent`（对外 API 不变）；Compute Budget 仍单独走原有指令路径，且在 DEX 事件派发 **之后** 运行，以便 Swap 等事件更早送达回调。

ShredStream 路径仍为下面的原始交易解析限制，不使用上述日志管线。

## 1. 数据源差异

| 数据 | gRPC | Shredstream |
|------|------|-------------|
| 账户列表 | 完整 resolved 列表（static + loaded_addresses） | 仅 `static_account_keys()` |
| Inner instructions (CPI) | 有（来自区块执行结果） | **无**（Entry 仅含原始交易） |
| block_time | 有 | **无**（恒为 0） |
| tx_index | slot 内交易索引 | entry 内交易索引（best-effort） |

## 2. 解析问题与遗漏

**漏掉事件小结**：Shred 会**整笔漏掉**「仅通过 CPI 触发的」所有协议事件（例如经 Jupiter 等聚合器路由的 PumpFun/PumpSwap 等），因为 shred 不解析 inner instructions。详见 2.3。

### 2.1 使用 Address Lookup Tables (ALT) 的交易

- **现象**：指令中的账户索引指向「static + loaded」的完整列表，shred 只传入 static。当前 ShredStream 路径会尽量解析外层指令；若某个指令账户来自 ALT-loaded keys，会以 `Pubkey::default()`（11111...）占位。
- **影响协议/指令**：所有依赖「按索引取账户」的指令，在交易使用 ALT 时都可能出现错误或默认账户；能靠指令 data / discriminator 识别的事件会 best-effort 产出。
- **program id 也来自 ALT 时**：无法从 shred 恢复真实程序 ID，会按启用的协议/filter 进行 discriminator-only best-effort 解析。若不同协议 discriminator 碰撞，建议使用窄 filter 降低误判或多候选输出风险。
- **典型表现**：
  - **PumpFun**：Create / CreateV2 的 token_program、global、event_authority 等为 11111...；Buy/Sell 的 creator_vault、token_program 等可能错误。
  - **PumpSwap / Bonk / Raydium / Meteora**：依赖高索引账户的指令同样可能得到错误或 default 账户。
- **建议**：若需完整且正确的账户字段，请使用 gRPC 订阅。

### 2.2 无 Inner Instructions → 无 CPI 合并

- **原因**：Shred/Entry 只包含原始 `VersionedTransaction`，inner instructions 是执行阶段产物，不在 shred 载荷中。
- **影响**：
  - **PumpFun**
    - Create / CreateV2：无 CPI 合并 → `timestamp`、`virtual_*_reserves`、`real_*_reserves`、`token_total_supply`、`token_program`（来自 log）等多为 0 或默认。
    - Trade：无 CPI 合并 → 无 log 中的成交额、reserves、fee 等明细，仅保留指令层数据。
    - Migrate：此前因「必须带 CPI」被直接跳过，**现已改为** shred 下仍发出仅含指令数据的 Migrate 事件（user/mint 等来自指令账户；mint_amount、sol_amount、timestamp、pool 等来自 CPI 的字段为 0/默认）。
  - **PumpSwap**：buy/sell/deposit/withdraw/create_pool 无 CPI 合并，无 log 中的金额、reserves 等；**swap_data** 依赖后续指令解析，inner 为空时恒为空。
  - **Bonk**：trade、pool_create 无 CPI 合并，缺少 log 明细。
  - **Meteora Damm V2**：swap、initialize_pool 无 CPI 合并。
  - **Raydium**：依赖 inner 的解析/合并与 gRPC 一致缺失。

### 2.3 漏掉的事件：仅通过 CPI 触发的调用

- **原因**：Shred 路径只遍历并解析**外层指令**（`transaction.message.instructions()`）。内层指令（inner instructions）只有在传入非空的 `inner_instructions` 时才会被解析；shred 传入的为 `&[]`，因此**从不**解析任何 inner。
- **结果**：当协议**仅作为 CPI 被调用**时（例如用户通过 Jupiter/Raydium 聚合器等路由，外层指令是聚合器，PumpFun/PumpSwap 等只在 inner 中出现），gRPC 会解析该 inner 并发出对应事件，**shred 则整笔交易都不会产生该协议的任何事件**。
- **影响**：所有协议（PumpFun、PumpSwap、Bonk、Raydium、Meteora 等）在「仅 CPI 调用」场景下，shred 都会**漏掉整笔事件**，不是字段缺失，而是事件本身不会出现。
- **建议**：若需要统计或处理通过聚合器/路由产生的交易，必须使用 gRPC 订阅；shred 只适合「用户直接与协议交互」的链路。

### 2.4 其他明确「漏掉」或弱化的解析

- **PumpFun Migrate**：shred 下**会**发出事件，但仅包含指令解析出的账户与部分字段（如 user、mint）；mint_amount、sol_amount、pool_migration_fee、timestamp、pool 等来自 CPI 的字段为 0/默认。
- **所有协议的 CPI 维度的数据**：shred 路径一律缺失（无 inner instructions 即无 CPI 解析与 merge）。

## 3. 使用建议

- 需要**完整、正确**的账户与 log 字段（reserves、timestamp、amounts、swap_data 等）时，使用 **gRPC 订阅**。
- Shredstream 更适合：对延迟更敏感、可接受「仅指令层 + 部分字段缺失/默认」的场景；使用 ALT 的交易会 best-effort 解析，但 ALT-loaded 账户字段可能为 default。

## 4. 各事件 Shred 路径字段完整性

以下为「直接外层调用」场景下，shred 能拿到的字段 vs 仅 CPI 合并才有的字段（shred 下为 0/默认）。若交易使用 ALT，标注为「指令」的账户类字段也可能错误或为 default。

**元数据（所有事件）**  
- Shred 有：signature, slot, recv_us, program_id, outer_index, tx_index（entry 内索引）, event_type, protocol  
- Shred 缺失：**block_time / block_time_ms**（恒为 0），**swap_data**（恒为 None，依赖 inner 后续指令解析）

### 4.1 PumpFun

| 事件 | 指令解析有（Shred 有） | 仅 CPI 合并有（Shred 缺失） |
|------|------------------------|-----------------------------|
| **CreateToken** | name, symbol, uri, creator, mint, 各账户(0..13) | timestamp, virtual_*_reserves, real_*_reserves, token_total_supply, token_program(来自 log), is_mayhem_mode, is_cashback_enabled |
| **CreateV2Token** | name, symbol, uri, creator, mint, 各账户(0..15) | timestamp, virtual_*_reserves, real_*_reserves, token_total_supply, token_program(来自 log), is_mayhem_mode, is_cashback_enabled |
| **Trade** (Buy/Sell) | is_buy, amount/max_sol_cost/min_sol_output, 各账户(含 user, mint, creator_vault 等) | sol_amount, token_amount, timestamp, virtual_*_reserves, real_*_reserves, fee_recipient, fee_basis_points, fee, creator, creator_fee_*, track_volume, total_unclaimed/claimed_tokens, current_sol_volume, last_update_timestamp, ix_name, mayhem_mode, cashback_* |
| **Migrate** | user, mint, bonding_curve, 全部 24 个账户 | mint_amount, sol_amount, pool_migration_fee, timestamp, pool（CPI 的 pool） |

### 4.2 PumpSwap

| 事件 | 指令解析有（Shred 有） | 仅 CPI 合并有（Shred 缺失） |
|------|------------------------|-----------------------------|
| **Buy** | base_amount_out, max_quote_amount_in, pool, user, base_mint, quote_mint, 各 token account / fee recipient / program，coin_creator_vault_ata/authority(若 accounts≥19) | timestamp, 实际 quote_amount_in, user/pool *_reserves, lp_fee, protocol_fee, coin_creator_fee_*, track_volume, total_unclaimed/claimed_tokens, current_sol_volume, last_update_timestamp |
| **Sell** | base_amount_in, min_quote_amount_out, pool, user, base_mint, quote_mint, 各账户，coin_creator_vault_* | timestamp, 实际 quote_amount_out, *_reserves, 各项 fee, coin_creator_fee_* |
| **CreatePool** | index, base_amount_in, quote_amount_in, coin_creator(若 data≥50), pool, creator, base/quote_mint, lp_mint, 各 token account | timestamp, base_mint_decimals, quote_mint_decimals, pool_base/quote_amount, minimum/initial_liquidity, lp_token_amount_out, pool_bump |
| **Deposit** | lp_token_amount_out, max_base/quote_amount_in, pool, user, 各 mint / token account | timestamp, user/pool *_reserves, base_amount_in, quote_amount_in, lp_mint_supply 等 |
| **Withdraw** | lp_token_amount_in, min_base/quote_amount_out, pool, user, 各账户 | timestamp, *_reserves, base/quote_amount_out, lp_mint_supply 等 |

说明：`PumpSwapCreatePool` 对齐链上 `CreatePoolEvent` IDL，只包含 `is_mayhem_mode`，不包含 `is_cashback_coin`。`is_cashback_coin` 属于 PumpSwap `Pool` account 字段；ShredStream 不带账户 body，无法从 Shred 的 CreatePool 事件恢复该字段。需要该字段时，请在 gRPC/account 流中订阅 `AccountPumpSwapPool`，读取 `PumpSwapPoolAccountEvent.pool.is_cashback_coin`。

### 4.3 Bonk

| 事件 | 指令解析有（Shred 有） | 仅 CPI 合并有（Shred 缺失） |
|------|------------------------|-----------------------------|
| **Trade** | amount_in/out, minimum/maximum_*, share_fee_rate, payer, pool_state, 各 vault/mint/program 账户, trade_direction | pool_state(来自 log), total_base_sell, virtual_base/quote, real_*_before/after, amount_in/out(实际成交), protocol_fee, platform_fee, creator_fee, share_fee, pool_status, exact_in |
| **PoolCreate** | payer, creator, global_config, platform_config, pool_state, base/quote_mint, base/quote_vault, base_mint_param, curve_param, vesting_param(, amm_fee_on for V2) | config, base_mint_param/curve_param/vesting_param(来自 log 的完整值), amm_fee_on(来自 log) |
| **MigrateToAmm / MigrateToCpswap** | 指令侧账户与参数 | base_lot_size, quote_lot_size, market_vault_signer_nonce（CPI 才有） |

### 4.4 Raydium / Meteora Damm V2

- **Raydium CLMM/CPMM/AMM**：指令解析会填账户与指令内参数（如 amount、min_out 等）；实际成交额、reserves、fee 等来自 log 的字段在 shred 下均为 0/默认。
- **Meteora Damm V2**：Swap / InitializePool 等同上，指令层有账户与部分参数，CPI 的 timestamp、reserves、实际 amount 等 shred 缺失。

## 5. 代码位置参考

- Shred 入口：`streaming/common/event_processor.rs` → `process_shred_transaction`
- 账户与 inner 传入：`accounts = tx.message.static_account_keys()`，`inner_instructions: &[]`
- 合并逻辑（CPI 覆盖/补充字段）：`streaming/event_parser/core/merger_event.rs` → `merge()`
