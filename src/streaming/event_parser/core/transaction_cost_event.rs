use crate::streaming::event_parser::common::{EventMetadata, EventType, ProtocolType};
use serde::{Deserialize, Serialize};
use sol_parser_sdk::TransactionCost;
pub use sol_parser_sdk::{SwqosProvider, TipPayment};
use solana_sdk::{pubkey::Pubkey, signature::Signature};

/// Opt-in transaction fee, compute-budget, and SWQoS tip details.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionCostEvent {
    pub metadata: EventMetadata,
    pub transaction_fee_lamports: Option<u64>,
    pub total_fee_and_tip_lamports: Option<u64>,
    pub compute_units_consumed: Option<u64>,
    pub compute_unit_limit: Option<u32>,
    pub compute_unit_price_micro_lamports: Option<u64>,
    pub priority_fee_lamports: Option<u64>,
    pub tip_payments_confirmed: bool,
    pub tip_lamports: u64,
    pub tip_payments: Vec<TipPayment>,
}

impl TransactionCostEvent {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_parser(
        cost: TransactionCost,
        signature: Signature,
        slot: u64,
        tx_index: Option<u64>,
        block_time_us: Option<i64>,
        recv_us: i64,
        recent_blockhash: Option<String>,
    ) -> Self {
        let block_time_us = block_time_us.unwrap_or_default();
        Self {
            metadata: EventMetadata::new(
                signature,
                slot,
                block_time_us / 1_000_000,
                block_time_us / 1_000,
                ProtocolType::Common,
                EventType::TransactionCost,
                Pubkey::default(),
                0,
                None,
                recv_us,
                tx_index,
                recent_blockhash,
            ),
            transaction_fee_lamports: cost.transaction_fee_lamports,
            total_fee_and_tip_lamports: cost.total_fee_and_tip_lamports,
            compute_units_consumed: cost.compute_units_consumed,
            compute_unit_limit: cost.compute_unit_limit,
            compute_unit_price_micro_lamports: cost.compute_unit_price_micro_lamports,
            priority_fee_lamports: cost.priority_fee_lamports,
            tip_payments_confirmed: cost.tip_payments_confirmed,
            tip_lamports: cost.tip_lamports,
            tip_payments: cost.tip_payments,
        }
    }

    #[inline]
    pub fn tip_lamports_for(&self, provider: SwqosProvider) -> u64 {
        self.tip_payments
            .iter()
            .filter(|payment| payment.provider == provider)
            .fold(0u64, |total, payment| total.saturating_add(payment.lamports))
    }
}
