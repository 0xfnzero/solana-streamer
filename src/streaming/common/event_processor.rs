use crate::common::AnyResult;
use crate::streaming::common::MetricsEventType;
use crate::streaming::event_parser::common::filter::EventTypeFilter;
use crate::streaming::event_parser::core::account_event_parser::AccountEventParser;
use crate::streaming::event_parser::core::common_event_parser::CommonEventParser;
use crate::streaming::event_parser::core::event_parser::EventParser;
use crate::streaming::event_parser::{core::traits::DexEvent, Protocol};
use crate::streaming::grpc::{EventPretty, MetricsManager};
use crate::streaming::shred::TransactionWithSlot;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;

/// High-performance Event processor using SegQueue for all strategies
pub struct EventProcessor {
    pub(crate) metrics_manager: MetricsManager,
    pub(crate) config: ClientConfig,
    pub(crate) parser_cache: OnceCell<Arc<EventParser>>,
    pub(crate) protocols: Vec<Protocol>,
    pub(crate) event_type_filter: Option<EventTypeFilter>,
    pub(crate) callback: Option<Arc<dyn Fn(Box<dyn UnifiedEvent>) + Send + Sync>>,
    pub(crate) enhanced_callback: Option<crate::streaming::event_parser::core::traits::EnhancedEventCallback>,
    pub(crate) backpressure_config: BackpressureConfig,
    pub(crate) grpc_queue: Arc<SegQueue<(EventPretty, Option<Pubkey>)>>,
    pub(crate) shred_queue: Arc<SegQueue<(TransactionWithSlot, Option<Pubkey>)>>,
    pub(crate) grpc_pending_count: Arc<AtomicUsize>,
    pub(crate) shred_pending_count: Arc<AtomicUsize>,
    pub(crate) processing_shutdown: Arc<AtomicBool>,
}

impl EventProcessor {
    pub fn new(metrics_manager: MetricsManager, config: ClientConfig) -> Self {
        let backpressure_config = config.backpressure.clone();
        let grpc_queue = Arc::new(SegQueue::new());
        let shred_queue = Arc::new(SegQueue::new());
        let grpc_pending_count = Arc::new(AtomicUsize::new(0));
        let shred_pending_count = Arc::new(AtomicUsize::new(0));
        let processing_shutdown = Arc::new(AtomicBool::new(false));

        Self {
            metrics_manager,
            config,
            parser_cache: OnceCell::new(),
            protocols: vec![],
            event_type_filter: None,
            backpressure_config,
            callback: None,
            enhanced_callback: None,
            grpc_queue,
            shred_queue,
            grpc_pending_count,
            shred_pending_count,
            processing_shutdown,
        }
    }

    pub fn set_protocols_and_event_type_filter(
        &mut self,
        source: EventSource,
        protocols: Vec<Protocol>,
        event_type_filter: Option<EventTypeFilter>,
        backpressure_config: BackpressureConfig,
        callback: Option<Arc<dyn Fn(Box<dyn UnifiedEvent>) + Send + Sync>>,
    ) {
        self.protocols = protocols;
        self.event_type_filter = event_type_filter;

        self.backpressure_config = backpressure_config;
        self.callback = callback;
        let protocols_ref = &self.protocols;
        let event_type_filter_ref = self.event_type_filter.as_ref();
        self.parser_cache.get_or_init(|| {
            Arc::new(EventParser::new(protocols_ref.clone(), event_type_filter_ref.cloned()))
        });

        if matches!(self.backpressure_config.strategy, BackpressureStrategy::Block) {
            self.start_block_processing_thread(source);
        }
    }

    pub fn set_protocols_and_event_type_filter_with_enhanced_callback(
        &mut self,
        source: EventSource,
        protocols: Vec<Protocol>,
        event_type_filter: Option<EventTypeFilter>,
        backpressure_config: BackpressureConfig,
        enhanced_callback: Option<crate::streaming::event_parser::core::traits::EnhancedEventCallback>,
    ) {
        self.protocols = protocols;
        self.event_type_filter = event_type_filter;
        self.backpressure_config = backpressure_config;
        self.enhanced_callback = enhanced_callback;
        self.callback = None; // Clear standard callback when using enhanced callback

        let protocols_ref = &self.protocols;
        let event_type_filter_ref = self.event_type_filter.as_ref();
        self.parser_cache.get_or_init(|| {
            Arc::new(EventParser::new(protocols_ref.clone(), event_type_filter_ref.cloned()))
        });

        if matches!(self.backpressure_config.strategy, BackpressureStrategy::Block) {
            self.start_block_processing_thread(source);
        }
    }

    pub fn get_parser(&self) -> Arc<EventParser> {
        self.parser_cache.get().unwrap().clone()
    }

    fn create_adapter_callback(&self) -> Arc<dyn Fn(Box<dyn UnifiedEvent>) + Send + Sync> {
        let callback = self.callback.clone().unwrap();
        let metrics_manager = self.metrics_manager.clone();

        Arc::new(move |event: Box<dyn UnifiedEvent>| {
            let processing_time_us = event.handle_us() as f64;
            callback(event);
            metrics_manager.update_metrics(MetricsEventType::Transaction, 1, processing_time_us);
        })
    }

    pub async fn process_grpc_event_transaction_with_metrics(
        &self,
        event_pretty: EventPretty,
        bot_wallet: Option<Pubkey>,
    ) -> AnyResult<()> {
        self.apply_backpressure_control(event_pretty, bot_wallet).await
    }

    async fn apply_backpressure_control(
        &self,
        event_pretty: EventPretty,
        bot_wallet: Option<Pubkey>,
    ) -> AnyResult<()> {
        match self.backpressure_config.strategy {
            BackpressureStrategy::Block => {
                loop {
                    let current_pending = self.grpc_pending_count.load(Ordering::Relaxed);
                    if current_pending < self.backpressure_config.permits {
                        self.grpc_queue.push((event_pretty, bot_wallet));
                        self.grpc_pending_count.fetch_add(1, Ordering::Relaxed);
                        break;
                    }

                    tokio::task::yield_now().await;
                }
                Ok(())
            }
            BackpressureStrategy::Drop => {
                let current_pending = self.grpc_pending_count.load(Ordering::Relaxed);
                if current_pending >= self.backpressure_config.permits {
                    self.metrics_manager.increment_dropped_events();
                    Ok(())
                } else {
                    self.grpc_pending_count.fetch_add(1, Ordering::Relaxed);
                    let processor = self.clone();
                    tokio::spawn(async move {
                        match processor
                            .process_grpc_event_transaction(event_pretty, bot_wallet)
                            .await
                        {
                            Ok(_) => {
                                processor.grpc_pending_count.fetch_sub(1, Ordering::Relaxed);
                            }
                            Err(e) => {
                                log::error!("Error in async gRPC processing: {}", e);
                            }
                        }
                    });
                    Ok(())
                }
            }
        }
    }

    async fn process_grpc_event_transaction(
        &self,
        event_pretty: EventPretty,
        bot_wallet: Option<Pubkey>,
    ) -> AnyResult<()> {
        if self.callback.is_none() {
            return Ok(());
        }
        match event_pretty {
            EventPretty::Account(account_pretty) => {
                self.metrics_manager.add_account_process_count();
                let account_event = AccountEventParser::parse_account_event(
                    &self.protocols,
                    account_pretty,
                    self.event_type_filter.as_ref(),
                );
                if let Some(event) = account_event {
                    let processing_time_us = event.handle_us() as f64;
                    self.invoke_callback(event);
                    self.update_metrics(MetricsEventType::Account, 1, processing_time_us);
                }
            }
            EventPretty::Transaction(transaction_pretty) => {
                self.metrics_manager.add_tx_process_count();
                let slot = transaction_pretty.slot;
                let signature = transaction_pretty.signature;
                let block_time = transaction_pretty.block_time;
                let recv_us = transaction_pretty.recv_us;
                let transaction_index = transaction_pretty.transaction_index;
                let grpc_tx = transaction_pretty.grpc_tx;

                let parser = self.get_parser();

                // Use enhanced callback if available, otherwise use standard callback
                if let Some(enhanced_callback) = &self.enhanced_callback {
                    let enhanced_callback_clone = enhanced_callback.clone();
                    parser
                        .parse_grpc_transaction_owned_with_raw_data(
                            grpc_tx,
                            signature,
                            Some(slot),
                            block_time,
                            recv_us,
                            bot_wallet,
                            transaction_index,
                            enhanced_callback_clone,
                        )
                        .await?;
                } else {
                    let adapter_callback = self.create_adapter_callback();
                    parser
                        .parse_grpc_transaction_owned(
                            grpc_tx,
                            signature,
                            Some(slot),
                            block_time,
                            recv_us,
                            bot_wallet,
                            transaction_index,
                            adapter_callback,
                        )
                        .await?;
                }
            }
            EventPretty::BlockMeta(block_meta_pretty) => {
                self.metrics_manager.add_block_meta_process_count();
                let block_time_ms = block_meta_pretty
                    .block_time
                    .map(|ts| ts.seconds * 1000 + ts.nanos as i64 / 1_000_000)
                    .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());
                let block_meta_event = CommonEventParser::generate_block_meta_event(
                    block_meta_pretty.slot,
                    block_meta_pretty.block_hash,
                    block_time_ms,
                    block_meta_pretty.recv_us,
                );
                let processing_time_us = block_meta_event.handle_us() as f64;
                self.invoke_callback(block_meta_event);
                self.update_metrics(MetricsEventType::BlockMeta, 1, processing_time_us);
            }
        }
        EventPretty::Transaction(transaction_pretty) => {
            MetricsManager::global().add_tx_process_count();

            let slot = transaction_pretty.slot;
            let signature = transaction_pretty.signature;
            let block_time = transaction_pretty.block_time;
            let recv_us = transaction_pretty.recv_us;
            let transaction_index = transaction_pretty.transaction_index;
            let grpc_tx = transaction_pretty.grpc_tx;

            let adapter_callback = create_metrics_callback(callback.clone());

            EventParser::parse_grpc_transaction_owned(
                protocols,
                event_type_filter,
                grpc_tx,
                signature,
                Some(slot),
                block_time,
                recv_us,
                bot_wallet,
                transaction_index,
                adapter_callback,
            )
            .await?;
        }
        EventPretty::BlockMeta(block_meta_pretty) => {
            MetricsManager::global().add_block_meta_process_count();

            let block_time_ms = block_meta_pretty
                .block_time
                .map(|ts| ts.seconds * 1000 + ts.nanos as i64 / 1_000_000)
                .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

            let block_meta_event = CommonEventParser::generate_block_meta_event(
                block_meta_pretty.slot,
                block_meta_pretty.block_hash,
                block_time_ms,
                block_meta_pretty.recv_us,
            );

            let processing_time_us = block_meta_event.metadata().handle_us as f64;
            callback(block_meta_event);
            update_metrics(MetricsEventType::BlockMeta, 1, processing_time_us);
        }
    }

    Ok(())
}

impl Clone for EventProcessor {
    fn clone(&self) -> Self {
        Self {
            metrics_manager: self.metrics_manager.clone(),
            config: self.config.clone(),
            parser_cache: self.parser_cache.clone(),
            protocols: self.protocols.clone(),
            event_type_filter: self.event_type_filter.clone(),
            backpressure_config: self.backpressure_config.clone(),
            callback: self.callback.clone(),
            enhanced_callback: self.enhanced_callback.clone(),
            grpc_queue: self.grpc_queue.clone(),
            shred_queue: self.shred_queue.clone(),
            grpc_pending_count: self.grpc_pending_count.clone(),
            shred_pending_count: self.shred_pending_count.clone(),
            processing_shutdown: self.processing_shutdown.clone(),
        }
    }
}
