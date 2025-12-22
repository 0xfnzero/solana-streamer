// Constants related to stream processing

// Default configuration constants
pub const DEFAULT_CONNECT_TIMEOUT: u64 = 10;
pub const DEFAULT_REQUEST_TIMEOUT: u64 = 60;
pub const DEFAULT_CHANNEL_SIZE: usize = 1000;
pub const DEFAULT_MAX_DECODING_MESSAGE_SIZE: usize = 1024 * 1024 * 10;

// Performance monitoring related constants
pub const DEFAULT_METRICS_WINDOW_SECONDS: u64 = 5;
pub const DEFAULT_METRICS_PRINT_INTERVAL_SECONDS: u64 = 10;
pub const SLOW_PROCESSING_THRESHOLD_US: f64 = 3000.0;

// gRPC latency monitoring
// Solana doesn't store milliseconds, so we use 500ms to calibrate for better approximation
pub const SOLANA_BLOCK_TIME_ADJUSTMENT_MS: i64 = 500;
// Default maximum latency threshold (milliseconds)
pub const MAX_LATENCY_THRESHOLD_MS: i64 = 1000;
