use crate::streaming::event_parser::{
    DexEvent, Protocol, common::{
        EventMetadata, filter::EventTypeFilter, high_performance_clock::elapsed_micros_since, parse_swap_data_from_next_grpc_instructions, parse_swap_data_from_next_instructions
    }, core::{
        dispatcher::EventDispatcher,
        global_state::{
            add_bonk_dev_address, add_dev_address, is_bonk_dev_address_in_signature,
            is_dev_address_in_signature,
        },
        merger_event::merge,
    }, protocols::raydium_amm_v4::parser::RAYDIUM_AMM_V4_PROGRAM_ID
};
use prost_types::Timestamp;
use solana_sdk::{
    message::compiled_instruction::CompiledInstruction, pubkey::Pubkey, signature::Signature,
    transaction::VersionedTransaction,
};
use solana_transaction_status::InnerInstructions;
use std::sync::Arc;
use yellowstone_grpc_proto::geyser::SubscribeUpdateTransactionInfo;

pub struct EventParser {}

impl EventParser {
    // ================================================================================================
    // Public API - Entry Points
    // ================================================================================================

    /// Parse transaction from gRPC stream
    ///
    /// This is the main entry point for parsing transactions received from gRPC streams.
    /// It extracts account keys, inner instructions, and delegates to instruction parsing.
    pub async fn parse_grpc_transaction(
        protocols: &[Protocol],
        event_type_filter: Option<&EventTypeFilter>,
        grpc_tx: SubscribeUpdateTransactionInfo,
        signature: Signature,
        slot: Option<u64>,
        block_time: Option<Timestamp>,
        recv_us: i64,
        bot_wallet: Option<Pubkey>,
        transaction_index: Option<u64>,
        callback: Arc<dyn Fn(DexEvent) + Send + Sync>,
    ) -> anyhow::Result<()> {
        // 创建适配器回调，将所有权回调转换为引用回调
        let adapter_callback = Arc::new(move |event: &DexEvent| {
            callback(event.clone());
        });
        if let Some(transition) = grpc_tx.transaction {
            if let Some(message) = &transition.message {
                let mut address_table_lookups: Vec<Vec<u8>> = vec![];
                let mut inner_instructions: Vec<
                    yellowstone_grpc_proto::solana::storage::confirmed_block::InnerInstructions,
                > = vec![];

                if let Some(meta) = grpc_tx.meta {
                    inner_instructions = meta.inner_instructions;
                    address_table_lookups.reserve(
                        meta.loaded_writable_addresses.len() + meta.loaded_readonly_addresses.len(),
                    );
                    let loaded_writable_addresses = meta.loaded_writable_addresses;
                    let loaded_readonly_addresses = meta.loaded_readonly_addresses;
                    address_table_lookups.extend(
                        loaded_writable_addresses.into_iter().chain(loaded_readonly_addresses),
                    );
                }

                let mut accounts_bytes: Vec<Vec<u8>> =
                    Vec::with_capacity(message.account_keys.len() + address_table_lookups.len());
                accounts_bytes.extend_from_slice(&message.account_keys);
                accounts_bytes.extend(address_table_lookups);
                // 转换为 Pubkey
                let accounts: Vec<Pubkey> = accounts_bytes
                    .iter()
                    .filter_map(|account| {
                        if account.len() == 32 {
                            Some(Pubkey::try_from(account.as_slice()).unwrap_or_default())
                        } else {
                            None
                        }
                    })
                    .collect();
                // 解析指令事件
                let instructions = &message.instructions;
                Self::parse_instruction_events_from_grpc_transaction(
                    protocols,
                    event_type_filter,
                    &instructions,
                    signature,
                    slot,
                    block_time,
                    recv_us,
                    &accounts,
                    &inner_instructions,
                    bot_wallet,
                    transaction_index,
                    adapter_callback,
                )
                .await?;
            }
        }

        Ok(())
    }

    /// Parse transaction from VersionedTransaction
    ///
    /// This is the entry point for parsing VersionedTransaction objects.
    /// It's used when working with RPC responses or historical data.
    #[allow(clippy::too_many_arguments)]
    pub async fn parse_instruction_events_from_versioned_transaction(
        protocols: &[Protocol],
        event_type_filter: Option<&EventTypeFilter>,
        transaction: &VersionedTransaction,
        signature: Signature,
        slot: Option<u64>,
        block_time: Option<Timestamp>,
        recv_us: i64,
        accounts: &[Pubkey],
        inner_instructions: &[InnerInstructions],
        bot_wallet: Option<Pubkey>,
        transaction_index: Option<u64>,
        callback: Arc<dyn Fn(DexEvent) + Send + Sync>,
    ) -> anyhow::Result<()> {
        // 创建适配器回调，将所有权回调转换为引用回调
        let adapter_callback = Arc::new(move |event: &DexEvent| {
            callback(event.clone());
        });
        // 获取交易的指令和账户
        let compiled_instructions = transaction.message.instructions();
        let mut accounts: Vec<Pubkey> = accounts.to_vec();
        // 检查交易中是否包含程序
        let has_program = accounts
            .iter()
            .any(|account| Self::should_handle(protocols, event_type_filter, account));
        if has_program {
            // 解析每个指令
            for (index, instruction) in compiled_instructions.iter().enumerate() {
                if let Some(program_id) = accounts.get(instruction.program_id_index as usize) {
                    let program_id = *program_id; // 克隆程序ID，避免借用冲突
                    let inner_instructions = inner_instructions
                        .iter()
                        .find(|inner_instruction| inner_instruction.index == index as u8);
                    if Self::should_handle(protocols, event_type_filter, &program_id) {
                        let max_idx = instruction.accounts.iter().max().unwrap_or(&0);
                        // 补齐accounts(使用Pubkey::default())
                        if *max_idx as usize >= accounts.len() {
                            accounts.resize(*max_idx as usize + 1, Pubkey::default());
                        }
                        Self::parse_events_from_instruction(
                            protocols,
                            event_type_filter,
                            instruction,
                            &accounts,
                            signature,
                            slot.unwrap_or(0),
                            block_time,
                            recv_us,
                            index as i64,
                            None,
                            bot_wallet,
                            transaction_index,
                            inner_instructions,
                            adapter_callback.clone(),
                        )?;
                    }
                    // Immediately process inner instructions for correct ordering
                    if let Some(inner_instructions) = inner_instructions {
                        for (inner_index, inner_instruction) in
                            inner_instructions.instructions.iter().enumerate()
                        {
                            Self::parse_events_from_instruction(
                                protocols,
                                event_type_filter,
                                &inner_instruction.instruction,
                                &accounts,
                                signature,
                                slot.unwrap_or(0),
                                block_time,
                                recv_us,
                                index as i64,
                                Some(inner_index as i64),
                                bot_wallet,
                                transaction_index,
                                Some(&inner_instructions),
                                adapter_callback.clone(),
                            )?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    // ================================================================================================
    // gRPC Transaction Processing
    // ================================================================================================

    /// Parse instruction events from gRPC transaction format
    ///
    /// Iterates through all instructions in a gRPC transaction, checks if they should be handled,
    /// and delegates to instruction-level parsing for both outer and inner instructions.
    #[allow(clippy::too_many_arguments)]
    async fn parse_instruction_events_from_grpc_transaction(
        protocols: &[Protocol],
        event_type_filter: Option<&EventTypeFilter>,
        compiled_instructions: &[yellowstone_grpc_proto::prelude::CompiledInstruction],
        signature: Signature,
        slot: Option<u64>,
        block_time: Option<Timestamp>,
        recv_us: i64,
        accounts: &[Pubkey],
        inner_instructions: &[yellowstone_grpc_proto::prelude::InnerInstructions],
        bot_wallet: Option<Pubkey>,
        transaction_index: Option<u64>,
        callback: Arc<dyn for<'a> Fn(&'a DexEvent) + Send + Sync>,
    ) -> anyhow::Result<()> {
        // 获取交易的指令和账户
        let mut accounts = accounts.to_vec();
        // 检查交易中是否包含程序
        let has_program = accounts
            .iter()
            .any(|account| Self::should_handle(protocols, event_type_filter, account));
        if has_program {
            // 解析每个指令
            for (index, instruction) in compiled_instructions.iter().enumerate() {
                if let Some(program_id) = accounts.get(instruction.program_id_index as usize) {
                    let program_id = *program_id; // 克隆程序ID，避免借用冲突
                    let inner_instructions = inner_instructions
                        .iter()
                        .find(|inner_instruction| inner_instruction.index == index as u32);
                    let max_idx = instruction.accounts.iter().max().unwrap_or(&0);
                    // 补齐accounts(使用Pubkey::default())
                    if *max_idx as usize >= accounts.len() {
                        accounts.resize(*max_idx as usize + 1, Pubkey::default());
                    }
                    if Self::should_handle(protocols, event_type_filter, &program_id) {
                        Self::parse_events_from_grpc_instruction(
                            protocols,
                            event_type_filter,
                            instruction,
                            &accounts,
                            signature,
                            slot.unwrap_or(0),
                            block_time,
                            recv_us,
                            index as i64,
                            None,
                            bot_wallet,
                            transaction_index,
                            inner_instructions,
                            callback.clone(),
                        )?;
                    }
                    // Immediately process inner instructions for correct ordering
                    if let Some(inner_instructions) = inner_instructions {
                        for (inner_index, inner_instruction) in
                            inner_instructions.instructions.iter().enumerate()
                        {
                            let inner_accounts = &inner_instruction.accounts;
                            let data = &inner_instruction.data;
                            let instruction =
                                yellowstone_grpc_proto::prelude::CompiledInstruction {
                                    program_id_index: inner_instruction.program_id_index,
                                    accounts: inner_accounts.to_vec(),
                                    data: data.to_vec(),
                                };
                            Self::parse_events_from_grpc_instruction(
                                protocols,
                                event_type_filter,
                                &instruction,
                                &accounts,
                                signature,
                                slot.unwrap_or(0),
                                block_time,
                                recv_us,
                                inner_instructions.index as i64,
                                Some(inner_index as i64),
                                bot_wallet,
                                transaction_index,
                                Some(&inner_instructions),
                                callback.clone(),
                            )?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Parse events from gRPC instruction
    ///
    /// Core parsing logic for a single gRPC instruction. Extracts discriminator, dispatches
    /// to protocol-specific parsers, handles inner instructions, and processes swap data.
    #[allow(clippy::too_many_arguments)]
    fn parse_events_from_grpc_instruction(
        protocols: &[Protocol],
        event_type_filter: Option<&EventTypeFilter>,
        instruction: &yellowstone_grpc_proto::prelude::CompiledInstruction,
        accounts: &[Pubkey],
        signature: Signature,
        slot: u64,
        block_time: Option<Timestamp>,
        recv_us: i64,
        outer_index: i64,
        inner_index: Option<i64>,
        bot_wallet: Option<Pubkey>,
        transaction_index: Option<u64>,
        inner_instructions: Option<&yellowstone_grpc_proto::prelude::InnerInstructions>,
        callback: Arc<dyn for<'a> Fn(&'a DexEvent) + Send + Sync>,
    ) -> anyhow::Result<()> {
        // 添加边界检查以防止越界访问
        let program_id_index = instruction.program_id_index as usize;
        if program_id_index >= accounts.len() {
            return Ok(());
        }
        let program_id = accounts[program_id_index];
        if !Self::should_handle(protocols, event_type_filter, &program_id) {
            return Ok(());
        }

        let is_cu_program = EventDispatcher::is_compute_budget_program(&program_id);

        let disc_len = match program_id {
            RAYDIUM_AMM_V4_PROGRAM_ID => 1,
            _ => 8,
        };

        // 检查指令数据长度（至少需要 disc_len 字节的 discriminator）
        if !is_cu_program && instruction.data.len() < disc_len {
            return Ok(());
        }
        // 创建元数据
        let timestamp = block_time.unwrap_or(Timestamp { seconds: 0, nanos: 0 });
        let block_time_ms = timestamp.seconds * 1000 + (timestamp.nanos as i64) / 1_000_000;
        let metadata = EventMetadata::new(
            signature,
            slot,
            timestamp.seconds,
            block_time_ms,
            Default::default(), // protocol will be set by dispatcher
            Default::default(), // event_type will be set by dispatcher
            program_id,
            outer_index,
            inner_index,
            recv_us,
            transaction_index,
        );

        if is_cu_program {
            if let Some(event) = EventDispatcher::dispatch_compute_budget_instruction(
                &instruction.data,
                metadata.clone(),
            ) {
                callback(&event);
            }
            return Ok(());
        }

        // 使用 EventDispatcher 匹配协议
        let protocol = match EventDispatcher::match_protocol_by_program_id(&program_id) {
            Some(p) => p,
            None => return Ok(()),
        };

        // 提取 discriminator 和数据
        let instruction_discriminator = &instruction.data[..disc_len];
        let instruction_data = &instruction.data[disc_len..];

        // 构建账户公钥列表
        let account_pubkeys: Vec<Pubkey> = instruction
            .accounts
            .iter()
            .filter_map(|&idx| accounts.get(idx as usize).copied())
            .collect();

        // 使用 EventDispatcher 解析 instruction 事件
        let mut event = match EventDispatcher::dispatch_instruction(
            protocol.clone(),
            instruction_discriminator,
            instruction_data,
            &account_pubkeys,
            metadata.clone(),
        ) {
            Some(e) => e,
            None => return Ok(()),
        };

        // 处理 inner instructions
        let mut inner_instruction_event: Option<DexEvent> = None;
        if let Some(inner_instructions_ref) = inner_instructions {
            // 并行执行两个任务: 解析 inner event 和提取 swap_data
            let (inner_event_result, swap_data_result) = std::thread::scope(|s| {
                let inner_event_handle = s.spawn(|| {
                    for inner_instruction in inner_instructions_ref.instructions.iter() {
                        let inner_data = &inner_instruction.data;
                        // 检查长度（需要 16 字节的 discriminator）
                        if inner_data.len() < 16 {
                            continue;
                        }
                        let inner_discriminator = &inner_data[..16];
                        let inner_instruction_data = &inner_data[16..];

                        if let Some(inner_event) = EventDispatcher::dispatch_inner_instruction(
                            protocol.clone(),
                            inner_discriminator,
                            inner_instruction_data,
                            metadata.clone(),
                        ) {
                            return Some(inner_event);
                        }
                    }
                    None
                });

                let swap_data_handle = s.spawn(|| {
                    if event.metadata().swap_data.is_none() {
                        parse_swap_data_from_next_grpc_instructions(
                            &event,
                            inner_instructions_ref,
                            inner_index.unwrap_or(-1_i64) as i8,
                            accounts,
                        )
                    } else {
                        None
                    }
                });

                // 等待两个任务完成
                (inner_event_handle.join().unwrap(), swap_data_handle.join().unwrap())
            });

            inner_instruction_event = inner_event_result;
            if let Some(swap_data) = swap_data_result {
                event.metadata_mut().set_swap_data(swap_data);
            }
        }

        // 特殊处理: PumpFun MIGRATE 指令需要 inner instruction data
        if matches!(protocol, Protocol::PumpFun) {
            const PUMPFUN_MIGRATE_IX: &[u8] = &[155, 234, 231, 146, 236, 158, 162, 30];
            if instruction_discriminator == PUMPFUN_MIGRATE_IX && inner_instruction_event.is_none()
            {
                return Ok(());
            }
        }

        // 合并事件
        if let Some(inner_instruction_event) = inner_instruction_event {
            merge(&mut event, inner_instruction_event);
        }

        // 设置处理时间（使用高性能时钟）
        event.metadata_mut().handle_us = elapsed_micros_since(recv_us);
        event = Self::process_event(event, bot_wallet);
        callback(&event);

        Ok(())
    }

    // ================================================================================================
    // Standard Instruction Processing
    // ================================================================================================

    /// Parse events from standard Solana instruction
    ///
    /// Similar to gRPC instruction parsing but works with standard CompiledInstruction format.
    /// Used when parsing VersionedTransaction or RPC data.
    #[allow(clippy::too_many_arguments)]
    fn parse_events_from_instruction(
        protocols: &[Protocol],
        event_type_filter: Option<&EventTypeFilter>,
        instruction: &CompiledInstruction,
        accounts: &[Pubkey],
        signature: Signature,
        slot: u64,
        block_time: Option<Timestamp>,
        recv_us: i64,
        outer_index: i64,
        inner_index: Option<i64>,
        bot_wallet: Option<Pubkey>,
        transaction_index: Option<u64>,
        inner_instructions: Option<&InnerInstructions>,
        callback: Arc<dyn for<'a> Fn(&'a DexEvent) + Send + Sync>,
    ) -> anyhow::Result<()> {
        // 添加边界检查以防止越界访问
        let program_id_index = instruction.program_id_index as usize;
        if program_id_index >= accounts.len() {
            return Ok(());
        }
        let program_id = accounts[program_id_index];
        if !Self::should_handle(protocols, event_type_filter, &program_id) {
            return Ok(());
        }

        let is_cu_program = EventDispatcher::is_compute_budget_program(&program_id);

        let disc_len = match program_id {
            RAYDIUM_AMM_V4_PROGRAM_ID => 1,
            _ => 8,
        };

        // 检查指令数据长度（至少需要 8 字节的 discriminator）
        if !is_cu_program && instruction.data.len() < disc_len {
            return Ok(());
        }

        // 创建元数据
        let timestamp = block_time.unwrap_or(Timestamp { seconds: 0, nanos: 0 });
        let block_time_ms = timestamp.seconds * 1000 + (timestamp.nanos as i64) / 1_000_000;
        let metadata = EventMetadata::new(
            signature,
            slot,
            timestamp.seconds,
            block_time_ms,
            Default::default(), // protocol will be set by dispatcher
            Default::default(), // event_type will be set by dispatcher
            program_id,
            outer_index,
            inner_index,
            recv_us,
            transaction_index,
        );

        if is_cu_program {
            if let Some(event) = EventDispatcher::dispatch_compute_budget_instruction(
                &instruction.data,
                metadata.clone(),
            ) {
                callback(&event);
            }
            return Ok(());
        }

        // 使用 EventDispatcher 匹配协议
        let protocol = match EventDispatcher::match_protocol_by_program_id(&program_id) {
            Some(p) => p,
            None => return Ok(()),
        };

        // 提取 discriminator 和数据
        let instruction_discriminator = &instruction.data[..disc_len];
        let instruction_data = &instruction.data[disc_len..];

        // 构建账户公钥列表
        let account_pubkeys: Vec<Pubkey> = instruction
            .accounts
            .iter()
            .filter_map(|&idx| accounts.get(idx as usize).copied())
            .collect();

        // 使用 EventDispatcher 解析 instruction 事件
        let mut event = match EventDispatcher::dispatch_instruction(
            protocol.clone(),
            instruction_discriminator,
            instruction_data,
            &account_pubkeys,
            metadata.clone(),
        ) {
            Some(e) => e,
            None => return Ok(()),
        };

        // 处理 inner instructions
        let mut inner_instruction_event: Option<DexEvent> = None;
        if let Some(inner_instructions_ref) = inner_instructions {
            // 并行执行两个任务: 解析 inner event 和提取 swap_data
            let (inner_event_result, swap_data_result) = std::thread::scope(|s| {
                let inner_event_handle = s.spawn(|| {
                    for inner_instruction in inner_instructions_ref.instructions.iter() {
                        let inner_data = &inner_instruction.instruction.data;
                        // 检查长度（需要 16 字节的 discriminator）
                        if inner_data.len() < 16 {
                            continue;
                        }
                        let inner_discriminator = &inner_data[..16];
                        let inner_instruction_data = &inner_data[16..];

                        if let Some(inner_event) = EventDispatcher::dispatch_inner_instruction(
                            protocol.clone(),
                            inner_discriminator,
                            inner_instruction_data,
                            metadata.clone(),
                        ) {
                            return Some(inner_event);
                        }
                    }
                    None
                });

                let swap_data_handle = s.spawn(|| {
                    if event.metadata().swap_data.is_none() {
                        parse_swap_data_from_next_instructions(
                            &event,
                            inner_instructions_ref,
                            inner_index.unwrap_or(-1_i64) as i8,
                            accounts,
                        )
                    } else {
                        None
                    }
                });

                // 等待两个任务完成
                (inner_event_handle.join().unwrap(), swap_data_handle.join().unwrap())
            });

            inner_instruction_event = inner_event_result;
            if let Some(swap_data) = swap_data_result {
                event.metadata_mut().set_swap_data(swap_data);
            }
        }

        // 特殊处理: PumpFun MIGRATE 指令需要 inner instruction data
        if matches!(protocol, Protocol::PumpFun) {
            const PUMPFUN_MIGRATE_IX: &[u8] = &[155, 234, 231, 146, 236, 158, 162, 30];
            if instruction_discriminator == PUMPFUN_MIGRATE_IX && inner_instruction_event.is_none()
            {
                return Ok(());
            }
        }

        // 合并事件
        if let Some(inner_instruction_event) = inner_instruction_event {
            merge(&mut event, inner_instruction_event);
        }

        // 设置处理时间（使用高性能时钟）
        event.metadata_mut().handle_us = elapsed_micros_since(recv_us);
        event = Self::process_event(event, bot_wallet);
        callback(&event);

        Ok(())
    }

    // ================================================================================================
    // Helper Functions
    // ================================================================================================

    /// Check if instruction should be processed based on protocol filter
    ///
    /// Determines whether a program_id matches any of the protocols we're interested in.
    fn should_handle(
        protocols: &[Protocol],
        _event_type_filter: Option<&EventTypeFilter>,
        program_id: &Pubkey,
    ) -> bool {
        // 使用 EventDispatcher 来匹配协议
        if let Some(protocol) = EventDispatcher::match_protocol_by_program_id(program_id) {
            protocols.contains(&protocol)
        } else if EventDispatcher::is_compute_budget_program(program_id) {
            return true;
        } else {
            false
        }
    }

    // ================================================================================================
    // Event Post-Processing
    // ================================================================================================

    /// Process and enrich parsed event with additional context
    ///
    /// Handles protocol-specific post-processing:
    /// - PumpFun: Tracks dev addresses and marks dev trades
    /// - PumpSwap: Fills swap data amounts
    /// - Bonk: Tracks pool creators and marks dev trades
    /// - General: Marks bot wallet trades
    fn process_event(event: DexEvent, bot_wallet: Option<Pubkey>) -> DexEvent {
        let signature = event.metadata().signature; // Copy the signature to avoid borrowing issues
        match event {
            DexEvent::PumpFunCreateTokenEvent(token_info) => {
                add_dev_address(&signature, token_info.user);
                if token_info.creator != Pubkey::default() && token_info.creator != token_info.user
                {
                    add_dev_address(&signature, token_info.creator);
                }
                DexEvent::PumpFunCreateTokenEvent(token_info)
            }
            DexEvent::PumpFunCreateV2TokenEvent(token_info) => {
                add_dev_address(&signature, token_info.user);
                if token_info.creator != Pubkey::default() && token_info.creator != token_info.user
                {
                    add_dev_address(&signature, token_info.creator);
                }
                DexEvent::PumpFunCreateV2TokenEvent(token_info)
            }
            DexEvent::PumpFunTradeEvent(mut trade_info) => {
                trade_info.is_dev_create_token_trade =
                    is_dev_address_in_signature(&signature, &trade_info.user)
                        || is_dev_address_in_signature(&signature, &trade_info.creator);
                trade_info.is_bot = Some(trade_info.user) == bot_wallet;

                if let Some(swap_data) = trade_info.metadata.swap_data.as_mut() {
                    swap_data.from_amount = if trade_info.is_buy {
                        trade_info.sol_amount
                    } else {
                        trade_info.token_amount
                    };
                    swap_data.to_amount = if trade_info.is_buy {
                        trade_info.token_amount
                    } else {
                        trade_info.sol_amount
                    };
                }
                DexEvent::PumpFunTradeEvent(trade_info)
            }
            DexEvent::PumpSwapBuyEvent(mut trade_info) => {
                if let Some(swap_data) = trade_info.metadata.swap_data.as_mut() {
                    swap_data.from_amount = trade_info.user_quote_amount_in;
                    swap_data.to_amount = trade_info.base_amount_out;
                }
                DexEvent::PumpSwapBuyEvent(trade_info)
            }
            DexEvent::PumpSwapSellEvent(mut trade_info) => {
                if let Some(swap_data) = trade_info.metadata.swap_data.as_mut() {
                    swap_data.from_amount = trade_info.base_amount_in;
                    swap_data.to_amount = trade_info.user_quote_amount_out;
                }
                DexEvent::PumpSwapSellEvent(trade_info)
            }
            DexEvent::BonkPoolCreateEvent(pool_info) => {
                add_bonk_dev_address(&signature, pool_info.creator);
                DexEvent::BonkPoolCreateEvent(pool_info)
            }
            DexEvent::BonkTradeEvent(mut trade_info) => {
                trade_info.is_dev_create_token_trade =
                    is_bonk_dev_address_in_signature(&signature, &trade_info.payer);
                trade_info.is_bot = Some(trade_info.payer) == bot_wallet;
                DexEvent::BonkTradeEvent(trade_info)
            }
            _ => event,
        }
    }
}
