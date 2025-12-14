use crate::streaming::event_parser::protocols::meteora_dlmm::discriminators;
use solana_sdk::pubkey::Pubkey;

/// Meteora DLMM 程序ID
pub const METEORA_DLMM_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");

/// 解析 Meteora DLMM 账户数据
///
/// 根据判别器路由到具体的账户解析函数
pub fn parse_meteora_dlmm_account_data(
    discriminator: &[u8],
    account: &crate::streaming::grpc::AccountPretty,
    metadata: crate::streaming::event_parser::common::EventMetadata,
) -> Option<crate::streaming::event_parser::DexEvent> {
    match discriminator {
        discriminators::LB_PAIR => {
            crate::streaming::event_parser::protocols::meteora_dlmm::types::lb_pair_parser(
                account,
                metadata,
            )
        }
        _ => None,
    }
}
