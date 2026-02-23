use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

use crate::streaming::event_parser::common::EventMetadata;
use crate::streaming::event_parser::protocols::pumpfun::types::{BondingCurve, Global};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PumpFunCreateTokenEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub mint: Pubkey,
    pub bonding_curve: Pubkey,
    pub user: Pubkey,
    pub creator: Pubkey,
    pub timestamp: i64,
    pub virtual_token_reserves: u64,
    pub virtual_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub token_total_supply: u64,
    #[borsh(skip)]
    pub token_program: Pubkey,
    #[borsh(skip)]
    pub is_mayhem_mode: bool,
    /// Whether cashback is enabled (IDL CreateEvent.is_cashback_enabled)
    #[borsh(skip)]
    pub is_cashback_enabled: bool,
    #[borsh(skip)]
    pub mint_authority: Pubkey,
    #[borsh(skip)]
    pub associated_bonding_curve: Pubkey,
    #[borsh(skip)]
    pub global: Pubkey,
    #[borsh(skip)]
    pub mpl_token_metadata: Pubkey,
    #[borsh(skip)]
    pub metadata_account: Pubkey,
    #[borsh(skip)]
    pub system_program: Pubkey,
    #[borsh(skip)]
    pub associated_token_program: Pubkey,
    #[borsh(skip)]
    pub rent: Pubkey,
    #[borsh(skip)]
    pub event_authority: Pubkey,
    #[borsh(skip)]
    pub program: Pubkey,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PumpFunCreateV2TokenEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub mint: Pubkey,
    pub bonding_curve: Pubkey,
    pub user: Pubkey,
    pub creator: Pubkey,
    pub timestamp: i64,
    pub virtual_token_reserves: u64,
    pub virtual_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub token_total_supply: u64,
    pub token_program: Pubkey,
    pub is_mayhem_mode: bool,
    /// Whether cashback is enabled (IDL CreateEvent.is_cashback_enabled)
    pub is_cashback_enabled: bool,
    #[borsh(skip)]
    pub mint_authority: Pubkey,
    #[borsh(skip)]
    pub associated_bonding_curve: Pubkey,
    #[borsh(skip)]
    pub global: Pubkey,
    #[borsh(skip)]
    pub system_program: Pubkey,
    #[borsh(skip)]
    pub associated_token_program: Pubkey,
    #[borsh(skip)]
    pub mayhem_program_id: Pubkey,
    #[borsh(skip)]
    pub global_params: Pubkey,
    #[borsh(skip)]
    pub sol_vault: Pubkey,
    #[borsh(skip)]
    pub mayhem_state: Pubkey,
    #[borsh(skip)]
    pub mayhem_token_vault: Pubkey,
    #[borsh(skip)]
    pub event_authority: Pubkey,
    #[borsh(skip)]
    pub program: Pubkey,
}

pub fn pumpfun_create_v2_token_event_log_decode(data: &[u8]) -> Option<PumpFunCreateV2TokenEvent> {
    let mut offset = 0;

    // Parse name string: [length (4 bytes u32)][string bytes]
    if data.len() < offset + 4 {
        return None;
    }
    let name_len = u32::from_le_bytes(data[offset..offset + 4].try_into().ok()?) as usize;
    offset += 4;
    if data.len() < offset + name_len {
        return None;
    }
    let name = String::from_utf8(data[offset..offset + name_len].to_vec()).ok()?;
    offset += name_len;

    // Parse symbol string
    if data.len() < offset + 4 {
        return None;
    }
    let symbol_len = u32::from_le_bytes(data[offset..offset + 4].try_into().ok()?) as usize;
    offset += 4;
    if data.len() < offset + symbol_len {
        return None;
    }
    let symbol = String::from_utf8(data[offset..offset + symbol_len].to_vec()).ok()?;
    offset += symbol_len;

    // Parse uri string
    if data.len() < offset + 4 {
        return None;
    }
    let uri_len = u32::from_le_bytes(data[offset..offset + 4].try_into().ok()?) as usize;
    offset += 4;
    if data.len() < offset + uri_len {
        return None;
    }
    let uri = String::from_utf8(data[offset..offset + uri_len].to_vec()).ok()?;
    offset += uri_len;

    // Parse Pubkey fields (32 bytes each)
    if data.len() < offset + 32 {
        return None;
    }
    let mint = Pubkey::new_from_array(data[offset..offset + 32].try_into().ok()?);
    offset += 32;

    if data.len() < offset + 32 {
        return None;
    }
    let bonding_curve = Pubkey::new_from_array(data[offset..offset + 32].try_into().ok()?);
    offset += 32;

    if data.len() < offset + 32 {
        return None;
    }
    let user = Pubkey::new_from_array(data[offset..offset + 32].try_into().ok()?);
    offset += 32;

    if data.len() < offset + 32 {
        return None;
    }
    let creator = Pubkey::new_from_array(data[offset..offset + 32].try_into().ok()?);
    offset += 32;

    // Parse numeric fields
    if data.len() < offset + 8 {
        return None;
    }
    let timestamp = i64::from_le_bytes(data[offset..offset + 8].try_into().ok()?);
    offset += 8;

    if data.len() < offset + 8 {
        return None;
    }
    let virtual_token_reserves = u64::from_le_bytes(data[offset..offset + 8].try_into().ok()?);
    offset += 8;

    if data.len() < offset + 8 {
        return None;
    }
    let virtual_sol_reserves = u64::from_le_bytes(data[offset..offset + 8].try_into().ok()?);
    offset += 8;

    if data.len() < offset + 8 {
        return None;
    }
    let real_token_reserves = u64::from_le_bytes(data[offset..offset + 8].try_into().ok()?);
    offset += 8;

    if data.len() < offset + 8 {
        return None;
    }
    let token_total_supply = u64::from_le_bytes(data[offset..offset + 8].try_into().ok()?);
    offset += 8;

    // If data length allows, parse V2 extra fields: token_program (32 bytes) + is_mayhem_mode (1 byte) + is_cashback_enabled (1 byte)
    let (token_program, is_mayhem_mode, is_cashback_enabled) = if data.len() >= offset + 34 {
        let token_program = Pubkey::new_from_array(data[offset..offset + 32].try_into().ok()?);
        let is_mayhem_mode = data[offset + 32] != 0;
        let is_cashback_enabled = data[offset + 33] != 0;
        (token_program, is_mayhem_mode, is_cashback_enabled)
    } else if data.len() >= offset + 33 {
        // Backward compat: only token_program + is_mayhem_mode, no is_cashback_enabled
        let token_program = Pubkey::new_from_array(data[offset..offset + 32].try_into().ok()?);
        let is_mayhem_mode = data[offset + 32] != 0;
        (token_program, is_mayhem_mode, false)
    } else {
        (Pubkey::default(), false, false)
    };

    Some(PumpFunCreateV2TokenEvent {
        name,
        symbol,
        uri,
        mint,
        bonding_curve,
        user,
        creator,
        timestamp,
        virtual_token_reserves,
        virtual_sol_reserves,
        real_token_reserves,
        token_total_supply,
        token_program,
        is_mayhem_mode,
        is_cashback_enabled,
        ..Default::default()
    })
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PumpFunTradeEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub mint: Pubkey,
    pub sol_amount: u64,
    pub token_amount: u64,
    pub is_buy: bool,
    pub user: Pubkey,
    pub timestamp: i64,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub fee_recipient: Pubkey,
    pub fee_basis_points: u64,
    pub fee: u64,
    pub creator: Pubkey,
    pub creator_fee_basis_points: u64,
    pub creator_fee: u64,
    pub track_volume: bool,
    pub total_unclaimed_tokens: u64,
    pub total_claimed_tokens: u64,
    pub current_sol_volume: u64,
    pub last_update_timestamp: i64,

    #[borsh(skip)]
    pub max_sol_cost: u64,
    #[borsh(skip)]
    pub min_sol_output: u64,
    #[borsh(skip)]
    pub amount: u64,
    #[borsh(skip)]
    pub is_bot: bool,
    #[borsh(skip)]
    pub is_dev_create_token_trade: bool, // Whether this is a dev-created token trade

    #[borsh(skip)]
    pub global: Pubkey,
    // #[borsh(skip)]
    // pub fee_recipient: Pubkey,
    // #[borsh(skip)]
    // pub mint: Pubkey,
    #[borsh(skip)]
    pub bonding_curve: Pubkey,
    #[borsh(skip)]
    pub associated_bonding_curve: Pubkey,
    #[borsh(skip)]
    pub associated_user: Pubkey,
    // #[borsh(skip)]
    // pub user: Pubkey,
    #[borsh(skip)]
    pub system_program: Pubkey,
    #[borsh(skip)]
    pub token_program: Pubkey,
    #[borsh(skip)]
    pub creator_vault: Pubkey,
    #[borsh(skip)]
    pub event_authority: Pubkey,
    #[borsh(skip)]
    pub program: Pubkey,
    #[borsh(skip)]
    pub global_volume_accumulator: Pubkey,
    #[borsh(skip)]
    pub user_volume_accumulator: Pubkey,
    #[borsh(skip)]
    pub fee_config: Pubkey,
    #[borsh(skip)]
    pub fee_program: Pubkey,

    // === IDL TradeEvent extension fields (aligned with sol-parser-sdk / pumpfun IDL) ===
    #[borsh(skip)]
    pub ix_name: String,
    #[borsh(skip)]
    pub mayhem_mode: bool,
    #[borsh(skip)]
    pub cashback_fee_basis_points: u64,
    #[borsh(skip)]
    pub cashback: u64,
    /// Whether this is a cashback coin (cashback_fee_basis_points > 0)
    #[borsh(skip)]
    pub is_cashback_coin: bool,
}

/// Borsh byte length of TradeEvent fixed fields (IDL order; excludes ix_name and following variable part).
/// Layout: mint(32)+sol_amount(8)+token_amount(8)+is_buy(1)+user(32)+timestamp(8)+virtual_sol(8)+virtual_token(8)+real_sol(8)+real_token(8)+fee_recipient(32)+fee_basis_points(8)+fee(8)+creator(32)+creator_fee_bps(8)+creator_fee(8)+track_volume(1)+total_unclaimed(8)+total_claimed(8)+current_sol_volume(8)+last_update_timestamp(8) = 250
pub const PUMPFUN_TRADE_EVENT_LOG_SIZE: usize = 250;

/// Decode TradeEvent log; if data.len() > 250 then parse ix_name, mayhem_mode, cashback (IDL-aligned).
pub fn pumpfun_trade_event_log_decode(data: &[u8]) -> Option<PumpFunTradeEvent> {
    if data.len() < PUMPFUN_TRADE_EVENT_LOG_SIZE {
        return None;
    }
    let mut event = borsh::from_slice::<PumpFunTradeEvent>(&data[..PUMPFUN_TRADE_EVENT_LOG_SIZE]).ok()?;
    let mut offset = PUMPFUN_TRADE_EVENT_LOG_SIZE;
    if offset < data.len() {
        let (ix_name, inc) = read_borsh_string(data, offset).unwrap_or((String::new(), 0));
        offset += inc;
        event.ix_name = ix_name;
    }
    if offset + 1 <= data.len() {
        event.mayhem_mode = data[offset] != 0;
        offset += 1;
    }
    if offset + 8 <= data.len() {
        event.cashback_fee_basis_points = u64::from_le_bytes(data[offset..offset + 8].try_into().ok()?);
        offset += 8;
    }
    if offset + 8 <= data.len() {
        event.cashback = u64::from_le_bytes(data[offset..offset + 8].try_into().ok()?);
    }
    event.is_cashback_coin = event.cashback_fee_basis_points > 0;
    Some(event)
}

#[inline]
fn read_borsh_string(data: &[u8], start: usize) -> Option<(String, usize)> {
    if start + 4 > data.len() {
        return None;
    }
    let len = u32::from_le_bytes(data[start..start + 4].try_into().ok()?) as usize;
    let start = start + 4;
    if start + len > data.len() {
        return None;
    }
    let s = String::from_utf8_lossy(&data[start..start + len]).to_string();
    Some((s, 4 + len))
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PumpFunMigrateEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,

    pub user: Pubkey,
    pub mint: Pubkey,
    pub mint_amount: u64,
    pub sol_amount: u64,
    pub pool_migration_fee: u64,
    pub bonding_curve: Pubkey,
    pub timestamp: i64,
    pub pool: Pubkey,

    #[borsh(skip)]
    pub global: Pubkey,
    #[borsh(skip)]
    pub withdraw_authority: Pubkey,
    #[borsh(skip)]
    pub associated_bonding_curve: Pubkey,
    #[borsh(skip)]
    pub system_program: Pubkey,
    #[borsh(skip)]
    pub token_program: Pubkey,
    #[borsh(skip)]
    pub pump_amm: Pubkey,
    #[borsh(skip)]
    pub pool_authority: Pubkey,
    #[borsh(skip)]
    pub pool_authority_mint_account: Pubkey,
    #[borsh(skip)]
    pub pool_authority_wsol_account: Pubkey,
    #[borsh(skip)]
    pub amm_global_config: Pubkey,
    #[borsh(skip)]
    pub wsol_mint: Pubkey,
    #[borsh(skip)]
    pub lp_mint: Pubkey,
    #[borsh(skip)]
    pub user_pool_token_account: Pubkey,
    #[borsh(skip)]
    pub pool_base_token_account: Pubkey,
    #[borsh(skip)]
    pub pool_quote_token_account: Pubkey,
    #[borsh(skip)]
    pub token_2022_program: Pubkey,
    #[borsh(skip)]
    pub associated_token_program: Pubkey,
    #[borsh(skip)]
    pub pump_amm_event_authority: Pubkey,
    #[borsh(skip)]
    pub event_authority: Pubkey,
    #[borsh(skip)]
    pub program: Pubkey,
}

pub const PUMPFUN_MIGRATE_EVENT_LOG_SIZE: usize = 160;

pub fn pumpfun_migrate_event_log_decode(data: &[u8]) -> Option<PumpFunMigrateEvent> {
    if data.len() < PUMPFUN_MIGRATE_EVENT_LOG_SIZE {
        return None;
    }
    borsh::from_slice::<PumpFunMigrateEvent>(&data[..PUMPFUN_MIGRATE_EVENT_LOG_SIZE]).ok()
}

/// Bonding curve
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PumpFunBondingCurveAccountEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub bonding_curve: BondingCurve,
}

/// Global config
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PumpFunGlobalAccountEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub global: Global,
}

/// Event discriminator constants
pub mod discriminators {
    // Event discriminators
    // pub const CREATE_TOKEN_EVENT: &str = "0xe445a52e51cb9a1d1b72a94ddeeb6376";
    pub const CREATE_TOKEN_EVENT: &[u8] =
        &[228, 69, 165, 46, 81, 203, 154, 29, 27, 114, 169, 77, 222, 235, 99, 118];
    // pub const TRADE_EVENT: &str = "0xe445a52e51cb9a1dbddb7fd34ee661ee";
    pub const TRADE_EVENT: &[u8] =
        &[228, 69, 165, 46, 81, 203, 154, 29, 189, 219, 127, 211, 78, 230, 97, 238];
    // pub const COMPLETE_PUMP_AMM_MIGRATION_EVENT: &str = "0xe445a52e51cb9a1dbde95db95c94ea94";
    pub const COMPLETE_PUMP_AMM_MIGRATION_EVENT: &[u8] =
        &[228, 69, 165, 46, 81, 203, 154, 29, 189, 233, 93, 185, 92, 148, 234, 148];

    // Instruction discriminators
    pub const CREATE_TOKEN_IX: &[u8] = &[24, 30, 200, 40, 5, 28, 7, 119];
    pub const CREATE_V2_TOKEN_IX: &[u8] = &[214, 144, 76, 236, 95, 139, 49, 180];
    pub const BUY_IX: &[u8] = &[102, 6, 61, 18, 1, 218, 235, 234];
    pub const BUY_EXACT_SOL_IN_IX: &[u8] = &[56, 252, 116, 8, 158, 223, 205, 95];
    pub const SELL_IX: &[u8] = &[51, 230, 133, 164, 1, 127, 131, 173];
    pub const MIGRATE_IX: &[u8] = &[155, 234, 231, 146, 236, 158, 162, 30];

    // Account discriminators
    pub const BONDING_CURVE_ACCOUNT: &[u8] = &[23, 183, 248, 55, 96, 216, 172, 96];
    pub const GLOBAL_ACCOUNT: &[u8] = &[167, 232, 232, 177, 200, 108, 114, 127];
}
