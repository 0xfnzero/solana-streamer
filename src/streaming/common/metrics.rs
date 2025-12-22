use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use super::constants::*;

/// Event type enumeration
#[derive(Debug, Clone, Copy)]
pub enum EventType {
    Transaction = 0,
    Account = 1,
    BlockMeta = 2,
}

/// Compatibility alias
pub type MetricsEventType = EventType;

impl EventType {
    #[inline]
    const fn as_index(self) -> usize {
        self as usize
    }

    const fn name(self) -> &'static str {
        match self {
            EventType::Transaction => "TX",
            EventType::Account => "Account",
            EventType::BlockMeta => "Block Meta",
        }
    }

    // Compatibility constants
    pub const TX: EventType = EventType::Transaction;
}

/// High-performance atomic event metrics
#[derive(Debug)]
struct AtomicEventMetrics {
    process_count: AtomicU64,
    events_processed: AtomicU64,
    events_in_window: AtomicU64,
    window_start_nanos: AtomicU64,
    // Processing time statistics per event type
    processing_stats: AtomicProcessingTimeStats,
}

impl AtomicEventMetrics {
    const fn new_const() -> Self {
        Self {
            process_count: AtomicU64::new(0),
            events_processed: AtomicU64::new(0),
            events_in_window: AtomicU64::new(0),
            window_start_nanos: AtomicU64::new(0),
            processing_stats: AtomicProcessingTimeStats::new_const(),
        }
    }

    /// Atomically increment process count
    #[inline]
    fn add_process_count(&self) {
        self.process_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Atomically increment event processing count
    #[inline]
    fn add_events_processed(&self, count: u64) {
        self.events_processed.fetch_add(count, Ordering::Relaxed);
        self.events_in_window.fetch_add(count, Ordering::Relaxed);
    }

    /// Get current count (non-blocking)
    #[inline]
    fn get_counts(&self) -> (u64, u64, u64) {
        (
            self.process_count.load(Ordering::Relaxed),
            self.events_processed.load(Ordering::Relaxed),
            self.events_in_window.load(Ordering::Relaxed),
        )
    }

    /// Reset window count
    #[inline]
    fn reset_window(&self, new_start_nanos: u64) {
        self.events_in_window.store(0, Ordering::Relaxed);
        self.window_start_nanos.store(new_start_nanos, Ordering::Relaxed);
    }

    #[inline]
    fn get_window_start(&self) -> u64 {
        self.window_start_nanos.load(Ordering::Relaxed)
    }

    /// Get processing time statistics for this event type
    #[inline]
    fn get_processing_stats(&self) -> ProcessingTimeStats {
        self.processing_stats.get_stats()
    }

    /// Update processing time statistics for this event type
    #[inline]
    fn update_processing_stats(&self, time_us: f64, event_count: u64) {
        self.processing_stats.update(time_us, event_count);
    }
}

/// High-performance atomic processing time statistics
#[derive(Debug)]
struct AtomicProcessingTimeStats {
    last_time_bits: AtomicU64, // Last processing time (f64 as u64 bits)
    total_time_us: AtomicU64,  // Store integer part of microseconds
    total_events: AtomicU64,
}

impl AtomicProcessingTimeStats {
    const fn new_const() -> Self {
        Self {
            last_time_bits: AtomicU64::new(0),
            total_time_us: AtomicU64::new(0),
            total_events: AtomicU64::new(0),
        }
    }

    /// Atomically update processing time statistics (hot path - no syscalls)
    #[inline]
    fn update(&self, time_us: f64, event_count: u64) {
        let time_bits = time_us.to_bits();

        // Update last processing time (simple store, no compare-exchange needed)
        self.last_time_bits.store(time_bits, Ordering::Relaxed);

        // Update cumulative values (convert microseconds to integers to avoid floating point accumulation issues)
        let total_time_us_int = (time_us * event_count as f64) as u64;
        self.total_time_us.fetch_add(total_time_us_int, Ordering::Relaxed);
        self.total_events.fetch_add(event_count, Ordering::Relaxed);
    }

    /// Get statistics (non-blocking)
    #[inline]
    fn get_stats(&self) -> ProcessingTimeStats {
        let last_bits = self.last_time_bits.load(Ordering::Relaxed);
        let total_time_us_int = self.total_time_us.load(Ordering::Relaxed);
        let total_events = self.total_events.load(Ordering::Relaxed);

        let last_time = f64::from_bits(last_bits);
        let avg_time =
            if total_events > 0 { total_time_us_int as f64 / total_events as f64 } else { 0.0 };

        ProcessingTimeStats { last_us: last_time, avg_us: avg_time }
    }
}

/// Processing time statistics result
#[derive(Debug, Clone)]
pub struct ProcessingTimeStats {
    pub last_us: f64, // Last processing time in microseconds
    pub avg_us: f64,  // Average processing time in microseconds
}

/// Event metrics snapshot
#[derive(Debug, Clone)]
pub struct EventMetricsSnapshot {
    pub process_count: u64,
    pub events_processed: u64,
    pub processing_stats: ProcessingTimeStats,
}

/// Compatibility structure - complete performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub uptime: std::time::Duration,
    pub tx_metrics: EventMetricsSnapshot,
    pub account_metrics: EventMetricsSnapshot,
    pub block_meta_metrics: EventMetricsSnapshot,
    pub processing_stats: ProcessingTimeStats,
    pub dropped_events_count: u64,
}

impl PerformanceMetrics {
    /// Create default performance metrics (compatibility method)
    pub fn new() -> Self {
        let default_stats = ProcessingTimeStats { last_us: 0.0, avg_us: 0.0 };
        let default_metrics = EventMetricsSnapshot {
            process_count: 0,
            events_processed: 0,
            processing_stats: default_stats.clone(),
        };

        Self {
            uptime: std::time::Duration::ZERO,
            tx_metrics: default_metrics.clone(),
            account_metrics: default_metrics.clone(),
            block_meta_metrics: default_metrics,
            processing_stats: default_stats,
            dropped_events_count: 0,
        }
    }
}

/// High-performance metrics system
#[derive(Debug)]
pub struct HighPerformanceMetrics {
    start_nanos: AtomicU64,
    event_metrics: [AtomicEventMetrics; 3],
    processing_stats: AtomicProcessingTimeStats,
    // Dropped events metrics
    dropped_events_count: AtomicU64,
}

impl HighPerformanceMetrics {
    /// Const constructor for static initialization (zero-cost)
    const fn new_const() -> Self {
        Self {
            start_nanos: AtomicU64::new(0), // Will be lazily initialized on first access
            event_metrics: [
                AtomicEventMetrics::new_const(),
                AtomicEventMetrics::new_const(),
                AtomicEventMetrics::new_const(),
            ],
            processing_stats: AtomicProcessingTimeStats::new_const(),
            dropped_events_count: AtomicU64::new(0),
        }
    }

    /// Get uptime (seconds)
    #[inline]
    pub fn get_uptime_seconds(&self) -> f64 {
        let now_nanos =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
                as u64;

        // Lazy initialization of start_nanos (compare-and-swap once)
        let mut start = self.start_nanos.load(Ordering::Relaxed);
        if start == 0 {
            // Try to initialize with current time
            match self.start_nanos.compare_exchange(
                0,
                now_nanos,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => start = now_nanos,
                Err(existing) => start = existing,
            }
        }

        (now_nanos - start) as f64 / 1_000_000_000.0
    }

    /// Get event metrics snapshot
    #[inline]
    pub fn get_event_metrics(&self, event_type: EventType) -> EventMetricsSnapshot {
        let index = event_type.as_index();
        let (process_count, events_processed, _) = self.event_metrics[index].get_counts();
        let processing_stats = self.event_metrics[index].get_processing_stats();

        EventMetricsSnapshot { process_count, events_processed, processing_stats }
    }

    /// Get processing time statistics
    #[inline]
    pub fn get_processing_stats(&self) -> ProcessingTimeStats {
        self.processing_stats.get_stats()
    }

    /// Get dropped events count
    #[inline]
    pub fn get_dropped_events_count(&self) -> u64 {
        self.dropped_events_count.load(Ordering::Relaxed)
    }

    /// Update window metrics (called by background task)
    fn update_window_metrics(&self, event_type: EventType, window_duration_nanos: u64) {
        let now_nanos =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
                as u64;

        let index = event_type.as_index();
        let event_metric = &self.event_metrics[index];

        let window_start = event_metric.get_window_start();
        if now_nanos.saturating_sub(window_start) >= window_duration_nanos {
            event_metric.reset_window(now_nanos);
        }
    }
}

/// Global singleton instance - zero-cost static allocation
static GLOBAL_METRICS: HighPerformanceMetrics = HighPerformanceMetrics::new_const();

/// Background task initialization flag
static BACKGROUND_TASK_STARTED: AtomicBool = AtomicBool::new(false);

/// Metrics enabled flag
static METRICS_ENABLED: AtomicBool = AtomicBool::new(true);

/// High-performance metrics manager (Singleton)
#[derive(Clone, Copy)]
pub struct MetricsManager;

impl MetricsManager {
    /// Get global singleton instance (zero-cost)
    #[inline]
    pub const fn global() -> Self {
        Self
    }

    /// Initialize and start background task (call once at startup)
    pub fn init(enable_metrics: bool) {
        METRICS_ENABLED.store(enable_metrics, Ordering::Relaxed);

        // Start background task only once
        if enable_metrics
            && BACKGROUND_TASK_STARTED
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
        {
            tokio::spawn(async {
                let mut interval = tokio::time::interval(std::time::Duration::from_millis(500));
                loop {
                    interval.tick().await;
                    let window_duration_nanos = DEFAULT_METRICS_WINDOW_SECONDS * 1_000_000_000;

                    // Update window metrics for all event types
                    GLOBAL_METRICS
                        .update_window_metrics(EventType::Transaction, window_duration_nanos);
                    GLOBAL_METRICS.update_window_metrics(EventType::Account, window_duration_nanos);
                    GLOBAL_METRICS
                        .update_window_metrics(EventType::BlockMeta, window_duration_nanos);
                }
            });
        }
    }

    #[inline]
    fn is_enabled(&self) -> bool {
        METRICS_ENABLED.load(Ordering::Relaxed)
    }

    /// Record process count (non-blocking)
    #[inline]
    pub fn record_process(&self, event_type: EventType) {
        if self.is_enabled() {
            GLOBAL_METRICS.event_metrics[event_type.as_index()].add_process_count();
        }
    }

    /// Record event processing (non-blocking)
    #[inline]
    pub fn record_events(&self, event_type: EventType, count: u64, processing_time_us: f64) {
        if !self.is_enabled() {
            return;
        }

        let index = event_type.as_index();

        // Atomically update event count
        GLOBAL_METRICS.event_metrics[index].add_events_processed(count);

        // Atomically update processing time statistics for this event type
        GLOBAL_METRICS.event_metrics[index].update_processing_stats(processing_time_us, count);

        // Maintain compatibility with global processing time statistics
        GLOBAL_METRICS.processing_stats.update(processing_time_us, count);
    }

    /// Record slow processing operation
    #[inline]
    pub fn log_slow_processing(&self, processing_time_us: f64, event_count: usize) {
        if processing_time_us > SLOW_PROCESSING_THRESHOLD_US {
            log::debug!("Slow processing: {:.2}us for {} events", processing_time_us, event_count);
        }
    }

    /// Check and warn about high latency (calibrated gRPC latency)
    /// latency = recv_time - (block_time + 500ms)
    #[inline]
    pub fn check_and_warn_high_latency(&self, recv_us: i64, block_time_ms: i64) {
        let recv_ms = recv_us / 1000;
        // Calibrate latency: recv_time - (block_time + 500ms)
        let adjusted_latency_ms = recv_ms - (block_time_ms + SOLANA_BLOCK_TIME_ADJUSTMENT_MS);

        if adjusted_latency_ms > MAX_LATENCY_THRESHOLD_MS {
            log::warn!(
                "⚠️  High gRPC latency: {}ms (threshold: {}ms, raw: recv={}ms, block={}ms)",
                adjusted_latency_ms,
                MAX_LATENCY_THRESHOLD_MS,
                recv_ms,
                block_time_ms
            );
        }
    }

    /// Get uptime
    pub fn get_uptime(&self) -> std::time::Duration {
        std::time::Duration::from_secs_f64(GLOBAL_METRICS.get_uptime_seconds())
    }

    /// Get event metrics
    pub fn get_event_metrics(&self, event_type: EventType) -> EventMetricsSnapshot {
        GLOBAL_METRICS.get_event_metrics(event_type)
    }

    /// Get processing time statistics
    pub fn get_processing_stats(&self) -> ProcessingTimeStats {
        GLOBAL_METRICS.get_processing_stats()
    }

    /// Get dropped events count
    pub fn get_dropped_events_count(&self) -> u64 {
        GLOBAL_METRICS.get_dropped_events_count()
    }

    /// Print performance metrics (non-blocking)
    pub fn print_metrics(&self) {
        println!("\n📊 Performance Metrics");
        println!("   Run Time: {:?}", self.get_uptime());

        // Print dropped events metrics
        let dropped_count = self.get_dropped_events_count();
        if dropped_count > 0 {
            println!("\n⚠️  Dropped Events: {}", dropped_count);
        }

        // Print event metrics table (including processing time statistics)
        println!("┌─────────────┬──────────────┬──────────────────┬─────────────┬─────────────┐");
        println!("│ Event Type  │ Process Count│ Events Processed │ Last(μs)    │ Avg(μs)     │");
        println!("├─────────────┼──────────────┼──────────────────┼─────────────┼─────────────┤");

        for event_type in [EventType::Transaction, EventType::Account, EventType::BlockMeta] {
            let metrics = self.get_event_metrics(event_type);
            println!(
                "│ {:11} │ {:12} │ {:16} │ {:9.2}   │ {:9.2}   │",
                event_type.name(),
                metrics.process_count,
                metrics.events_processed,
                metrics.processing_stats.last_us,
                metrics.processing_stats.avg_us
            );
        }

        println!("└─────────────┴──────────────┴──────────────────┴─────────────┴─────────────┘");
        println!();
    }

    /// Start automatic performance monitoring task
    pub async fn start_auto_monitoring(&self) -> Option<tokio::task::JoinHandle<()>> {
        if !self.is_enabled() {
            return None;
        }

        let handle = tokio::spawn(async {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(
                DEFAULT_METRICS_PRINT_INTERVAL_SECONDS,
            ));
            loop {
                interval.tick().await;
                MetricsManager::global().print_metrics();
            }
        });
        Some(handle)
    }

    /// Get complete performance metrics (compatibility method)
    pub fn get_metrics(&self) -> PerformanceMetrics {
        PerformanceMetrics {
            uptime: self.get_uptime(),
            tx_metrics: self.get_event_metrics(EventType::Transaction),
            account_metrics: self.get_event_metrics(EventType::Account),
            block_meta_metrics: self.get_event_metrics(EventType::BlockMeta),
            processing_stats: self.get_processing_stats(),
            dropped_events_count: self.get_dropped_events_count(),
        }
    }

    /// Compatibility method - add transaction process count
    #[inline]
    pub fn add_tx_process_count(&self) {
        self.record_process(EventType::Transaction);
    }

    /// Compatibility method - add account process count
    #[inline]
    pub fn add_account_process_count(&self) {
        self.record_process(EventType::Account);
    }

    /// Compatibility method - add block meta process count
    #[inline]
    pub fn add_block_meta_process_count(&self) {
        self.record_process(EventType::BlockMeta);
    }

    /// Compatibility method - update metrics
    #[inline]
    pub fn update_metrics(
        &self,
        event_type: MetricsEventType,
        events_processed: u64,
        processing_time_us: f64,
    ) {
        self.record_events(event_type, events_processed, processing_time_us);
        self.log_slow_processing(processing_time_us, events_processed as usize);
    }

    /// Update metrics and check latency
    #[inline]
    pub fn update_metrics_with_latency(
        &self,
        event_type: MetricsEventType,
        events_processed: u64,
        processing_time_us: f64,
        recv_us: i64,
        block_time_ms: i64,
    ) {
        self.check_and_warn_high_latency(recv_us, block_time_ms);
        self.update_metrics(event_type, events_processed, processing_time_us);
    }

    /// Increment dropped events count
    #[inline]
    pub fn increment_dropped_events(&self) {
        if !self.is_enabled() {
            return;
        }

        // Atomically increment dropped events count
        let new_count = GLOBAL_METRICS.dropped_events_count.fetch_add(1, Ordering::Relaxed) + 1;

        // Log warning every 1000 dropped events
        if new_count % 1000 == 0 {
            log::debug!("Dropped events count reached: {}", new_count);
        }
    }

    /// Batch increment dropped events count
    #[inline]
    pub fn increment_dropped_events_by(&self, count: u64) {
        if !self.is_enabled() || count == 0 {
            return;
        }

        // Atomically increment dropped events count
        let new_count =
            GLOBAL_METRICS.dropped_events_count.fetch_add(count, Ordering::Relaxed) + count;

        // Log batch dropped events
        if count > 1 {
            log::debug!("Dropped batch of {} events, total dropped: {}", count, new_count);
        }

        // Log warning every 1000 dropped events
        if new_count % 1000 == 0 || (new_count / 1000) != ((new_count - count) / 1000) {
            log::debug!("Dropped events count reached: {}", new_count);
        }
    }
}
