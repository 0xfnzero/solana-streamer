use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Channel;

use crate::common::AnyResult;
use crate::protos::shredstream::shredstream_proxy_client::ShredstreamProxyClient;
use crate::streaming::common::{
    MetricsManager, PerformanceMetrics, StreamClientConfig, SubscriptionHandle,
};

/// ShredStream gRPC 客户端
#[derive(Clone)]
pub struct ShredStreamGrpc {
    pub shredstream_client: Arc<ShredstreamProxyClient<Channel>>,
    pub sdk_shredstream_client: Arc<sol_parser_sdk::shredstream::ShredStreamClient>,
    pub config: StreamClientConfig,
    pub subscription_handle: Arc<Mutex<Option<SubscriptionHandle>>>,
}

impl ShredStreamGrpc {
    /// 创建客户端，使用默认配置
    pub async fn new(endpoint: String) -> AnyResult<Self> {
        Self::new_with_config(endpoint, StreamClientConfig::default()).await
    }

    /// 创建客户端，使用自定义配置
    pub async fn new_with_config(endpoint: String, config: StreamClientConfig) -> AnyResult<Self> {
        let shredstream_client = ShredstreamProxyClient::connect(endpoint.clone()).await?;
        let sdk_config = sol_parser_sdk::shredstream::ShredStreamConfig {
            connection_timeout_ms: config.connection.connect_timeout.saturating_mul(1000),
            request_timeout_ms: config.connection.request_timeout.saturating_mul(1000),
            max_decoding_message_size: config.connection.max_decoding_message_size,
            ..Default::default()
        };
        let sdk_shredstream_client =
            sol_parser_sdk::shredstream::ShredStreamClient::new_with_config(endpoint, sdk_config)
                .await
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        MetricsManager::init(config.enable_metrics);
        Ok(Self {
            shredstream_client: Arc::new(shredstream_client),
            sdk_shredstream_client: Arc::new(sdk_shredstream_client),
            config,
            subscription_handle: Arc::new(Mutex::new(None)),
        })
    }

    /// 获取当前配置
    pub fn get_config(&self) -> &StreamClientConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: StreamClientConfig) {
        self.config = config;
    }

    /// 获取性能指标
    pub fn get_metrics(&self) -> PerformanceMetrics {
        MetricsManager::global().get_metrics()
    }

    /// 启用或禁用性能监控
    pub fn set_enable_metrics(&mut self, enabled: bool) {
        self.config.enable_metrics = enabled;
    }

    /// 打印性能指标
    pub fn print_metrics(&self) {
        MetricsManager::global().print_metrics();
    }

    /// 启动自动性能监控任务
    pub async fn start_auto_metrics_monitoring(&self) {
        MetricsManager::global().start_auto_monitoring().await;
    }

    /// 停止当前订阅
    pub async fn stop(&self) {
        self.sdk_shredstream_client.stop().await;
        let mut handle_guard = self.subscription_handle.lock().await;
        if let Some(handle) = handle_guard.take() {
            handle.stop();
        }
    }
}
