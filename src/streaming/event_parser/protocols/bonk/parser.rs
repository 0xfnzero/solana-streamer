use prost_types::Timestamp;
use solana_sdk::{instruction::CompiledInstruction, pubkey::Pubkey};
use solana_transaction_status::UiCompiledInstruction;

use crate::streaming::event_parser::{
    common::{utils::*, EventMetadata, EventType, ProtocolType},
    core::traits::{EventParser, GenericEventParseConfig, GenericEventParser, UnifiedEvent},
    protocols::bonk::{
        discriminators, BonkPoolCreateEvent, BonkTradeEvent, ConstantCurve, CurveParams,
        FixedCurve, LinearCurve, MintParams, TradeDirection, VestingParams,
    },
};

/// Bonk程序ID
pub const BONK_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj");

/// Bonk事件解析器
pub struct BonkEventParser {
    inner: GenericEventParser,
}

impl BonkEventParser {
    pub fn new() -> Self {
        // 配置所有事件类型
        let configs = vec![
            GenericEventParseConfig {
                inner_instruction_discriminator: discriminators::TRADE_EVENT,
                instruction_discriminator: discriminators::BUY_EXACT_IN,
                event_type: EventType::BonkBuyExactIn,
                inner_instruction_parser: Self::parse_trade_inner_instruction,
                instruction_parser: Self::parse_buy_exact_in_instruction,
            },
            GenericEventParseConfig {
                inner_instruction_discriminator: discriminators::TRADE_EVENT,
                instruction_discriminator: discriminators::BUY_EXACT_OUT,
                event_type: EventType::BonkBuyExactOut,
                inner_instruction_parser: Self::parse_trade_inner_instruction,
                instruction_parser: Self::parse_buy_exact_out_instruction,
            },
            GenericEventParseConfig {
                inner_instruction_discriminator: discriminators::TRADE_EVENT,
                instruction_discriminator: discriminators::SELL_EXACT_IN,
                event_type: EventType::BonkSellExactIn,
                inner_instruction_parser: Self::parse_trade_inner_instruction,
                instruction_parser: Self::parse_sell_exact_in_instruction,
            },
            GenericEventParseConfig {
                inner_instruction_discriminator: discriminators::TRADE_EVENT,
                instruction_discriminator: discriminators::SELL_EXACT_OUT,
                event_type: EventType::BonkSellExactOut,
                inner_instruction_parser: Self::parse_trade_inner_instruction,
                instruction_parser: Self::parse_sell_exact_out_instruction,
            },
            GenericEventParseConfig {
                inner_instruction_discriminator: discriminators::POOL_CREATE_EVENT,
                instruction_discriminator: discriminators::INITIALIZE,
                event_type: EventType::BonkInitialize,
                inner_instruction_parser: Self::parse_pool_create_inner_instruction,
                instruction_parser: Self::parse_initialize_instruction,
            },
        ];

        let inner = GenericEventParser::new(BONK_PROGRAM_ID, ProtocolType::Bonk, configs);

        Self { inner }
    }

    /// 解析创建池事件
    fn parse_pool_create_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if let Ok(event) = borsh::from_slice::<BonkPoolCreateEvent>(data) {
            let mut metadata = metadata;
            metadata.set_id(format!("{}", metadata.signature,));
            Some(Box::new(BonkPoolCreateEvent {
                metadata: metadata,
                ..event
            }))
        } else {
            None
        }
    }

    /// 解析交易事件
    fn parse_trade_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if let Ok(event) = borsh::from_slice::<BonkTradeEvent>(data) {
            let mut metadata = metadata;
            metadata.set_id(format!(
                "{}-{}",
                metadata.signature,
                event.pool_state.to_string()
            ));
            if metadata.event_type == EventType::BonkBuyExactIn
                || metadata.event_type == EventType::BonkBuyExactOut
            {
                if event.trade_direction != TradeDirection::Buy {
                    return None;
                }
            } else if metadata.event_type == EventType::BonkSellExactIn
                || metadata.event_type == EventType::BonkSellExactOut
            {
                if event.trade_direction != TradeDirection::Sell {
                    return None;
                }
            }
            Some(Box::new(BonkTradeEvent {
                metadata: metadata,
                ..event
            }))
        } else {
            None
        }
    }

    /// 解析买入指令事件
    fn parse_buy_exact_in_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 16 || accounts.len() < 11 {
            return None;
        }

        let amount_in = read_u64_le(data, 0)?;
        let minimum_amount_out = read_u64_le(data, 8)?;
        let share_fee_rate = read_u64_le(data, 16)?;

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}", metadata.signature, accounts[4]));

        Some(Box::new(BonkTradeEvent {
            metadata,
            amount_in,
            minimum_amount_out,
            share_fee_rate,
            payer: accounts[0],
            pool_state: accounts[4],
            user_base_token: accounts[5],
            user_quote_token: accounts[6],
            base_vault: accounts[7],
            quote_vault: accounts[8],
            base_token_mint: accounts[9],
            quote_token_mint: accounts[10],
            trade_direction: TradeDirection::Buy,
            ..Default::default()
        }))
    }

    fn parse_buy_exact_out_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 16 || accounts.len() < 11 {
            return None;
        }

        let amount_out = read_u64_le(data, 0)?;
        let maximum_amount_in = read_u64_le(data, 8)?;
        let share_fee_rate = read_u64_le(data, 16)?;

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}", metadata.signature, accounts[4]));

        Some(Box::new(BonkTradeEvent {
            metadata,
            amount_out,
            maximum_amount_in,
            share_fee_rate,
            payer: accounts[0],
            pool_state: accounts[4],
            user_base_token: accounts[5],
            user_quote_token: accounts[6],
            base_vault: accounts[7],
            quote_vault: accounts[8],
            base_token_mint: accounts[9],
            quote_token_mint: accounts[10],
            trade_direction: TradeDirection::Buy,
            ..Default::default()
        }))
    }

    fn parse_sell_exact_in_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 16 || accounts.len() < 11 {
            return None;
        }

        let amount_in = read_u64_le(data, 0)?;
        let minimum_amount_out = read_u64_le(data, 8)?;
        let share_fee_rate = read_u64_le(data, 16)?;

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}", metadata.signature, accounts[4]));

        Some(Box::new(BonkTradeEvent {
            metadata,
            amount_in,
            minimum_amount_out,
            share_fee_rate,
            payer: accounts[0],
            pool_state: accounts[4],
            user_base_token: accounts[5],
            user_quote_token: accounts[6],
            base_vault: accounts[7],
            quote_vault: accounts[8],
            base_token_mint: accounts[9],
            quote_token_mint: accounts[10],
            trade_direction: TradeDirection::Sell,
            ..Default::default()
        }))
    }

    fn parse_sell_exact_out_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 16 || accounts.len() < 11 {
            return None;
        }

        let amount_out = read_u64_le(data, 0)?;
        let maximum_amount_in = read_u64_le(data, 8)?;
        let share_fee_rate = read_u64_le(data, 16)?;

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}", metadata.signature, accounts[4]));

        Some(Box::new(BonkTradeEvent {
            metadata,
            amount_out,
            maximum_amount_in,
            share_fee_rate,
            payer: accounts[0],
            pool_state: accounts[4],
            user_base_token: accounts[5],
            user_quote_token: accounts[6],
            base_vault: accounts[7],
            quote_vault: accounts[8],
            base_token_mint: accounts[9],
            quote_token_mint: accounts[10],
            trade_direction: TradeDirection::Sell,
            ..Default::default()
        }))
    }

    /// 解析初始化事件
    fn parse_initialize_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 24 {
            return None;
        }

        let mut offset = 0;
        let base_mint_param = Self::parse_mint_params(data, &mut offset)?;
        let curve_param = Self::parse_curve_params(data, &mut offset)?;
        let vesting_param = Self::parse_vesting_params(data, &mut offset)?;

        let mut metadata = metadata;
        metadata.set_id(format!("{}", metadata.signature));

        Some(Box::new(BonkPoolCreateEvent {
            metadata,
            payer: accounts[0],
            creator: accounts[1],
            global_config: accounts[2],
            platform_config: accounts[3],
            pool_state: accounts[5],
            base_mint: accounts[6],
            quote_mint: accounts[7],
            base_vault: accounts[8],
            quote_vault: accounts[9],
            base_mint_param,
            curve_param,
            vesting_param,
            ..Default::default()
        }))
    }

    /// 解析 MintParams 结构
    fn parse_mint_params(data: &[u8], offset: &mut usize) -> Option<MintParams> {
        // 读取decimals (1字节)
        let decimals = read_u8(data, *offset)?;
        *offset += 1;

        // 读取name字符串长度和内容
        let name_len = read_u32_le(data, *offset)? as usize;
        *offset += 4;
        if data.len() < *offset + name_len {
            return None;
        }
        let name = String::from_utf8(data[*offset..*offset + name_len].to_vec()).ok()?;
        *offset += name_len;

        // 读取symbol字符串长度和内容
        let symbol_len = read_u32_le(data, *offset)? as usize;
        *offset += 4;
        if data.len() < *offset + symbol_len {
            return None;
        }
        let symbol = String::from_utf8(data[*offset..*offset + symbol_len].to_vec()).ok()?;
        *offset += symbol_len;

        // 读取uri字符串长度和内容
        let uri_len = read_u32_le(data, *offset)? as usize;
        *offset += 4;
        if data.len() < *offset + uri_len {
            return None;
        }
        let uri = String::from_utf8(data[*offset..*offset + uri_len].to_vec()).ok()?;
        *offset += uri_len;

        Some(MintParams {
            decimals,
            name,
            symbol,
            uri,
        })
    }

    /// 解析 CurveParams 结构
    fn parse_curve_params(data: &[u8], offset: &mut usize) -> Option<CurveParams> {
        // 读取curve类型标识符 (1字节)
        let curve_type = read_u8(data, *offset)?;
        *offset += 1;

        match curve_type {
            0 => {
                // Constant curve
                let supply = read_u64_le(data, *offset)?;
                *offset += 8;
                let total_base_sell = read_u64_le(data, *offset)?;
                *offset += 8;
                let total_quote_fund_raising = read_u64_le(data, *offset)?;
                *offset += 8;
                let migrate_type = read_u8(data, *offset)?;
                *offset += 1;

                Some(CurveParams::Constant {
                    data: ConstantCurve {
                        supply,
                        total_base_sell,
                        total_quote_fund_raising,
                        migrate_type,
                    },
                })
            }
            1 => {
                // Fixed curve
                let supply = read_u64_le(data, *offset)?;
                *offset += 8;
                let total_quote_fund_raising = read_u64_le(data, *offset)?;
                *offset += 8;
                let migrate_type = read_u8(data, *offset)?;
                *offset += 1;

                Some(CurveParams::Fixed {
                    data: FixedCurve {
                        supply,
                        total_quote_fund_raising,
                        migrate_type,
                    },
                })
            }
            2 => {
                // Linear curve
                let supply = read_u64_le(data, *offset)?;
                *offset += 8;
                let total_quote_fund_raising = read_u64_le(data, *offset)?;
                *offset += 8;
                let migrate_type = read_u8(data, *offset)?;
                *offset += 1;

                Some(CurveParams::Linear {
                    data: LinearCurve {
                        supply,
                        total_quote_fund_raising,
                        migrate_type,
                    },
                })
            }
            _ => None,
        }
    }

    /// 解析 VestingParams 结构
    fn parse_vesting_params(data: &[u8], offset: &mut usize) -> Option<VestingParams> {
        let total_locked_amount = read_u64_le(data, *offset)?;
        *offset += 8;
        let cliff_period = read_u64_le(data, *offset)?;
        *offset += 8;
        let unlock_period = read_u64_le(data, *offset)?;
        *offset += 8;

        Some(VestingParams {
            total_locked_amount,
            cliff_period,
            unlock_period,
        })
    }
}

#[async_trait::async_trait]
impl EventParser for BonkEventParser {
    fn parse_events_from_inner_instruction(
        &self,
        inner_instruction: &UiCompiledInstruction,
        signature: &str,
        slot: u64,
        block_time: Option<Timestamp>,
        program_received_time_ms: i64,
        index: String,
    ) -> Vec<Box<dyn UnifiedEvent>> {
        self.inner.parse_events_from_inner_instruction(
            inner_instruction,
            signature,
            slot,
            block_time,
            program_received_time_ms,
            index,
        )
    }

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
        self.inner.parse_events_from_instruction(
            instruction,
            accounts,
            signature,
            slot,
            block_time,
            program_received_time_ms,
            index,
        )
    }

    fn should_handle(&self, program_id: &Pubkey) -> bool {
        self.inner.should_handle(program_id)
    }

    fn supported_program_ids(&self) -> Vec<Pubkey> {
        self.inner.supported_program_ids()
    }

    fn get_inner_instruction_configs(&self) -> &std::collections::HashMap<&'static str, Vec<crate::streaming::event_parser::core::traits::GenericEventParseConfig>> {
        self.inner.get_inner_instruction_configs()
    }
    
    fn get_protocol_type(&self) -> ProtocolType {
        ProtocolType::Bonk
    }
    
    fn get_program_id(&self) -> Pubkey {
        BONK_PROGRAM_ID
    }
}
