use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Configuration for the profiler
#[derive(Debug, Clone)]
pub struct ProfilerConfig {
    pub sampling_interval: Duration,
    pub max_samples: usize,
    pub track_memory: bool,
    pub track_cpu: bool,
    pub track_allocation_patterns: bool,
    pub auto_report_interval: Option<Duration>,
}

impl Default for ProfilerConfig {
    fn default() -> Self {
        Self {
            sampling_interval: Duration::from_millis(100),
            max_samples: 10000,
            track_memory: true,
            track_cpu: true,
            track_allocation_patterns: true,
            auto_report_interval: Some(Duration::from_secs(60)),
        }
    }
}

/// A single profiling sample
#[derive(Debug, Clone)]
pub struct ProfileSample {
    pub timestamp: Instant,
    pub memory_usage: Option<usize>,
    pub allocation_rate: Option<f64>,
    pub cpu_usage: Option<f64>,
    pub active_threads: Option<usize>,
    pub custom_metrics: HashMap<String, f64>,
}

/// Result of a profiling session
#[derive(Debug, Clone)]
pub struct ProfileResult {
    pub start_time: Instant,
    pub end_time: Instant,
    pub samples: Vec<ProfileSample>,
    pub summary: ProfileSummary,
}

/// Summary statistics from profiling
#[derive(Debug, Clone)]
pub struct ProfileSummary {
    pub duration: Duration,
    pub total_samples: usize,
    pub memory_stats: Option<MemoryProfileStats>,
    pub allocation_stats: Option<AllocationProfileStats>,
    pub cpu_stats: Option<CpuProfileStats>,
    pub hotspots: Vec<Hotspot>,
}

/// Memory profiling statistics
#[derive(Debug, Clone)]
pub struct MemoryProfileStats {
    pub peak_usage: usize,
    pub average_usage: f64,
    pub min_usage: usize,
    pub growth_rate: f64, // bytes per second
    pub fragmentation_estimate: f64,
}

/// Allocation profiling statistics
#[derive(Debug, Clone)]
pub struct AllocationProfileStats {
    pub peak_rate: f64,
    pub average_rate: f64,
    pub total_allocations: u64,
    pub allocation_spikes: Vec<AllocationSpike>,
}

/// CPU profiling statistics
#[derive(Debug, Clone)]
pub struct CpuProfileStats {
    pub peak_usage: f64,
    pub average_usage: f64,
    pub cpu_spikes: Vec<CpuSpike>,
}

/// Performance hotspot identification
#[derive(Debug, Clone)]
pub struct Hotspot {
    pub metric_name: String,
    pub severity: HotspotSeverity,
    pub value: f64,
    pub threshold: f64,
    pub description: String,
    pub timestamp: Instant,
}

/// Severity levels for hotspots
#[derive(Debug, Clone, PartialEq)]
pub enum HotspotSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Allocation spike detection
#[derive(Debug, Clone)]
pub struct AllocationSpike {
    pub timestamp: Instant,
    pub rate: f64,
    pub duration: Duration,
}

/// CPU usage spike detection
#[derive(Debug, Clone)]
pub struct CpuSpike {
    pub timestamp: Instant,
    pub usage: f64,
    pub duration: Duration,
}

/// Main profiler implementation
pub struct Profiler {
    config: ProfilerConfig,
    samples: Arc<Mutex<Vec<ProfileSample>>>,
    is_running: Arc<Mutex<bool>>,
    start_time: Option<Instant>,
    background_handle: Option<thread::JoinHandle<()>>,
}

impl Profiler {
    /// Create a new profiler with the given configuration
    pub fn new(config: ProfilerConfig) -> Self {
        Self {
            config,
            samples: Arc::new(Mutex::new(Vec::new())),
            is_running: Arc::new(Mutex::new(false)),
            start_time: None,
            background_handle: None,
        }
    }

    /// Start profiling
    pub fn start(&mut self) -> Result<(), ProfilerError> {
        {
            let mut running = self
                .is_running
                .lock()
                .map_err(|_| ProfilerError::LockError)?;
            if *running {
                return Err(ProfilerError::AlreadyRunning);
            }
            *running = true;
        }

        self.start_time = Some(Instant::now());

        // Clear previous samples
        if let Ok(mut samples) = self.samples.lock() {
            samples.clear();
        }

        // Start background sampling thread
        let samples_clone = Arc::clone(&self.samples);
        let is_running_clone = Arc::clone(&self.is_running);
        let config = self.config.clone();

        let handle = thread::spawn(move || {
            Self::sampling_loop(samples_clone, is_running_clone, config);
        });

        self.background_handle = Some(handle);
        Ok(())
    }

    /// Stop profiling and return results
    pub fn stop(&mut self) -> Result<ProfileResult, ProfilerError> {
        {
            let mut running = self
                .is_running
                .lock()
                .map_err(|_| ProfilerError::LockError)?;
            if !*running {
                return Err(ProfilerError::NotRunning);
            }
            *running = false;
        }

        // Wait for background thread to finish
        if let Some(handle) = self.background_handle.take() {
            handle.join().map_err(|_| ProfilerError::ThreadJoinError)?;
        }

        let end_time = Instant::now();
        let start_time = self.start_time.take().ok_or(ProfilerError::InvalidState)?;

        let samples = self
            .samples
            .lock()
            .map_err(|_| ProfilerError::LockError)?
            .clone();

        let summary = self.generate_summary(&samples, start_time, end_time);

        Ok(ProfileResult {
            start_time,
            end_time,
            samples,
            summary,
        })
    }

    /// Check if profiler is currently running
    pub fn is_running(&self) -> bool {
        self.is_running.lock().map(|r| *r).unwrap_or(false)
    }

    /// Get current sample count
    pub fn sample_count(&self) -> usize {
        self.samples.lock().map(|s| s.len()).unwrap_or(0)
    }

    /// Main sampling loop (runs in background thread)
    fn sampling_loop(
        samples: Arc<Mutex<Vec<ProfileSample>>>,
        is_running: Arc<Mutex<bool>>,
        config: ProfilerConfig,
    ) {
        let mut last_report = Instant::now();

        while Self::should_continue(&is_running) {
            let sample = Self::collect_sample(&config);

            // Add sample to collection
            if let Ok(mut samples_guard) = samples.lock() {
                samples_guard.push(sample);

                // Trim if we exceed max samples
                if samples_guard.len() > config.max_samples {
                    samples_guard.remove(0);
                }
            }

            // Auto-report if configured
            if let Some(report_interval) = config.auto_report_interval {
                if last_report.elapsed() >= report_interval {
                    if let Ok(samples_guard) = samples.lock() {
                        Self::print_interim_report(&samples_guard);
                    }
                    last_report = Instant::now();
                }
            }

            thread::sleep(config.sampling_interval);
        }
    }

    /// Check if sampling should continue
    fn should_continue(is_running: &Arc<Mutex<bool>>) -> bool {
        is_running.lock().map(|r| *r).unwrap_or(false)
    }

    /// Collect a single profiling sample
    fn collect_sample(config: &ProfilerConfig) -> ProfileSample {
        let timestamp = Instant::now();
        let mut sample = ProfileSample {
            timestamp,
            memory_usage: None,
            allocation_rate: None,
            cpu_usage: None,
            active_threads: None,
            custom_metrics: HashMap::new(),
        };

        // Collect memory usage
        if config.track_memory {
            sample.memory_usage = Self::get_memory_usage();
        }

        // Collect allocation rate
        if config.track_allocation_patterns {
            sample.allocation_rate = Self::get_allocation_rate();
        }

        // Collect CPU usage (simplified)
        if config.track_cpu {
            sample.cpu_usage = Self::get_cpu_usage();
        }

        // Thread count
        sample.active_threads = Some(Self::get_thread_count());

        sample
    }

    /// Get current memory usage
    fn get_memory_usage() -> Option<usize> {
        // Use global statistics from performance module
        let stats = crate::performance::get_global_stats();
        // Approximate memory usage based on tracked allocations
        Some(
            (stats
                .allocations_tracked
                .saturating_sub(stats.deallocations_tracked)
                * 1024) as usize,
        )
    }

    /// Get current allocation rate
    fn get_allocation_rate() -> Option<f64> {
        let stats = crate::performance::get_global_stats();
        Some(stats.tracking_rate())
    }

    /// Get current CPU usage (simplified estimation)
    fn get_cpu_usage() -> Option<f64> {
        // This is a simplified estimation - in a real implementation,
        // you'd use platform-specific APIs to get actual CPU usage
        Some(0.5) // Placeholder
    }

    /// Get current thread count
    fn get_thread_count() -> usize {
        // This is a simplified implementation
        num_cpus::get()
    }

    /// Generate summary statistics from samples
    fn generate_summary(
        &self,
        samples: &[ProfileSample],
        start_time: Instant,
        end_time: Instant,
    ) -> ProfileSummary {
        let duration = end_time.duration_since(start_time);

        let memory_stats = if self.config.track_memory {
            Some(Self::analyze_memory_stats(samples))
        } else {
            None
        };

        let allocation_stats = if self.config.track_allocation_patterns {
            Some(Self::analyze_allocation_stats(samples))
        } else {
            None
        };

        let cpu_stats = if self.config.track_cpu {
            Some(Self::analyze_cpu_stats(samples))
        } else {
            None
        };

        let hotspots = Self::detect_hotspots(samples, &memory_stats, &allocation_stats, &cpu_stats);

        ProfileSummary {
            duration,
            total_samples: samples.len(),
            memory_stats,
            allocation_stats,
            cpu_stats,
            hotspots,
        }
    }

    /// Analyze memory usage statistics
    fn analyze_memory_stats(samples: &[ProfileSample]) -> MemoryProfileStats {
        let memory_values: Vec<usize> = samples.iter().filter_map(|s| s.memory_usage).collect();

        if memory_values.is_empty() {
            return MemoryProfileStats {
                peak_usage: 0,
                average_usage: 0.0,
                min_usage: 0,
                growth_rate: 0.0,
                fragmentation_estimate: 0.0,
            };
        }

        let peak_usage = *memory_values.iter().max().unwrap();
        let min_usage = *memory_values.iter().min().unwrap();
        let average_usage = memory_values.iter().sum::<usize>() as f64 / memory_values.len() as f64;

        // Calculate growth rate
        let growth_rate = if memory_values.len() > 1 {
            let first = memory_values[0] as f64;
            let last = memory_values[memory_values.len() - 1] as f64;
            let time_diff = samples
                .last()
                .unwrap()
                .timestamp
                .duration_since(samples[0].timestamp)
                .as_secs_f64();
            if time_diff > 0.0 {
                (last - first) / time_diff
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Estimate fragmentation (simplified)
        let fragmentation_estimate = if peak_usage > 0 {
            (peak_usage - min_usage) as f64 / peak_usage as f64
        } else {
            0.0
        };

        MemoryProfileStats {
            peak_usage,
            average_usage,
            min_usage,
            growth_rate,
            fragmentation_estimate,
        }
    }

    /// Analyze allocation statistics
    fn analyze_allocation_stats(samples: &[ProfileSample]) -> AllocationProfileStats {
        let allocation_rates: Vec<f64> = samples.iter().filter_map(|s| s.allocation_rate).collect();

        if allocation_rates.is_empty() {
            return AllocationProfileStats {
                peak_rate: 0.0,
                average_rate: 0.0,
                total_allocations: 0,
                allocation_spikes: Vec::new(),
            };
        }

        let peak_rate = allocation_rates.iter().fold(0.0f64, |a, &b| a.max(b));
        let average_rate = allocation_rates.iter().sum::<f64>() / allocation_rates.len() as f64;
        let total_allocations = crate::performance::get_global_stats().allocations_tracked;

        // Detect allocation spikes
        let allocation_spikes = Self::detect_allocation_spikes(samples, average_rate);

        AllocationProfileStats {
            peak_rate,
            average_rate,
            total_allocations,
            allocation_spikes,
        }
    }

    /// Analyze CPU statistics
    fn analyze_cpu_stats(samples: &[ProfileSample]) -> CpuProfileStats {
        let cpu_values: Vec<f64> = samples.iter().filter_map(|s| s.cpu_usage).collect();

        if cpu_values.is_empty() {
            return CpuProfileStats {
                peak_usage: 0.0,
                average_usage: 0.0,
                cpu_spikes: Vec::new(),
            };
        }

        let peak_usage = cpu_values.iter().fold(0.0f64, |a, &b| a.max(b));
        let average_usage = cpu_values.iter().sum::<f64>() / cpu_values.len() as f64;

        let cpu_spikes = Self::detect_cpu_spikes(samples, average_usage);

        CpuProfileStats {
            peak_usage,
            average_usage,
            cpu_spikes,
        }
    }

    /// Detect allocation spikes
    fn detect_allocation_spikes(
        samples: &[ProfileSample],
        average_rate: f64,
    ) -> Vec<AllocationSpike> {
        let threshold = average_rate * 2.0; // Spike if 2x average
        let mut spikes = Vec::new();

        for sample in samples {
            if let Some(rate) = sample.allocation_rate {
                if rate > threshold {
                    spikes.push(AllocationSpike {
                        timestamp: sample.timestamp,
                        rate,
                        duration: Duration::from_millis(100), // Sampling interval
                    });
                }
            }
        }

        spikes
    }

    /// Detect CPU spikes
    fn detect_cpu_spikes(samples: &[ProfileSample], average_usage: f64) -> Vec<CpuSpike> {
        let threshold = (average_usage * 1.5).min(0.9); // Spike if 1.5x average or >90%
        let mut spikes = Vec::new();

        for sample in samples {
            if let Some(usage) = sample.cpu_usage {
                if usage > threshold {
                    spikes.push(CpuSpike {
                        timestamp: sample.timestamp,
                        usage,
                        duration: Duration::from_millis(100), // Sampling interval
                    });
                }
            }
        }

        spikes
    }

    /// Detect performance hotspots
    fn detect_hotspots(
        _samples: &[ProfileSample],
        memory_stats: &Option<MemoryProfileStats>,
        allocation_stats: &Option<AllocationProfileStats>,
        cpu_stats: &Option<CpuProfileStats>,
    ) -> Vec<Hotspot> {
        let mut hotspots = Vec::new();
        let now = Instant::now();

        // Memory growth hotspot
        if let Some(mem_stats) = memory_stats {
            if mem_stats.growth_rate > 1024.0 * 1024.0 {
                // 1MB/s growth
                let severity = if mem_stats.growth_rate > 10.0 * 1024.0 * 1024.0 {
                    HotspotSeverity::Critical
                } else if mem_stats.growth_rate > 5.0 * 1024.0 * 1024.0 {
                    HotspotSeverity::High
                } else {
                    HotspotSeverity::Medium
                };

                hotspots.push(Hotspot {
                    metric_name: "memory_growth_rate".to_string(),
                    severity,
                    value: mem_stats.growth_rate,
                    threshold: 1024.0 * 1024.0,
                    description: "High memory growth rate detected".to_string(),
                    timestamp: now,
                });
            }
        }

        // High allocation rate hotspot
        if let Some(alloc_stats) = allocation_stats {
            if alloc_stats.peak_rate > 1000.0 {
                // 1000 allocations/sec
                hotspots.push(Hotspot {
                    metric_name: "allocation_rate".to_string(),
                    severity: HotspotSeverity::Medium,
                    value: alloc_stats.peak_rate,
                    threshold: 1000.0,
                    description: "High allocation rate detected".to_string(),
                    timestamp: now,
                });
            }
        }

        // High CPU usage hotspot
        if let Some(cpu_stats) = cpu_stats {
            if cpu_stats.peak_usage > 0.8 {
                // 80% CPU
                hotspots.push(Hotspot {
                    metric_name: "cpu_usage".to_string(),
                    severity: HotspotSeverity::High,
                    value: cpu_stats.peak_usage,
                    threshold: 0.8,
                    description: "High CPU usage detected".to_string(),
                    timestamp: now,
                });
            }
        }

        hotspots
    }

    /// Print interim report during auto-reporting
    fn print_interim_report(samples: &[ProfileSample]) {
        if !samples.is_empty() {
            let latest = &samples[samples.len() - 1];
            tracing::info!(
                "Profiler update - Samples: {}, Latest memory: {:?}, Latest allocation rate: {:?}",
                samples.len(),
                latest.memory_usage,
                latest.allocation_rate
            );
        }
    }
}

impl Drop for Profiler {
    fn drop(&mut self) {
        if self.is_running() {
            let _ = self.stop();
        }
    }
}

/// Profiler errors
#[derive(Debug, thiserror::Error)]
pub enum ProfilerError {
    #[error("Profiler is already running")]
    AlreadyRunning,

    #[error("Profiler is not running")]
    NotRunning,

    #[error("Lock error occurred")]
    LockError,

    #[error("Thread join error")]
    ThreadJoinError,

    #[error("Invalid profiler state")]
    InvalidState,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_profiler_lifecycle() {
        let mut profiler = Profiler::new(ProfilerConfig::default());

        assert!(!profiler.is_running());

        profiler.start().unwrap();
        assert!(profiler.is_running());

        thread::sleep(Duration::from_millis(200));
        assert!(profiler.sample_count() > 0);

        let result = profiler.stop().unwrap();
        assert!(!profiler.is_running());
        assert!(result.samples.is_empty());
        assert!(result.summary.duration > Duration::from_millis(100));
    }

    #[test]
    fn test_profiler_config() {
        let config = ProfilerConfig {
            sampling_interval: Duration::from_millis(50),
            max_samples: 100,
            track_memory: true,
            track_cpu: false,
            track_allocation_patterns: true,
            auto_report_interval: None,
        };

        let mut profiler = Profiler::new(config);
        profiler.start().unwrap();
        thread::sleep(Duration::from_millis(100));
        let result = profiler.stop().unwrap();

        // Should have collected samples at 50ms interval
        assert!(!result.samples.is_empty());

        // Memory tracking should be enabled
        assert!(result.samples.iter().any(|s| s.memory_usage.is_some()));

        // CPU tracking should be disabled
        assert!(result.samples.iter().all(|s| s.cpu_usage.is_none()));
    }

    #[test]
    fn test_hotspot_detection() {
        let samples = vec![ProfileSample {
            timestamp: Instant::now(),
            memory_usage: Some(1024),
            allocation_rate: Some(2000.0), // High rate
            cpu_usage: Some(0.9),          // High CPU
            active_threads: Some(4),
            custom_metrics: HashMap::new(),
        }];

        let memory_stats = MemoryProfileStats {
            peak_usage: 1024,
            average_usage: 1024.0,
            min_usage: 1024,
            growth_rate: 5.0 * 1024.0 * 1024.0, // High growth
            fragmentation_estimate: 0.1,
        };

        let allocation_stats = AllocationProfileStats {
            peak_rate: 2000.0,
            average_rate: 1500.0,
            total_allocations: 10000,
            allocation_spikes: Vec::new(),
        };

        let cpu_stats = CpuProfileStats {
            peak_usage: 0.9,
            average_usage: 0.7,
            cpu_spikes: Vec::new(),
        };

        let hotspots = Profiler::detect_hotspots(
            &samples,
            &Some(memory_stats),
            &Some(allocation_stats),
            &Some(cpu_stats),
        );

        // Should detect multiple hotspots
        assert!(hotspots.len() >= 2);
        assert!(hotspots
            .iter()
            .any(|h| h.metric_name == "memory_growth_rate"));
        assert!(hotspots.iter().any(|h| h.metric_name == "allocation_rate"));
        assert!(hotspots.iter().any(|h| h.metric_name == "cpu_usage"));
    }
}
