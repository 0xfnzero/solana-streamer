use std::fmt::Debug;
use std::time::Instant;

/// High-performance clock manager, reduces system call overhead and minimizes latency
#[derive(Debug)]
pub struct HighPerformanceClock {
    /// Base time point (monotonic clock time at program startup)
    base_instant: Instant,
    /// UTC timestamp (microseconds) corresponding to base time point
    base_timestamp_us: i64,
    /// Last calibration time (used to detect if recalibration is needed)
    last_calibration: Instant,
    /// Calibration interval (seconds)
    calibration_interval_secs: u64,
}

impl HighPerformanceClock {
    /// Create new high-performance clock
    pub fn new() -> Self {
        Self::new_with_calibration_interval(300) // Default: calibrate every 5 minutes
    }

    /// Create high-performance clock with custom calibration interval
    pub fn new_with_calibration_interval(calibration_interval_secs: u64) -> Self {
        // Reduce initialization error through multiple samples
        let mut best_offset = i64::MAX;
        let mut best_instant = Instant::now();
        let mut best_timestamp = chrono::Utc::now().timestamp_micros();

        // Perform 3 samples, choose the one with minimum latency
        for _ in 0..3 {
            let instant_before = Instant::now();
            let timestamp = chrono::Utc::now().timestamp_micros();
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

    /// Get current timestamp (microseconds), calculated using monotonic clock to avoid system calls
    #[inline(always)]
    pub fn now_micros(&self) -> i64 {
        let elapsed = self.base_instant.elapsed();
        self.base_timestamp_us + elapsed.as_micros() as i64
    }

    /// Get high-precision current timestamp (microseconds), calibrate when necessary
    pub fn now_micros_with_calibration(&mut self) -> i64 {
        // Check if recalibration is needed
        if self.last_calibration.elapsed().as_secs() >= self.calibration_interval_secs {
            self.recalibrate();
        }
        self.now_micros()
    }

    /// Recalibrate clock to reduce accumulated drift
    fn recalibrate(&mut self) {
        let current_monotonic = Instant::now();
        let current_utc = chrono::Utc::now().timestamp_micros();

        // Calculate expected UTC timestamp (based on monotonic clock)
        let expected_utc = self.base_timestamp_us
            + current_monotonic.duration_since(self.base_instant).as_micros() as i64;

        // Calculate drift amount
        let drift_us = current_utc - expected_utc;

        // If drift exceeds 1 millisecond, perform calibration
        if drift_us.abs() > 1000 {
            self.base_instant = current_monotonic;
            self.base_timestamp_us = current_utc;
        }

        self.last_calibration = current_monotonic;
    }

    /// Calculate elapsed time (microseconds) from specified timestamp to now
    #[inline(always)]
    pub fn elapsed_micros_since(&self, start_timestamp_us: i64) -> i64 {
        self.now_micros() - start_timestamp_us
    }

    /// Get high-precision nanosecond timestamp
    #[inline(always)]
    pub fn now_nanos(&self) -> i128 {
        let elapsed = self.base_instant.elapsed();
        (self.base_timestamp_us as i128 * 1000) + elapsed.as_nanos() as i128
    }

    /// Reset clock (force re-initialization)
    pub fn reset(&mut self) {
        *self = Self::new_with_calibration_interval(self.calibration_interval_secs);
    }
}

impl Default for HighPerformanceClock {
    fn default() -> Self {
        Self::new()
    }
}

/// Global high-performance clock instance
static HIGH_PERF_CLOCK: once_cell::sync::OnceCell<HighPerformanceClock> =
    once_cell::sync::OnceCell::new();

/// Get global high-performance clock instance (simplest implementation)
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
