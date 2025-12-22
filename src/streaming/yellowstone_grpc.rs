use crate::common::AnyResult;
use crate::streaming::common::{
    process_grpc_transaction, MetricsManager, PerformanceMetrics, StreamClientConfig,
    SubscriptionHandle,
};
use crate::streaming::event_parser::common::filter::EventTypeFilter;
use crate::streaming::event_parser::{Protocol, DexEvent};
use crate::streaming::grpc::pool::factory;
use crate::streaming::grpc::{EventPretty, SubscriptionManager};
use anyhow::anyhow;
use chrono::Local;
use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use log::error;
use solana_sdk::pubkey::Pubkey;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof;
use yellowstone_grpc_proto::geyser::{
    CommitmentLevel, SubscribeRequest, SubscribeRequestFilterAccountsFilter, SubscribeRequestPing,
};

/// Transaction filter
#[derive(Debug, Clone)]
pub struct TransactionFilter {
    pub account_include: Vec<String>,
    pub account_exclude: Vec<String>,
    pub account_required: Vec<String>,
}

/// Account filter
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
    /// Create client with default configuration
    pub fn new(endpoint: String, x_token: Option<String>) -> AnyResult<Self> {
        Self::new_with_config(endpoint, x_token, StreamClientConfig::default())
    }

    /// Create client with custom configuration
    pub fn new_with_config(
        endpoint: String,
        x_token: Option<String>,
        config: StreamClientConfig,
    ) -> AnyResult<Self> {
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

    /// Get configuration
    pub fn get_config(&self) -> &StreamClientConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: StreamClientConfig) {
        self.config = config;
    }

    /// Get performance metrics
    pub fn get_metrics(&self) -> PerformanceMetrics {
        MetricsManager::global().get_metrics()
    }

    /// Print performance metrics
    pub fn print_metrics(&self) {
        MetricsManager::global().print_metrics();
    }

    /// Enable or disable performance monitoring
    pub fn set_enable_metrics(&mut self, enabled: bool) {
        self.config.enable_metrics = enabled;
    }

    /// Stop current subscription
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

        let mut metrics_handle = None;
        // Start automatic performance monitoring (if enabled)
        if self.config.enable_metrics {
            metrics_handle = MetricsManager::global().start_auto_monitoring().await;
        }

        let transactions = self
            .subscription_manager
            .get_subscribe_request_filter(transaction_filter, event_type_filter.as_ref());
        let accounts = self
            .subscription_manager
            .subscribe_with_account_request(account_filter, event_type_filter.as_ref());

        // Subscribe to events
        let (subscribe_tx, mut stream, subscribe_request) = self
            .subscription_manager
            .subscribe_with_request(transactions, accounts, commitment, event_type_filter.as_ref())
            .await?;

        // Wrap subscribe_tx with Arc<Mutex<>> to support multi-threaded sharing
        let subscribe_tx = Arc::new(Mutex::new(subscribe_tx));
        *self.current_request.write().await = Some(subscribe_request);
        let (control_tx, mut control_rx) = mpsc::channel(100);
        *self.control_tx.lock().await = Some(control_tx);

        // Wrap callback once before the async block
        let callback = Arc::new(callback);

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
                                        let transaction_pretty = factory::create_transaction_pretty_pooled(sut, created_at);
                                        log::debug!(
                                            "Received transaction: {} at slot {}",
                                            transaction_pretty.signature,
                                            transaction_pretty.slot
                                        );
                                        if let Err(e) = process_grpc_transaction(
                                            EventPretty::Transaction(transaction_pretty),
                                            &protocols,
                                            event_type_filter.as_ref(),
                                            callback.clone(),
                                            bot_wallet,
                                        )
                                        .await
                                        {
                                            error!("Error processing transaction event: {e:?}");
                                        }
                                    }
                                    Some(UpdateOneof::Ping(_)) => {
                                        // Only acquire lock when needed and release immediately
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

        // Save subscription handle
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

// Implement Clone trait to support sharing between modules
impl Clone for YellowstoneGrpc {
    fn clone(&self) -> Self {
        Self {
            endpoint: self.endpoint.clone(),
            x_token: self.x_token.clone(),
            config: self.config.clone(),
            subscription_manager: self.subscription_manager.clone(),
            subscription_handle: self.subscription_handle.clone(), // Share the same Arc<Mutex<>>
            active_subscription: self.active_subscription.clone(),
            control_tx: self.control_tx.clone(),
            event_type_filter: self.event_type_filter.clone(),
            current_request: self.current_request.clone(),
        }
    }
}
