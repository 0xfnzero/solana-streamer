use borsh::{BorshDeserialize, BorshSerialize};
use crossbeam_queue::ArrayQueue;
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::{borrow::Cow, fmt, str::FromStr, sync::Arc};

use crate::streaming::{common::SimdUtils, event_parser::DexEvent};

// Object pool size configuration
const EVENT_METADATA_POOL_SIZE: usize = 1000;

/// Event metadata object pool
pub struct EventMetadataPool {
    pool: Arc<ArrayQueue<EventMetadata>>,
}

impl Default for EventMetadataPool {
    fn default() -> Self {
        Self::new()
    }
}

impl EventMetadataPool {
    pub fn new() -> Self {
        Self { pool: Arc::new(ArrayQueue::new(EVENT_METADATA_POOL_SIZE)) }
    }

    pub fn acquire(&self) -> Option<EventMetadata> {
        self.pool.pop()
    }

    pub fn release(&self, metadata: EventMetadata) {
        // If queue is full, push will fail but won't block
        let _ = self.pool.push(metadata);
    }
}

// Global object pool instances
lazy_static::lazy_static! {
    pub static ref EVENT_METADATA_POOL: EventMetadataPool = EventMetadataPool::new();
}

#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
pub enum ProtocolType {
    #[default]
    PumpSwap,
    PumpFun,
    Bonk,
    RaydiumCpmm,
    RaydiumClmm,
    RaydiumAmmV4,
    MeteoraDammV2,
    Common,
}

/// Event type enumeration
#[derive(
    Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
pub enum EventType {
    // PumpSwap events
    #[default]
    PumpSwapBuy,
    PumpSwapBuyExactQuoteIn,
    PumpSwapSell,
    PumpSwapCreatePool,
    PumpSwapDeposit,
    PumpSwapWithdraw,

    // PumpFun events
    PumpFunCreateToken,
    PumpFunCreateV2Token,
    PumpFunBuy,
    PumpFunSell,
    PumpFunMigrate,

    // Bonk events
    BonkBuyExactIn,
    BonkBuyExactOut,
    BonkSellExactIn,
    BonkSellExactOut,
    BonkInitialize,
    BonkInitializeV2,
    BonkInitializeWithToken2022,
    BonkMigrateToAmm,
    BonkMigrateToCpswap,

    // Raydium CPMM events
    RaydiumCpmmSwapBaseInput,
    RaydiumCpmmSwapBaseOutput,
    RaydiumCpmmDeposit,
    RaydiumCpmmInitialize,
    RaydiumCpmmWithdraw,

    // Raydium CLMM events
    RaydiumClmmSwap,
    RaydiumClmmSwapV2,
    RaydiumClmmClosePosition,
    RaydiumClmmIncreaseLiquidityV2,
    RaydiumClmmDecreaseLiquidityV2,
    RaydiumClmmCreatePool,
    RaydiumClmmOpenPositionWithToken22Nft,
    RaydiumClmmOpenPositionV2,

    // Raydium AMM V4 events
    RaydiumAmmV4SwapBaseIn,
    RaydiumAmmV4SwapBaseOut,
    RaydiumAmmV4Deposit,
    RaydiumAmmV4Initialize2,
    RaydiumAmmV4Withdraw,
    RaydiumAmmV4WithdrawPnl,

    // Meteora DAMM v2 events
    MeteoraDammV2Swap,
    MeteoraDammV2Swap2,
    MeteoraDammV2InitializePool,
    MeteoraDammV2InitializeCustomizablePool,
    MeteoraDammV2InitializePoolWithDynamicConfig,

    // Account events
    AccountRaydiumAmmV4AmmInfo,
    AccountPumpSwapGlobalConfig,
    AccountPumpSwapPool,
    AccountBonkPoolState,
    AccountBonkGlobalConfig,
    AccountBonkPlatformConfig,
    AccountBonkVestingRecord,
    AccountPumpFunBondingCurve,
    AccountPumpFunGlobal,
    AccountRaydiumClmmAmmConfig,
    AccountRaydiumClmmPoolState,
    AccountRaydiumClmmTickArrayState,
    AccountRaydiumCpmmAmmConfig,
    AccountRaydiumCpmmPoolState,

    NonceAccount,
    TokenAccount,

    // Common events
    BlockMeta,
    SetComputeUnitLimit,
    SetComputeUnitPrice,
    Unknown,
}

pub const ACCOUNT_EVENT_TYPES: &[EventType] = &[
    EventType::AccountRaydiumAmmV4AmmInfo,
    EventType::AccountPumpSwapGlobalConfig,
    EventType::AccountPumpSwapPool,
    EventType::AccountBonkPoolState,
    EventType::AccountBonkGlobalConfig,
    EventType::AccountBonkPlatformConfig,
    EventType::AccountBonkVestingRecord,
    EventType::AccountPumpFunBondingCurve,
    EventType::AccountPumpFunGlobal,
    EventType::AccountRaydiumClmmAmmConfig,
    EventType::AccountRaydiumClmmPoolState,
    EventType::AccountRaydiumClmmTickArrayState,
    EventType::AccountRaydiumCpmmAmmConfig,
    EventType::AccountRaydiumCpmmPoolState,
    EventType::TokenAccount,
    EventType::NonceAccount,
];
pub const BLOCK_EVENT_TYPES: &[EventType] = &[EventType::BlockMeta];

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventType::PumpSwapBuy => write!(f, "PumpSwapBuy"),
            EventType::PumpSwapBuyExactQuoteIn => write!(f, "PumpSwapBuyExactQuoteIn"),
            EventType::PumpSwapSell => write!(f, "PumpSwapSell"),
            EventType::PumpSwapCreatePool => write!(f, "PumpSwapCreatePool"),
            EventType::PumpSwapDeposit => write!(f, "PumpSwapDeposit"),
            EventType::PumpSwapWithdraw => write!(f, "PumpSwapWithdraw"),
            EventType::PumpFunCreateToken => write!(f, "PumpFunCreateToken"),
            EventType::PumpFunCreateV2Token => write!(f, "PumpFunCreateV2Token"),
            EventType::PumpFunBuy => write!(f, "PumpFunBuy"),
            EventType::PumpFunSell => write!(f, "PumpFunSell"),
            EventType::PumpFunMigrate => write!(f, "PumpFunMigrate"),
            EventType::BonkBuyExactIn => write!(f, "BonkBuyExactIn"),
            EventType::BonkBuyExactOut => write!(f, "BonkBuyExactOut"),
            EventType::BonkSellExactIn => write!(f, "BonkSellExactIn"),
            EventType::BonkSellExactOut => write!(f, "BonkSellExactOut"),
            EventType::BonkInitialize => write!(f, "BonkInitialize"),
            EventType::BonkInitializeV2 => write!(f, "BonkInitializeV2"),
            EventType::BonkInitializeWithToken2022 => write!(f, "BonkInitializeWithToken2022"),
            EventType::BonkMigrateToAmm => write!(f, "BonkMigrateToAmm"),
            EventType::BonkMigrateToCpswap => write!(f, "BonkMigrateToCpswap"),
            EventType::RaydiumCpmmSwapBaseInput => write!(f, "RaydiumCpmmSwapBaseInput"),
            EventType::RaydiumCpmmSwapBaseOutput => write!(f, "RaydiumCpmmSwapBaseOutput"),
            EventType::RaydiumCpmmDeposit => write!(f, "RaydiumCpmmDeposit"),
            EventType::RaydiumCpmmInitialize => write!(f, "RaydiumCpmmInitialize"),
            EventType::RaydiumCpmmWithdraw => write!(f, "RaydiumCpmmWithdraw"),
            EventType::RaydiumClmmSwap => write!(f, "RaydiumClmmSwap"),
            EventType::RaydiumClmmSwapV2 => write!(f, "RaydiumClmmSwapV2"),
            EventType::RaydiumClmmClosePosition => write!(f, "RaydiumClmmClosePosition"),
            EventType::RaydiumClmmDecreaseLiquidityV2 => {
                write!(f, "RaydiumClmmDecreaseLiquidityV2")
            }
            EventType::RaydiumClmmCreatePool => write!(f, "RaydiumClmmCreatePool"),
            EventType::RaydiumClmmIncreaseLiquidityV2 => {
                write!(f, "RaydiumClmmIncreaseLiquidityV2")
            }
            EventType::RaydiumClmmOpenPositionWithToken22Nft => {
                write!(f, "RaydiumClmmOpenPositionWithToken22Nft")
            }
            EventType::RaydiumClmmOpenPositionV2 => write!(f, "RaydiumClmmOpenPositionV2"),
            EventType::RaydiumAmmV4SwapBaseIn => write!(f, "RaydiumAmmV4SwapBaseIn"),
            EventType::RaydiumAmmV4SwapBaseOut => write!(f, "RaydiumAmmV4SwapBaseOut"),
            EventType::RaydiumAmmV4Deposit => write!(f, "RaydiumAmmV4Deposit"),
            EventType::RaydiumAmmV4Initialize2 => write!(f, "RaydiumAmmV4Initialize2"),
            EventType::RaydiumAmmV4Withdraw => write!(f, "RaydiumAmmV4Withdraw"),
            EventType::RaydiumAmmV4WithdrawPnl => write!(f, "RaydiumAmmV4WithdrawPnl"),
            EventType::MeteoraDammV2Swap => write!(f, "MeteoraDammV2Swap"),
            EventType::MeteoraDammV2Swap2 => write!(f, "MeteoraDammV2Swap2"),
            EventType::MeteoraDammV2InitializePool => write!(f, "MeteoraDammV2InitializePool"),
            EventType::MeteoraDammV2InitializeCustomizablePool => write!(f, "MeteoraDammV2InitializeCustomizablePool"),
            EventType::MeteoraDammV2InitializePoolWithDynamicConfig => write!(f, "MeteoraDammV2InitializePoolWithDynamicConfig"),
            EventType::AccountRaydiumAmmV4AmmInfo => write!(f, "AccountRaydiumAmmV4AmmInfo"),
            EventType::AccountPumpSwapGlobalConfig => write!(f, "AccountPumpSwapGlobalConfig"),
            EventType::AccountPumpSwapPool => write!(f, "AccountPumpSwapPool"),
            EventType::AccountBonkPoolState => write!(f, "AccountBonkPoolState"),
            EventType::AccountBonkGlobalConfig => write!(f, "AccountBonkGlobalConfig"),
            EventType::AccountBonkPlatformConfig => write!(f, "AccountBonkPlatformConfig"),
            EventType::AccountBonkVestingRecord => write!(f, "AccountBonkVestingRecord"),
            EventType::AccountPumpFunBondingCurve => write!(f, "AccountPumpFunBondingCurve"),
            EventType::AccountPumpFunGlobal => write!(f, "AccountPumpFunGlobal"),
            EventType::AccountRaydiumClmmAmmConfig => write!(f, "AccountRaydiumClmmAmmConfig"),
            EventType::AccountRaydiumClmmPoolState => write!(f, "AccountRaydiumClmmPoolState"),
            EventType::AccountRaydiumClmmTickArrayState => {
                write!(f, "AccountRaydiumClmmTickArrayState")
            }
            EventType::AccountRaydiumCpmmAmmConfig => write!(f, "AccountRaydiumCpmmAmmConfig"),
            EventType::AccountRaydiumCpmmPoolState => write!(f, "AccountRaydiumCpmmPoolState"),
            EventType::TokenAccount => write!(f, "TokenAccount"),
            EventType::NonceAccount => write!(f, "NonceAccount"),
            EventType::BlockMeta => write!(f, "BlockMeta"),
            EventType::SetComputeUnitLimit => write!(f, "SetComputeUnitLimit"),
            EventType::SetComputeUnitPrice => write!(f, "SetComputeUnitPrice"),
            EventType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Parse result
#[derive(Debug, Clone)]
pub struct ParseResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ParseResult<T> {
    pub fn success(data: T) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    pub fn failure(error: String) -> Self {
        Self { success: false, data: None, error: Some(error) }
    }

    pub fn is_success(&self) -> bool {
        self.success
    }

    pub fn is_failure(&self) -> bool {
        !self.success
    }
}

/// Protocol information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolInfo {
    pub name: String,
    pub program_ids: Vec<Pubkey>,
}

impl ProtocolInfo {
    pub fn new(name: String, program_ids: Vec<Pubkey>) -> Self {
        Self { name, program_ids }
    }

    pub fn supports_program(&self, program_id: &Pubkey) -> bool {
        self.program_ids.contains(program_id)
    }
}

#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
pub struct SwapData {
    pub from_mint: Pubkey,
    pub to_mint: Pubkey,
    pub from_amount: u64,
    pub to_amount: u64,
    pub description: Option<Cow<'static, str>>,
}

/// Event metadata
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventMetadata {
    pub signature: Signature,
    pub slot: u64,
    pub transaction_index: Option<u64>, // New: transaction index within the slot
    pub block_time: i64,
    pub block_time_ms: i64,
    pub recv_us: i64,
    pub handle_us: i64,
    pub protocol: ProtocolType,
    pub event_type: EventType,
    pub program_id: Pubkey,
    pub swap_data: Option<SwapData>,
    pub outer_index: i64,
    pub inner_index: Option<i64>,
}

impl EventMetadata {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        signature: Signature,
        slot: u64,
        block_time: i64,
        block_time_ms: i64,
        protocol: ProtocolType,
        event_type: EventType,
        program_id: Pubkey,
        outer_index: i64,
        inner_index: Option<i64>,
        recv_us: i64,
        transaction_index: Option<u64>,
    ) -> Self {
        Self {
            signature,
            slot,
            block_time,
            block_time_ms,
            recv_us,
            handle_us: 0,
            protocol,
            event_type,
            program_id,
            swap_data: None,
            outer_index,
            inner_index,
            transaction_index,
        }
    }

    pub fn set_swap_data(&mut self, swap_data: SwapData) {
        self.swap_data = Some(swap_data);
    }

    /// Recycle EventMetadata to object pool
    pub fn recycle(self) {
        EVENT_METADATA_POOL.release(self);
    }
}

lazy_static::lazy_static! {
    static ref SOL_MINT: Pubkey = Pubkey::from_str("So11111111111111111111111111111111111111111").unwrap();
    static ref SYSTEM_PROGRAMS: [Pubkey; 3] = [
        Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap(),
        Pubkey::from_str("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb").unwrap(),
        Pubkey::from_str("11111111111111111111111111111111").unwrap(),
    ];
}

/// Parse token transfer data from next instructions
pub fn parse_swap_data_from_next_instructions(
    event: &DexEvent,
    inner_instruction: &solana_transaction_status::InnerInstructions,
    current_index: i8,
    accounts: &[Pubkey],
) -> Option<SwapData> {
    let mut swap_data = SwapData {
        from_mint: Pubkey::default(),
        to_mint: Pubkey::default(),
        from_amount: 0,
        to_amount: 0,
        description: None,
    };

    // First extract key information from event
    // let mut user: Option<Pubkey> = None;
    let mut from_mint: Option<Pubkey> = None;
    let mut to_mint: Option<Pubkey> = None;
    let mut user_from_token: Option<Pubkey> = None;
    let mut user_to_token: Option<Pubkey> = None;
    let mut from_vault: Option<Pubkey> = None;
    let mut to_vault: Option<Pubkey> = None;

    match event {
        DexEvent::BonkTradeEvent(e) => {
            // user = Some(e.payer);
            from_mint = Some(e.base_token_mint);
            to_mint = Some(e.quote_token_mint);
            user_from_token = Some(e.user_base_token);
            user_to_token = Some(e.user_quote_token);
            from_vault = Some(e.base_vault);
            to_vault = Some(e.quote_vault);
        }
        DexEvent::PumpFunTradeEvent(e) => {
            swap_data.from_mint = if e.is_buy { *SOL_MINT } else { e.mint };
            swap_data.to_mint = if e.is_buy { e.mint } else { *SOL_MINT };
        }
        DexEvent::PumpSwapBuyEvent(e) => {
            swap_data.from_mint = e.quote_mint;
            swap_data.to_mint = e.base_mint;
        }
        DexEvent::PumpSwapBuyExactQuoteInEvent(e) => {
            swap_data.from_mint = e.quote_mint;
            swap_data.to_mint = e.base_mint;
        }
        DexEvent::PumpSwapSellEvent(e) => {
            swap_data.from_mint = e.base_mint;
            swap_data.to_mint = e.quote_mint;
        }
        DexEvent::RaydiumCpmmSwapEvent(e) => {
            // user = Some(e.payer);
            from_mint = Some(e.input_token_mint);
            to_mint = Some(e.output_token_mint);
            user_from_token = Some(e.input_token_account);
            user_to_token = Some(e.output_token_account);
            from_vault = Some(e.input_vault);
            to_vault = Some(e.output_vault);
        }
        DexEvent::RaydiumClmmSwapEvent(e) => {
            // user = Some(e.payer);
            swap_data.description =
                Some("Unable to get from_mint and to_mint from RaydiumClmmSwapEvent".into());
            user_from_token = Some(e.input_token_account);
            user_to_token = Some(e.output_token_account);
            from_vault = Some(e.input_vault);
            to_vault = Some(e.output_vault);
        }
        DexEvent::RaydiumClmmSwapV2Event(e) => {
            // user = Some(e.payer);
            from_mint = Some(e.input_vault_mint);
            to_mint = Some(e.output_vault_mint);
            user_from_token = Some(e.input_token_account);
            user_to_token = Some(e.output_token_account);
            from_vault = Some(e.input_vault);
            to_vault = Some(e.output_vault);
        }
        DexEvent::RaydiumAmmV4SwapEvent(e) => {
            // user = Some(e.user_source_owner);
            swap_data.description =
                Some("Unable to get from_mint and to_mint from RaydiumAmmV4SwapEvent".into());
            user_from_token = Some(e.user_source_token_account);
            user_to_token = Some(e.user_destination_token_account);
            from_vault = Some(e.pool_pc_token_account);
            to_vault = Some(e.pool_coin_token_account);
        }
        _ => {}
    }

    let user_to_token = user_to_token.unwrap_or_default();
    let user_from_token = user_from_token.unwrap_or_default();
    let to_vault = to_vault.unwrap_or_default();
    let from_vault = from_vault.unwrap_or_default();
    let to_mint = to_mint.unwrap_or_default();
    let from_mint = from_mint.unwrap_or_default();

    // Single loop to complete extraction and validation
    for instruction in inner_instruction.instructions.iter().skip((current_index + 1) as usize) {
        let compiled = &instruction.instruction;
        let program_id = accounts[compiled.program_id_index as usize];
        if !SYSTEM_PROGRAMS.contains(&program_id) {
            break;
        }
        let data = &compiled.data;

        // Use SIMD to validate data format
        if !SimdUtils::validate_data_format(data, 8) {
            continue;
        }

        let get_pubkey = |i: usize| accounts[compiled.accounts[i] as usize];
        let (source, destination, amount) = match data[0] {
            12 if compiled.accounts.len() >= 4 => {
                let amt = u64::from_le_bytes(data[1..9].try_into().unwrap());
                (get_pubkey(0), get_pubkey(2), amt)
            }
            3 if compiled.accounts.len() >= 3 => {
                let amt = u64::from_le_bytes(data[1..9].try_into().unwrap());
                (get_pubkey(0), get_pubkey(1), amt)
            }
            2 if compiled.accounts.len() >= 2 => {
                let amt = u64::from_le_bytes(data[4..12].try_into().unwrap());
                (get_pubkey(0), get_pubkey(1), amt)
            }
            _ => continue,
        };

        match (source, destination) {
            (s, d) if s == user_to_token && d == to_vault => {
                swap_data.from_mint = to_mint;
                swap_data.from_amount = amount;
            }
            (s, d) if s == from_vault && d == user_from_token => {
                swap_data.to_mint = from_mint;
                swap_data.to_amount = amount;
            }
            (s, d) if s == user_from_token && d == from_vault => {
                swap_data.from_mint = from_mint;
                swap_data.from_amount = amount;
            }
            (s, d) if s == to_vault && d == user_to_token => {
                swap_data.to_mint = to_mint;
                swap_data.to_amount = amount;
            }
            (s, d) if s == user_from_token && d == to_vault => {
                swap_data.from_mint = from_mint;
                swap_data.from_amount = amount;
            }
            (s, d) if s == from_vault && d == user_to_token => {
                swap_data.to_mint = to_mint;
                swap_data.to_amount = amount;
            }
            _ => {}
        }
        if swap_data.from_mint != Pubkey::default() && swap_data.to_mint != Pubkey::default() {
            break;
        }
        if swap_data.from_amount != 0 && swap_data.to_amount != 0 {
            break;
        }
    }

    if swap_data.from_mint != Pubkey::default()
        || swap_data.to_mint != Pubkey::default()
        || swap_data.from_amount != 0
        || swap_data.to_amount != 0
    {
        Some(swap_data)
    } else {
        None
    }
}

/// Parse token transfer data from next instructions
/// TODO: - wait refactor
pub fn parse_swap_data_from_next_grpc_instructions(
    event: &DexEvent,
    inner_instruction: &yellowstone_grpc_proto::prelude::InnerInstructions,
    current_index: i8,
    accounts: &[Pubkey],
) -> Option<SwapData> {
    let mut swap_data = SwapData {
        from_mint: Pubkey::default(),
        to_mint: Pubkey::default(),
        from_amount: 0,
        to_amount: 0,
        description: None,
    };

    // First extract key information from event
    // let mut user: Option<Pubkey> = None;
    let mut from_mint: Option<Pubkey> = None;
    let mut to_mint: Option<Pubkey> = None;
    let mut user_from_token: Option<Pubkey> = None;
    let mut user_to_token: Option<Pubkey> = None;
    let mut from_vault: Option<Pubkey> = None;
    let mut to_vault: Option<Pubkey> = None;

    match event {
        DexEvent::BonkTradeEvent(e) => {
            // user = Some(e.payer);
            from_mint = Some(e.base_token_mint);
            to_mint = Some(e.quote_token_mint);
            user_from_token = Some(e.user_base_token);
            user_to_token = Some(e.user_quote_token);
            from_vault = Some(e.base_vault);
            to_vault = Some(e.quote_vault);
        }
        DexEvent::PumpFunTradeEvent(e) => {
            swap_data.from_mint = if e.is_buy { *SOL_MINT } else { e.mint };
            swap_data.to_mint = if e.is_buy { e.mint } else { *SOL_MINT };
        }
        DexEvent::PumpSwapBuyEvent(e) => {
            swap_data.from_mint = e.quote_mint;
            swap_data.to_mint = e.base_mint;
        }
        DexEvent::PumpSwapBuyExactQuoteInEvent(e) => {
            swap_data.from_mint = e.quote_mint;
            swap_data.to_mint = e.base_mint;
        }
        DexEvent::PumpSwapSellEvent(e) => {
            swap_data.from_mint = e.base_mint;
            swap_data.to_mint = e.quote_mint;
        }
        DexEvent::RaydiumCpmmSwapEvent(e) => {
            // user = Some(e.payer);
            from_mint = Some(e.input_token_mint);
            to_mint = Some(e.output_token_mint);
            user_from_token = Some(e.input_token_account);
            user_to_token = Some(e.output_token_account);
            from_vault = Some(e.input_vault);
            to_vault = Some(e.output_vault);
        }
        DexEvent::RaydiumClmmSwapEvent(e) => {
            // user = Some(e.payer);
            swap_data.description =
                Some("Unable to get from_mint and to_mint from RaydiumClmmSwapEvent".into());
            user_from_token = Some(e.input_token_account);
            user_to_token = Some(e.output_token_account);
            from_vault = Some(e.input_vault);
            to_vault = Some(e.output_vault);
        }
        DexEvent::RaydiumClmmSwapV2Event(e) => {
            // user = Some(e.payer);
            from_mint = Some(e.input_vault_mint);
            to_mint = Some(e.output_vault_mint);
            user_from_token = Some(e.input_token_account);
            user_to_token = Some(e.output_token_account);
            from_vault = Some(e.input_vault);
            to_vault = Some(e.output_vault);
        }
        DexEvent::RaydiumAmmV4SwapEvent(e) => {
            // user = Some(e.user_source_owner);
            swap_data.description =
                Some("Unable to get from_mint and to_mint from RaydiumAmmV4SwapEvent".into());
            user_from_token = Some(e.user_source_token_account);
            user_to_token = Some(e.user_destination_token_account);
            from_vault = Some(e.pool_pc_token_account);
            to_vault = Some(e.pool_coin_token_account);
        }
        _ => {}
    }

    let user_to_token = user_to_token.unwrap_or_default();
    let user_from_token = user_from_token.unwrap_or_default();
    let to_vault = to_vault.unwrap_or_default();
    let from_vault = from_vault.unwrap_or_default();
    let to_mint = to_mint.unwrap_or_default();
    let from_mint = from_mint.unwrap_or_default();

    // Single loop to complete extraction and validation
    for instruction in inner_instruction.instructions.iter().skip((current_index + 1) as usize) {
        let compiled = &instruction;
        let program_id = accounts[compiled.program_id_index as usize];
        if !SYSTEM_PROGRAMS.contains(&program_id) {
            break;
        }
        let data = &compiled.data;

        // Use SIMD to validate data format
        if !SimdUtils::validate_data_format(data, 8) {
            continue;
        }

        let get_pubkey = |i: usize| accounts[compiled.accounts[i] as usize];
        let (source, destination, amount) = match data[0] {
            12 if compiled.accounts.len() >= 4 => {
                let amt = u64::from_le_bytes(data[1..9].try_into().unwrap());
                (get_pubkey(0), get_pubkey(2), amt)
            }
            3 if compiled.accounts.len() >= 3 => {
                let amt = u64::from_le_bytes(data[1..9].try_into().unwrap());
                (get_pubkey(0), get_pubkey(1), amt)
            }
            2 if compiled.accounts.len() >= 2 => {
                let amt = u64::from_le_bytes(data[4..12].try_into().unwrap());
                (get_pubkey(0), get_pubkey(1), amt)
            }
            _ => continue,
        };

        match (source, destination) {
            (s, d) if s == user_to_token && d == to_vault => {
                swap_data.from_mint = to_mint;
                swap_data.from_amount = amount;
            }
            (s, d) if s == from_vault && d == user_from_token => {
                swap_data.to_mint = from_mint;
                swap_data.to_amount = amount;
            }
            (s, d) if s == user_from_token && d == from_vault => {
                swap_data.from_mint = from_mint;
                swap_data.from_amount = amount;
            }
            (s, d) if s == to_vault && d == user_to_token => {
                swap_data.to_mint = to_mint;
                swap_data.to_amount = amount;
            }
            (s, d) if s == user_from_token && d == to_vault => {
                swap_data.from_mint = from_mint;
                swap_data.from_amount = amount;
            }
            (s, d) if s == from_vault && d == user_to_token => {
                swap_data.to_mint = to_mint;
                swap_data.to_amount = amount;
            }
            _ => {}
        }
        if swap_data.from_mint != Pubkey::default() && swap_data.to_mint != Pubkey::default() {
            break;
        }
        if swap_data.from_amount != 0 && swap_data.to_amount != 0 {
            break;
        }
    }

    if swap_data.from_mint != Pubkey::default()
        || swap_data.to_mint != Pubkey::default()
        || swap_data.from_amount != 0
        || swap_data.to_amount != 0
    {
        Some(swap_data)
    } else {
        None
    }
}
