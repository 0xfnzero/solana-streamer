use borsh::{BorshDeserialize, BorshSerialize};
use crossbeam_queue::ArrayQueue;
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::{borrow::Cow, fmt, sync::Arc};

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
        // 如果队列已满，push 会失败，但不会阻塞
        let _ = self.pool.push(metadata);
    }
}

// Global object pool instances
pub static EVENT_METADATA_POOL: std::sync::LazyLock<EventMetadataPool> =
    std::sync::LazyLock::new(EventMetadataPool::new);

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
    MeteoraDbc,
    OrcaWhirlpool,
    MeteoraPools,
    MeteoraDlmm,
    Common,
    PumpFees,
}

/// Event type enumeration
#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    BorshSerialize,
    BorshDeserialize,
)]
pub enum EventType {
    // PumpSwap events
    #[default]
    PumpSwapBuy,
    PumpSwapSell,
    PumpSwapCreatePool,
    PumpSwapDeposit,
    PumpSwapWithdraw,

    // PumpFun events
    PumpFunCreateToken,
    PumpFunCreateV2Token,
    PumpFunBuy,
    PumpFunBuyExactSolIn,
    PumpFunSell,
    PumpFunMigrate,
    PumpFeesCreateFeeSharingConfig,
    PumpFeesInitializeFeeConfig,
    PumpFeesResetFeeSharingConfig,
    PumpFeesRevokeFeeSharingAuthority,
    PumpFeesTransferFeeSharingAuthority,
    PumpFeesUpdateAdmin,
    PumpFeesUpdateFeeConfig,
    PumpFeesUpdateFeeShares,
    PumpFeesUpsertFeeTiers,
    PumpFunMigrateBondingCurveCreator,

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
    RaydiumClmmLiquidityChange,
    RaydiumClmmConfigChange,
    RaydiumClmmCreatePersonalPosition,
    RaydiumClmmLiquidityCalculate,
    RaydiumClmmOpenLimitOrder,
    RaydiumClmmIncreaseLimitOrder,
    RaydiumClmmDecreaseLimitOrder,
    RaydiumClmmSettleLimitOrder,
    RaydiumClmmUpdateRewardInfos,
    RaydiumClmmCreatePool,
    RaydiumClmmOpenPositionWithToken22Nft,
    RaydiumClmmOpenPositionV2,
    RaydiumClmmCollectFee,

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
    MeteoraDammV2CreatePosition,
    MeteoraDammV2ClosePosition,
    MeteoraDammV2AddLiquidity,
    MeteoraDammV2RemoveLiquidity,

    // Meteora DBC
    MeteoraDbcSwap,
    MeteoraDbcInitializePool,
    MeteoraDbcCurveComplete,

    // Orca Whirlpool
    OrcaWhirlpoolSwap,
    OrcaWhirlpoolLiquidityIncreased,
    OrcaWhirlpoolLiquidityDecreased,
    OrcaWhirlpoolPoolInitialized,

    // Meteora Pools
    MeteoraPoolsSwap,
    MeteoraPoolsAddLiquidity,
    MeteoraPoolsRemoveLiquidity,
    MeteoraPoolsBootstrapLiquidity,
    MeteoraPoolsPoolCreated,
    MeteoraPoolsSetPoolFees,

    // Meteora DLMM
    MeteoraDlmmSwap,
    MeteoraDlmmAddLiquidity,
    MeteoraDlmmRemoveLiquidity,
    MeteoraDlmmInitializePool,
    MeteoraDlmmInitializeBinArray,
    MeteoraDlmmCreatePosition,
    MeteoraDlmmClosePosition,
    MeteoraDlmmClaimFee,

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
    AccountPumpFunFeeConfig,
    AccountPumpFunSharingConfig,
    AccountPumpFunGlobalVolumeAccumulator,
    AccountPumpFunUserVolumeAccumulator,
    AccountRaydiumClmmAmmConfig,
    AccountRaydiumClmmPoolState,
    AccountRaydiumClmmTickArrayState,
    AccountRaydiumCpmmAmmConfig,
    AccountRaydiumCpmmPoolState,
    AccountOrcaWhirlpool,
    AccountOrcaPosition,
    AccountOrcaTickArray,
    AccountOrcaFeeTier,
    AccountOrcaWhirlpoolsConfig,

    NonceAccount,
    TokenAccount,
    TokenInfo,

    // Common events
    BlockMeta,
    SetComputeUnitLimit,
    SetComputeUnitPrice,
    ParserSdkError,
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
    EventType::AccountPumpFunFeeConfig,
    EventType::AccountPumpFunSharingConfig,
    EventType::AccountPumpFunGlobalVolumeAccumulator,
    EventType::AccountPumpFunUserVolumeAccumulator,
    EventType::AccountRaydiumClmmAmmConfig,
    EventType::AccountRaydiumClmmPoolState,
    EventType::AccountRaydiumClmmTickArrayState,
    EventType::AccountRaydiumCpmmAmmConfig,
    EventType::AccountRaydiumCpmmPoolState,
    EventType::AccountOrcaWhirlpool,
    EventType::AccountOrcaPosition,
    EventType::AccountOrcaTickArray,
    EventType::AccountOrcaFeeTier,
    EventType::AccountOrcaWhirlpoolsConfig,
    EventType::TokenAccount,
    EventType::TokenInfo,
    EventType::NonceAccount,
];
pub const BLOCK_EVENT_TYPES: &[EventType] = &[EventType::BlockMeta];

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
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
    pub tx_index: Option<u64>, // 新增：交易在slot中的索引
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
    /// Transaction message recent blockhash as base58 string (same encoding as signature), when available.
    #[serde(default)]
    pub recent_blockhash: Option<String>,
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
        tx_index: Option<u64>,
        recent_blockhash: Option<String>,
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
            tx_index,
            recent_blockhash,
        }
    }

    pub fn set_swap_data(&mut self, swap_data: SwapData) {
        self.swap_data = Some(swap_data);
    }
}
