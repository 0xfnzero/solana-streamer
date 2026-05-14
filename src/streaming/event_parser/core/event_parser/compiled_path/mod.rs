//! Standard [`VersionedTransaction`] / [`CompiledInstruction`] path for RPC and replay.
//!
//! | Module | Responsibility |
//! |--------|------|
//! | [`compiled_transaction`] | top-level ix loop |
//! | [`compiled_instruction`] | single Solana `CompiledInstruction` |

mod compiled_instruction;
mod compiled_transaction;

pub(super) use compiled_transaction::parse_instruction_events_from_versioned_transaction;
