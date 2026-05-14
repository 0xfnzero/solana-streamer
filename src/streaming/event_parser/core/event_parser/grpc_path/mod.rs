//! Yellowstone gRPC path for `SubscribeUpdateTransactionInfo` and SDK aggregate parsing.
//!
//! | Module | Responsibility |
//! |--------|------|
//! | [`grpc_ix_mode`] | `GrpcIxParseMode` |
//! | [`grpc_transaction`] | whole subscription message and top-level ix loop |
//! | [`grpc_instruction`] | single Yellowstone `CompiledInstruction` |

mod grpc_instruction;
mod grpc_ix_mode;
mod grpc_transaction;

pub(super) use grpc_transaction::parse_grpc_transaction;
