use crate::streaming::{
    common::SimdUtils,
    event_parser::{
        common::{
            filter::EventTypeFilter,
            high_performance_clock::{elapsed_micros_since, get_high_perf_clock},
            parse_swap_data_from_next_grpc_instructions, parse_swap_data_from_next_instructions,
            EventMetadata, EventType, ProtocolType,
        },
        core::global_state::{
            add_bonk_dev_address, add_dev_address, is_bonk_dev_address_in_signature,
            is_dev_address_in_signature,
        },
        protocols::{
            bonk::{parser::BONK_PROGRAM_ID, BonkPoolCreateEvent, BonkTradeEvent},
            pumpfun::{parser::PUMPFUN_PROGRAM_ID, PumpFunCreateTokenEvent, PumpFunTradeEvent},
            pumpswap::{parser::PUMPSWAP_PROGRAM_ID, PumpSwapBuyEvent, PumpSwapSellEvent},
            raydium_amm_v4::parser::RAYDIUM_AMM_V4_PROGRAM_ID,
            raydium_clmm::parser::RAYDIUM_CLMM_PROGRAM_ID,
            raydium_cpmm::parser::RAYDIUM_CPMM_PROGRAM_ID,
        },
        Protocol, UnifiedEvent,
    },
};
use prost_types::Timestamp;
use solana_sdk::{bs58, message::compiled_instruction::CompiledInstruction, pubkey::Pubkey, signature::Signature, transaction::VersionedTransaction};
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, InnerInstruction, InnerInstructions, UiInstruction,
};
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};
use yellowstone_grpc_proto::geyser::SubscribeUpdateTransactionInfo;

/// 高性能账户公钥缓存，避免重复Vec分配
#[derive(Debug)]
pub struct AccountPubkeyCache {
    /// 预分配的账户公钥向量，避免每次重新分配
    cache: Vec<Pubkey>,
}

impl AccountPubkeyCache {
    /// 创建新的账户公钥缓存
    pub fn new() -> Self {
        Self {
            cache: Vec::with_capacity(32), // 预分配32个位置，覆盖大多数交易
        }
    }

    /// 从指令账户索引构建账户公钥向量，重用缓存内存
    #[inline]
    pub fn build_account_pubkeys(
        &mut self,
        instruction_accounts: &[u8],
        all_accounts: &[Pubkey],
    ) -> &[Pubkey] {
        self.cache.clear();

        // 确保容量足够，避免动态扩容
        if self.cache.capacity() < instruction_accounts.len() {
            self.cache.reserve(instruction_accounts.len() - self.cache.capacity());
        }

        // 快速填充账户公钥
        for &idx in instruction_accounts.iter() {
            if (idx as usize) < all_accounts.len() {
                self.cache.push(all_accounts[idx as usize]);
            }
        }

        &self.cache
    }
}

impl Default for AccountPubkeyCache {
    fn default() -> Self {
        Self::new()
    }
}

/// 内联指令事件解析器
pub type InnerInstructionEventParser =
    fn(data: &[u8], metadata: EventMetadata) -> Option<Box<dyn UnifiedEvent>>;

/// 指令事件解析器
pub type InstructionEventParser =
    fn(data: &[u8], accounts: &[Pubkey], metadata: EventMetadata) -> Option<Box<dyn UnifiedEvent>>;

/// 通用事件解析器配置
#[derive(Debug, Clone)]
pub struct GenericEventParseConfig {
    pub program_id: Pubkey,
    pub protocol_type: ProtocolType,
    pub inner_instruction_discriminator: &'static [u8],
    pub instruction_discriminator: &'static [u8],
    pub event_type: EventType,
    pub inner_instruction_parser: Option<InnerInstructionEventParser>,
    pub instruction_parser: Option<InstructionEventParser>,
    pub requires_inner_instruction: bool,
}

pub static EVENT_PARSERS: LazyLock<HashMap<Protocol, (Pubkey, &[GenericEventParseConfig])>> =
    LazyLock::new(|| {
        // 预分配容量，避免动态扩容
        let mut parsers: HashMap<Protocol, (Pubkey, &[GenericEventParseConfig])> =
            HashMap::with_capacity(6);
        parsers.insert(
            Protocol::PumpSwap,
            (
                PUMPSWAP_PROGRAM_ID,
                crate::streaming::event_parser::protocols::pumpswap::parser::CONFIGS,
            ),
        );
        parsers.insert(
            Protocol::PumpFun,
            (
                PUMPFUN_PROGRAM_ID,
                crate::streaming::event_parser::protocols::pumpfun::parser::CONFIGS,
            ),
        );
        parsers.insert(
            Protocol::Bonk,
            (BONK_PROGRAM_ID, crate::streaming::event_parser::protocols::bonk::parser::CONFIGS),
        );
        parsers.insert(
            Protocol::RaydiumCpmm,
            (
                RAYDIUM_CPMM_PROGRAM_ID,
                crate::streaming::event_parser::protocols::raydium_cpmm::parser::CONFIGS,
            ),
        );
        parsers.insert(
            Protocol::RaydiumClmm,
            (
                RAYDIUM_CLMM_PROGRAM_ID,
                crate::streaming::event_parser::protocols::raydium_clmm::parser::CONFIGS,
            ),
        );
        parsers.insert(
            Protocol::RaydiumAmmV4,
            (
                RAYDIUM_AMM_V4_PROGRAM_ID,
                crate::streaming::event_parser::protocols::raydium_amm_v4::parser::CONFIGS,
            ),
        );
        parsers
    });

/// 通用事件解析器基类
pub struct EventParser {
    pub program_ids: Vec<Pubkey>,
    // pub inner_instruction_configs: HashMap<Vec<u8>, Vec<GenericEventParseConfig>>,
    pub instruction_configs: HashMap<Vec<u8>, Vec<GenericEventParseConfig>>,
    /// 账户公钥缓存，避免重复分配
    pub account_cache: parking_lot::Mutex<AccountPubkeyCache>,
}

impl EventParser {
    pub fn new(protocols: Vec<Protocol>, event_type_filter: Option<EventTypeFilter>) -> Self {
        let mut instruction_configs = HashMap::with_capacity(protocols.len());
        let mut program_ids = Vec::with_capacity(protocols.len());
        // Configure all event types
        for protocol in protocols {
            let parse = EVENT_PARSERS.get(&protocol).unwrap();
            // Merge instruction_configs, append configurations to existing Vec
            parse
                .1
                .iter()
                .filter(|config| {
                    event_type_filter
                        .as_ref()
                        .map(|filter| filter.include.contains(&config.event_type))
                        .unwrap_or(true)
                })
                .for_each(|config| {
                    instruction_configs
                        .entry(config.instruction_discriminator.to_vec())
                        .or_insert_with(Vec::new)
                        .push(config.clone());
                });

            // Append program_ids (this is already appending)
            program_ids.push(parse.0);
        }
        let account_cache = parking_lot::Mutex::new(AccountPubkeyCache::new());

        Self { program_ids, instruction_configs, account_cache }
    }

    #[allow(clippy::too_many_arguments)]
    async fn parse_instruction_events_from_grpc_transaction(
        &self,
        compiled_instructions: &[yellowstone_grpc_proto::prelude::CompiledInstruction],
        signature: Signature,
        slot: Option<u64>,
        block_time: Option<Timestamp>,
        recv_us: i64,
        accounts: &[Pubkey],
        inner_instructions: &[yellowstone_grpc_proto::prelude::InnerInstructions],
        bot_wallet: Option<Pubkey>,
        transaction_index: Option<u64>,
        callback: Arc<dyn for<'a> Fn(&'a Box<dyn UnifiedEvent>) + Send + Sync>,
    ) -> anyhow::Result<()> {
        // 获取交易的指令和账户
        let mut accounts = accounts.to_vec();
        // 检查交易中是否包含程序
        let has_program = accounts.iter().any(|account| self.should_handle(account));
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
                    if self.should_handle(&program_id) {
                        self.parse_events_from_grpc_instruction(
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
                            Arc::clone(&callback),
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
                            self.parse_events_from_grpc_instruction(
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
                                Arc::clone(&callback),
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
        &self,
        transaction: &VersionedTransaction,
        signature: Signature,
        slot: Option<u64>,
        block_time: Option<Timestamp>,
        recv_us: i64,
        accounts: &[Pubkey],
        inner_instructions: &[InnerInstructions],
        bot_wallet: Option<Pubkey>,
        transaction_index: Option<u64>,
        callback: Arc<dyn for<'a> Fn(&'a Box<dyn UnifiedEvent>) + Send + Sync>,
    ) -> anyhow::Result<()> {
        // 获取交易的指令和账户
        let compiled_instructions = transaction.message.instructions();
        let mut accounts: Vec<Pubkey> = accounts.to_vec();
        // 检查交易中是否包含程序
        let has_program = accounts.iter().any(|account| self.should_handle(account));
        if has_program {
            // 解析每个指令
            for (index, instruction) in compiled_instructions.iter().enumerate() {
                if let Some(program_id) = accounts.get(instruction.program_id_index as usize) {
                    let program_id = *program_id; // 克隆程序ID，避免借用冲突
                    let inner_instructions = inner_instructions
                        .iter()
                        .find(|inner_instruction| inner_instruction.index == index as u8);
                    if self.should_handle(&program_id) {
                        let max_idx = instruction.accounts.iter().max().unwrap_or(&0);
                        // 补齐accounts(使用Pubkey::default())
                        if *max_idx as usize >= accounts.len() {
                            accounts.resize(*max_idx as usize + 1, Pubkey::default());
                        }
                        self.parse_events_from_instruction(
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
                            Arc::clone(&callback),
                        )?;
                    }
                    // Immediately process inner instructions for correct ordering
                    if let Some(inner_instructions) = inner_instructions {
                        for (inner_index, inner_instruction) in
                            inner_instructions.instructions.iter().enumerate()
                        {
                            self.parse_events_from_instruction(
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
                                Arc::clone(&callback),
                            )?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn parse_versioned_transaction_owned(
        &self,
        versioned_tx: VersionedTransaction,
        signature: Signature,
        slot: Option<u64>,
        block_time: Option<Timestamp>,
        recv_us: i64,
        bot_wallet: Option<Pubkey>,
        transaction_index: Option<u64>,
        inner_instructions: &[InnerInstructions],
        callback: Arc<dyn Fn(Box<dyn UnifiedEvent>) + Send + Sync>,
    ) -> anyhow::Result<()> {
        // 创建适配器回调，将所有权回调转换为引用回调
        let adapter_callback = Arc::new(move |event: &Box<dyn UnifiedEvent>| {
            callback(event.clone_boxed());
        });
        self.parse_versioned_transaction(
            &versioned_tx,
            signature,
            slot,
            block_time,
            recv_us,
            bot_wallet,
            transaction_index,
            inner_instructions,
            adapter_callback,
        )
        .await?;
        Ok(())
    }

    async fn parse_versioned_transaction(
        &self,
        versioned_tx: &VersionedTransaction,
        signature: Signature,
        slot: Option<u64>,
        block_time: Option<Timestamp>,
        recv_us: i64,
        bot_wallet: Option<Pubkey>,
        transaction_index: Option<u64>,
        inner_instructions: &[InnerInstructions],
        callback: Arc<dyn for<'a> Fn(&'a Box<dyn UnifiedEvent>) + Send + Sync>,
    ) -> anyhow::Result<()> {
        let accounts: Vec<Pubkey> = versioned_tx.message.static_account_keys().to_vec();
        self.parse_instruction_events_from_versioned_transaction(
            versioned_tx,
            signature,
            slot,
            block_time,
            recv_us,
            &accounts,
            inner_instructions,
            bot_wallet,
            transaction_index,
            callback,
        )
        .await?;
        Ok(())
    }

    pub async fn parse_grpc_transaction_owned(
        &self,
        grpc_tx: SubscribeUpdateTransactionInfo,
        signature: Signature,
        slot: Option<u64>,
        block_time: Option<Timestamp>,
        recv_us: i64,
        bot_wallet: Option<Pubkey>,
        transaction_index: Option<u64>,
        callback: Arc<dyn Fn(Box<dyn UnifiedEvent>) + Send + Sync>,
    ) -> anyhow::Result<()> {
        // 创建适配器回调，将所有权回调转换为引用回调
        let adapter_callback = Arc::new(move |event: &Box<dyn UnifiedEvent>| {
            callback(event.clone_boxed());
        });
        // 调用原始方法
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
        &self,
        grpc_tx: SubscribeUpdateTransactionInfo,
        signature: Signature,
        slot: Option<u64>,
        block_time: Option<Timestamp>,
        recv_us: i64,
        bot_wallet: Option<Pubkey>,
        transaction_index: Option<u64>,
        callback: Arc<dyn for<'a> Fn(&'a Box<dyn UnifiedEvent>) + Send + Sync>,
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
                // 使用 Arc 包装共享数据，避免不必要的克隆
                let accounts_arc = Arc::new(accounts);
                let inner_instructions_arc = Arc::new(inner_instructions);
                // 解析指令事件
                let instructions = &message.instructions;
                self.parse_instruction_events_from_grpc_transaction(
                    &instructions,
                    signature,
                    slot,
                    block_time,
                    recv_us,
                    &accounts_arc,
                    &inner_instructions_arc,
                    bot_wallet,
                    transaction_index,
                    callback.clone(),
                )
                .await?;
            }
        }

        Ok(())
    }

    pub async fn parse_encoded_confirmed_transaction_with_status_meta(
        &self,
        signature: Signature,
        transaction: EncodedConfirmedTransactionWithStatusMeta,
        callback: Arc<dyn for<'a> Fn(&'a Box<dyn UnifiedEvent>) + Send + Sync>,
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
                                    accounts: ui_compiled.accounts.clone(),
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
        // 使用 Arc 包装共享数据，避免不必要的克隆
        let accounts_arc = Arc::new(accounts);
        let inner_instructions_arc = Arc::new(inner_instructions);

        let slot = transaction.slot;
        let block_time = transaction.block_time.map(|t| Timestamp { seconds: t as i64, nanos: 0 });
        let recv_us = get_high_perf_clock();
        let bot_wallet = None;
        let transaction_index = None;
        // 解析指令事件
        self.parse_instruction_events_from_versioned_transaction(
            &versioned_tx,
            signature,
            Some(slot),
            block_time,
            recv_us,
            &accounts_arc,
            &inner_instructions_arc,
            bot_wallet,
            transaction_index,
            callback.clone(),
        )
        .await?;

        Ok(())
    }

    /// 通用的内联指令解析方法
    #[allow(clippy::too_many_arguments)]
    fn parse_inner_instruction_event(
        &self,
        config: &GenericEventParseConfig,
        data: &[u8],
        signature: Signature,
        slot: u64,
        block_time: Option<Timestamp>,
        recv_us: i64,
        outer_index: i64,
        inner_index: Option<i64>,
        transaction_index: Option<u64>,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if let Some(parser) = config.inner_instruction_parser {
            let timestamp = block_time.unwrap_or(Timestamp { seconds: 0, nanos: 0 });
            let block_time_ms = timestamp.seconds * 1000 + (timestamp.nanos as i64) / 1_000_000;
            let metadata = EventMetadata::new(
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
            );
            parser(data, metadata)
        } else {
            None
        }
    }

    /// 通用的指令解析方法
    #[allow(clippy::too_many_arguments)]
    fn parse_instruction_event(
        &self,
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
    ) -> Option<Box<dyn UnifiedEvent>> {
        if let Some(parser) = config.instruction_parser {
            let timestamp = block_time.unwrap_or(Timestamp { seconds: 0, nanos: 0 });
            let block_time_ms = timestamp.seconds * 1000 + (timestamp.nanos as i64) / 1_000_000;
            let metadata = EventMetadata::new(
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
            );
            parser(data, account_pubkeys, metadata)
        } else {
            None
        }
    }

    /// 从内联指令中解析事件数据
    #[allow(clippy::too_many_arguments)]
    fn parse_events_from_inner_instruction(
        &self,
        inner_instruction: &CompiledInstruction,
        signature: Signature,
        slot: u64,
        block_time: Option<Timestamp>,
        recv_us: i64,
        outer_index: i64,
        inner_index: Option<i64>,
        transaction_index: Option<u64>,
        config: &GenericEventParseConfig,
    ) -> Vec<Box<dyn UnifiedEvent>> {
        // Use SIMD-optimized data validation with correct discriminator length
        let discriminator_len = config.inner_instruction_discriminator.len();
        if !SimdUtils::validate_instruction_data_simd(
            &inner_instruction.data,
            16,
            discriminator_len,
        ) {
            return Vec::new();
        }

        // Use SIMD-optimized discriminator matching
        if !SimdUtils::fast_discriminator_match(
            &inner_instruction.data,
            config.inner_instruction_discriminator,
        ) {
            return Vec::new();
        }

        let data = &inner_instruction.data[16..];
        let mut events = Vec::new();
        if let Some(event) = self.parse_inner_instruction_event(
            config,
            data,
            signature,
            slot,
            block_time,
            recv_us,
            outer_index,
            inner_index,
            transaction_index,
        ) {
            events.push(event);
        }
        events
    }

    /// 从内联指令中解析事件数据
    #[allow(clippy::too_many_arguments)]
    fn parse_events_from_grpc_inner_instruction(
        &self,
        inner_instruction: &yellowstone_grpc_proto::prelude::InnerInstruction,
        signature: Signature,
        slot: u64,
        block_time: Option<Timestamp>,
        recv_us: i64,
        outer_index: i64,
        inner_index: Option<i64>,
        transaction_index: Option<u64>,
        config: &GenericEventParseConfig,
    ) -> Vec<Box<dyn UnifiedEvent>> {
        // Use SIMD-optimized data validation with correct discriminator length
        let discriminator_len = config.inner_instruction_discriminator.len();
        if !SimdUtils::validate_instruction_data_simd(
            &inner_instruction.data,
            16,
            discriminator_len,
        ) {
            return Vec::new();
        }

        // Use SIMD-optimized discriminator matching
        if !SimdUtils::fast_discriminator_match(
            &inner_instruction.data,
            config.inner_instruction_discriminator,
        ) {
            return Vec::new();
        }

        let data = &inner_instruction.data[16..];
        let mut events = Vec::new();
        if let Some(event) = self.parse_inner_instruction_event(
            config,
            data,
            signature,
            slot,
            block_time,
            recv_us,
            outer_index,
            inner_index,
            transaction_index,
        ) {
            events.push(event);
        }
        events
    }

    /// 从指令中解析事件
    #[allow(clippy::too_many_arguments)]
    fn parse_events_from_instruction(
        &self,
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
        callback: Arc<dyn for<'a> Fn(&'a Box<dyn UnifiedEvent>) + Send + Sync>,
    ) -> anyhow::Result<()> {
        let program_id = accounts[instruction.program_id_index as usize];
        if !self.should_handle(&program_id) {
            return Ok(());
        }
        // 一维化并行处理：将所有 (discriminator, config) 组合展开并行处理
        let all_processing_params: Vec<_> = self
            .instruction_configs
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

        // 使用缓存构建账户公钥列表，避免重复分配 (只需构建一次)
        let account_pubkeys = {
            let mut cache_guard = self.account_cache.lock();
            cache_guard.build_account_pubkeys(&instruction.accounts, accounts).to_vec()
        };

        // 并行处理所有 (discriminator, config) 组合
        let all_results: Vec<_> = all_processing_params
            .iter()
            .filter_map(|(disc, config)| {
                let data = &instruction.data[disc.len()..];
                self.parse_instruction_event(
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
                .map(|event| ((*disc).clone(), (*config).clone(), event))
            })
            .collect();

        for (_disc, config, mut event) in all_results {
            // 阻塞处理：原有的同步逻辑
            let mut inner_instruction_event: Option<Box<dyn UnifiedEvent>> = None;
            if inner_instructions.is_some() {
                let inner_instructions_ref = inner_instructions.unwrap();

                // 并行执行两个任务
                let (inner_event_result, swap_data_result) = std::thread::scope(|s| {
                    let inner_event_handle = s.spawn(|| {
                        for inner_instruction in inner_instructions_ref.instructions.iter() {
                            let result = self.parse_events_from_inner_instruction(
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
                            if result.len() > 0 {
                                return Some(result[0].clone());
                            }
                        }
                        None
                    });

                    let swap_data_handle = s.spawn(|| {
                        if !event.swap_data_is_parsed() {
                            parse_swap_data_from_next_instructions(
                                &*event,
                                inner_instructions_ref,
                                inner_index.unwrap_or(-1_i64) as i8,
                                &accounts,
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
                    event.set_swap_data(swap_data);
                }
            }

            // Skip events that require inner instruction data but don't have it
            if config.requires_inner_instruction && inner_instruction_event.is_none() {
                continue;
            }

            // 合并事件
            if let Some(inner_instruction_event) = inner_instruction_event {
                event.merge(&*inner_instruction_event);
            }
            // 设置处理时间（使用高性能时钟）
            event.set_handle_us(elapsed_micros_since(recv_us));
            event = process_event(event, bot_wallet);
            callback(&event);
        }
        Ok(())
    }

    /// 从指令中解析事件
    /// TODO: - wait refactor
    #[allow(clippy::too_many_arguments)]
    fn parse_events_from_grpc_instruction(
        &self,
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
        callback: Arc<dyn for<'a> Fn(&'a Box<dyn UnifiedEvent>) + Send + Sync>,
    ) -> anyhow::Result<()> {
        let program_id = accounts[instruction.program_id_index as usize];
        if !self.should_handle(&program_id) {
            return Ok(());
        }
        // 一维化并行处理：将所有 (discriminator, config) 组合展开并行处理
        let all_processing_params: Vec<_> = self
            .instruction_configs
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

        // 使用缓存构建账户公钥列表，避免重复分配 (只需构建一次)
        let account_pubkeys = {
            let mut cache_guard = self.account_cache.lock();
            cache_guard.build_account_pubkeys(&instruction.accounts, accounts).to_vec()
        };

        // 并行处理所有 (discriminator, config) 组合
        let all_results: Vec<_> = all_processing_params
            .iter()
            .filter_map(|(disc, config)| {
                let data = &instruction.data[disc.len()..];
                self.parse_instruction_event(
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
                .map(|event| ((*disc).clone(), (*config).clone(), event))
            })
            .collect();

        for (_disc, config, mut event) in all_results {
            // 阻塞处理：原有的同步逻辑
            let mut inner_instruction_event: Option<Box<dyn UnifiedEvent>> = None;
            if inner_instructions.is_some() {
                let inner_instructions_ref = inner_instructions.unwrap();

                // 并行执行两个任务
                let (inner_event_result, swap_data_result) = std::thread::scope(|s| {
                    let inner_event_handle = s.spawn(|| {
                        for inner_instruction in inner_instructions_ref.instructions.iter() {
                            let result = self.parse_events_from_grpc_inner_instruction(
                                &inner_instruction,
                                signature,
                                slot,
                                block_time,
                                recv_us,
                                outer_index,
                                inner_index,
                                transaction_index,
                                &config,
                            );
                            if result.len() > 0 {
                                return Some(result[0].clone());
                            }
                        }
                        None
                    });

                    let swap_data_handle = s.spawn(|| {
                        if !event.swap_data_is_parsed() {
                            parse_swap_data_from_next_grpc_instructions(
                                &*event,
                                inner_instructions_ref,
                                inner_index.unwrap_or(-1_i64) as i8,
                                &accounts,
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
                    event.set_swap_data(swap_data);
                }
            }

            // Skip events that require inner instruction data but don't have it
            if config.requires_inner_instruction && inner_instruction_event.is_none() {
                continue;
            }

            // 合并事件
            if let Some(inner_instruction_event) = inner_instruction_event {
                event.merge(&*inner_instruction_event);
            }
            // 设置处理时间（使用高性能时钟）
            event.set_handle_us(elapsed_micros_since(recv_us));
            event = process_event(event, bot_wallet);
            callback(&event);
        }
        Ok(())
    }

    fn should_handle(&self, program_id: &Pubkey) -> bool {
        self.program_ids.contains(program_id)
    }

    // fn supported_program_ids(&self) -> Vec<Pubkey> {
    //     self.program_ids.clone()
    // }
}

fn process_event(
    mut event: Box<dyn UnifiedEvent>,
    bot_wallet: Option<Pubkey>,
) -> Box<dyn UnifiedEvent> {
    let signature = *event.signature(); // Copy the signature to avoid borrowing issues
    if let Some(token_info) = event.as_any().downcast_ref::<PumpFunCreateTokenEvent>() {
        add_dev_address(&signature, token_info.user);
        if token_info.creator != Pubkey::default() && token_info.creator != token_info.user {
            add_dev_address(&signature, token_info.creator);
        }
    } else if let Some(trade_info) = event.as_any_mut().downcast_mut::<PumpFunTradeEvent>() {
        if is_dev_address_in_signature(&signature, &trade_info.user)
            || is_dev_address_in_signature(&signature, &trade_info.creator)
        {
            trade_info.is_dev_create_token_trade = true;
        } else if Some(trade_info.user) == bot_wallet {
            trade_info.is_bot = true;
        } else {
            trade_info.is_dev_create_token_trade = false;
        }
        if trade_info.metadata.swap_data.is_some() {
            trade_info.metadata.swap_data.as_mut().unwrap().from_amount =
                if trade_info.is_buy { trade_info.sol_amount } else { trade_info.token_amount };
            trade_info.metadata.swap_data.as_mut().unwrap().to_amount =
                if trade_info.is_buy { trade_info.token_amount } else { trade_info.sol_amount };
        }
    } else if let Some(trade_info) = event.as_any_mut().downcast_mut::<PumpSwapBuyEvent>() {
        if trade_info.metadata.swap_data.is_some() {
            trade_info.metadata.swap_data.as_mut().unwrap().from_amount =
                trade_info.user_quote_amount_in;
            trade_info.metadata.swap_data.as_mut().unwrap().to_amount = trade_info.base_amount_out;
        }
    } else if let Some(trade_info) = event.as_any_mut().downcast_mut::<PumpSwapSellEvent>() {
        if trade_info.metadata.swap_data.is_some() {
            trade_info.metadata.swap_data.as_mut().unwrap().from_amount = trade_info.base_amount_in;
            trade_info.metadata.swap_data.as_mut().unwrap().to_amount =
                trade_info.user_quote_amount_out;
        }
    } else if let Some(pool_info) = event.as_any().downcast_ref::<BonkPoolCreateEvent>() {
        add_bonk_dev_address(&signature, pool_info.creator);
    } else if let Some(trade_info) = event.as_any_mut().downcast_mut::<BonkTradeEvent>() {
        if is_bonk_dev_address_in_signature(&signature, &trade_info.payer) {
            trade_info.is_dev_create_token_trade = true;
        } else if Some(trade_info.payer) == bot_wallet {
            trade_info.is_bot = true;
        } else {
            trade_info.is_dev_create_token_trade = false;
        }
    }
    event
}
