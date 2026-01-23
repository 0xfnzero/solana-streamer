use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

use crate::streaming::{
    event_parser::{
        common::{EventMetadata, EventType},
        protocols::bonk::{
            BonkGlobalConfigAccountEvent, BonkPlatformConfigAccountEvent, BonkPoolStateAccountEvent,
        },
        DexEvent,
    },
    grpc::AccountPretty,
};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub enum TradeDirection {
    #[default]
    Buy,
    Sell,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub enum PoolStatus {
    #[default]
    Fund,
    Migrate,
    Trade,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct MintParams {
    pub decimals: u8,
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct VestingParams {
    pub total_locked_amount: u64,
    pub cliff_period: u64,
    pub unlock_period: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub enum AmmFeeOn {
    QuoteToken,
    BothToken,
}

impl Default for AmmFeeOn {
    fn default() -> Self {
        Self::QuoteToken
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
#[borsh(use_discriminant = true)]
#[repr(u8)]
pub enum AmmCreatorFeeOn {
    QuoteToken = 0,
    BothToken = 1,
}

impl Default for AmmCreatorFeeOn {
    fn default() -> Self {
        Self::QuoteToken
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct ConstantCurve {
    pub supply: u64,
    pub total_base_sell: u64,
    pub total_quote_fund_raising: u64,
    pub migrate_type: u8,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct FixedCurve {
    pub supply: u64,
    pub total_quote_fund_raising: u64,
    pub migrate_type: u8,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct LinearCurve {
    pub supply: u64,
    pub total_quote_fund_raising: u64,
    pub migrate_type: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub enum CurveParams {
    Constant { data: ConstantCurve },
    Fixed { data: FixedCurve },
    Linear { data: LinearCurve },
}

impl Default for CurveParams {
    fn default() -> Self {
        Self::Constant { data: ConstantCurve::default() }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct VestingSchedule {
    pub total_locked_amount: u64,
    pub cliff_period: u64,
    pub unlock_period: u64,
    pub start_time: u64,
    pub allocated_share_amount: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PoolState {
    pub epoch: u64,
    pub auth_bump: u8,
    pub status: u8,
    pub base_decimals: u8,
    pub quote_decimals: u8,
    pub migrate_type: u8,
    pub supply: u64,
    pub total_base_sell: u64,
    pub virtual_base: u64,
    pub virtual_quote: u64,
    pub real_base: u64,
    pub real_quote: u64,
    pub total_quote_fund_raising: u64,
    pub quote_protocol_fee: u64,
    pub platform_fee: u64,
    pub migrate_fee: u64,
    pub vesting_schedule: VestingSchedule,
    pub global_config: Pubkey,
    pub platform_config: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub creator: Pubkey,
    pub token_program_flag: u8,
    pub amm_creator_fee_on: AmmCreatorFeeOn,
    pub platform_vesting_share: u64,
    #[serde(with = "serde_big_array::BigArray")]
    pub padding: [u8; 54],
}

impl Default for PoolState {
    fn default() -> Self {
        Self {
            epoch: 0,
            auth_bump: 0,
            status: 0,
            base_decimals: 0,
            quote_decimals: 0,
            migrate_type: 0,
            supply: 0,
            total_base_sell: 0,
            virtual_base: 0,
            virtual_quote: 0,
            real_base: 0,
            real_quote: 0,
            total_quote_fund_raising: 0,
            quote_protocol_fee: 0,
            platform_fee: 0,
            migrate_fee: 0,
            vesting_schedule: VestingSchedule::default(),
            global_config: Pubkey::default(),
            platform_config: Pubkey::default(),
            base_mint: Pubkey::default(),
            quote_mint: Pubkey::default(),
            base_vault: Pubkey::default(),
            quote_vault: Pubkey::default(),
            creator: Pubkey::default(),
            token_program_flag: 0,
            amm_creator_fee_on: AmmCreatorFeeOn::default(),
            platform_vesting_share: 0,
            padding: [0u8; 54],
        }
    }
}

pub const POOL_STATE_SIZE: usize = 8 + 1 * 5 + 8 * 10 + 32 * 7 + 8 * 8 + 8 * 5 + 1 + 1 + 8 + 54;

pub fn pool_state_decode(data: &[u8]) -> Option<PoolState> {
    if data.len() < POOL_STATE_SIZE {
        return None;
    }
    borsh::from_slice::<PoolState>(&data[..POOL_STATE_SIZE]).ok()
}

pub fn pool_state_parser(account: &AccountPretty, mut metadata: EventMetadata) -> Option<DexEvent> {
    metadata.event_type = EventType::AccountBonkPoolState;

    if account.data.len() < POOL_STATE_SIZE + 8 {
        return None;
    }
    if let Some(pool_state) = pool_state_decode(&account.data[8..POOL_STATE_SIZE + 8]) {
        Some(DexEvent::BonkPoolStateAccountEvent(BonkPoolStateAccountEvent {
            metadata,
            pubkey: account.pubkey,
            executable: account.executable,
            lamports: account.lamports,
            owner: account.owner,
            rent_epoch: account.rent_epoch,
            pool_state,
        }))
    } else {
        None
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct GlobalConfig {
    pub epoch: u64,
    pub curve_type: u8,
    pub index: u16,
    pub migrate_fee: u64,
    pub trade_fee_rate: u64,
    pub max_share_fee_rate: u64,
    pub min_base_supply: u64,
    pub max_lock_rate: u64,
    pub min_base_sell_rate: u64,
    pub min_base_migrate_rate: u64,
    pub min_quote_fund_raising: u64,
    pub quote_mint: Pubkey,
    pub protocol_fee_owner: Pubkey,
    pub migrate_fee_owner: Pubkey,
    pub migrate_to_amm_wallet: Pubkey,
    pub migrate_to_cpswap_wallet: Pubkey,
    pub padding: [u64; 16],
}

pub const GLOBAL_CONFIG_SIZE: usize = 8 + 1 + 2 + 8 * 8 + 32 * 5 + 8 * 16;

pub fn global_config_decode(data: &[u8]) -> Option<GlobalConfig> {
    if data.len() < GLOBAL_CONFIG_SIZE {
        return None;
    }
    borsh::from_slice::<GlobalConfig>(&data[..GLOBAL_CONFIG_SIZE]).ok()
}

pub fn global_config_parser(
    account: &AccountPretty,
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::AccountBonkGlobalConfig;

    if account.data.len() < GLOBAL_CONFIG_SIZE + 8 {
        return None;
    }
    if let Some(global_config) = global_config_decode(&account.data[8..GLOBAL_CONFIG_SIZE + 8]) {
        Some(DexEvent::BonkGlobalConfigAccountEvent(BonkGlobalConfigAccountEvent {
            metadata,
            pubkey: account.pubkey,
            executable: account.executable,
            lamports: account.lamports,
            owner: account.owner,
            rent_epoch: account.rent_epoch,
            global_config,
        }))
    } else {
        None
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct BondingCurveParam {
    pub migrate_type: u8,
    pub migrate_cpmm_fee_on: u8,
    pub supply: u64,
    pub total_base_sell: u64,
    pub total_quote_fund_raising: u64,
    pub total_locked_amount: u64,
    pub cliff_period: u64,
    pub unlock_period: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PlatformCurveParam {
    pub epoch: u64,
    pub index: u8,
    pub global_config: Pubkey,
    pub bonding_curve_param: BondingCurveParam,
    #[serde(with = "serde_big_array::BigArray")]
    pub padding: [u64; 50],
}

impl Default for PlatformCurveParam {
    fn default() -> Self {
        Self {
            epoch: 0,
            index: 0,
            global_config: Pubkey::default(),
            bonding_curve_param: BondingCurveParam::default(),
            padding: [0u64; 50],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PlatformConfig {
    pub epoch: u64,
    pub platform_fee_wallet: Pubkey,
    pub platform_nft_wallet: Pubkey,
    pub platform_scale: u64,
    pub creator_scale: u64,
    pub burn_scale: u64,
    pub fee_rate: u64,
    #[serde(with = "serde_big_array::BigArray")]
    pub name: [u8; 64],
    #[serde(with = "serde_big_array::BigArray")]
    pub web: [u8; 256],
    #[serde(with = "serde_big_array::BigArray")]
    pub img: [u8; 256],
    pub cpswap_config: Pubkey,
    pub creator_fee_rate: u64,
    pub transfer_fee_extension_auth: Pubkey,
    pub platform_vesting_wallet: Pubkey,
    pub platform_vesting_scale: u64,
    pub platform_cp_creator: Pubkey,
    #[serde(with = "serde_big_array::BigArray")]
    pub padding: [u8; 108],
    pub curve_params: Vec<PlatformCurveParam>,
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            epoch: 0,
            platform_fee_wallet: Pubkey::default(),
            platform_nft_wallet: Pubkey::default(),
            platform_scale: 0,
            creator_scale: 0,
            burn_scale: 0,
            fee_rate: 0,
            name: [0u8; 64],
            web: [0u8; 256],
            img: [0u8; 256],
            cpswap_config: Pubkey::default(),
            creator_fee_rate: 0,
            transfer_fee_extension_auth: Pubkey::default(),
            platform_vesting_wallet: Pubkey::default(),
            platform_vesting_scale: 0,
            platform_cp_creator: Pubkey::default(),
            padding: [0u8; 108],
            curve_params: Vec::new(),
        }
    }
}

pub const PLATFORM_CONFIG_SIZE: usize = 8 + 32 * 2 + 8 * 4 + 64 + 256 + 256 + 32 + 8 + 32 + 32 + 8 + 32 + 108;

pub fn platform_config_decode(data: &[u8]) -> Option<PlatformConfig> {
    if data.len() < PLATFORM_CONFIG_SIZE {
        return None;
    }
    borsh::from_slice::<PlatformConfig>(&data[..PLATFORM_CONFIG_SIZE]).ok()
}

pub fn platform_config_parser(
    account: &AccountPretty,
    mut metadata: EventMetadata,
) -> Option<DexEvent> {
    metadata.event_type = EventType::AccountBonkPlatformConfig;

    if account.data.len() < PLATFORM_CONFIG_SIZE + 8 {
        return None;
    }
    if let Some(platform_config) =
        platform_config_decode(&account.data[8..PLATFORM_CONFIG_SIZE + 8])
    {
        Some(DexEvent::BonkPlatformConfigAccountEvent(BonkPlatformConfigAccountEvent {
            metadata,
            pubkey: account.pubkey,
            executable: account.executable,
            lamports: account.lamports,
            owner: account.owner,
            rent_epoch: account.rent_epoch,
            platform_config,
        }))
    } else {
        None
    }
}
