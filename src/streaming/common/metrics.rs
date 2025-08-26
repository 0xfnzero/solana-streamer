use std::sync::Arc;
use crossbeam::utils::Backoff;
use crossbeam::atomic::AtomicCell;
use std::sync::RwLock;

use super::config::StreamClientConfig;
use super::constants::*;

/// 单个事件类型的指标
#[derive(Debug, Clone)]
pub struct EventMetrics {
    pub process_count: u64,
    pub events_processed: u64,
    pub events_per_second: f64,
    pub events_in_window: u64,
    pub window_start_time: std::time::Instant,
}

impl EventMetrics {
    fn new(now: std::time::Instant) -> Self {
        Self {
            process_count: 0,
            events_processed: 0,
            events_per_second: 0.0,
            events_in_window: 0,
            window_start_time: now,
        }
    }
}

/// 通用性能监控指标
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub start_time: std::time::Instant,
    pub event_metrics: [EventMetrics; 3], // [Tx, Account, BlockMeta]
    pub average_processing_time_us: f64,
    pub min_processing_time_us: f64,
    pub max_processing_time_us: f64,
    pub last_update_time: std::time::Instant,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

pub enum MetricsEventType {
    Tx,
    Account,
    BlockMeta,
}

impl MetricsEventType {
    fn as_index(&self) -> usize {
        match self {
            MetricsEventType::Tx => 0,
            MetricsEventType::Account => 1,
            MetricsEventType::BlockMeta => 2,
        }
    }
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        let now = std::time::Instant::now();
        Self {
            start_time: now,
            event_metrics: [EventMetrics::new(now), EventMetrics::new(now), EventMetrics::new(now)],
            average_processing_time_us: 0.0,
            min_processing_time_us: 0.0,
            max_processing_time_us: 0.0,
            last_update_time: now,
        }
    }

    /// 更新时间窗口指标
    fn update_window_metrics(
        &mut self,
        event_type: &MetricsEventType,
        now: std::time::Instant,
        window_duration: std::time::Duration,
    ) {
        let index = event_type.as_index();
        let event_metric = &mut self.event_metrics[index];

        if now.duration_since(event_metric.window_start_time) >= window_duration {
            let window_seconds = now.duration_since(event_metric.window_start_time).as_secs_f64();
            // 修复：正确计算每秒事件数，避免除零错误
            event_metric.events_per_second = if window_seconds > 0.001 {
                // 避免极小的时间差
                event_metric.events_in_window as f64 / window_seconds
            } else {
                0.0 // 时间太短时设为0，而不是事件总数
            };

            // 重置窗口
            event_metric.events_in_window = 0;
            event_metric.window_start_time = now;
        }
    }

    /// 计算实时每秒事件数（用于显示）
    fn calculate_real_time_events_per_second(
        &self,
        event_type: &MetricsEventType,
        now: std::time::Instant,
    ) -> f64 {
        let index = event_type.as_index();
        let event_metric = &self.event_metrics[index];

        let current_window_duration =
            now.duration_since(event_metric.window_start_time).as_secs_f64();

        // 如果当前窗口有足够的时间和事件，使用当前窗口的数据
        if current_window_duration > 1.0 && event_metric.events_in_window > 0 {
            event_metric.events_in_window as f64 / current_window_duration
        }
        // 如果当前窗口时间太短或没有事件，使用上一个完整窗口的值
        else if event_metric.events_per_second > 0.0 {
            event_metric.events_per_second
        }
        // 如果都没有，计算总体平均值
        else {
            let total_duration = now.duration_since(self.start_time).as_secs_f64();
            if total_duration > 1.0 && event_metric.events_processed > 0 {
                event_metric.events_processed as f64 / total_duration
            } else {
                0.0
            }
        }
    }
}

/// 通用性能监控管理器
pub struct MetricsManager {
    metrics: Arc<RwLock<PerformanceMetrics>>,
    config: Arc<StreamClientConfig>,
    stream_name: String,
}

impl MetricsManager {
    /// 创建新的性能监控管理器
    pub fn new(
        metrics: Arc<RwLock<PerformanceMetrics>>,
        config: Arc<StreamClientConfig>,
        stream_name: String,
    ) -> Self {
        Self { metrics, config, stream_name }
    }

    /// 获取性能指标
    pub fn get_metrics(&self) -> PerformanceMetrics {
        // 使用 Backoff 策略进行读取尝试
        let backoff = Backoff::new();
        loop {
            match self.metrics.read() {
                Ok(metrics) => return metrics.clone(),
                Err(_) => {
                    // 如果获取读锁失败，使用指数退避策略
                    backoff.snooze();
                    continue;
                }
            }
        }
    }

    /// 打印性能指标
    pub fn print_metrics(&self) {
        let metrics = self.get_metrics();
        let event_names = ["TX", "Account", "Block Meta"];
        let event_types =
            [MetricsEventType::Tx, MetricsEventType::Account, MetricsEventType::BlockMeta];
        let now = std::time::Instant::now();

        println!("\n📊 {} Performance Metrics", self.stream_name);
        println!("   Run Time: {:?}", metrics.start_time.elapsed());
        
        // 打印表格头部
        println!("┌─────────────┬──────────────┬──────────────────┬─────────────────┐");
        println!("│ Event Type  │ Process Count│ Events Processed │ Events/Second   │");
        println!("├─────────────┼──────────────┼──────────────────┼─────────────────┤");

        // 打印每种事件类型的数据
        for (i, name) in event_names.iter().enumerate() {
            let event_metric = &metrics.event_metrics[i];
            // 使用实时计算的每秒事件数，而不是窗口更新的值
            let real_time_eps = metrics.calculate_real_time_events_per_second(&event_types[i], now);

            println!(
                "│ {:11} │ {:12} │ {:16} │ {:13.2}   │",
                name,
                event_metric.process_count,
                event_metric.events_processed,
                real_time_eps
            );
        }

        println!("└─────────────┴──────────────┴──────────────────┴─────────────────┘");

        // 打印处理时间统计表格
        println!("\n⏱️  Processing Time Statistics");
        println!("┌─────────────────────┬─────────────┐");
        println!("│ Metric              │ Value (us)  │");
        println!("├─────────────────────┼─────────────┤");
        println!("│ Average             │ {:9.2}   │", metrics.average_processing_time_us);
        println!("│ Minimum             │ {:9.2}   │", metrics.min_processing_time_us);
        println!("│ Maximum             │ {:9.2}   │", metrics.max_processing_time_us);
        println!("└─────────────────────┴─────────────┘");
        println!();
    }

    /// 启动自动性能监控任务
    pub async fn start_auto_monitoring(&self) -> Option<tokio::task::JoinHandle<()>> {
        // 检查是否启用性能监控
        if !self.config.enable_metrics {
            return None; // 如果未启用性能监控，不启动监控任务
        }

        let metrics_manager = self.clone();
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
                DEFAULT_METRICS_PRINT_INTERVAL_SECONDS,
            ));
            loop {
                interval.tick().await;
                metrics_manager.print_metrics();
            }
        });
        Some(handle)
    }

    /// 更新处理次数
    pub fn add_process_count(&self, event_type: MetricsEventType) {
        if !self.config.enable_metrics {
            return;
        }
        
        // 使用 Backoff 策略进行写入尝试
        let backoff = Backoff::new();
        loop {
            match self.metrics.write() {
                Ok(mut metrics) => {
                    metrics.event_metrics[event_type.as_index()].process_count += 1;
                    break;
                },
                Err(_) => {
                    // 如果获取写锁失败，使用指数退避策略
                    backoff.snooze();
                    continue;
                }
            }
        }
    }

    // 保持向后兼容的方法
    pub fn add_tx_process_count(&self) {
        self.add_process_count(MetricsEventType::Tx);
    }

    pub fn add_account_process_count(&self) {
        self.add_process_count(MetricsEventType::Account);
    }

    pub fn add_block_meta_process_count(&self) {
        self.add_process_count(MetricsEventType::BlockMeta);
    }

    /// 更新性能指标
    pub fn update_metrics(
        &self,
        event_type: MetricsEventType,
        events_processed: u64,
        processing_time_us: f64,
    ) {
        // 检查是否启用性能监控
        if !self.config.enable_metrics {
            return;
        }
        
        // 使用 Backoff 策略进行写入尝试
        let backoff = Backoff::new();
        loop {
            match self.metrics.write() {
                Ok(mut metrics) => {
                    let now = std::time::Instant::now();
                    let index = event_type.as_index();

                    // 更新事件计数
                    metrics.event_metrics[index].events_processed += events_processed;
                    metrics.event_metrics[index].events_in_window += events_processed;

                    metrics.last_update_time = now;

                    // 更新处理时间统计
                    if processing_time_us < metrics.min_processing_time_us
                        || metrics.min_processing_time_us == 0.0
                    {
                        metrics.min_processing_time_us = processing_time_us;
                    }
                    if processing_time_us > metrics.max_processing_time_us {
                        metrics.max_processing_time_us = processing_time_us;
                    }

                    // 计算平均处理时间 - 使用增量更新避免重复计算
                    let total_events = metrics.event_metrics[index].events_processed;
                    if total_events > 0 {
                        let total_events_f64 = total_events as f64;
                        let old_total = (total_events_f64 - events_processed as f64).max(0.0);

                        metrics.average_processing_time_us = if old_total > 0.0 {
                            (metrics.average_processing_time_us * old_total
                                + processing_time_us * events_processed as f64)
                                / total_events_f64
                        } else {
                            processing_time_us
                        };
                    }

                    // 更新时间窗口指标
                    let window_duration = std::time::Duration::from_secs(DEFAULT_METRICS_WINDOW_SECONDS);
                    metrics.update_window_metrics(&event_type, now, window_duration);
                    break;
                },
                Err(_) => {
                    // 如果获取写锁失败，使用指数退避策略
                    backoff.snooze();
                    continue;
                }
            }
        }
    }

    /// 记录慢处理操作
    pub fn log_slow_processing(&self, processing_time_us: f64, event_count: usize) {
        if processing_time_us > SLOW_PROCESSING_THRESHOLD_US {
            log::warn!(
                "{} slow processing: {processing_time_us}us for {event_count} events",
                self.stream_name
            );
        }
    }
}

impl Clone for MetricsManager {
    fn clone(&self) -> Self {
        Self {
            metrics: self.metrics.clone(),
            config: self.config.clone(),
            stream_name: self.stream_name.clone(),
        }
    }
}
