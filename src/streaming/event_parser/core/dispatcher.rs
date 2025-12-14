//! 中心事件解析调度器
//!
//! 根据协议类型路由到对应的解析函数，替代原有的静态 CONFIGS 数组架构
//!
//! ## 设计原则
//! - **单一职责**: 每个函数只负责一件事（路由、解析、合并分离）
//! - **灵活性**: 调用方可以选择是否合并，或自定义合并逻辑
//! - **可测试性**: 每个函数都可以独立测试

use crate::streaming::event_parser::{
    common::EventMetadata,
    core::common_event_parser::{CommonEventParser, COMPUTE_BUDGET_PROGRAM_ID},
    protocols::{
        bonk::parser as bonk, meteora_damm_v2::parser as meteora_damm_v2,
        meteora_dlmm::parser as meteora_dlmm, pumpfun::parser as pumpfun,
        pumpswap::parser as pumpswap, raydium_amm_v4::parser as raydium_amm_v4,
        raydium_clmm::parser as raydium_clmm, raydium_cpmm::parser as raydium_cpmm,
        whirlpool::parser as whirlpool,
    },
    DexEvent, Protocol,
};
use solana_sdk::pubkey::Pubkey;

/// 中心事件解析调度器
///
/// 负责将解析请求路由到对应协议的解析函数
pub struct EventDispatcher;

impl EventDispatcher {
    /// 解析 instruction 事件（只解析，不合并）
    ///
    /// # 参数
    /// - `protocol`: 协议类型
    /// - `instruction_discriminator`: 指令判别器 (8 bytes)
    /// - `instruction_data`: 指令数据
    /// - `accounts`: 账户公钥列表
    /// - `metadata`: 事件元数据
    ///
    /// # 返回
    /// 解析成功返回 `Some(DexEvent)`，否则返回 `None`
    #[inline]
    pub fn dispatch_instruction(
        protocol: Protocol,
        instruction_discriminator: &[u8],
        instruction_data: &[u8],
        accounts: &[Pubkey],
        mut metadata: EventMetadata,
    ) -> Option<DexEvent> {
        // 根据协议类型设置 metadata.protocol
        use crate::streaming::event_parser::common::ProtocolType;
        metadata.protocol = match protocol {
            Protocol::PumpFun => ProtocolType::PumpFun,
            Protocol::PumpSwap => ProtocolType::PumpSwap,
            Protocol::Bonk => ProtocolType::Bonk,
            Protocol::RaydiumCpmm => ProtocolType::RaydiumCpmm,
            Protocol::RaydiumClmm => ProtocolType::RaydiumClmm,
            Protocol::RaydiumAmmV4 => ProtocolType::RaydiumAmmV4,
            Protocol::MeteoraDammV2 => ProtocolType::MeteoraDammV2,
            Protocol::MeteoraDlmm => ProtocolType::MeteoraDlmm,
            Protocol::Whirlpool => ProtocolType::Whirlpool,
        };

        match protocol {
            Protocol::PumpFun => pumpfun::parse_pumpfun_instruction_data(
                instruction_discriminator,
                instruction_data,
                accounts,
                metadata,
            ),
            Protocol::PumpSwap => pumpswap::parse_pumpswap_instruction_data(
                instruction_discriminator,
                instruction_data,
                accounts,
                metadata,
            ),
            Protocol::Bonk => bonk::parse_bonk_instruction_data(
                instruction_discriminator,
                instruction_data,
                accounts,
                metadata,
            ),
            Protocol::RaydiumCpmm => raydium_cpmm::parse_raydium_cpmm_instruction_data(
                instruction_discriminator,
                instruction_data,
                accounts,
                metadata,
            ),
            Protocol::RaydiumClmm => raydium_clmm::parse_raydium_clmm_instruction_data(
                instruction_discriminator,
                instruction_data,
                accounts,
                metadata,
            ),
            Protocol::RaydiumAmmV4 => raydium_amm_v4::parse_raydium_amm_v4_instruction_data(
                instruction_discriminator,
                instruction_data,
                accounts,
                metadata,
            ),
            Protocol::MeteoraDammV2 => meteora_damm_v2::parse_meteora_damm_v2_instruction_data(
                instruction_discriminator,
                instruction_data,
                accounts,
                metadata,
            ),
            Protocol::MeteoraDlmm => {
                // Meteora DLMM 目前不需要解析指令数据，返回 None
                None
            }
            Protocol::Whirlpool => {
                // Whirlpool 目前不需要解析指令数据，返回 None
                None
            }
        }
    }

    /// 解析 inner instruction 事件（只解析，不合并）
    ///
    /// # 参数
    /// - `protocol`: 协议类型
    /// - `inner_instruction_discriminator`: 内联指令判别器 (16 bytes)
    /// - `inner_instruction_data`: 内联指令数据
    /// - `metadata`: 事件元数据
    ///
    /// # 返回
    /// 解析成功返回 `Some(DexEvent)`，否则返回 `None`
    #[inline]
    pub fn dispatch_inner_instruction(
        protocol: Protocol,
        inner_instruction_discriminator: &[u8],
        inner_instruction_data: &[u8],
        mut metadata: EventMetadata,
    ) -> Option<DexEvent> {
        // 根据协议类型设置 metadata.protocol
        use crate::streaming::event_parser::common::ProtocolType;
        metadata.protocol = match protocol {
            Protocol::PumpFun => ProtocolType::PumpFun,
            Protocol::PumpSwap => ProtocolType::PumpSwap,
            Protocol::Bonk => ProtocolType::Bonk,
            Protocol::RaydiumCpmm => ProtocolType::RaydiumCpmm,
            Protocol::RaydiumClmm => ProtocolType::RaydiumClmm,
            Protocol::RaydiumAmmV4 => ProtocolType::RaydiumAmmV4,
            Protocol::MeteoraDammV2 => ProtocolType::MeteoraDammV2,
            Protocol::MeteoraDlmm => ProtocolType::MeteoraDlmm,
            Protocol::Whirlpool => ProtocolType::Whirlpool,
        };

        match protocol {
            Protocol::PumpFun => pumpfun::parse_pumpfun_inner_instruction_data(
                inner_instruction_discriminator,
                inner_instruction_data,
                metadata,
            ),
            Protocol::PumpSwap => pumpswap::parse_pumpswap_inner_instruction_data(
                inner_instruction_discriminator,
                inner_instruction_data,
                metadata,
            ),
            Protocol::Bonk => bonk::parse_bonk_inner_instruction_data(
                inner_instruction_discriminator,
                inner_instruction_data,
                metadata,
            ),
            Protocol::RaydiumCpmm => raydium_cpmm::parse_raydium_cpmm_inner_instruction_data(
                inner_instruction_discriminator,
                inner_instruction_data,
                metadata,
            ),
            Protocol::RaydiumClmm => raydium_clmm::parse_raydium_clmm_inner_instruction_data(
                inner_instruction_discriminator,
                inner_instruction_data,
                metadata,
            ),
            Protocol::RaydiumAmmV4 => raydium_amm_v4::parse_raydium_amm_v4_inner_instruction_data(
                inner_instruction_discriminator,
                inner_instruction_data,
                metadata,
            ),
            Protocol::MeteoraDammV2 => meteora_damm_v2::parse_meteora_damm_v2_inner_instruction_data(
                inner_instruction_discriminator,
                inner_instruction_data,
                metadata,
            ),
            Protocol::MeteoraDlmm => {
                // Meteora DLMM 目前不需要解析 inner instruction 数据，返回 None
                None
            }
            Protocol::Whirlpool => {
                // Whirlpool 目前不需要解析 inner instruction 数据，返回 None
                None
            }
        }
    }

    /// 通过 program_id 匹配协议类型
    #[inline]
    pub fn match_protocol_by_program_id(program_id: &Pubkey) -> Option<Protocol> {
        if program_id == &pumpfun::PUMPFUN_PROGRAM_ID {
            Some(Protocol::PumpFun)
        } else if program_id == &pumpswap::PUMPSWAP_PROGRAM_ID {
            Some(Protocol::PumpSwap)
        } else if program_id == &bonk::BONK_PROGRAM_ID {
            Some(Protocol::Bonk)
        } else if program_id == &raydium_cpmm::RAYDIUM_CPMM_PROGRAM_ID {
            Some(Protocol::RaydiumCpmm)
        } else if program_id == &raydium_clmm::RAYDIUM_CLMM_PROGRAM_ID {
            Some(Protocol::RaydiumClmm)
        } else if program_id == &raydium_amm_v4::RAYDIUM_AMM_V4_PROGRAM_ID {
            Some(Protocol::RaydiumAmmV4)
        } else if program_id == &meteora_damm_v2::METEORA_DAMM_V2_PROGRAM_ID {
            Some(Protocol::MeteoraDammV2)
        } else if program_id == &meteora_dlmm::METEORA_DLMM_PROGRAM_ID {
            Some(Protocol::MeteoraDlmm)
        } else if program_id == &whirlpool::WHIRLPOOL_PROGRAM_ID {
            Some(Protocol::Whirlpool)
        } else {
            None
        }
    }

    /// 检查是否为 Compute Budget Program
    #[inline]
    pub fn is_compute_budget_program(program_id: &Pubkey) -> bool {
        program_id == &COMPUTE_BUDGET_PROGRAM_ID
    }

    /// 解析 Compute Budget 指令
    ///
    /// # 参数
    /// - `instruction_data`: 指令数据
    /// - `metadata`: 事件元数据
    ///
    /// # 返回
    /// 解析成功返回 `Some(DexEvent)`，否则返回 `None`
    #[inline]
    pub fn dispatch_compute_budget_instruction(
        instruction_data: &[u8],
        metadata: EventMetadata,
    ) -> Option<DexEvent> {
        CommonEventParser::parse_compute_budget_instruction(instruction_data, metadata)
    }

    /// 获取指定协议的 program_id
    #[inline]
    pub fn get_program_id(protocol: Protocol) -> Pubkey {
        match protocol {
            Protocol::PumpFun => pumpfun::PUMPFUN_PROGRAM_ID,
            Protocol::PumpSwap => pumpswap::PUMPSWAP_PROGRAM_ID,
            Protocol::Bonk => bonk::BONK_PROGRAM_ID,
            Protocol::RaydiumCpmm => raydium_cpmm::RAYDIUM_CPMM_PROGRAM_ID,
            Protocol::RaydiumClmm => raydium_clmm::RAYDIUM_CLMM_PROGRAM_ID,
            Protocol::RaydiumAmmV4 => raydium_amm_v4::RAYDIUM_AMM_V4_PROGRAM_ID,
            Protocol::MeteoraDammV2 => meteora_damm_v2::METEORA_DAMM_V2_PROGRAM_ID,
            Protocol::MeteoraDlmm => meteora_dlmm::METEORA_DLMM_PROGRAM_ID,
            Protocol::Whirlpool => whirlpool::WHIRLPOOL_PROGRAM_ID,
        }
    }

    /// 批量获取 program_ids
    pub fn get_program_ids(protocols: &[Protocol]) -> Vec<Pubkey> {
        protocols.iter().map(|p| Self::get_program_id(p.clone())).collect()
    }

    /// 解析账户数据
    ///
    /// 根据账户的 discriminator 路由到对应协议的账户解析函数
    ///
    /// # 参数
    /// - `protocol`: 协议类型
    /// - `discriminator`: 账户判别器
    /// - `account`: 账户信息
    /// - `metadata`: 事件元数据
    ///
    /// # 返回
    /// 解析成功返回 `Some(DexEvent)`，否则返回 `None`
    pub fn dispatch_account(
        protocol: Protocol,
        discriminator: &[u8],
        account: &crate::streaming::grpc::AccountPretty,
        mut metadata: crate::streaming::event_parser::common::EventMetadata,
    ) -> Option<DexEvent> {
        // 根据协议类型设置 metadata.protocol
        use crate::streaming::event_parser::common::ProtocolType;
        metadata.protocol = match protocol {
            Protocol::PumpFun => ProtocolType::PumpFun,
            Protocol::PumpSwap => ProtocolType::PumpSwap,
            Protocol::Bonk => ProtocolType::Bonk,
            Protocol::RaydiumCpmm => ProtocolType::RaydiumCpmm,
            Protocol::RaydiumClmm => ProtocolType::RaydiumClmm,
            Protocol::RaydiumAmmV4 => ProtocolType::RaydiumAmmV4,
            Protocol::MeteoraDammV2 => ProtocolType::MeteoraDammV2,
            Protocol::MeteoraDlmm => ProtocolType::MeteoraDlmm,
            Protocol::Whirlpool => ProtocolType::Whirlpool,
        };

        match protocol {
            Protocol::PumpFun => {
                pumpfun::parse_pumpfun_account_data(discriminator, account, metadata)
            }
            Protocol::PumpSwap => {
                pumpswap::parse_pumpswap_account_data(discriminator, account, metadata)
            }
            Protocol::Bonk => bonk::parse_bonk_account_data(discriminator, account, metadata),
            Protocol::RaydiumCpmm => {
                raydium_cpmm::parse_raydium_cpmm_account_data(discriminator, account, metadata)
            }
            Protocol::RaydiumClmm => {
                raydium_clmm::parse_raydium_clmm_account_data(discriminator, account, metadata)
            }
            Protocol::RaydiumAmmV4 => {
                raydium_amm_v4::parse_raydium_amm_v4_account_data(discriminator, account, metadata)
            }
            Protocol::MeteoraDammV2 => {
                // Meteora DAMM 目前不需要解析账户数据，返回 None
                None
            }
            Protocol::MeteoraDlmm => {
                meteora_dlmm::parse_meteora_dlmm_account_data(discriminator, account, metadata)
            }
            Protocol::Whirlpool => {
                whirlpool::parse_whirlpool_account_data(discriminator, account, metadata)
            }
        }
    }
}
