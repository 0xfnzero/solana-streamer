use crate::common::AnyResult;
use crate::streaming::common::{
    EventProcessor, MetricsManager, PerformanceMetrics, StreamClientConfig, SubscriptionHandle,
};
use crate::streaming::event_parser::common::filter::EventTypeFilter;
use crate::streaming::event_parser::{Protocol, UnifiedEvent};
use crate::streaming::grpc::pool::factory;
use crate::streaming::grpc::{EventPretty, SubscriptionManager};
use anyhow::anyhow;
use chrono::Local;
use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use log::error;
use solana_sdk::pubkey::Pubkey;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use tokio::sync::Mutex;
use yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof;
use yellowstone_grpc_proto::geyser::{
    CommitmentLevel, SubscribeRequest, SubscribeRequestFilterAccountsFilter, SubscribeRequestPing,
};

/// 交易过滤器
#[derive(Debug, Clone)]
pub struct TransactionFilter {
    pub account_include: Vec<String>,
    pub account_exclude: Vec<String>,
    pub account_required: Vec<String>,
}

/// 账户过滤器
#[derive(Debug, Clone)]
pub struct AccountFilter {
    pub account: Vec<String>,
    pub owner: Vec<String>,
    pub filters: Vec<SubscribeRequestFilterAccountsFilter>,
}

pub struct YellowstoneGrpc {
    pub endpoint: String,
    pub x_token: Option<String>,
    pub config: StreamClientConfig,
    pub metrics: Arc<RwLock<PerformanceMetrics>>,
    pub subscription_manager: SubscriptionManager,
    pub metrics_manager: MetricsManager,
    pub event_processor: EventProcessor,
    pub subscription_handle: Arc<Mutex<Option<SubscriptionHandle>>>,
    // Dynamic subscription management fields
    pub active_subscription: Arc<AtomicBool>,
    pub control_tx: Arc<tokio::sync::Mutex<Option<mpsc::Sender<SubscribeRequest>>>>,
    pub current_request: Arc<tokio::sync::RwLock<Option<SubscribeRequest>>>,

    pub event_type_filter: Arc<tokio::sync::RwLock<Option<EventTypeFilter>>>,
}

impl YellowstoneGrpc {
    /// 创建客户端，使用默认配置
    pub fn new(endpoint: String, x_token: Option<String>) -> AnyResult<Self> {
        Self::new_with_config(endpoint, x_token, StreamClientConfig::default())
    }

    /// 创建客户端，使用自定义配置
    pub fn new_with_config(
        endpoint: String,
        x_token: Option<String>,
        config: StreamClientConfig,
    ) -> AnyResult<Self> {
        let _ = rustls::crypto::ring::default_provider().install_default().ok();
        let metrics = Arc::new(RwLock::new(PerformanceMetrics::new()));

        let subscription_manager =
            SubscriptionManager::new(endpoint.clone(), x_token.clone(), config.clone());
        let metrics_manager = MetricsManager::new_with_metrics(
            metrics.clone(),
            config.enable_metrics,
            "YellowstoneGrpc".to_string(),
        );
        let event_processor = EventProcessor::new(metrics_manager.clone(), config.clone());

        Ok(Self {
            endpoint,
            x_token,
            config,
            metrics: metrics.clone(),
            subscription_manager,
            metrics_manager,
            event_processor,
            subscription_handle: Arc::new(Mutex::new(None)),
            active_subscription: Arc::new(AtomicBool::new(false)),
            control_tx: Arc::new(tokio::sync::Mutex::new(None)),
            current_request: Arc::new(tokio::sync::RwLock::new(None)),
            event_type_filter: Arc::new(tokio::sync::RwLock::new(None)),
        })
    }

    /// Creates a new YellowstoneGrpcClient with high-throughput configuration.
    ///
    /// This is a convenience method that creates a client optimized for high-concurrency scenarios
    /// where throughput is prioritized over latency. See `StreamClientConfig::high_throughput()`
    /// for detailed configuration information.
    pub fn new_high_throughput(endpoint: String, x_token: Option<String>) -> AnyResult<Self> {
        Self::new_with_config(endpoint, x_token, StreamClientConfig::high_throughput())
    }

    /// Creates a new YellowstoneGrpcClient with low-latency configuration.
    ///
    /// This is a convenience method that creates a client optimized for real-time scenarios
    /// where latency is prioritized over throughput. See `StreamClientConfig::low_latency()`
    /// for detailed configuration information.
    pub fn new_low_latency(endpoint: String, x_token: Option<String>) -> AnyResult<Self> {
        Self::new_with_config(endpoint, x_token, StreamClientConfig::low_latency())
    }

    /// 获取配置
    pub fn get_config(&self) -> &StreamClientConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: StreamClientConfig) {
        self.config = config;
    }

    /// 获取性能指标
    pub fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics_manager.get_metrics()
    }

    /// 打印性能指标
    pub fn print_metrics(&self) {
        self.metrics_manager.print_metrics();
    }

    /// 启用或禁用性能监控
    pub fn set_enable_metrics(&mut self, enabled: bool) {
        self.config.enable_metrics = enabled;
    }

    /// 停止当前订阅
    pub async fn stop(&self) {
        let mut handle_guard = self.subscription_handle.lock().await;
        if let Some(handle) = handle_guard.take() {
            handle.stop();
        }
        *self.control_tx.lock().await = None;
        *self.current_request.write().await = None;
        self.active_subscription.store(false, Ordering::Release);
    }

    /// Simplified immediate event subscription (recommended for simple scenarios)
    ///
    /// # Parameters
    /// * `protocols` - List of protocols to monitor
    /// * `bot_wallet` - Optional bot wallet address for filtering related transactions
    /// * `transaction_filter` - Transaction filter specifying accounts to include/exclude
    /// * `account_filter` - Account filter specifying accounts and owners to monitor
    /// * `event_filter` - Optional event filter for further event filtering, no filtering if None
    /// * `commitment` - Optional commitment level, defaults to Confirmed
    /// * `callback` - Event callback function that receives parsed unified events
    ///
    /// # Returns
    /// Returns `AnyResult<()>`, `Ok(())` on success, error information on failure
    pub async fn subscribe_events_immediate<F>(
        &self,
        protocols: Vec<Protocol>,
        bot_wallet: Option<Pubkey>,
        transaction_filter: Vec<TransactionFilter>,
        account_filter: Vec<AccountFilter>,
        event_type_filter: Option<EventTypeFilter>,
        commitment: Option<CommitmentLevel>,
        callback: F,
    ) -> AnyResult<()>
    where
        F: Fn(Box<dyn UnifiedEvent>) + Send + Sync + 'static,
    {
        *self.event_type_filter.write().await = event_type_filter.clone();
        if self
            .active_subscription
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return Err(anyhow!("Already subscribed. Use update_subscription() to modify filters"));
        }

        let mut metrics_handle = None;
        // 启动自动性能监控（如果启用）
        if self.config.enable_metrics {
            metrics_handle = self.metrics_manager.start_auto_monitoring().await;
        }

        let transactions = self
            .subscription_manager
            .get_subscribe_request_filter(transaction_filter, event_type_filter.as_ref());
        let accounts = self
            .subscription_manager
            .subscribe_with_account_request(account_filter, event_type_filter.as_ref());

        // 订阅事件
        let (mut subscribe_tx, mut stream, subscribe_request) = self
            .subscription_manager
            .subscribe_with_request(transactions, accounts, commitment, event_type_filter.as_ref())
            .await?;

        // 用 Arc<Mutex<>> 包装 subscribe_tx 以支持多线程共享
        let subscribe_tx = Arc::new(Mutex::new(subscribe_tx));
        *self.current_request.write().await = Some(subscribe_request);
        let (control_tx, mut control_rx) = mpsc::channel(100);
        *self.control_tx.lock().await = Some(control_tx);

        // 启动流处理任务
        let mut event_processor = self.event_processor.clone();
        event_processor.set_protocols_and_event_type_filter(
            super::common::EventSource::Grpc,
            protocols,
            event_type_filter,
            self.config.backpressure.clone(),
            Some(Arc::new(callback)),
        );
        let stream_handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    message = stream.next() => {
                        match message {
                            Some(Ok(msg)) => {
                                let created_at = msg.created_at;
                                match msg.update_oneof {
                                    Some(UpdateOneof::Account(account)) => {
                                        let account_pretty = factory::create_account_pretty_pooled(account);
                                        log::debug!("Received account: {:?}", account_pretty);
                                        if let Err(e) = event_processor
                                            .process_grpc_event_transaction_with_metrics(
                                                EventPretty::Account(account_pretty),
                                                bot_wallet,
                                            )
                                            .await
                                        {
                                            error!("Error processing account event: {e:?}");
                                        }
                                    }
                                    Some(UpdateOneof::BlockMeta(sut)) => {
                                        let block_meta_pretty = factory::create_block_meta_pretty_pooled(sut, created_at);
                                        log::debug!("Received block meta: {:?}", block_meta_pretty);
                                        if let Err(e) = event_processor
                                            .process_grpc_event_transaction_with_metrics(
                                                EventPretty::BlockMeta(block_meta_pretty),
                                                bot_wallet,
                                            )
                                            .await
                                        {
                                            error!("Error processing block meta event: {e:?}");
                                        }
                                    }
                                    Some(UpdateOneof::Transaction(sut)) => {
                                        let transaction_pretty = factory::create_transaction_pretty_pooled(sut, created_at);
                                        log::debug!(
                                            "Received transaction: {} at slot {}",
                                            transaction_pretty.signature,
                                            transaction_pretty.slot
                                        );
                                        if let Err(e) = event_processor
                                            .process_grpc_event_transaction_with_metrics(
                                                EventPretty::Transaction(transaction_pretty),
                                                bot_wallet,
                                            )
                                            .await
                                        {
                                            error!("Error processing transaction event: {e:?}");
                                        }
                                    }
                                    Some(UpdateOneof::Ping(_)) => {
                                        // 只在需要时获取锁，并立即释放
                                        if let Ok(mut tx_guard) = subscribe_tx.try_lock() {
                                            let _ = tx_guard
                                                .send(SubscribeRequest {
                                                    ping: Some(SubscribeRequestPing { id: 1 }),
                                                    ..Default::default()
                                                })
                                                .await;
                                        }
                                        log::debug!("service is ping: {}", Local::now());
                                    }
                                    Some(UpdateOneof::Pong(_)) => {
                                        log::debug!("service is pong: {}", Local::now());
                                    }
                                    _ => {
                                        log::debug!("Received other message type");
                                    }
                                }
                            }
                            Some(Err(error)) => {
                                error!("Stream error: {error:?}");
                                break;
                            }
                            None => break,
                        }
                    }
                    Some(update) = control_rx.next() => {
                        if let Err(e) = subscribe_tx.lock().await.send(update).await {
                            error!("Failed to send subscription update: {}", e);
                            break;
                        }
                    }
                }
            }
        });

        // 保存订阅句柄
        let subscription_handle = SubscriptionHandle::new(stream_handle, None, metrics_handle);
        let mut handle_guard = self.subscription_handle.lock().await;
        *handle_guard = Some(subscription_handle);

        Ok(())
    }

    /// Update subscription filters at runtime without reconnection
    ///
    /// # Parameters
    /// * `transaction_filter` - New transaction filter to apply
    /// * `account_filter` - New account filter to apply
    ///
    /// # Returns
    /// Returns `AnyResult<()>` on success, error on failure
    pub async fn update_subscription(
        &self,
        transaction_filter: Vec<TransactionFilter>,
        account_filter: Vec<AccountFilter>,
    ) -> AnyResult<()> {
        let mut control_sender = {
            let control_guard = self.control_tx.lock().await;

            if !self.active_subscription.load(Ordering::Acquire) {
                return Err(anyhow!("No active subscription to update"));
            }

            control_guard
                .as_ref()
                .ok_or_else(|| anyhow!("No active subscription to update"))?
                .clone()
        };

        let mut request = self
            .current_request
            .read()
            .await
            .as_ref()
            .ok_or_else(|| anyhow!("No active subscription"))?
            .clone();

        request.transactions = self
            .subscription_manager
            .get_subscribe_request_filter(
                transaction_filter,
                self.event_type_filter.read().await.as_ref(),
            )
            .unwrap_or_default();

        request.accounts = self
            .subscription_manager
            .subscribe_with_account_request(
                account_filter,
                self.event_type_filter.read().await.as_ref(),
            )
            .unwrap_or_default();

        control_sender
            .send(request.clone())
            .await
            .map_err(|e| anyhow!("Failed to send update: {}", e))?;

        *self.current_request.write().await = Some(request);

        Ok(())
    }
}

// 实现 Clone trait 以支持模块间共享
impl Clone for YellowstoneGrpc {
    fn clone(&self) -> Self {
        Self {
            endpoint: self.endpoint.clone(),
            x_token: self.x_token.clone(),
            config: self.config.clone(),
            metrics: self.metrics.clone(),
            subscription_manager: self.subscription_manager.clone(),
            metrics_manager: self.metrics_manager.clone(),
            event_processor: self.event_processor.clone(),
            subscription_handle: self.subscription_handle.clone(), // 共享同一个 Arc<Mutex<>>
            active_subscription: self.active_subscription.clone(),
            control_tx: self.control_tx.clone(),
            event_type_filter: self.event_type_filter.clone(),
            current_request: self.current_request.clone(),
        }
    }
}
