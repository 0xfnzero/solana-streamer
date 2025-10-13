use super::constants::*;

/// Connection configuration
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// Connection timeout in seconds (default: 10)
    pub connect_timeout: u64,
    /// Request timeout in seconds (default: 60)
    pub request_timeout: u64,
    /// Maximum decoding message size in bytes (default: 10MB)
    pub max_decoding_message_size: usize,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
            request_timeout: DEFAULT_REQUEST_TIMEOUT,
            max_decoding_message_size: DEFAULT_MAX_DECODING_MESSAGE_SIZE,
        }
    }
}

/// Common client configuration
#[derive(Debug, Clone)]
pub struct StreamClientConfig {
    /// Connection configuration
    pub connection: ConnectionConfig,
    /// Whether performance monitoring is enabled (default: false)
    pub enable_metrics: bool,
}

impl Default for StreamClientConfig {
    fn default() -> Self {
        Self { connection: ConnectionConfig::default(), enable_metrics: false }
    }
}
