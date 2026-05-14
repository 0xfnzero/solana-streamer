//! Top-level ix parsing strategy for the gRPC path.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GrpcIxParseMode {
    /// Parse all subscribed instructions, used when transaction meta is missing.
    Full,
    /// Parse only ComputeBudget locally; DEX events come from sol-parser-sdk.
    ComputeBudgetOnly,
}
