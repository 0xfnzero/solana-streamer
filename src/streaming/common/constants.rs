// 流处理相关的常量定义

// 默认配置常量
pub const DEFAULT_CONNECT_TIMEOUT: u64 = 10;
pub const DEFAULT_REQUEST_TIMEOUT: u64 = 60;
pub const DEFAULT_CHANNEL_SIZE: usize = 1000;
pub const DEFAULT_MAX_DECODING_MESSAGE_SIZE: usize = 1024 * 1024 * 10;

// 性能监控相关常量
pub const DEFAULT_METRICS_WINDOW_SECONDS: u64 = 5;
pub const DEFAULT_METRICS_PRINT_INTERVAL_SECONDS: u64 = 10;
pub const SLOW_PROCESSING_THRESHOLD_US: f64 = 3000.0;

// gRPC 延迟监控
// Solana 不存储毫秒，所以我们用500ms来校准以获得更好的近似值
pub const SOLANA_BLOCK_TIME_ADJUSTMENT_MS: i64 = 500;
// 默认最大延迟阈值（毫秒）
pub const MAX_LATENCY_THRESHOLD_MS: i64 = 1000;
