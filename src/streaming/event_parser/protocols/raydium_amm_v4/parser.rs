use crate::streaming::event_parser::{
    common::{read_u64_le, EventMetadata, EventType, ProtocolType},
    core::GenericEventParseConfig,
    protocols::raydium_amm_v4::{
        discriminators, RaydiumAmmV4DepositEvent, RaydiumAmmV4Initialize2Event,
        RaydiumAmmV4SwapEvent, RaydiumAmmV4WithdrawEvent, RaydiumAmmV4WithdrawPnlEvent,
    },
    DexEvent,
};
use solana_sdk::pubkey::Pubkey;

/// Raydium CPMM程序ID
pub const RAYDIUM_AMM_V4_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");

// 配置所有事件类型
pub const CONFIGS: &[GenericEventParseConfig] = &[
    GenericEventParseConfig {
        program_id: RAYDIUM_AMM_V4_PROGRAM_ID,
        protocol_type: ProtocolType::RaydiumAmmV4,
        inner_instruction_discriminator: &[],
        instruction_discriminator: discriminators::SWAP_BASE_IN,
        event_type: EventType::RaydiumAmmV4SwapBaseIn,
        inner_instruction_parser: None,
        instruction_parser: Some(parse_swap_base_input_instruction),
        requires_inner_instruction: false,
    },
    GenericEventParseConfig {
        program_id: RAYDIUM_AMM_V4_PROGRAM_ID,
        protocol_type: ProtocolType::RaydiumAmmV4,
        inner_instruction_discriminator: &[],
        instruction_discriminator: discriminators::SWAP_BASE_OUT,
        event_type: EventType::RaydiumAmmV4SwapBaseOut,
        inner_instruction_parser: None,
        instruction_parser: Some(parse_swap_base_output_instruction),
        requires_inner_instruction: false,
    },
    GenericEventParseConfig {
        program_id: RAYDIUM_AMM_V4_PROGRAM_ID,
        protocol_type: ProtocolType::RaydiumAmmV4,
        inner_instruction_discriminator: &[],
        instruction_discriminator: discriminators::DEPOSIT,
        event_type: EventType::RaydiumAmmV4Deposit,
        inner_instruction_parser: None,
        instruction_parser: Some(parse_deposit_instruction),
        requires_inner_instruction: false,
    },
    GenericEventParseConfig {
        program_id: RAYDIUM_AMM_V4_PROGRAM_ID,
        protocol_type: ProtocolType::RaydiumAmmV4,
        inner_instruction_discriminator: &[],
        instruction_discriminator: discriminators::INITIALIZE2,
        event_type: EventType::RaydiumAmmV4Initialize2,
        inner_instruction_parser: None,
        instruction_parser: Some(parse_initialize2_instruction),
        requires_inner_instruction: false,
    },
    GenericEventParseConfig {
        program_id: RAYDIUM_AMM_V4_PROGRAM_ID,
        protocol_type: ProtocolType::RaydiumAmmV4,
        inner_instruction_discriminator: &[],
        instruction_discriminator: discriminators::WITHDRAW,
        event_type: EventType::RaydiumAmmV4Withdraw,
        inner_instruction_parser: None,
        instruction_parser: Some(parse_withdraw_instruction),
        requires_inner_instruction: false,
    },
    GenericEventParseConfig {
        program_id: RAYDIUM_AMM_V4_PROGRAM_ID,
        protocol_type: ProtocolType::RaydiumAmmV4,
        inner_instruction_discriminator: &[],
        instruction_discriminator: discriminators::WITHDRAW_PNL,
        event_type: EventType::RaydiumAmmV4WithdrawPnl,
        inner_instruction_parser: None,
        instruction_parser: Some(parse_withdraw_pnl_instruction),
        requires_inner_instruction: false,
    },
];

/// 解析提现指令事件
fn parse_withdraw_pnl_instruction(
    _data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    if accounts.len() < 17 {
        return None;
    }

    Some(DexEvent::RaydiumAmmV4WithdrawPnlEvent(RaydiumAmmV4WithdrawPnlEvent {
        metadata,
        token_program: accounts[0],
        amm: accounts[1],
        amm_config: accounts[2],
        amm_authority: accounts[3],
        amm_open_orders: accounts[4],
        pool_coin_token_account: accounts[5],
        pool_pc_token_account: accounts[6],
        coin_pnl_token_account: accounts[7],
        pc_pnl_token_account: accounts[8],
        pnl_owner_account: accounts[9],
        amm_target_orders: accounts[10],
        serum_program: accounts[11],
        serum_market: accounts[12],
        serum_event_queue: accounts[13],
        serum_coin_vault_account: accounts[14],
        serum_pc_vault_account: accounts[15],
        serum_vault_signer: accounts[16],
    }))
}

/// 解析移除流动性指令事件
fn parse_withdraw_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    if data.len() < 8 || accounts.len() < 22 {
        return None;
    }
    let amount = read_u64_le(data, 0)?;

    Some(DexEvent::RaydiumAmmV4WithdrawEvent(RaydiumAmmV4WithdrawEvent {
        metadata,
        amount,

        token_program: accounts[0],
        amm: accounts[1],
        amm_authority: accounts[2],
        amm_open_orders: accounts[3],
        amm_target_orders: accounts[4],
        lp_mint_address: accounts[5],
        pool_coin_token_account: accounts[6],
        pool_pc_token_account: accounts[7],
        pool_withdraw_queue: accounts[8],
        pool_temp_lp_token_account: accounts[9],
        serum_program: accounts[10],
        serum_market: accounts[11],
        serum_coin_vault_account: accounts[12],
        serum_pc_vault_account: accounts[13],
        serum_vault_signer: accounts[14],
        user_lp_token_account: accounts[15],
        user_coin_token_account: accounts[16],
        user_pc_token_account: accounts[17],
        user_owner: accounts[18],
        serum_event_queue: accounts[19],
        serum_bids: accounts[20],
        serum_asks: accounts[21],
    }))
}

/// 解析初始化指令事件
fn parse_initialize2_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    if data.len() < 25 || accounts.len() < 21 {
        return None;
    }
    let nonce = data[0];
    let open_time = read_u64_le(data, 1)?;
    let init_pc_amount = read_u64_le(data, 9)?;
    let init_coin_amount = read_u64_le(data, 17)?;

    Some(DexEvent::RaydiumAmmV4Initialize2Event(RaydiumAmmV4Initialize2Event {
        metadata,
        nonce,
        open_time,
        init_pc_amount,
        init_coin_amount,

        token_program: accounts[0],
        spl_associated_token_account: accounts[1],
        system_program: accounts[2],
        rent: accounts[3],
        amm: accounts[4],
        amm_authority: accounts[5],
        amm_open_orders: accounts[6],
        lp_mint: accounts[7],
        coin_mint: accounts[8],
        pc_mint: accounts[9],
        pool_coin_token_account: accounts[10],
        pool_pc_token_account: accounts[11],
        pool_withdraw_queue: accounts[12],
        amm_target_orders: accounts[13],
        pool_temp_lp: accounts[14],
        serum_program: accounts[15],
        serum_market: accounts[16],
        user_wallet: accounts[17],
        user_token_coin: accounts[18],
        user_token_pc: accounts[19],
        user_lp_token_account: accounts[20],
    }))
}

/// 解析添加流动性指令事件
fn parse_deposit_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    if data.len() < 24 || accounts.len() < 14 {
        return None;
    }
    let max_coin_amount = read_u64_le(data, 0)?;
    let max_pc_amount = read_u64_le(data, 8)?;
    let base_side = read_u64_le(data, 16)?;

    Some(DexEvent::RaydiumAmmV4DepositEvent(RaydiumAmmV4DepositEvent {
        metadata,
        max_coin_amount,
        max_pc_amount,
        base_side,

        token_program: accounts[0],
        amm: accounts[1],
        amm_authority: accounts[2],
        amm_open_orders: accounts[3],
        amm_target_orders: accounts[4],
        lp_mint_address: accounts[5],
        pool_coin_token_account: accounts[6],
        pool_pc_token_account: accounts[7],
        serum_market: accounts[8],
        user_coin_token_account: accounts[9],
        user_pc_token_account: accounts[10],
        user_lp_token_account: accounts[11],
        user_owner: accounts[12],
        serum_event_queue: accounts[13],
    }))
}

/// 解析买入指令事件
fn parse_swap_base_output_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    if data.len() < 16 || accounts.len() < 17 {
        return None;
    }
    let max_amount_in = read_u64_le(data, 0)?;
    let amount_out = read_u64_le(data, 8)?;

    let mut accounts = accounts.to_vec();
    if accounts.len() == 17 {
        // 添加一个默认的 Pubkey 作为 amm_target_orders 的占位符
        // 因为在某些情况下，amm_target_orders 可能是可选的
        accounts.insert(4, Pubkey::default());
    }

    Some(DexEvent::RaydiumAmmV4SwapEvent(RaydiumAmmV4SwapEvent {
        metadata,
        max_amount_in,
        amount_out,

        token_program: accounts[0],
        amm: accounts[1],
        amm_authority: accounts[2],
        amm_open_orders: accounts[3],
        amm_target_orders: Some(accounts[4]),
        pool_coin_token_account: accounts[5],
        pool_pc_token_account: accounts[6],
        serum_program: accounts[7],
        serum_market: accounts[8],
        serum_bids: accounts[9],
        serum_asks: accounts[10],
        serum_event_queue: accounts[11],
        serum_coin_vault_account: accounts[12],
        serum_pc_vault_account: accounts[13],
        serum_vault_signer: accounts[14],
        user_source_token_account: accounts[15],
        user_destination_token_account: accounts[16],
        user_source_owner: accounts[17],

        ..Default::default()
    }))
}

/// 解析买入指令事件
fn parse_swap_base_input_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<DexEvent> {
    if data.len() < 16 || accounts.len() < 17 {
        return None;
    }
    let amount_in = read_u64_le(data, 0)?;
    let minimum_amount_out = read_u64_le(data, 8)?;

    let mut accounts = accounts.to_vec();
    if accounts.len() == 17 {
        // 添加一个默认的 Pubkey 作为 amm_target_orders 的占位符
        // 因为在某些情况下，amm_target_orders 可能是可选的
        accounts.insert(4, Pubkey::default());
    }

    Some(DexEvent::RaydiumAmmV4SwapEvent(RaydiumAmmV4SwapEvent {
        metadata,
        amount_in,
        minimum_amount_out,

        token_program: accounts[0],
        amm: accounts[1],
        amm_authority: accounts[2],
        amm_open_orders: accounts[3],
        amm_target_orders: Some(accounts[4]),
        pool_coin_token_account: accounts[5],
        pool_pc_token_account: accounts[6],
        serum_program: accounts[7],
        serum_market: accounts[8],
        serum_bids: accounts[9],
        serum_asks: accounts[10],
        serum_event_queue: accounts[11],
        serum_coin_vault_account: accounts[12],
        serum_pc_vault_account: accounts[13],
        serum_vault_signer: accounts[14],
        user_source_token_account: accounts[15],
        user_destination_token_account: accounts[16],
        user_source_owner: accounts[17],

        ..Default::default()
    }))
}
