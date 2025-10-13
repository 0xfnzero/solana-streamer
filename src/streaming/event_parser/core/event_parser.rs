use crate::streaming::{
    common::SimdUtils,
    event_parser::{
        common::{
            filter::EventTypeFilter,
            high_performance_clock::{elapsed_micros_since, get_high_perf_clock},
            parse_swap_data_from_next_grpc_instructions, parse_swap_data_from_next_instructions,
            EventMetadata,
        },
        core::{
            global_state::{
                add_bonk_dev_address, add_dev_address, is_bonk_dev_address_in_signature,
                is_dev_address_in_signature,
            },
            merger_event::merge,
            parser_cache::{
                build_account_pubkeys_with_cache, get_global_instruction_configs,
                get_global_program_ids, GenericEventParseConfig,
            },
        },
        DexEvent, Protocol,
    },
};
use prost_types::Timestamp;
use solana_sdk::{
    bs58, message::compiled_instruction::CompiledInstruction, pubkey::Pubkey, signature::Signature,
    transaction::VersionedTransaction,
};
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, InnerInstruction, InnerInstructions, UiInstruction,
};
use std::sync::Arc;
use yellowstone_grpc_proto::geyser::SubscribeUpdateTransactionInfo;

pub struct EventParser {}

impl EventParser {
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

    /// 从VersionedTransaction中解析指令事件的通用方法
    #[allow(clippy::too_many_arguments)]
    async fn parse_instruction_events_from_versioned_transaction(
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
        callback: Arc<dyn for<'a> Fn(&'a DexEvent) + Send + Sync>,
    ) -> anyhow::Result<()> {
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
                            callback.clone(),
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
                                callback.clone(),
                            )?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn parse_versioned_transaction_owned(
        protocols: &[Protocol],
        event_type_filter: Option<&EventTypeFilter>,
        versioned_tx: VersionedTransaction,
        signature: Signature,
        slot: Option<u64>,
        block_time: Option<Timestamp>,
        recv_us: i64,
        bot_wallet: Option<Pubkey>,
        transaction_index: Option<u64>,
        inner_instructions: &[InnerInstructions],
        callback: Arc<dyn Fn(DexEvent) + Send + Sync>,
    ) -> anyhow::Result<()> {
        // 创建适配器回调，将所有权回调转换为引用回调
        let adapter_callback = Arc::new(move |event: &DexEvent| {
            callback(event.clone());
        });
        let accounts = versioned_tx.message.static_account_keys();
        Self::parse_instruction_events_from_versioned_transaction(
            protocols,
            event_type_filter,
            &versioned_tx,
            signature,
            slot,
            block_time,
            recv_us,
            accounts,
            inner_instructions,
            bot_wallet,
            transaction_index,
            adapter_callback,
        )
        .await
    }

    pub async fn parse_grpc_transaction_owned(
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
        // 调用原始方法
        Self::parse_grpc_transaction(
            protocols,
            event_type_filter,
            grpc_tx,
            signature,
            slot,
            block_time,
            recv_us,
            bot_wallet,
            transaction_index,
            adapter_callback,
        )
        .await
    }

    /// Enhanced version that passes raw gRPC transaction data to the callback
    pub async fn parse_grpc_transaction_owned_with_raw_data(
        &self,
        grpc_tx: SubscribeUpdateTransactionInfo,
        signature: Signature,
        slot: Option<u64>,
        block_time: Option<Timestamp>,
        recv_us: i64,
        bot_wallet: Option<Pubkey>,
        transaction_index: Option<u64>,
        enhanced_callback: crate::streaming::event_parser::core::traits::EnhancedEventCallback,
    ) -> anyhow::Result<()> {
        // Convert gRPC transaction to EncodedTransactionWithStatusMeta for raw data access
        let raw_transaction = match yellowstone_grpc_proto::convert_from::create_tx_with_meta(grpc_tx.clone()) {
            Ok(tx_with_meta) => {
                Some(tx_with_meta.encode(
                    solana_transaction_status::UiTransactionEncoding::JsonParsed,
                    Some(u8::MAX),
                    true
                ).unwrap())
            }
            Err(_) => None
        };

        // Create adapter callback that passes both event and raw transaction data
        let adapter_callback = Arc::new(move |event: &Box<dyn UnifiedEvent>| {
            enhanced_callback(event.clone_boxed(), raw_transaction.as_ref());
        });

        // Call the original parsing method
        self.parse_grpc_transaction(
            grpc_tx,
            signature,
            slot,
            block_time,
            recv_us,
            bot_wallet,
            transaction_index,
            adapter_callback,
        )
        .await
    }

    async fn parse_grpc_transaction(
        protocols: &[Protocol],
        event_type_filter: Option<&EventTypeFilter>,
        grpc_tx: SubscribeUpdateTransactionInfo,
        signature: Signature,
        slot: Option<u64>,
        block_time: Option<Timestamp>,
        recv_us: i64,
        bot_wallet: Option<Pubkey>,
        transaction_index: Option<u64>,
        callback: Arc<dyn for<'a> Fn(&'a DexEvent) + Send + Sync>,
    ) -> anyhow::Result<()> {
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
                    callback,
                )
                .await?;
            }
        }

        Ok(())
    }

    pub async fn parse_encoded_confirmed_transaction_with_status_meta(
        protocols: &[Protocol],
        event_type_filter: Option<&EventTypeFilter>,
        signature: Signature,
        transaction: EncodedConfirmedTransactionWithStatusMeta,
        callback: Arc<dyn for<'a> Fn(&'a DexEvent) + Send + Sync>,
    ) -> anyhow::Result<()> {
        let versioned_tx = match transaction.transaction.transaction.decode() {
            Some(tx) => tx,
            None => {
                return Ok(());
            }
        };
        let mut inner_instructions_vec: Vec<InnerInstructions> = Vec::new();
        if let Some(meta) = &transaction.transaction.meta {
            // 从meta中获取inner_instructions，处理OptionSerializer类型
            if let solana_transaction_status::option_serializer::OptionSerializer::Some(
                ui_inner_insts,
            ) = &meta.inner_instructions
            {
                // 将UiInnerInstructions转换为InnerInstructions
                for ui_inner in ui_inner_insts {
                    let mut converted_instructions = Vec::new();

                    // 转换每个UiInstruction为InnerInstruction
                    for ui_instruction in &ui_inner.instructions {
                        if let UiInstruction::Compiled(ui_compiled) = ui_instruction {
                            // 解码base58编码的data
                            if let Ok(data) = bs58::decode(&ui_compiled.data).into_vec() {
                                // base64解码
                                let compiled_instruction = CompiledInstruction {
                                    program_id_index: ui_compiled.program_id_index,
                                    accounts: ui_compiled.accounts.to_vec(),
                                    data,
                                };

                                let inner_instruction = InnerInstruction {
                                    instruction: compiled_instruction,
                                    stack_height: ui_compiled.stack_height,
                                };

                                converted_instructions.push(inner_instruction);
                            }
                        }
                    }

                    let inner_instructions = InnerInstructions {
                        index: ui_inner.index,
                        instructions: converted_instructions,
                    };

                    inner_instructions_vec.push(inner_instructions);
                }
            }
        }
        let inner_instructions: &[InnerInstructions] = &inner_instructions_vec;

        let meta = transaction.transaction.meta;
        let mut address_table_lookups: Vec<Pubkey> = vec![];
        if let Some(meta) = meta {
            if let solana_transaction_status::option_serializer::OptionSerializer::Some(
                loaded_addresses,
            ) = &meta.loaded_addresses
            {
                address_table_lookups
                    .reserve(loaded_addresses.writable.len() + loaded_addresses.readonly.len());
                address_table_lookups.extend(
                    loaded_addresses
                        .writable
                        .iter()
                        .filter_map(|s| s.parse::<Pubkey>().ok())
                        .chain(
                            loaded_addresses
                                .readonly
                                .iter()
                                .filter_map(|s| s.parse::<Pubkey>().ok()),
                        ),
                );
            }
        }
        let mut accounts = Vec::with_capacity(
            versioned_tx.message.static_account_keys().len() + address_table_lookups.len(),
        );
        accounts.extend_from_slice(versioned_tx.message.static_account_keys());
        accounts.extend(address_table_lookups);

        let slot = transaction.slot;
        let block_time = transaction.block_time.map(|t| Timestamp { seconds: t as i64, nanos: 0 });
        let recv_us = get_high_perf_clock();
        let bot_wallet = None;
        let transaction_index = None;
        // 解析指令事件
        Self::parse_instruction_events_from_versioned_transaction(
            protocols,
            event_type_filter,
            &versioned_tx,
            signature,
            Some(slot),
            block_time,
            recv_us,
            &accounts,
            inner_instructions,
            bot_wallet,
            transaction_index,
            callback,
        )
        .await
    }

    /// Helper function to create EventMetadata from common parameters
    #[inline]
    fn create_metadata(
        config: &GenericEventParseConfig,
        signature: Signature,
        slot: u64,
        block_time: Option<Timestamp>,
        recv_us: i64,
        outer_index: i64,
        inner_index: Option<i64>,
        transaction_index: Option<u64>,
    ) -> EventMetadata {
        let timestamp = block_time.unwrap_or(Timestamp { seconds: 0, nanos: 0 });
        let block_time_ms = timestamp.seconds * 1000 + (timestamp.nanos as i64) / 1_000_000;
        EventMetadata::new(
            signature,
            slot,
            timestamp.seconds,
            block_time_ms,
            config.protocol_type.clone(),
            config.event_type.clone(),
            config.program_id,
            outer_index,
            inner_index,
            recv_us,
            transaction_index,
        )
    }

    /// 通用的内联指令解析方法
    #[allow(clippy::too_many_arguments)]
    fn parse_inner_instruction_event(
        config: &GenericEventParseConfig,
        data: &[u8],
        signature: Signature,
        slot: u64,
        block_time: Option<Timestamp>,
        recv_us: i64,
        outer_index: i64,
        inner_index: Option<i64>,
        transaction_index: Option<u64>,
    ) -> Option<DexEvent> {
        config.inner_instruction_parser.and_then(|parser| {
            let metadata = Self::create_metadata(
                config,
                signature,
                slot,
                block_time,
                recv_us,
                outer_index,
                inner_index,
                transaction_index,
            );
            parser(data, metadata)
        })
    }

    /// 通用的指令解析方法
    #[allow(clippy::too_many_arguments)]
    fn parse_instruction_event(
        config: &GenericEventParseConfig,
        data: &[u8],
        account_pubkeys: &[Pubkey],
        signature: Signature,
        slot: u64,
        block_time: Option<Timestamp>,
        recv_us: i64,
        outer_index: i64,
        inner_index: Option<i64>,
        transaction_index: Option<u64>,
    ) -> Option<DexEvent> {
        config.instruction_parser.and_then(|parser| {
            let metadata = Self::create_metadata(
                config,
                signature,
                slot,
                block_time,
                recv_us,
                outer_index,
                inner_index,
                transaction_index,
            );
            parser(data, account_pubkeys, metadata)
        })
    }

    /// 从内联指令中解析事件数据 - 通用实现
    #[allow(clippy::too_many_arguments)]
    fn parse_events_from_inner_instruction_data(
        data: &[u8],
        signature: Signature,
        slot: u64,
        block_time: Option<Timestamp>,
        recv_us: i64,
        outer_index: i64,
        inner_index: Option<i64>,
        transaction_index: Option<u64>,
        config: &GenericEventParseConfig,
    ) -> Vec<DexEvent> {
        // Use SIMD-optimized data validation with correct discriminator length
        let discriminator_len = config.inner_instruction_discriminator.len();
        if !SimdUtils::validate_instruction_data_simd(data, 16, discriminator_len) {
            return Vec::new();
        }

        // Use SIMD-optimized discriminator matching
        if !SimdUtils::fast_discriminator_match(data, config.inner_instruction_discriminator) {
            return Vec::new();
        }

        let data = &data[16..];
        Self::parse_inner_instruction_event(
            config,
            data,
            signature,
            slot,
            block_time,
            recv_us,
            outer_index,
            inner_index,
            transaction_index,
        )
        .into_iter()
        .collect()
    }

    /// 从内联指令中解析事件数据
    #[allow(clippy::too_many_arguments)]
    fn parse_events_from_inner_instruction(
        inner_instruction: &CompiledInstruction,
        signature: Signature,
        slot: u64,
        block_time: Option<Timestamp>,
        recv_us: i64,
        outer_index: i64,
        inner_index: Option<i64>,
        transaction_index: Option<u64>,
        config: &GenericEventParseConfig,
    ) -> Vec<DexEvent> {
        Self::parse_events_from_inner_instruction_data(
            &inner_instruction.data,
            signature,
            slot,
            block_time,
            recv_us,
            outer_index,
            inner_index,
            transaction_index,
            config,
        )
    }

    /// 从内联指令中解析事件数据
    #[allow(clippy::too_many_arguments)]
    fn parse_events_from_grpc_inner_instruction(
        inner_instruction: &yellowstone_grpc_proto::prelude::InnerInstruction,
        signature: Signature,
        slot: u64,
        block_time: Option<Timestamp>,
        recv_us: i64,
        outer_index: i64,
        inner_index: Option<i64>,
        transaction_index: Option<u64>,
        config: &GenericEventParseConfig,
    ) -> Vec<DexEvent> {
        Self::parse_events_from_inner_instruction_data(
            &inner_instruction.data,
            signature,
            slot,
            block_time,
            recv_us,
            outer_index,
            inner_index,
            transaction_index,
            config,
        )
    }

    /// 从指令中解析事件
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
        // 一维化并行处理：将所有 (discriminator, config) 组合展开并行处理
        let instruction_configs = get_global_instruction_configs(protocols, event_type_filter);
        let all_processing_params: Vec<_> = instruction_configs
            .iter()
            .filter(|(disc, _)| {
                // Use SIMD-optimized data validation and discriminator matching
                SimdUtils::validate_instruction_data_simd(&instruction.data, disc.len(), disc.len())
                    && SimdUtils::fast_discriminator_match(&instruction.data, disc)
            })
            .flat_map(|(disc, configs)| {
                configs
                    .iter()
                    .filter(|config| config.program_id == program_id)
                    .map(move |config| (disc, config))
            })
            .collect();

        // Use SIMD-optimized account indices validation (只需检查一次)
        if !SimdUtils::validate_account_indices_simd(&instruction.accounts, accounts.len()) {
            return Ok(());
        }

        // 使用线程局部缓存构建账户公钥列表，避免重复分配 (只需构建一次)
        let account_pubkeys = build_account_pubkeys_with_cache(&instruction.accounts, accounts);

        // 并行处理所有 (discriminator, config) 组合
        let all_results: Vec<_> = all_processing_params
            .iter()
            .filter_map(|(disc, config)| {
                let data = &instruction.data[disc.len()..];
                Self::parse_instruction_event(
                    config,
                    data,
                    &account_pubkeys,
                    signature,
                    slot,
                    block_time,
                    recv_us,
                    outer_index,
                    inner_index,
                    transaction_index,
                )
                .map(|event| (*disc, *config, event))
            })
            .collect();

        for (_disc, config, mut event) in all_results {
            // 阻塞处理：原有的同步逻辑
            let mut inner_instruction_event: Option<DexEvent> = None;
            if let Some(inner_instructions_ref) = inner_instructions {
                // 并行执行两个任务
                let (inner_event_result, swap_data_result) = std::thread::scope(|s| {
                    let inner_event_handle = s.spawn(|| {
                        for inner_instruction in inner_instructions_ref.instructions.iter() {
                            let result = Self::parse_events_from_inner_instruction(
                                &inner_instruction.instruction,
                                signature,
                                slot,
                                block_time,
                                recv_us,
                                outer_index,
                                inner_index,
                                transaction_index,
                                &config,
                            );
                            if !result.is_empty() {
                                return result.into_iter().next();
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

            // Skip events that require inner instruction data but don't have it
            if config.requires_inner_instruction && inner_instruction_event.is_none() {
                continue;
            }

            // 合并事件
            if let Some(inner_instruction_event) = inner_instruction_event {
                merge(&mut event, inner_instruction_event);
            }
            // 设置处理时间（使用高性能时钟）
            event.metadata_mut().handle_us = elapsed_micros_since(recv_us);
            event = Self::process_event(event, bot_wallet);
            callback(&event);
        }
        Ok(())
    }

    /// 从指令中解析事件
    /// TODO: - wait refactor
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
        // 一维化并行处理：将所有 (discriminator, config) 组合展开并行处理
        let instruction_configs = get_global_instruction_configs(protocols, event_type_filter);
        let all_processing_params: Vec<_> = instruction_configs
            .iter()
            .filter(|(disc, _)| {
                // Use SIMD-optimized data validation and discriminator matching
                SimdUtils::validate_instruction_data_simd(&instruction.data, disc.len(), disc.len())
                    && SimdUtils::fast_discriminator_match(&instruction.data, disc)
            })
            .flat_map(|(disc, configs)| {
                configs
                    .iter()
                    .filter(|config| config.program_id == program_id)
                    .map(move |config| (disc, config))
            })
            .collect();

        // Use SIMD-optimized account indices validation (只需检查一次)
        if !SimdUtils::validate_account_indices_simd(&instruction.accounts, accounts.len()) {
            return Ok(());
        }

        // 使用线程局部缓存构建账户公钥列表，避免重复分配 (只需构建一次)
        let account_pubkeys = build_account_pubkeys_with_cache(&instruction.accounts, accounts);

        // 并行处理所有 (discriminator, config) 组合
        let all_results: Vec<_> = all_processing_params
            .iter()
            .filter_map(|(disc, config)| {
                let data = &instruction.data[disc.len()..];
                Self::parse_instruction_event(
                    config,
                    data,
                    &account_pubkeys,
                    signature,
                    slot,
                    block_time,
                    recv_us,
                    outer_index,
                    inner_index,
                    transaction_index,
                )
                .map(|event| (*disc, *config, event))
            })
            .collect();

        for (_disc, config, mut event) in all_results {
            // 阻塞处理：原有的同步逻辑
            let mut inner_instruction_event: Option<DexEvent> = None;
            if let Some(inner_instructions_ref) = inner_instructions {
                // 并行执行两个任务
                let (inner_event_result, swap_data_result) = std::thread::scope(|s| {
                    let inner_event_handle = s.spawn(|| {
                        for inner_instruction in inner_instructions_ref.instructions.iter() {
                            let result = Self::parse_events_from_grpc_inner_instruction(
                                inner_instruction,
                                signature,
                                slot,
                                block_time,
                                recv_us,
                                outer_index,
                                inner_index,
                                transaction_index,
                                &config,
                            );
                            if !result.is_empty() {
                                return result.into_iter().next();
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

            // Skip events that require inner instruction data but don't have it
            if config.requires_inner_instruction && inner_instruction_event.is_none() {
                continue;
            }

            // 合并事件
            if let Some(inner_instruction_event) = inner_instruction_event {
                merge(&mut event, inner_instruction_event);
            }
            // 设置处理时间（使用高性能时钟）
            event.metadata_mut().handle_us = elapsed_micros_since(recv_us);
            event = Self::process_event(event, bot_wallet);
            callback(&event);
        }
        Ok(())
    }

    fn should_handle(
        protocols: &[Protocol],
        event_type_filter: Option<&EventTypeFilter>,
        program_id: &Pubkey,
    ) -> bool {
        get_global_program_ids(protocols, event_type_filter).contains(program_id)
    }

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
