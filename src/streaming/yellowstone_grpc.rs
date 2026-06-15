use crate::common::AnyResult;
use crate::streaming::common::{
    parse_grpc_transaction_events, process_grpc_transaction, transaction_metrics_callback,
    MetricsManager, MicroBatchBuffer, PerformanceMetrics, SlotBuffer, StreamClientConfig,
    SubscriptionHandle,
};
use crate::streaming::event_parser::common::filter::EventTypeFilter;
use crate::streaming::event_parser::{DexEvent, Protocol};
use crate::streaming::grpc::pool::factory;
use crate::streaming::grpc::{EventPretty, SubscriptionManager, TransactionPretty};
use anyhow::anyhow;
use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use log::error;
use sol_parser_sdk::grpc::OrderMode;
use solana_sdk::pubkey::Pubkey;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};
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
    pub subscription_manager: SubscriptionManager,
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
        sol_parser_sdk::warmup_parser();
        let _ = rustls::crypto::ring::default_provider().install_default().ok();
        let subscription_manager =
            SubscriptionManager::new(endpoint.clone(), x_token.clone(), config.clone());
        MetricsManager::init(config.enable_metrics);

        Ok(Self {
            endpoint,
            x_token,
            config,
            subscription_manager,
            subscription_handle: Arc::new(Mutex::new(None)),
            active_subscription: Arc::new(AtomicBool::new(false)),
            control_tx: Arc::new(tokio::sync::Mutex::new(None)),
            current_request: Arc::new(tokio::sync::RwLock::new(None)),
            event_type_filter: Arc::new(tokio::sync::RwLock::new(None)),
        })
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
        MetricsManager::global().get_metrics()
    }

    /// 打印性能指标
    pub fn print_metrics(&self) {
        MetricsManager::global().print_metrics();
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
    #[allow(clippy::too_many_arguments)]
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
        F: Fn(DexEvent) + Send + Sync + 'static,
    {
        *self.event_type_filter.write().await = event_type_filter.clone();
        if self
            .active_subscription
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return Err(anyhow!("Already subscribed. Use update_subscription() to modify filters"));
        }

        let mut start_guard = SubscriptionStartGuard::new(self.active_subscription.clone());
        // 启动自动性能监控（如果启用）
        if self.config.enable_metrics {
            start_guard.set_metrics_handle(MetricsManager::global().start_auto_monitoring().await);
        }

        let transactions = self
            .subscription_manager
            .get_subscribe_request_filter(transaction_filter, event_type_filter.as_ref());
        let accounts = self
            .subscription_manager
            .subscribe_with_account_request(account_filter, event_type_filter.as_ref());

        // 订阅事件
        let (subscribe_tx, mut stream, subscribe_request) = self
            .subscription_manager
            .subscribe_with_request(transactions, accounts, commitment, event_type_filter.as_ref())
            .await?;

        // 用 Arc<Mutex<>> 包装 subscribe_tx 以支持多线程共享
        let subscribe_tx = Arc::new(Mutex::new(subscribe_tx));
        *self.current_request.write().await = Some(subscribe_request);
        let (control_tx, mut control_rx) = mpsc::channel(100);
        *self.control_tx.lock().await = Some(control_tx);

        // Wrap callback once before the async block
        let callback = Arc::new(callback);
        let transaction_callback = transaction_metrics_callback(callback.clone());
        let order_mode = self.config.order_mode;
        let order_timeout_ms = self.config.order_timeout_ms;
        let micro_batch_us = self.config.micro_batch_us;

        let active_subscription = self.active_subscription.clone();
        let control_tx_cleanup = self.control_tx.clone();
        let current_request_cleanup = self.current_request.clone();
        let metrics_handle = start_guard.disarm();
        let stream_handle = tokio::spawn(async move {
            let cleanup = SubscriptionCleanupGuard::new(
                active_subscription,
                control_tx_cleanup,
                current_request_cleanup,
                metrics_handle,
            );
            let mut slot_buffer = SlotBuffer::new();
            let mut micro_batch = MicroBatchBuffer::new();
            let mut last_slot = 0u64;
            let check_interval = match order_mode {
                OrderMode::MicroBatch => Duration::from_micros(micro_batch_us.max(1)),
                _ => Duration::from_millis((order_timeout_ms / 2).max(1)),
            };
            let mut next_check = Instant::now() + check_interval;

            loop {
                if has_buffered_events(order_mode, &slot_buffer, &micro_batch) {
                    flush_ordered_timeouts(
                        order_mode,
                        &mut slot_buffer,
                        &mut micro_batch,
                        transaction_callback.clone(),
                        order_timeout_ms,
                        micro_batch_us,
                        &mut next_check,
                        check_interval,
                    );
                }

                tokio::select! {
                    message = stream.next() => {
                        match message {
                            Some(Ok(msg)) => {
                                let created_at = msg.created_at;
                                match msg.update_oneof {
                                    Some(UpdateOneof::Account(account)) => {
                                        let Some(account_pretty) =
                                            factory::try_create_account_pretty_pooled(account)
                                        else {
                                            continue;
                                        };
                                        log::debug!("Received account: {:?}", account_pretty);
                                        if let Err(e) = process_grpc_transaction(
                                            EventPretty::Account(account_pretty),
                                            &protocols,
                                            event_type_filter.as_ref(),
                                            callback.clone(),
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
                                        if let Err(e) = process_grpc_transaction(
                                            EventPretty::BlockMeta(block_meta_pretty),
                                            &protocols,
                                            event_type_filter.as_ref(),
                                            callback.clone(),
                                            bot_wallet,
                                        )
                                        .await
                                        {
                                            error!("Error processing block meta event: {e:?}");
                                        }
                                    }
                                    Some(UpdateOneof::Transaction(sut)) => {
                                        let Some(transaction_pretty) =
                                            factory::try_create_transaction_pretty_pooled(sut, created_at)
                                        else {
                                            continue;
                                        };
                                        log::debug!(
                                            "Received transaction: {} at slot {}",
                                            transaction_pretty.signature,
                                            transaction_pretty.slot
                                        );
                                        handle_ordered_transaction(
                                            transaction_pretty,
                                            &protocols,
                                            event_type_filter.as_ref(),
                                            transaction_callback.clone(),
                                            bot_wallet,
                                            order_mode,
                                            &mut slot_buffer,
                                            &mut micro_batch,
                                            &mut last_slot,
                                            micro_batch_us,
                                        );
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
                                        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                                        log::debug!("service is ping: {}", ts);
                                    }
                                    Some(UpdateOneof::Pong(_)) => {
                                        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                                        log::debug!("service is pong: {}", ts);
                                    }
                                    _ => {
                                        log::debug!("Received other message type");
                                    }
                                }
                            }
                            Some(Err(error)) => {
                                error!("Stream error: {error:?}");
                                flush_ordered_on_disconnect(
                                    order_mode,
                                    &mut slot_buffer,
                                    &mut micro_batch,
                                    transaction_callback.clone(),
                                );
                                break;
                            }
                            None => {
                                flush_ordered_on_disconnect(
                                    order_mode,
                                    &mut slot_buffer,
                                    &mut micro_batch,
                                    transaction_callback.clone(),
                                );
                                break;
                            }
                        }
                    }
                    Some(update) = control_rx.next() => {
                        if let Err(e) = subscribe_tx.lock().await.send(update).await {
                            error!("Failed to send subscription update: {}", e);
                            flush_ordered_on_disconnect(
                                order_mode,
                                &mut slot_buffer,
                                &mut micro_batch,
                                transaction_callback.clone(),
                            );
                            break;
                        }
                    }
                    _ = tokio::time::sleep_until(next_check), if has_buffered_events(order_mode, &slot_buffer, &micro_batch) => {
                        flush_ordered_timeouts(
                            order_mode,
                            &mut slot_buffer,
                            &mut micro_batch,
                            transaction_callback.clone(),
                            order_timeout_ms,
                            micro_batch_us,
                            &mut next_check,
                            check_interval,
                        );
                    }
                }
            }
            cleanup.finish().await;
        });

        // 保存订阅句柄
        let subscription_handle = SubscriptionHandle::new(stream_handle, None, None);
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

#[allow(clippy::too_many_arguments)]
fn handle_ordered_transaction(
    transaction_pretty: TransactionPretty,
    protocols: &[Protocol],
    event_type_filter: Option<&EventTypeFilter>,
    callback: Arc<dyn Fn(DexEvent) + Send + Sync>,
    bot_wallet: Option<Pubkey>,
    order_mode: OrderMode,
    slot_buffer: &mut SlotBuffer,
    micro_batch: &mut MicroBatchBuffer,
    last_slot: &mut u64,
    micro_batch_us: u64,
) {
    let fallback_slot = transaction_pretty.slot;
    let fallback_tx_index = transaction_pretty.tx_index.unwrap_or(0);
    let recv_us = transaction_pretty.recv_us;
    let events =
        parse_grpc_transaction_events(transaction_pretty, protocols, event_type_filter, bot_wallet);

    match order_mode {
        OrderMode::Unordered => {
            for event in events {
                callback(event);
            }
        }
        OrderMode::Ordered => {
            if fallback_slot > *last_slot && *last_slot > 0 {
                deliver_events(callback.clone(), slot_buffer.flush_before(fallback_slot));
            }
            *last_slot = fallback_slot;
            for event in events {
                let (slot, tx_index) = event_order_key(&event, fallback_slot, fallback_tx_index);
                slot_buffer.push(slot, tx_index, event);
            }
        }
        OrderMode::StreamingOrdered => {
            // Events parsed from one transaction share its (slot, tx_index); push them as a
            // single group so the streaming watermark advances per transaction and no event of
            // a multi-event transaction is dropped.
            for ((slot, tx_index), group) in
                group_events_by_order_key(events, fallback_slot, fallback_tx_index)
            {
                deliver_events(callback.clone(), slot_buffer.push_streaming(slot, tx_index, group));
            }
        }
        OrderMode::MicroBatch => {
            for event in events {
                let (slot, tx_index) = event_order_key(&event, fallback_slot, fallback_tx_index);
                if micro_batch.push(slot, tx_index, event, recv_us, micro_batch_us) {
                    deliver_events(callback.clone(), micro_batch.flush());
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn flush_ordered_timeouts(
    order_mode: OrderMode,
    slot_buffer: &mut SlotBuffer,
    micro_batch: &mut MicroBatchBuffer,
    callback: Arc<dyn Fn(DexEvent) + Send + Sync>,
    order_timeout_ms: u64,
    micro_batch_us: u64,
    next_check: &mut Instant,
    check_interval: Duration,
) {
    if Instant::now() < *next_check {
        return;
    }
    *next_check = Instant::now() + check_interval;

    match order_mode {
        OrderMode::Ordered => {
            if slot_buffer.should_timeout(order_timeout_ms) {
                deliver_events(callback, slot_buffer.flush_all());
            }
        }
        OrderMode::StreamingOrdered => {
            if slot_buffer.should_timeout(order_timeout_ms) {
                deliver_events(callback, slot_buffer.flush_streaming_timeout());
            }
        }
        OrderMode::MicroBatch => {
            let now_us =
                crate::streaming::event_parser::common::high_performance_clock::get_high_perf_clock(
                );
            if micro_batch.should_flush(now_us, micro_batch_us) {
                deliver_events(callback, micro_batch.flush());
            }
        }
        OrderMode::Unordered => {}
    }
}

fn flush_ordered_on_disconnect(
    order_mode: OrderMode,
    slot_buffer: &mut SlotBuffer,
    micro_batch: &mut MicroBatchBuffer,
    callback: Arc<dyn Fn(DexEvent) + Send + Sync>,
) {
    match order_mode {
        OrderMode::Ordered => deliver_events(callback, slot_buffer.flush_all()),
        OrderMode::StreamingOrdered => {
            deliver_events(callback, slot_buffer.flush_streaming_timeout())
        }
        OrderMode::MicroBatch => deliver_events(callback, micro_batch.flush()),
        OrderMode::Unordered => {}
    }
}

#[inline]
fn event_order_key(event: &DexEvent, fallback_slot: u64, fallback_tx_index: u64) -> (u64, u64) {
    let metadata = event.metadata();
    (metadata.slot.max(fallback_slot), metadata.tx_index.unwrap_or(fallback_tx_index))
}

/// Group consecutive events that share the same `(slot, tx_index)` order key, preserving order.
/// Events of a single transaction carry the same key, so this normally yields one group, but it
/// stays correct if a transaction ever spans multiple keys.
fn group_events_by_order_key(
    events: Vec<DexEvent>,
    fallback_slot: u64,
    fallback_tx_index: u64,
) -> Vec<((u64, u64), Vec<DexEvent>)> {
    let mut groups: Vec<((u64, u64), Vec<DexEvent>)> = Vec::new();
    for event in events {
        let key = event_order_key(&event, fallback_slot, fallback_tx_index);
        match groups.last_mut() {
            Some((last_key, group)) if *last_key == key => group.push(event),
            _ => groups.push((key, vec![event])),
        }
    }
    groups
}

#[inline]
fn deliver_events(callback: Arc<dyn Fn(DexEvent) + Send + Sync>, events: Vec<DexEvent>) {
    for event in events {
        callback(event);
    }
}

#[inline]
fn has_buffered_events(
    order_mode: OrderMode,
    slot_buffer: &SlotBuffer,
    micro_batch: &MicroBatchBuffer,
) -> bool {
    match order_mode {
        OrderMode::Ordered | OrderMode::StreamingOrdered => !slot_buffer.is_empty(),
        OrderMode::MicroBatch => !micro_batch.is_empty(),
        OrderMode::Unordered => false,
    }
}

struct SubscriptionStartGuard {
    active_subscription: Arc<AtomicBool>,
    metrics_handle: Option<tokio::task::JoinHandle<()>>,
    disarmed: bool,
}

impl SubscriptionStartGuard {
    fn new(active_subscription: Arc<AtomicBool>) -> Self {
        Self { active_subscription, metrics_handle: None, disarmed: false }
    }

    fn set_metrics_handle(&mut self, metrics_handle: Option<tokio::task::JoinHandle<()>>) {
        self.metrics_handle = metrics_handle;
    }

    fn disarm(&mut self) -> Option<tokio::task::JoinHandle<()>> {
        self.disarmed = true;
        self.metrics_handle.take()
    }
}

impl Drop for SubscriptionStartGuard {
    fn drop(&mut self) {
        if !self.disarmed {
            self.active_subscription.store(false, Ordering::Release);
            if let Some(handle) = self.metrics_handle.take() {
                handle.abort();
            }
        }
    }
}

struct SubscriptionCleanupGuard {
    active_subscription: Arc<AtomicBool>,
    control_tx: Arc<tokio::sync::Mutex<Option<mpsc::Sender<SubscribeRequest>>>>,
    current_request: Arc<tokio::sync::RwLock<Option<SubscribeRequest>>>,
    metrics_handle: Option<tokio::task::JoinHandle<()>>,
    finished: bool,
}

impl SubscriptionCleanupGuard {
    fn new(
        active_subscription: Arc<AtomicBool>,
        control_tx: Arc<tokio::sync::Mutex<Option<mpsc::Sender<SubscribeRequest>>>>,
        current_request: Arc<tokio::sync::RwLock<Option<SubscribeRequest>>>,
        metrics_handle: Option<tokio::task::JoinHandle<()>>,
    ) -> Self {
        Self { active_subscription, control_tx, current_request, metrics_handle, finished: false }
    }

    async fn finish(mut self) {
        *self.control_tx.lock().await = None;
        *self.current_request.write().await = None;
        self.active_subscription.store(false, Ordering::Release);
        if let Some(handle) = self.metrics_handle.take() {
            handle.abort();
        }
        self.finished = true;
    }
}

impl Drop for SubscriptionCleanupGuard {
    fn drop(&mut self) {
        if !self.finished {
            self.active_subscription.store(false, Ordering::Release);
            if let Some(handle) = self.metrics_handle.take() {
                handle.abort();
            }
        }
    }
}

// 实现 Clone trait 以支持模块间共享
impl Clone for YellowstoneGrpc {
    fn clone(&self) -> Self {
        Self {
            endpoint: self.endpoint.clone(),
            x_token: self.x_token.clone(),
            config: self.config.clone(),
            subscription_manager: self.subscription_manager.clone(),
            subscription_handle: self.subscription_handle.clone(), // 共享同一个 Arc<Mutex<>>
            active_subscription: self.active_subscription.clone(),
            control_tx: self.control_tx.clone(),
            event_type_filter: self.event_type_filter.clone(),
            current_request: self.current_request.clone(),
        }
    }
}
