use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// Memory tracking statistics
///
/// Provides tracking completeness monitoring and quality warnings
/// to ensure users can monitor memory tracking quality in real-time.
#[derive(Debug)]
pub struct TrackingStats {
    /// Total tracking attempts
    pub total_attempts: AtomicUsize,
    /// Successful tracking count  
    pub successful_tracks: AtomicUsize,
    /// Failed attempts due to contention
    pub missed_due_to_contention: AtomicUsize,
    /// Last warning time for rate limiting
    pub last_warning_time: std::sync::Mutex<Option<Instant>>,
}

impl TrackingStats {
    /// Create new tracking statistics instance
    pub fn new() -> Self {
        Self {
            total_attempts: AtomicUsize::new(0),
            successful_tracks: AtomicUsize::new(0),
            missed_due_to_contention: AtomicUsize::new(0),
            last_warning_time: std::sync::Mutex::new(None),
        }
    }

    /// Record a tracking attempt
    #[inline]
    pub fn record_attempt(&self) {
        self.total_attempts.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a successful tracking
    #[inline]
    pub fn record_success(&self) {
        self.successful_tracks.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a failed tracking due to contention
    #[inline]
    pub fn record_miss(&self) {
        self.missed_due_to_contention
            .fetch_add(1, Ordering::Relaxed);
        self.maybe_warn();
    }

    /// Get tracking completeness percentage
    ///
    /// # Returns
    ///
    /// Returns value between 0.0-1.0 representing successful tracking ratio
    pub fn get_completeness(&self) -> f64 {
        let attempts = self.total_attempts.load(Ordering::Relaxed);
        let successful = self.successful_tracks.load(Ordering::Relaxed);
        if attempts == 0 {
            1.0
        } else {
            successful as f64 / attempts as f64
        }
    }

    /// Get detailed statistics
    pub fn get_detailed_stats(&self) -> DetailedStats {
        let attempts = self.total_attempts.load(Ordering::Relaxed);
        let successful = self.successful_tracks.load(Ordering::Relaxed);
        let missed = self.missed_due_to_contention.load(Ordering::Relaxed);

        DetailedStats {
            total_attempts: attempts,
            successful_tracks: successful,
            missed_due_to_contention: missed,
            completeness: self.get_completeness(),
            contention_rate: if attempts > 0 {
                missed as f64 / attempts as f64
            } else {
                0.0
            },
        }
    }

    /// Reset all statistics counters
    pub fn reset(&self) {
        self.total_attempts.store(0, Ordering::Relaxed);
        self.successful_tracks.store(0, Ordering::Relaxed);
        self.missed_due_to_contention.store(0, Ordering::Relaxed);

        if let Ok(mut last_warning) = self.last_warning_time.lock() {
            *last_warning = None;
        }
    }

    /// Check if warning should be issued
    ///
    /// Warns when tracking completeness drops below 90%, limited to once per 10 seconds
    fn maybe_warn(&self) {
        let completeness = self.get_completeness();

        // Only consider warning when completeness is below 90%
        if completeness < 0.9 {
            if let Ok(mut last_warning) = self.last_warning_time.lock() {
                let now = Instant::now();
                let should_warn = last_warning
                    .map(|last| now.duration_since(last) > Duration::from_secs(10))
                    .unwrap_or(true);

                if should_warn {
                    let stats = self.get_detailed_stats();
                    eprintln!(
                        "WARNING: Memory tracking completeness: {:.1}% ({}/{} successful, {} missed due to contention)",
                        completeness * 100.0,
                        stats.successful_tracks,
                        stats.total_attempts,
                        stats.missed_due_to_contention
                    );
                    *last_warning = Some(now);
                }
            }
        }
    }
}

impl Default for TrackingStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Detailed tracking statistics
#[derive(Debug, Clone)]
pub struct DetailedStats {
    /// Total attempts count
    pub total_attempts: usize,
    /// Successful tracking count
    pub successful_tracks: usize,
    /// Failed attempts due to contention
    pub missed_due_to_contention: usize,
    /// Tracking completeness (0.0-1.0)
    pub completeness: f64,
    /// Lock contention rate (0.0-1.0)
    pub contention_rate: f64,
}

impl DetailedStats {
    /// Check if tracking quality is healthy
    pub fn is_healthy(&self) -> bool {
        self.completeness >= 0.95 && self.contention_rate <= 0.05
    }

    /// Get quality grade description
    pub fn quality_grade(&self) -> &'static str {
        match self.completeness {
            x if x >= 0.98 => "Excellent",
            x if x >= 0.95 => "Good",
            x if x >= 0.90 => "Fair",
            x if x >= 0.80 => "Poor",
            _ => "Critical",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_tracking_stats_basic() {
        let stats = TrackingStats::new();

        // Initial state
        assert_eq!(stats.get_completeness(), 1.0);

        // Record some operations
        stats.record_attempt();
        stats.record_success();
        assert_eq!(stats.get_completeness(), 1.0);

        stats.record_attempt();
        stats.record_miss();
        assert_eq!(stats.get_completeness(), 0.5);
    }

    #[test]
    fn test_detailed_stats() {
        let stats = TrackingStats::new();

        // Simulate some tracking operations
        for _ in 0..100 {
            stats.record_attempt();
            stats.record_success();
        }

        for _ in 0..5 {
            stats.record_attempt();
            stats.record_miss();
        }

        let detailed = stats.get_detailed_stats();
        assert_eq!(detailed.total_attempts, 105);
        assert_eq!(detailed.successful_tracks, 100);
        assert_eq!(detailed.missed_due_to_contention, 5);
        assert!((detailed.completeness - 0.9523).abs() < 0.001);
        assert!(detailed.is_healthy());
    }

    #[test]
    fn test_quality_grades() {
        let stats = TrackingStats::new();

        // 测试不同质量等级
        let test_cases = vec![
            (100, 100, "Excellent"),
            (100, 96, "Good"),
            (100, 92, "Fair"),
            (100, 85, "Poor"),
            (100, 70, "Critical"),
        ];

        for (attempts, successes, expected_grade) in test_cases {
            stats.reset();

            for _ in 0..attempts {
                stats.record_attempt();
            }
            for _ in 0..successes {
                stats.record_success();
            }

            let detailed = stats.get_detailed_stats();
            assert_eq!(detailed.quality_grade(), expected_grade);
        }
    }

    #[test]
    fn test_concurrent_access() {
        let stats = std::sync::Arc::new(TrackingStats::new());
        let mut handles = vec![];

        // 启动多个线程并发访问
        for _ in 0..4 {
            let stats_clone = stats.clone();
            let handle = thread::spawn(move || {
                for _ in 0..1000 {
                    stats_clone.record_attempt();
                    if thread_local_random() % 10 != 0 {
                        stats_clone.record_success();
                    } else {
                        stats_clone.record_miss();
                    }
                }
            });
            handles.push(handle);
        }

        // 等待所有线程完成
        for handle in handles {
            handle.join().unwrap();
        }

        let detailed = stats.get_detailed_stats();
        assert_eq!(detailed.total_attempts, 4000);
        assert!(detailed.successful_tracks >= 3000); // 大约90%成功率
        assert!(detailed.completeness >= 0.8);
    }

    fn thread_local_random() -> usize {
        use std::cell::Cell;
        thread_local! {
            static RNG: Cell<usize> = Cell::new(1);
        }

        RNG.with(|rng| {
            let x = rng.get();
            let next = x.wrapping_mul(1103515245).wrapping_add(12345);
            rng.set(next);
            next
        })
    }
}
