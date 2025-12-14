use crate::streaming::event_parser::protocols::whirlpool::discriminators;
use solana_sdk::pubkey::Pubkey;

/// Whirlpool 程序ID
pub const WHIRLPOOL_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc");

/// 解析 Whirlpool 账户数据
///
/// 根据判别器路由到具体的账户解析函数
pub fn parse_whirlpool_account_data(
    discriminator: &[u8],
    account: &crate::streaming::grpc::AccountPretty,
    metadata: crate::streaming::event_parser::common::EventMetadata,
) -> Option<crate::streaming::event_parser::DexEvent> {
    match discriminator {
        discriminators::WHIRLPOOL => {
            crate::streaming::event_parser::protocols::whirlpool::types::whirlpool_parser(
                account,
                metadata,
            )
        }
        _ => None,
    }
}
