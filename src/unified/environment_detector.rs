// Runtime Environment Detection System
// Intelligently detects execution context to optimize memory tracking strategy
// Supports single-thread, multi-thread, async, and hybrid runtime detection

use crate::unified::backend::{AsyncRuntimeType, BackendError, RuntimeEnvironment};
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Advanced environment detector with runtime analysis capabilities
/// Provides deep inspection of execution context for optimal tracking strategy selection
pub struct EnvironmentDetector {
    /// Detection configuration parameters
    config: DetectionConfig,
    /// Runtime statistics collector
    runtime_stats: RuntimeStatistics,
}

/// Configuration for environment detection behavior
#[derive(Debug, Clone)]
pub struct DetectionConfig {
    /// Enable deep async runtime analysis
    pub deep_async_detection: bool,
    /// Sampling period for runtime analysis (milliseconds)
    pub analysis_period_ms: u64,
    /// Threshold for considering environment as multi-threaded
    pub multi_thread_threshold: usize,
    /// Maximum time to spend on detection
    pub max_detection_time_ms: u64,
}

/// Runtime statistics collected during detection
#[derive(Debug, Clone)]
pub struct RuntimeStatistics {
    /// Active thread count observed
    pub active_threads: Arc<AtomicUsize>,
    /// Async task count (if detectable)
    pub async_tasks: Arc<AtomicUsize>,
    /// Peak thread utilization
    pub peak_thread_count: usize,
    /// Detection duration
    pub detection_duration_ms: u64,
}

/// Detailed environment analysis result
#[derive(Debug, Clone)]
pub struct EnvironmentAnalysis {
    /// Primary detected environment
    pub environment: RuntimeEnvironment,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Alternative environments considered
    pub alternatives: Vec<RuntimeEnvironment>,
    /// Detection metadata
    pub detection_metadata: DetectionMetadata,
}

/// Metadata about detection process
#[derive(Debug, Clone)]
pub struct DetectionMetadata {
    /// Time spent on detection
    pub detection_time_ms: u64,
    /// Number of samples taken
    pub sample_count: usize,
    /// Detection method used
    pub method: DetectionMethod,
    /// Warnings or issues during detection
    pub warnings: Vec<String>,
}

/// Method used for environment detection
#[derive(Debug, Clone, PartialEq)]
pub enum DetectionMethod {
    /// Static analysis only
    Static,
    /// Runtime sampling
    Dynamic,
    /// Combined static and dynamic
    Hybrid,
    /// Forced by configuration
    Manual,
}

impl Default for DetectionConfig {
    /// Default detection configuration optimized for accuracy and performance
    fn default() -> Self {
        Self {
            deep_async_detection: true,
            analysis_period_ms: 100,
            multi_thread_threshold: 2,
            max_detection_time_ms: 500,
        }
    }
}

impl Default for RuntimeStatistics {
    /// Initialize runtime statistics with zero values
    fn default() -> Self {
        Self {
            active_threads: Arc::new(AtomicUsize::new(1)),
            async_tasks: Arc::new(AtomicUsize::new(0)),
            peak_thread_count: 1,
            detection_duration_ms: 0,
        }
    }
}

impl EnvironmentDetector {
    /// Create new environment detector with configuration
    pub fn new(config: DetectionConfig) -> Self {
        debug!("Creating environment detector with config: {:?}", config);

        Self {
            config,
            runtime_stats: RuntimeStatistics::default(),
        }
    }

    /// Perform comprehensive environment detection and analysis
    /// Returns detailed analysis with confidence levels and alternatives
    pub fn analyze_environment(&mut self) -> Result<EnvironmentAnalysis, BackendError> {
        let start_time = std::time::Instant::now();
        info!("Starting comprehensive environment analysis");

        let mut warnings = Vec::new();

        // Phase 1: Static analysis
        let static_env = self.perform_static_analysis(&mut warnings)?;
        debug!("Static analysis result: {:?}", static_env);

        // Phase 2: Dynamic runtime analysis (if enabled)
        let dynamic_env = if self.config.deep_async_detection {
            Some(self.perform_dynamic_analysis(&mut warnings)?)
        } else {
            None
        };

        // Phase 3: Combine results and calculate confidence
        let has_dynamic = dynamic_env.is_some();
        let (final_env, confidence, alternatives) =
            self.synthesize_results(static_env, dynamic_env, &warnings)?;

        let detection_time = start_time.elapsed().as_millis() as u64;
        self.runtime_stats.detection_duration_ms = detection_time;

        let analysis = EnvironmentAnalysis {
            environment: final_env,
            confidence,
            alternatives,
            detection_metadata: DetectionMetadata {
                detection_time_ms: detection_time,
                sample_count: self.calculate_sample_count(),
                method: self.determine_detection_method(has_dynamic),
                warnings,
            },
        };

        info!(
            "Environment analysis completed: {:?} (confidence: {:.2})",
            analysis.environment, analysis.confidence
        );

        Ok(analysis)
    }

    /// Perform static environment analysis based on available system information
    fn perform_static_analysis(
        &self,
        warnings: &mut Vec<String>,
    ) -> Result<RuntimeEnvironment, BackendError> {
        debug!("Performing static environment analysis");

        // Detect available parallelism
        let logical_cores = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or_else(|e| {
                warnings.push(format!("Could not detect CPU cores: {}", e));
                1
            });

        debug!("Detected {} logical CPU cores", logical_cores);

        // Check for async runtime indicators
        let async_runtime = self.detect_async_runtime_static(warnings);

        // Determine base environment from static analysis
        let environment = match (async_runtime, logical_cores) {
            (Some(runtime_type), 0) => {
                warnings.push("Zero cores detected with async runtime".to_string());
                RuntimeEnvironment::AsyncRuntime { runtime_type }
            }
            (Some(runtime_type), 1) => RuntimeEnvironment::AsyncRuntime { runtime_type },
            (Some(_runtime_type), cores) if cores >= self.config.multi_thread_threshold => {
                RuntimeEnvironment::Hybrid {
                    thread_count: cores,
                    async_task_count: 0, // Will be determined dynamically
                }
            }
            (Some(runtime_type), _cores) => {
                // Low core count with async runtime
                RuntimeEnvironment::AsyncRuntime { runtime_type }
            }
            (None, 1) => RuntimeEnvironment::SingleThreaded,
            (None, cores) if cores >= self.config.multi_thread_threshold => {
                RuntimeEnvironment::MultiThreaded {
                    thread_count: cores,
                }
            }
            (None, cores) => {
                warnings.push(format!("Low core count {} but above single-thread", cores));
                RuntimeEnvironment::SingleThreaded
            }
        };

        debug!("Static analysis determined environment: {:?}", environment);
        Ok(environment)
    }

    /// Detect async runtime presence using static indicators
    fn detect_async_runtime_static(&self, warnings: &mut Vec<String>) -> Option<AsyncRuntimeType> {
        debug!("Detecting async runtime using static analysis");

        // Check for Tokio runtime
        if self.is_tokio_runtime_present() {
            debug!("Tokio runtime detected via static analysis");
            return Some(AsyncRuntimeType::Tokio);
        }

        // Check for async-std runtime
        if self.is_async_std_runtime_present() {
            debug!("async-std runtime detected via static analysis");
            return Some(AsyncRuntimeType::AsyncStd);
        }

        // Check environment variables for async indicators
        if let Ok(async_env) = std::env::var("ASYNC_RUNTIME") {
            match async_env.to_lowercase().as_str() {
                "tokio" => {
                    debug!("Tokio runtime detected via environment variable");
                    return Some(AsyncRuntimeType::Tokio);
                }
                "async-std" => {
                    debug!("async-std runtime detected via environment variable");
                    return Some(AsyncRuntimeType::AsyncStd);
                }
                other => {
                    warnings.push(format!("Unknown async runtime specified: {}", other));
                    return Some(AsyncRuntimeType::Custom);
                }
            }
        }

        debug!("No async runtime detected in static analysis");
        None
    }

    /// Check for Tokio runtime presence using available detection methods
    fn is_tokio_runtime_present(&self) -> bool {
        // Method 1: Check for Tokio environment variables
        if std::env::var("TOKIO_WORKER_THREADS").is_ok() {
            return true;
        }

        // Note: tokio::runtime::Handle::try_current() requires tokio dependency
        // Would be enabled when tokio feature is available

        false
    }

    /// Check for async-std runtime presence
    fn is_async_std_runtime_present(&self) -> bool {
        // async-std detection is more challenging as it has fewer runtime introspection APIs
        // Check for async-std specific environment variables or patterns

        // Method 1: Check for async-std environment indicators
        if std::env::var("ASYNC_STD_THREAD_COUNT").is_ok() {
            return true;
        }

        // Method 2: Check if we're running inside async-std executor
        // This would require async-std specific detection logic

        false
    }

    /// Perform dynamic runtime analysis through sampling and observation
    fn perform_dynamic_analysis(
        &mut self,
        warnings: &mut Vec<String>,
    ) -> Result<RuntimeEnvironment, BackendError> {
        debug!("Performing dynamic runtime analysis");

        let analysis_start = std::time::Instant::now();
        let max_duration = std::time::Duration::from_millis(self.config.max_detection_time_ms);

        // Sample runtime characteristics over time
        let mut sample_count = 0;
        let mut thread_samples = Vec::new();
        let mut async_indicators = Vec::new();

        while analysis_start.elapsed() < max_duration {
            // Sample current thread activity
            let current_threads = self.sample_thread_activity();
            thread_samples.push(current_threads);

            // Sample async task activity (if possible)
            let async_activity = self.sample_async_activity();
            async_indicators.push(async_activity);

            sample_count += 1;

            // Sleep for sampling interval
            std::thread::sleep(std::time::Duration::from_millis(
                self.config.analysis_period_ms / 10,
            ));
        }

        // Analyze collected samples
        let avg_threads = if thread_samples.is_empty() {
            1
        } else {
            thread_samples.iter().sum::<usize>() / thread_samples.len()
        };

        let peak_threads = thread_samples.into_iter().max().unwrap_or(1);
        self.runtime_stats.peak_thread_count = peak_threads;

        let has_async_activity = async_indicators.iter().any(|&active| active);

        debug!(
            "Dynamic analysis: avg_threads={}, peak_threads={}, async_activity={}",
            avg_threads, peak_threads, has_async_activity
        );

        // Determine environment from dynamic analysis
        let environment = match (has_async_activity, peak_threads) {
            (true, 0) => {
                warnings.push("Async activity detected with zero threads".to_string());
                let runtime_type = self
                    .detect_async_runtime_static(warnings)
                    .unwrap_or(AsyncRuntimeType::Custom);
                RuntimeEnvironment::AsyncRuntime { runtime_type }
            }
            (true, 1) => {
                // Async activity on single thread suggests async runtime
                let runtime_type = self
                    .detect_async_runtime_static(warnings)
                    .unwrap_or(AsyncRuntimeType::Custom);
                RuntimeEnvironment::AsyncRuntime { runtime_type }
            }
            (true, threads) => {
                // Both async and multi-thread activity
                RuntimeEnvironment::Hybrid {
                    thread_count: threads,
                    async_task_count: sample_count, // Use sample count as proxy
                }
            }
            (false, 1) => RuntimeEnvironment::SingleThreaded,
            (false, threads) => RuntimeEnvironment::MultiThreaded {
                thread_count: threads,
            },
        };

        debug!("Dynamic analysis determined environment: {:?}", environment);
        Ok(environment)
    }

    /// Sample current thread activity level
    fn sample_thread_activity(&self) -> usize {
        // This is a simplified implementation
        // Real implementation would use platform-specific APIs to count active threads

        // For now, use available parallelism as baseline
        std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1)
    }

    /// Sample async task activity
    fn sample_async_activity(&self) -> bool {
        // This would use runtime-specific APIs to detect async task activity
        // For now, use simple heuristics

        // Check if we can detect any async runtime activity
        self.is_tokio_runtime_present() || self.is_async_std_runtime_present()
    }

    /// Synthesize static and dynamic analysis results
    fn synthesize_results(
        &self,
        static_result: RuntimeEnvironment,
        dynamic_result: Option<RuntimeEnvironment>,
        warnings: &[String],
    ) -> Result<(RuntimeEnvironment, f64, Vec<RuntimeEnvironment>), BackendError> {
        debug!("Synthesizing analysis results");

        let mut alternatives = Vec::new();
        let _base_confidence = 0.7; // Base confidence

        let (final_environment, mut confidence) = match dynamic_result {
            Some(dynamic_env) => {
                // Compare static and dynamic results
                if std::mem::discriminant(&static_result) == std::mem::discriminant(&dynamic_env) {
                    // Results agree - high confidence
                    let confidence = 0.95;
                    alternatives.push(static_result);
                    (dynamic_env, confidence)
                } else {
                    // Results disagree - medium confidence, prefer dynamic
                    let confidence = 0.75;
                    alternatives.push(static_result);
                    (dynamic_env, confidence)
                }
            }
            None => {
                // Only static analysis available
                let confidence = 0.80;
                (static_result, confidence)
            }
        };

        // Adjust confidence based on warnings
        if !warnings.is_empty() {
            confidence -= 0.1 * warnings.len() as f64;
            confidence = confidence.max(0.3); // Minimum confidence threshold
        }

        debug!(
            "Final synthesis: {:?} with confidence {:.2}",
            final_environment, confidence
        );
        Ok((final_environment, confidence, alternatives))
    }

    /// Calculate total number of samples taken during analysis
    fn calculate_sample_count(&self) -> usize {
        // This would be tracked during dynamic analysis
        // For now, estimate based on detection duration
        let samples = self.runtime_stats.detection_duration_ms / self.config.analysis_period_ms;
        samples.max(1) as usize
    }

    /// Determine which detection method was primarily used
    fn determine_detection_method(&self, used_dynamic: bool) -> DetectionMethod {
        if used_dynamic {
            DetectionMethod::Hybrid
        } else {
            DetectionMethod::Static
        }
    }

    /// Get current runtime statistics
    pub fn runtime_statistics(&self) -> &RuntimeStatistics {
        &self.runtime_stats
    }
}

/// Convenience function for quick environment detection
/// Uses default configuration for most common use cases
pub fn detect_environment() -> Result<RuntimeEnvironment, BackendError> {
    let mut detector = EnvironmentDetector::new(DetectionConfig::default());
    let analysis = detector.analyze_environment()?;

    if analysis.confidence < 0.5 {
        warn!(
            "Low confidence environment detection: {:.2}",
            analysis.confidence
        );
    }

    Ok(analysis.environment)
}

/// Advanced environment detection with custom configuration
/// Provides detailed analysis results for advanced use cases
pub fn detect_environment_detailed(
    config: DetectionConfig,
) -> Result<EnvironmentAnalysis, BackendError> {
    let mut detector = EnvironmentDetector::new(config);
    detector.analyze_environment()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let config = DetectionConfig::default();
        let detector = EnvironmentDetector::new(config);
        assert_eq!(detector.runtime_stats.peak_thread_count, 1);
    }

    #[test]
    fn test_static_analysis() {
        let config = DetectionConfig::default();
        let detector = EnvironmentDetector::new(config);
        let mut warnings = Vec::new();

        let result = detector.perform_static_analysis(&mut warnings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_tokio_detection() {
        let config = DetectionConfig::default();
        let detector = EnvironmentDetector::new(config);

        // This test will pass regardless of Tokio presence
        let _has_tokio = detector.is_tokio_runtime_present();
        // Just ensure the method doesn't panic
    }

    #[test]
    fn test_environment_analysis_confidence() {
        let mut detector = EnvironmentDetector::new(DetectionConfig::default());
        let analysis = detector.analyze_environment();

        assert!(analysis.is_ok());
        let analysis = analysis.unwrap();
        assert!(analysis.confidence >= 0.0 && analysis.confidence <= 1.0);
    }

    #[test]
    fn test_quick_detection() {
        let result = detect_environment();
        assert!(result.is_ok());
    }

    #[test]
    fn test_detailed_detection() {
        let config = DetectionConfig {
            deep_async_detection: false, // Disable for faster test
            max_detection_time_ms: 50,   // Short timeout for test
            ..Default::default()
        };

        let result = detect_environment_detailed(config);
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert!(analysis.detection_metadata.detection_time_ms <= 100); // Should be quick
    }
}
