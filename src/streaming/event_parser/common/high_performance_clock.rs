use std::fmt::Debug;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// 高性能时钟管理器，减少系统调用开销并最小化延迟
#[derive(Debug)]
pub struct HighPerformanceClock {
    /// 基准时间点（程序启动时的单调时钟时间）
    base_instant: Instant,
    /// 基准时间点对应的UTC时间戳（微秒）
    base_timestamp_us: i64,
    /// 上次校准时间（用于检测是否需要重新校准）
    last_calibration: Instant,
    /// 校准间隔（秒）
    calibration_interval_secs: u64,
}

impl HighPerformanceClock {
    /// 创建新的高性能时钟
    pub fn new() -> Self {
        Self::new_with_calibration_interval(300) // 默认5分钟校准一次
    }

    /// 创建带自定义校准间隔的高性能时钟
    pub fn new_with_calibration_interval(calibration_interval_secs: u64) -> Self {
        // 通过多次采样来减少初始化误差
        let mut best_offset = i64::MAX;
        let mut best_instant = Instant::now();
        let mut best_timestamp =
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros() as i64;

        // 进行3次采样，选择延迟最小的
        for _ in 0..3 {
            let instant_before = Instant::now();
            let timestamp =
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros() as i64;
            let instant_after = Instant::now();

            let sample_latency = instant_after.duration_since(instant_before).as_nanos() as i64;

            if sample_latency < best_offset {
                best_offset = sample_latency;
                best_instant = instant_before;
                best_timestamp = timestamp;
            }
        }

        Self {
            base_instant: best_instant,
            base_timestamp_us: best_timestamp,
            last_calibration: best_instant,
            calibration_interval_secs,
        }
    }

    /// 获取当前时间戳（微秒），使用单调时钟计算，避免系统调用
    #[inline(always)]
    pub fn now_micros(&self) -> i64 {
        let elapsed = self.base_instant.elapsed();
        self.base_timestamp_us + elapsed.as_micros() as i64
    }

    /// 获取高精度当前时间戳（微秒），在必要时进行校准
    pub fn now_micros_with_calibration(&mut self) -> i64 {
        // 检查是否需要重新校准
        if self.last_calibration.elapsed().as_secs() >= self.calibration_interval_secs {
            self.recalibrate();
        }
        self.now_micros()
    }

    /// 重新校准时钟，减少累积漂移
    fn recalibrate(&mut self) {
        let current_monotonic = Instant::now();
        let current_utc = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros() as i64;

        // 计算预期的UTC时间戳（基于单调时钟）
        let expected_utc = self.base_timestamp_us
            + current_monotonic.duration_since(self.base_instant).as_micros() as i64;

        // 计算漂移量
        let drift_us = current_utc - expected_utc;

        // 如果漂移超过1毫秒，进行校准
        if drift_us.abs() > 1000 {
            self.base_instant = current_monotonic;
            self.base_timestamp_us = current_utc;
        }

        self.last_calibration = current_monotonic;
    }

    /// 计算从指定时间戳到现在的消耗时间（微秒）
    #[inline(always)]
    pub fn elapsed_micros_since(&self, start_timestamp_us: i64) -> i64 {
        self.now_micros() - start_timestamp_us
    }

    /// 获取高精度纳秒时间戳
    #[inline(always)]
    pub fn now_nanos(&self) -> i128 {
        let elapsed = self.base_instant.elapsed();
        (self.base_timestamp_us as i128 * 1000) + elapsed.as_nanos() as i128
    }

    /// 重置时钟（强制重新初始化）
    pub fn reset(&mut self) {
        *self = Self::new_with_calibration_interval(self.calibration_interval_secs);
    }
}

impl Default for HighPerformanceClock {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局高性能时钟实例
static HIGH_PERF_CLOCK: std::sync::OnceLock<HighPerformanceClock> = std::sync::OnceLock::new();

/// 获取全局高性能时钟实例（最简单的实现）
#[inline(always)]
pub fn get_high_perf_clock() -> i64 {
    let clock = HIGH_PERF_CLOCK.get_or_init(HighPerformanceClock::new);
    clock.now_micros()
}

/// 计算从指定时间戳到现在的消耗时间（微秒）
#[inline(always)]
pub fn elapsed_micros_since(start_timestamp_us: i64) -> i64 {
    get_high_perf_clock() - start_timestamp_us
}
