//! Memory Analysis Detectors Module
//!
//! This module provides a unified interface for memory issue detection.
//! All detectors implement the `Detector` trait and can be registered
//! with the `AnalysisManager` for comprehensive memory analysis.
//!
//! # Architecture
//!
//! The detector system is built around the following core concepts:
//!
//! - **Detector Trait**: Base trait that all detectors must implement
//! - **DetectionResult**: Standardized output format for all detectors
//! - **Issue**: Unified issue representation with severity and category
//! - **DetectorConfig**: Configuration interface for detector customization
//!
//! # Example
//!
//! ```rust,ignore
//! use memscope_rs::analysis::detectors::{Detector, LeakDetector, LeakDetectorConfig};
//!
//! fn main() {
//!     let config = LeakDetectorConfig::default();
//!     let detector = LeakDetector::new(config);
//!
//!     let allocations = vec![];
//!     let result = detector.detect(&allocations);
//!
//!     println!("Found {} issues", result.issues.len());
//! }
//! ```
//!
//! # Available Detectors
//!
//! - [`LeakDetector`](leak_detector::LeakDetector) - Memory leak detection
//! - [`UafDetector`](uaf_detector::UafDetector) - Use-after-free detection
//! - [`OverflowDetector`](overflow_detector::OverflowDetector) - Buffer overflow detection
//! - [`SafetyDetector`](safety_detector::SafetyDetector) - Unified safety violations
//! - [`LifecycleDetector`](lifecycle_detector::LifecycleDetector) - Lifecycle pattern detection
//! - [`DoubleFreeDetector`](double_free_detector::DoubleFreeDetector) - Double-free detection
//! - [`DataRaceDetector`](data_race_detector::DataRaceDetector) - Data race detection

pub mod types;

// Detector implementations
pub mod data_race_detector;
pub mod double_free_detector;
pub mod leak_detector;
pub mod lifecycle_detector;
pub mod overflow_detector;
pub mod safety_detector;
pub mod uaf_detector;

// Re-export core types
pub use types::{
    DetectionResult, DetectionStatistics, DetectorConfig, DetectorError, Issue, IssueCategory,
    IssueSeverity, Location,
};

// Re-export detectors
pub use data_race_detector::{DataRaceConfig, DataRaceDetector, DataRaceDetectorWithEvents};
pub use double_free_detector::{
    DoubleFreeConfig, DoubleFreeDetector, DoubleFreeDetectorWithEvents,
};
pub use leak_detector::{LeakDetector, LeakDetectorConfig};
pub use lifecycle_detector::{LifecycleDetector, LifecycleDetectorConfig};
pub use overflow_detector::{OverflowDetector, OverflowDetectorConfig};
pub use safety_detector::{SafetyDetector, SafetyDetectorConfig};
pub use uaf_detector::{UafDetector, UafDetectorConfig};

use crate::capture::types::AllocationInfo;
use std::fmt;

/// Base trait for all memory detectors
///
/// All detectors must implement this trait to provide a unified interface
/// for memory issue detection.
///
/// # Required Methods
///
/// - [`name()`](Detector::name) - Returns the detector name
/// - [`version()`](Detector::version) - Returns the detector version
/// - [`detect()`](Detector::detect) - Performs detection on allocations
/// - [`config()`](Detector::config) - Returns the current configuration
/// - [`update_config()`](Detector::update_config) - Updates the configuration
///
/// # Example
///
/// ```rust,ignore
/// use memscope_rs::analysis::detectors::{Detector, DetectionResult, DetectorConfig};
/// use memscope_rs::capture::types::AllocationInfo;
///
/// struct MyDetector {
///     config: DetectorConfig,
/// }
///
/// impl Detector for MyDetector {
///     fn name(&self) -> &str {
///         "MyDetector"
///     }
///
///     fn version(&self) -> &str {
///         "1.0.0"
///     }
///
///     fn detect(&self, allocations: &[AllocationInfo]) -> DetectionResult {
///         DetectionResult {
///             detector_name: self.name().to_string(),
///             issues: vec![],
///             statistics: DetectionStatistics::default(),
///             detection_time_ms: 0,
///         }
///     }
///
///     fn config(&self) -> &DetectorConfig {
///         &self.config
///     }
///
///     fn update_config(&mut self, config: DetectorConfig) -> Result<(), DetectorError> {
///         self.config = config;
///         Ok(())
///     }
/// }
/// ```
pub trait Detector: Send + Sync + std::fmt::Debug {
    /// Get detector name
    ///
    /// Returns a unique identifier for this detector.
    fn name(&self) -> &str;

    /// Get detector version
    ///
    /// Returns the version string following semantic versioning.
    fn version(&self) -> &str;

    /// Detect issues in allocations
    ///
    /// Analyzes the provided allocations and returns detected issues.
    ///
    /// # Arguments
    ///
    /// * `allocations` - Slice of allocation information to analyze
    ///
    /// # Returns
    ///
    /// A `DetectionResult` containing detected issues and statistics.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use memscope_rs::analysis::detectors::Detector;
    ///
    /// fn analyze_leaks<D: Detector>(detector: &D, allocations: &[AllocationInfo]) {
    ///     let result = detector.detect(allocations);
    ///     for issue in result.issues {
    ///         println!("Found issue: {}", issue.description);
    ///     }
    /// }
    /// ```
    fn detect(&self, allocations: &[AllocationInfo]) -> DetectionResult;

    /// Get configuration
    ///
    /// Returns a reference to the current detector configuration.
    fn config(&self) -> &DetectorConfig;

    /// Update configuration
    ///
    /// Updates the detector configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - New configuration to apply
    ///
    /// # Errors
    ///
    /// Returns a `DetectorError` if the configuration is invalid.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use memscope_rs::analysis::detectors::{Detector, DetectorConfig};
    ///
    /// fn configure_detector<D: Detector>(detector: &mut D) -> Result<(), DetectorError> {
    ///     let new_config = DetectorConfig {
    ///         enabled: true,
    ///         max_reported_issues: 100,
    ///         ..Default::default()
    ///     };
    ///     detector.update_config(new_config)
    /// }
    /// ```
    fn update_config(&mut self, config: DetectorConfig) -> Result<(), DetectorError>;
}

/// Detector registry for managing multiple detectors
///
/// Provides functionality to register, unregister, and run detectors.
#[derive(Debug, Default)]
pub struct DetectorRegistry {
    detectors: Vec<Box<dyn Detector>>,
}

impl DetectorRegistry {
    /// Create a new detector registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a detector
    ///
    /// # Arguments
    ///
    /// * `detector` - Boxed detector to register
    ///
    /// # Example
    ///
    /// ```rust
    /// use memscope_rs::analysis::detectors::{DetectorRegistry, LeakDetector, LeakDetectorConfig};
    ///
    /// let mut registry = DetectorRegistry::new();
    /// let detector = LeakDetector::new(LeakDetectorConfig::default());
    /// registry.register(Box::new(detector));
    /// ```
    pub fn register(&mut self, detector: Box<dyn Detector>) {
        self.detectors.push(detector);
    }

    /// Unregister a detector by name
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the detector to unregister
    ///
    /// # Returns
    ///
    /// `true` if a detector was found and removed, `false` otherwise.
    pub fn unregister(&mut self, name: &str) -> bool {
        let initial_len = self.detectors.len();
        self.detectors.retain(|d| d.name() != name);
        self.detectors.len() < initial_len
    }

    /// Get detector by name
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the detector to retrieve
    ///
    /// # Returns
    ///
    /// Option reference to the detector if found.
    pub fn get_detector(&self, name: &str) -> Option<&dyn Detector> {
        self.detectors
            .iter()
            .find(|d| d.name() == name)
            .map(|d| d.as_ref())
    }

    /// Run all registered detectors
    ///
    /// # Arguments
    ///
    /// * `allocations` - Allocations to analyze
    ///
    /// # Returns
    ///
    /// Vector of detection results from all detectors.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use memscope_rs::analysis::detectors::DetectorRegistry;
    ///
    /// fn run_all_analysis(registry: &DetectorRegistry, allocations: &[AllocationInfo]) {
    ///     let results = registry.run_all(allocations);
    ///     for result in results {
    ///         println!("{} found {} issues", result.detector_name, result.issues.len());
    ///     }
    /// }
    /// ```
    pub fn run_all(&self, allocations: &[AllocationInfo]) -> Vec<DetectionResult> {
        self.detectors
            .iter()
            .map(|detector| detector.detect(allocations))
            .collect()
    }

    /// Run a specific detector by name
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the detector to run
    /// * `allocations` - Allocations to analyze
    ///
    /// # Returns
    ///
    /// Option of detection result if the detector was found.
    pub fn run_detector(
        &self,
        name: &str,
        allocations: &[AllocationInfo],
    ) -> Option<DetectionResult> {
        self.get_detector(name)
            .map(|detector| detector.detect(allocations))
    }

    /// Get all registered detector names
    ///
    /// # Returns
    ///
    /// Vector of detector names.
    pub fn detector_names(&self) -> Vec<&str> {
        self.detectors.iter().map(|d| d.name()).collect()
    }

    /// Get count of registered detectors
    ///
    /// # Returns
    ///
    /// Number of registered detectors.
    pub fn len(&self) -> usize {
        self.detectors.len()
    }

    /// Check if registry is empty
    ///
    /// # Returns
    ///
    /// `true` if no detectors are registered.
    pub fn is_empty(&self) -> bool {
        self.detectors.is_empty()
    }
}

impl fmt::Display for DetectorRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DetectorRegistry({} detectors: {})",
            self.len(),
            self.detector_names().join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::types::AllocationInfo;

    #[test]
    fn test_detector_registry_new() {
        let registry = DetectorRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_detector_registry_register() {
        let mut registry = DetectorRegistry::new();
        assert_eq!(registry.len(), 0);

        // Create a simple test detector
        #[derive(Debug)]
        struct TestDetector {
            config: DetectorConfig,
        }

        impl Detector for TestDetector {
            fn name(&self) -> &str {
                "TestDetector"
            }

            fn version(&self) -> &str {
                "1.0.0"
            }

            fn detect(&self, _allocations: &[AllocationInfo]) -> DetectionResult {
                DetectionResult {
                    detector_name: self.name().to_string(),
                    issues: vec![],
                    statistics: DetectionStatistics::default(),
                    detection_time_ms: 0,
                }
            }

            fn config(&self) -> &DetectorConfig {
                &self.config
            }

            fn update_config(&mut self, config: DetectorConfig) -> Result<(), DetectorError> {
                self.config = config;
                Ok(())
            }
        }

        registry.register(Box::new(TestDetector {
            config: DetectorConfig::default(),
        }));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_detector_registry_unregister() {
        let mut registry = DetectorRegistry::new();

        #[derive(Debug)]
        struct TestDetector {
            config: DetectorConfig,
        }

        impl Detector for TestDetector {
            fn name(&self) -> &str {
                "TestDetector"
            }

            fn version(&self) -> &str {
                "1.0.0"
            }

            fn detect(&self, _allocations: &[AllocationInfo]) -> DetectionResult {
                DetectionResult {
                    detector_name: self.name().to_string(),
                    issues: vec![],
                    statistics: DetectionStatistics::default(),
                    detection_time_ms: 0,
                }
            }

            fn config(&self) -> &DetectorConfig {
                &self.config
            }

            fn update_config(&mut self, config: DetectorConfig) -> Result<(), DetectorError> {
                self.config = config;
                Ok(())
            }
        }

        registry.register(Box::new(TestDetector {
            config: DetectorConfig::default(),
        }));
        assert_eq!(registry.len(), 1);

        let removed = registry.unregister("TestDetector");
        assert!(removed);
        assert_eq!(registry.len(), 0);

        let not_removed = registry.unregister("NonExistent");
        assert!(!not_removed);
    }

    #[test]
    fn test_detector_registry_get_detector() {
        let mut registry = DetectorRegistry::new();

        #[derive(Debug)]
        struct TestDetector {
            config: DetectorConfig,
        }

        impl Detector for TestDetector {
            fn name(&self) -> &str {
                "TestDetector"
            }

            fn version(&self) -> &str {
                "1.0.0"
            }

            fn detect(&self, _allocations: &[AllocationInfo]) -> DetectionResult {
                DetectionResult {
                    detector_name: self.name().to_string(),
                    issues: vec![],
                    statistics: DetectionStatistics::default(),
                    detection_time_ms: 0,
                }
            }

            fn config(&self) -> &DetectorConfig {
                &self.config
            }

            fn update_config(&mut self, config: DetectorConfig) -> Result<(), DetectorError> {
                self.config = config;
                Ok(())
            }
        }

        registry.register(Box::new(TestDetector {
            config: DetectorConfig::default(),
        }));

        let detector = registry.get_detector("TestDetector");
        assert!(detector.is_some());
        assert_eq!(detector.unwrap().name(), "TestDetector");

        let not_found = registry.get_detector("NonExistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_detector_registry_run_all() {
        let mut registry = DetectorRegistry::new();

        #[derive(Debug)]
        struct TestDetector {
            config: DetectorConfig,
        }

        impl Detector for TestDetector {
            fn name(&self) -> &str {
                "TestDetector"
            }

            fn version(&self) -> &str {
                "1.0.0"
            }

            fn detect(&self, _allocations: &[AllocationInfo]) -> DetectionResult {
                DetectionResult {
                    detector_name: self.name().to_string(),
                    issues: vec![],
                    statistics: DetectionStatistics::default(),
                    detection_time_ms: 0,
                }
            }

            fn config(&self) -> &DetectorConfig {
                &self.config
            }

            fn update_config(&mut self, config: DetectorConfig) -> Result<(), DetectorError> {
                self.config = config;
                Ok(())
            }
        }

        registry.register(Box::new(TestDetector {
            config: DetectorConfig::default(),
        }));
        let allocations = vec![
            AllocationInfo::new(0x1000, 1024),
            AllocationInfo::new(0x2000, 2048),
        ];

        let results = registry.run_all(&allocations);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].detector_name, "TestDetector");
    }

    #[test]
    fn test_detector_registry_run_detector() {
        let mut registry = DetectorRegistry::new();

        #[derive(Debug)]
        struct TestDetector {
            config: DetectorConfig,
        }

        impl Detector for TestDetector {
            fn name(&self) -> &str {
                "TestDetector"
            }

            fn version(&self) -> &str {
                "1.0.0"
            }

            fn detect(&self, _allocations: &[AllocationInfo]) -> DetectionResult {
                DetectionResult {
                    detector_name: self.name().to_string(),
                    issues: vec![],
                    statistics: DetectionStatistics::default(),
                    detection_time_ms: 0,
                }
            }

            fn config(&self) -> &DetectorConfig {
                &self.config
            }

            fn update_config(&mut self, config: DetectorConfig) -> Result<(), DetectorError> {
                self.config = config;
                Ok(())
            }
        }

        registry.register(Box::new(TestDetector {
            config: DetectorConfig::default(),
        }));

        let allocations = vec![AllocationInfo::new(0x1000, 1024)];

        let result = registry.run_detector("TestDetector", &allocations);
        assert!(result.is_some());
        assert_eq!(result.unwrap().detector_name, "TestDetector");

        let not_found = registry.run_detector("NonExistent", &allocations);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_detector_registry_detector_names() {
        let mut registry = DetectorRegistry::new();

        #[derive(Debug)]
        struct TestDetector1 {
            config: DetectorConfig,
        }

        impl Detector for TestDetector1 {
            fn name(&self) -> &str {
                "TestDetector1"
            }

            fn version(&self) -> &str {
                "1.0.0"
            }

            fn detect(&self, _allocations: &[AllocationInfo]) -> DetectionResult {
                DetectionResult {
                    detector_name: self.name().to_string(),
                    issues: vec![],
                    statistics: DetectionStatistics::default(),
                    detection_time_ms: 0,
                }
            }

            fn config(&self) -> &DetectorConfig {
                &self.config
            }

            fn update_config(&mut self, config: DetectorConfig) -> Result<(), DetectorError> {
                self.config = config;
                Ok(())
            }
        }

        #[derive(Debug)]
        struct TestDetector2 {
            config: DetectorConfig,
        }

        impl Detector for TestDetector2 {
            fn name(&self) -> &str {
                "TestDetector2"
            }

            fn version(&self) -> &str {
                "1.0.0"
            }

            fn detect(&self, _allocations: &[AllocationInfo]) -> DetectionResult {
                DetectionResult {
                    detector_name: self.name().to_string(),
                    issues: vec![],
                    statistics: DetectionStatistics::default(),
                    detection_time_ms: 0,
                }
            }

            fn config(&self) -> &DetectorConfig {
                &self.config
            }

            fn update_config(&mut self, config: DetectorConfig) -> Result<(), DetectorError> {
                self.config = config;
                Ok(())
            }
        }

        registry.register(Box::new(TestDetector1 {
            config: DetectorConfig::default(),
        }));
        registry.register(Box::new(TestDetector2 {
            config: DetectorConfig::default(),
        }));

        let names = registry.detector_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"TestDetector1"));
        assert!(names.contains(&"TestDetector2"));
    }

    #[test]
    fn test_detector_registry_display() {
        let registry = DetectorRegistry::new();
        let display = format!("{}", registry);
        assert!(display.contains("DetectorRegistry"));
        assert!(display.contains("0 detectors"));

        let mut registry = DetectorRegistry::new();

        #[derive(Debug)]
        struct TestDetector {
            config: DetectorConfig,
        }

        impl Detector for TestDetector {
            fn name(&self) -> &str {
                "TestDetector"
            }

            fn version(&self) -> &str {
                "1.0.0"
            }

            fn detect(&self, _allocations: &[AllocationInfo]) -> DetectionResult {
                DetectionResult {
                    detector_name: self.name().to_string(),
                    issues: vec![],
                    statistics: DetectionStatistics::default(),
                    detection_time_ms: 0,
                }
            }

            fn config(&self) -> &DetectorConfig {
                &self.config
            }

            fn update_config(&mut self, config: DetectorConfig) -> Result<(), DetectorError> {
                self.config = config;
                Ok(())
            }
        }

        registry.register(Box::new(TestDetector {
            config: DetectorConfig::default(),
        }));

        let display = format!("{}", registry);
        assert!(display.contains("DetectorRegistry"));
        assert!(display.contains("1 detectors"));
        assert!(display.contains("TestDetector"));
    }
}
