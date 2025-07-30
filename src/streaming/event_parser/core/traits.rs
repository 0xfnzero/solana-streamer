use anyhow::Result;
use prost_types::Timestamp;
use solana_sdk::{
    instruction::CompiledInstruction, pubkey::Pubkey, transaction::VersionedTransaction,
};
use solana_transaction_status::{
    EncodedTransactionWithStatusMeta, UiCompiledInstruction, UiInnerInstructions, UiInstruction,
};
use std::fmt::Debug;
use std::{collections::HashMap, str::FromStr};

use crate::streaming::event_parser::common::{
    parse_transfer_datas_from_next_instructions, TransferData,
};
use crate::streaming::event_parser::{
    common::{utils::*, EventMetadata, EventType, ProtocolType},
    protocols::{
        bonk::{BonkPoolCreateEvent, BonkTradeEvent},
        pumpfun::{PumpFunCreateTokenEvent, PumpFunTradeEvent},
    },
};

/// Unified Event Interface - All protocol events must implement this trait
pub trait UnifiedEvent: Debug + Send + Sync {
    /// Get event ID
    fn id(&self) -> &str;

    /// Get event type
    fn event_type(&self) -> EventType;

    /// Get transaction signature
    fn signature(&self) -> &str;

    /// Get slot number
    fn slot(&self) -> u64;

    /// Get program received timestamp (milliseconds)
    fn program_received_time_ms(&self) -> i64;

    /// Processing time consumption (milliseconds)
    fn program_handle_time_consuming_ms(&self) -> i64;

    /// Set processing time consumption (milliseconds)
    fn set_program_handle_time_consuming_ms(&mut self, program_handle_time_consuming_ms: i64);

    /// Convert event to Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;

    /// Convert event to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    /// Clone the event
    fn clone_boxed(&self) -> Box<dyn UnifiedEvent>;

    /// Merge events (optional implementation)
    fn merge(&mut self, _other: Box<dyn UnifiedEvent>) {
        // Default implementation: no merging operation
    }

    /// Set transfer datas
    fn set_transfer_datas(&mut self, transfer_datas: Vec<TransferData>);

    /// Get index
    fn index(&self) -> String;
}

/// 事件解析器trait - 定义了事件解析的核心方法
#[async_trait::async_trait]
pub trait EventParser: Send + Sync {
    /// 从内联指令中解析事件数据
    fn parse_events_from_inner_instruction(
        &self,
        instruction: &UiCompiledInstruction,
        signature: &str,
        slot: u64,
        block_time: Option<Timestamp>,
        program_received_time_ms: i64,
        index: String,
    ) -> Vec<Box<dyn UnifiedEvent>>;

    /// 从指令中解析事件数据
    fn parse_events_from_instruction(
        &self,
        instruction: &CompiledInstruction,
        accounts: &[Pubkey],
        signature: &str,
        slot: u64,
        block_time: Option<Timestamp>,
        program_received_time_ms: i64,
        index: String,
    ) -> Vec<Box<dyn UnifiedEvent>>;

    /// 从VersionedTransaction中解析指令事件的通用方法
    async fn parse_instruction_events_from_versioned_transaction(
        &self,
        versioned_tx: &VersionedTransaction,
        signature: &str,
        slot: Option<u64>,
        block_time: Option<Timestamp>,
        program_received_time_ms: i64,
        accounts: &[Pubkey],
        inner_instructions: &[UiInnerInstructions],
    ) -> Result<Vec<Box<dyn UnifiedEvent>>> {
        let mut instruction_events = Vec::new();
        // 获取交易的指令和账户
        let compiled_instructions = versioned_tx.message.instructions();
        let mut accounts: Vec<Pubkey> = accounts.to_vec();

        // 检查交易中是否包含程序
        let has_program = accounts.iter().any(|account| self.should_handle(account));
        if has_program {
            // 解析每个指令
            for (index, instruction) in compiled_instructions.iter().enumerate() {
                if let Some(program_id) = accounts.get(instruction.program_id_index as usize) {
                    if self.should_handle(program_id) {
                        let max_idx = instruction.accounts.iter().max().unwrap_or(&0);
                        // 补齐accounts(使用Pubkey::default())
                        if *max_idx as usize > accounts.len() {
                            for _i in accounts.len()..*max_idx as usize {
                                accounts.push(Pubkey::default());
                            }
                        }
                        if let Ok(mut events) = self
                            .parse_instruction(
                                instruction,
                                &accounts,
                                signature,
                                slot,
                                block_time,
                                program_received_time_ms,
                                format!("{}", index),
                            )
                            .await
                        {
                            if events.len() > 0 {
                                if let Some(inn) =
                                    inner_instructions.iter().find(|inner_instruction| {
                                        inner_instruction.index == index as u8
                                    })
                                {
                                    events.iter_mut().for_each(|event| {
                                        let transfer_datas =
                                            parse_transfer_datas_from_next_instructions(
                                                &inn,
                                                -1 as i8,
                                                &accounts,
                                                event.event_type(),
                                            );
                                        event.set_transfer_datas(transfer_datas.clone());
                                    });
                                }
                                instruction_events.extend(events);
                            }
                        }
                    }
                }
            }
        }
        Ok(instruction_events)
    }

    async fn parse_versioned_transaction(
        &self,
        versioned_tx: &VersionedTransaction,
        signature: &str,
        slot: Option<u64>,
        block_time: Option<Timestamp>,
        program_received_time_ms: i64,
        bot_wallet: Option<Pubkey>,
    ) -> Result<Vec<Box<dyn UnifiedEvent>>> {
        let accounts: Vec<Pubkey> = versioned_tx.message.static_account_keys().to_vec();
        let events = self
            .parse_instruction_events_from_versioned_transaction(
                versioned_tx,
                signature,
                slot,
                block_time,
                program_received_time_ms,
                &accounts,
                &vec![],
            )
            .await
            .unwrap_or_else(|_e| vec![]);
        Ok(self.process_events(events, bot_wallet))
    }

    async fn parse_transaction(
        &self,
        tx: EncodedTransactionWithStatusMeta,
        signature: &str,
        slot: Option<u64>,
        block_time: Option<Timestamp>,
        program_received_time_ms: i64,
        bot_wallet: Option<Pubkey>,
    ) -> Result<Vec<Box<dyn UnifiedEvent>>> {
        let transaction = tx.transaction;
        // 检查交易元数据
        let meta = tx
            .meta
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing transaction metadata"))?;

        let mut address_table_lookups: Vec<Pubkey> = vec![];
        let mut inner_instructions: Vec<UiInnerInstructions> = vec![];
        if meta.err.is_none() {
            inner_instructions = meta.inner_instructions.as_ref().unwrap().clone();
            let loaded_addresses = meta.loaded_addresses.as_ref().unwrap();
            for lookup in &loaded_addresses.writable {
                address_table_lookups.push(Pubkey::from_str(lookup).unwrap());
            }
            for lookup in &loaded_addresses.readonly {
                address_table_lookups.push(Pubkey::from_str(lookup).unwrap());
            }
        }
        let mut accounts: Vec<Pubkey> = vec![];

        let mut instruction_events = Vec::new();

        // 解析指令事件
        if let Some(versioned_tx) = transaction.decode() {
            accounts = versioned_tx.message.static_account_keys().to_vec();
            accounts.extend(address_table_lookups.clone());

            instruction_events = self
                .parse_instruction_events_from_versioned_transaction(
                    &versioned_tx,
                    signature,
                    slot,
                    block_time,
                    program_received_time_ms,
                    &accounts,
                    &inner_instructions,
                )
                .await
                .unwrap_or_else(|_e| vec![]);
        } else {
            accounts.extend(address_table_lookups.clone());
        }

        // Parse inner instruction events
        let mut inner_instruction_events = Vec::new();
        // Check if transaction was successful
        if meta.err.is_none() {
            for inner_instruction in &inner_instructions {
                for (index, instruction) in inner_instruction.instructions.iter().enumerate() {
                    match instruction {
                        UiInstruction::Compiled(compiled) => {
                            // 解析嵌套指令
                            let compiled_instruction = CompiledInstruction {
                                program_id_index: compiled.program_id_index,
                                accounts: compiled.accounts.clone(),
                                data: bs58::decode(compiled.data.clone()).into_vec().unwrap(),
                            };
                            if let Ok(mut events) = self
                                .parse_instruction(
                                    &compiled_instruction,
                                    &accounts,
                                    signature,
                                    slot,
                                    block_time,
                                    program_received_time_ms,
                                    format!("{}.{}", inner_instruction.index, index),
                                )
                                .await
                            {
                                if events.len() > 0 {
                                    events.iter_mut().for_each(|event| {
                                        let transfer_datas =
                                            parse_transfer_datas_from_next_instructions(
                                                &inner_instruction,
                                                index as i8,
                                                &accounts,
                                                event.event_type(),
                                            );
                                        event.set_transfer_datas(transfer_datas.clone());
                                    });
                                    instruction_events.extend(events);
                                }
                            }
                            if let Ok(mut events) = self
                                .parse_inner_instruction(
                                    compiled,
                                    signature,
                                    slot,
                                    block_time,
                                    program_received_time_ms,
                                    format!("{}.{}", inner_instruction.index, index),
                                )
                                .await
                            {
                                if events.len() > 0 {
                                    events.iter_mut().for_each(|event| {
                                        let transfer_datas =
                                            parse_transfer_datas_from_next_instructions(
                                                &inner_instruction,
                                                index as i8,
                                                &accounts,
                                                event.event_type(),
                                            );
                                        event.set_transfer_datas(transfer_datas.clone());
                                    });
                                    inner_instruction_events.extend(events);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Parse events from transaction logs
        let mut log_events = Vec::new();
        if let solana_transaction_status::option_serializer::OptionSerializer::Some(log_messages) = &meta.log_messages {
            log_events = self
                .parse_events_from_logs(
                    log_messages,
                    signature,
                    slot,
                    block_time,
                    &inner_instructions,
                )
                .await
                .unwrap_or_else(|_e| vec![]);
        }

        // Merge log events with inner instruction events
        inner_instruction_events.extend(log_events);

        if instruction_events.len() > 0 && inner_instruction_events.len() > 0 {
            for instruction_event in &mut instruction_events {
                for inner_instruction_event in &inner_instruction_events {
                    if instruction_event.id() == inner_instruction_event.id() {
                        let i_index = instruction_event.index();
                        let in_index = inner_instruction_event.index();
                        
                        // Handle log events specially - they should merge with matching ID
                        if in_index == "log" {
                            instruction_event.merge(inner_instruction_event.clone_boxed());
                            continue; // Don't break, might have multiple matches
                        }
                        
                        if !i_index.contains(".") && in_index.contains(".") {
                            let in_index_parent_index = in_index.split(".").nth(0).unwrap();
                            if in_index_parent_index == i_index {
                                instruction_event.merge(inner_instruction_event.clone_boxed());
                                break;
                            }
                        } else if i_index.contains(".") && in_index.contains(".") {
                            // 嵌套指令
                            let i_index_parent_index = i_index.split(".").nth(0).unwrap();
                            let in_index_parent_index = in_index.split(".").nth(0).unwrap();
                            if i_index_parent_index == in_index_parent_index {
                                let i_index_child_index = i_index
                                    .split(".")
                                    .nth(1)
                                    .unwrap()
                                    .parse::<u32>()
                                    .unwrap_or(0);
                                let in_index_child_index = in_index
                                    .split(".")
                                    .nth(1)
                                    .unwrap()
                                    .parse::<u32>()
                                    .unwrap_or(0);
                                if in_index_child_index > i_index_child_index {
                                    instruction_event.merge(inner_instruction_event.clone_boxed());
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(self.process_events(instruction_events, bot_wallet))
    }

    fn process_events(
        &self,
        mut events: Vec<Box<dyn UnifiedEvent>>,
        bot_wallet: Option<Pubkey>,
    ) -> Vec<Box<dyn UnifiedEvent>> {
        let mut dev_address = vec![];
        let mut bonk_dev_address = None;
        for event in &mut events {
            if let Some(token_info) = event.as_any().downcast_ref::<PumpFunCreateTokenEvent>() {
                dev_address.push(token_info.user);
                if token_info.creator != Pubkey::default() && token_info.creator != token_info.user
                {
                    dev_address.push(token_info.creator);
                }
            } else if let Some(trade_info) = event.as_any_mut().downcast_mut::<PumpFunTradeEvent>()
            {
                if dev_address.contains(&trade_info.user)
                    || dev_address.contains(&trade_info.creator)
                {
                    trade_info.is_dev_create_token_trade = true;
                } else if Some(trade_info.user) == bot_wallet {
                    trade_info.is_bot = true;
                } else {
                    trade_info.is_dev_create_token_trade = false;
                }
            }
            if let Some(pool_info) = event.as_any().downcast_ref::<BonkPoolCreateEvent>() {
                bonk_dev_address = Some(pool_info.creator);
            } else if let Some(trade_info) = event.as_any_mut().downcast_mut::<BonkTradeEvent>() {
                if Some(trade_info.payer) == bonk_dev_address {
                    trade_info.is_dev_create_token_trade = true;
                } else if Some(trade_info.payer) == bot_wallet {
                    trade_info.is_bot = true;
                } else {
                    trade_info.is_dev_create_token_trade = false;
                }
            }
            let now = chrono::Utc::now().timestamp_millis();
            event.set_program_handle_time_consuming_ms(now - event.program_received_time_ms());
        }
        events
    }

    async fn parse_inner_instruction(
        &self,
        instruction: &UiCompiledInstruction,
        signature: &str,
        slot: Option<u64>,
        block_time: Option<Timestamp>,
        program_received_time_ms: i64,
        index: String,
    ) -> Result<Vec<Box<dyn UnifiedEvent>>> {
        let slot = slot.unwrap_or(0);
        let events = self.parse_events_from_inner_instruction(
            instruction,
            signature,
            slot,
            block_time,
            program_received_time_ms,
            index,
        );
        Ok(events)
    }

    async fn parse_instruction(
        &self,
        instruction: &CompiledInstruction,
        accounts: &[Pubkey],
        signature: &str,
        slot: Option<u64>,
        block_time: Option<Timestamp>,
        program_received_time_ms: i64,
        index: String,
    ) -> Result<Vec<Box<dyn UnifiedEvent>>> {
        let slot = slot.unwrap_or(0);
        let events = self.parse_events_from_instruction(
            instruction,
            accounts,
            signature,
            slot,
            block_time,
            program_received_time_ms,
            index,
        );
        Ok(events)
    }

    /// Parse event data from log messages
    async fn parse_events_from_logs(
        &self,
        logs: &[String],
        signature: &str,
        slot: Option<u64>,
        block_time: Option<Timestamp>,
        _inner_instructions: &[UiInnerInstructions],
    ) -> Result<Vec<Box<dyn UnifiedEvent>>> {
        use crate::streaming::event_parser::common::utils::{decode_base64, extract_program_data};
        
        let mut events = Vec::new();
        
        for log in logs {
            if let Some(data_str) = extract_program_data(log) {
                if let Ok(decoded) = decode_base64(data_str) {
                    if decoded.len() >= 16 {
                        let hex_str = format!("0x{}", hex::encode(&decoded));
                        
                        let discriminators = self.get_inner_instruction_configs();
                        
                        // Check both full 16-byte and 8-byte discriminators for log events
                        for (discriminator, configs) in discriminators {
                            // Try full discriminator match first
                            if hex_str.starts_with(discriminator) {
                                let data = &decoded[16..]; // Skip full 16-byte discriminator
                                
                                for config in configs {
                                    if let Some(event) = (config.inner_instruction_parser)(
                                        data,
                                        EventMetadata::new(
                                            signature.to_string(),
                                            signature.to_string(),
                                            slot.unwrap_or(0),
                                            block_time.map(|bt| bt.seconds).unwrap_or(0),
                                            block_time.map(|bt| bt.seconds * 1000 + (bt.nanos as i64) / 1_000_000).unwrap_or(0),
                                            self.get_protocol_type(),
                                            config.event_type.clone(),
                                            self.get_program_id(),
                                            "log".to_string(),
                                            0, // program_received_time_ms
                                        ),
                                    ) {
                                        events.push(event);
                                    }
                                }
                            } else {
                                // Try 8-byte discriminator (second half) for log events
                                let discriminator_without_prefix = discriminator.strip_prefix("0x").unwrap_or(discriminator);
                                if discriminator_without_prefix.len() >= 16 {
                                    let second_half = &discriminator_without_prefix[16..]; // Take last 8 bytes
                                    let second_half_with_prefix = format!("0x{}", second_half);
                                    
                                    if hex_str.starts_with(&second_half_with_prefix) {
                                        let data = &decoded[8..]; // Skip 8-byte discriminator
                                        
                                        for config in configs {
                                            if let Some(event) = (config.inner_instruction_parser)(
                                                data,
                                                EventMetadata::new(
                                                    signature.to_string(),
                                                    signature.to_string(),
                                                    slot.unwrap_or(0),
                                                    block_time.map(|bt| bt.seconds).unwrap_or(0),
                                                    block_time.map(|bt| bt.seconds * 1000 + (bt.nanos as i64) / 1_000_000).unwrap_or(0),
                                                    self.get_protocol_type(),
                                                    config.event_type.clone(),
                                                    self.get_program_id(),
                                                    "log".to_string(),
                                                    0, // program_received_time_ms
                                                ),
                                            ) {
                                                events.push(event);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(events)
    }

    /// Get inner instruction configurations
    fn get_inner_instruction_configs(&self) -> &std::collections::HashMap<&'static str, Vec<GenericEventParseConfig>> {
        // Default implementation returns empty map - parsers should override this
        use std::sync::LazyLock;
        static EMPTY_MAP: LazyLock<std::collections::HashMap<&'static str, Vec<GenericEventParseConfig>>> = LazyLock::new(|| std::collections::HashMap::new());
        &EMPTY_MAP
    }
    
    /// 获取协议类型（需要实现）
    fn get_protocol_type(&self) -> ProtocolType {
        // Default implementation - parsers should override this
        ProtocolType::PumpFun
    }
    
    /// 获取程序ID（需要实现）
    fn get_program_id(&self) -> Pubkey {
        // Default implementation - parsers should override this
        Pubkey::default()
    }

    /// 检查是否应该处理此程序ID
    fn should_handle(&self, program_id: &Pubkey) -> bool;

    /// 获取支持的程序ID列表
    fn supported_program_ids(&self) -> Vec<Pubkey>;
}

// 为Box<dyn UnifiedEvent>实现Clone
impl Clone for Box<dyn UnifiedEvent> {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}

/// 通用事件解析器配置
#[derive(Debug, Clone)]
pub struct GenericEventParseConfig {
    pub inner_instruction_discriminator: &'static str,
    pub instruction_discriminator: &'static [u8],
    pub event_type: EventType,
    pub inner_instruction_parser: InnerInstructionEventParser,
    pub instruction_parser: InstructionEventParser,
}

/// 内联指令事件解析器
pub type InnerInstructionEventParser =
    fn(data: &[u8], metadata: EventMetadata) -> Option<Box<dyn UnifiedEvent>>;

/// 指令事件解析器
pub type InstructionEventParser =
    fn(data: &[u8], accounts: &[Pubkey], metadata: EventMetadata) -> Option<Box<dyn UnifiedEvent>>;

/// 通用事件解析器基类
pub struct GenericEventParser {
    program_id: Pubkey,
    protocol_type: ProtocolType,
    inner_instruction_configs: HashMap<&'static str, Vec<GenericEventParseConfig>>,
    instruction_configs: HashMap<Vec<u8>, Vec<GenericEventParseConfig>>,
}

impl GenericEventParser {
    /// 创建新的通用事件解析器
    pub fn new(
        program_id: Pubkey,
        protocol_type: ProtocolType,
        configs: Vec<GenericEventParseConfig>,
    ) -> Self {
        let mut inner_instruction_configs = HashMap::new();
        let mut instruction_configs = HashMap::new();

        for config in configs {
            inner_instruction_configs
                .entry(config.inner_instruction_discriminator)
                .or_insert(vec![])
                .push(config.clone());
            instruction_configs
                .entry(config.instruction_discriminator.to_vec())
                .or_insert(vec![])
                .push(config);
        }

        Self {
            program_id,
            protocol_type,
            inner_instruction_configs,
            instruction_configs,
        }
    }

    /// 通用的内联指令解析方法
    fn parse_inner_instruction_event(
        &self,
        config: &GenericEventParseConfig,
        data: &[u8],
        signature: &str,
        slot: u64,
        block_time: Option<Timestamp>,
        program_received_time_ms: i64,
        index: String,
    ) -> Option<Box<dyn UnifiedEvent>> {
        let timestamp = block_time.unwrap_or(Timestamp {
            seconds: 0,
            nanos: 0,
        });
        let block_time_ms = timestamp.seconds * 1000 + (timestamp.nanos as i64) / 1_000_000;
        let metadata = EventMetadata::new(
            signature.to_string(),
            signature.to_string(),
            slot,
            timestamp.seconds,
            block_time_ms,
            self.protocol_type.clone(),
            config.event_type.clone(),
            self.program_id,
            index,
            program_received_time_ms,
        );
        (config.inner_instruction_parser)(data, metadata)
    }

    /// 通用的指令解析方法
    fn parse_instruction_event(
        &self,
        config: &GenericEventParseConfig,
        data: &[u8],
        account_pubkeys: &[Pubkey],
        signature: &str,
        slot: u64,
        block_time: Option<Timestamp>,
        program_received_time_ms: i64,
        index: String,
    ) -> Option<Box<dyn UnifiedEvent>> {
        let timestamp = block_time.unwrap_or(Timestamp {
            seconds: 0,
            nanos: 0,
        });
        let block_time_ms = timestamp.seconds * 1000 + (timestamp.nanos as i64) / 1_000_000;
        let metadata = EventMetadata::new(
            signature.to_string(),
            signature.to_string(),
            slot,
            timestamp.seconds,
            block_time_ms,
            self.protocol_type.clone(),
            config.event_type.clone(),
            self.program_id,
            index,
            program_received_time_ms,
        );
        (config.instruction_parser)(data, account_pubkeys, metadata)
    }
}

#[async_trait::async_trait]
impl EventParser for GenericEventParser {
    /// 从内联指令中解析事件数据
    fn parse_events_from_inner_instruction(
        &self,
        inner_instruction: &UiCompiledInstruction,
        signature: &str,
        slot: u64,
        block_time: Option<Timestamp>,
        program_received_time_ms: i64,
        index: String,
    ) -> Vec<Box<dyn UnifiedEvent>> {
        let inner_instruction_data = inner_instruction.data.clone();
        let inner_instruction_data_decoded =
            bs58::decode(inner_instruction_data).into_vec().unwrap();
        if inner_instruction_data_decoded.len() < 16 {
            return Vec::new();
        }
        let inner_instruction_data_decoded_str =
            format!("0x{}", hex::encode(&inner_instruction_data_decoded));
        let data = &inner_instruction_data_decoded[16..];
        let mut events = Vec::new();
        for (disc, configs) in &self.inner_instruction_configs {
            if discriminator_matches(&inner_instruction_data_decoded_str, disc) {
                for config in configs {
                    if let Some(event) = self.parse_inner_instruction_event(
                        config,
                        data,
                        signature,
                        slot,
                        block_time,
                        program_received_time_ms,
                        index.clone(),
                    ) {
                        events.push(event);
                    }
                }
            }
        }
        events
    }

    /// 从指令中解析事件
    fn parse_events_from_instruction(
        &self,
        instruction: &CompiledInstruction,
        accounts: &[Pubkey],
        signature: &str,
        slot: u64,
        block_time: Option<Timestamp>,
        program_received_time_ms: i64,
        index: String,
    ) -> Vec<Box<dyn UnifiedEvent>> {
        // Check bounds for program_id_index
        if instruction.program_id_index as usize >= accounts.len() {
            return Vec::new();
        }
        let program_id = accounts[instruction.program_id_index as usize];
        if !self.should_handle(&program_id) {
            return Vec::new();
        }
        let mut events = Vec::new();
        for (disc, configs) in &self.instruction_configs {
            if instruction.data.len() < disc.len() {
                continue;
            }
            let discriminator = &instruction.data[..disc.len()];
            let data = &instruction.data[disc.len()..];
            if discriminator == disc {
                // 验证账户索引
                if !validate_account_indices(&instruction.accounts, accounts.len()) {
                    continue;
                }

                let account_pubkeys: Vec<Pubkey> = instruction
                    .accounts
                    .iter()
                    .map(|&idx| accounts[idx as usize])
                    .collect();
                for config in configs {
                    if let Some(event) = self.parse_instruction_event(
                        config,
                        data,
                        &account_pubkeys,
                        signature,
                        slot,
                        block_time,
                        program_received_time_ms,
                        index.clone(),
                    ) {
                        events.push(event);
                    }
                }
            }
        }

        events
    }

    fn get_inner_instruction_configs(&self) -> &std::collections::HashMap<&'static str, Vec<GenericEventParseConfig>> {
        &self.inner_instruction_configs
    }
    
    fn get_protocol_type(&self) -> ProtocolType {
        self.protocol_type.clone()
    }
    
    fn get_program_id(&self) -> Pubkey {
        self.program_id
    }

    fn should_handle(&self, program_id: &Pubkey) -> bool {
        *program_id == self.program_id
    }

    fn supported_program_ids(&self) -> Vec<Pubkey> {
        vec![self.program_id]
    }
}

pub struct SDKSystemEventParser {}
impl SDKSystemEventParser {}
