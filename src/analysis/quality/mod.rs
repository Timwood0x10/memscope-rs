//! Code quality assurance and validation system
//!
//! Provides automated quality checks, code analysis, and validation
//! specifically for memory analysis operations. Ensures reliability
//! and performance standards are maintained.

pub mod analyzer;
pub mod checker;
pub mod validator;

pub use analyzer::{AnalysisReport, CodeAnalyzer, QualityMetric};
pub use checker::{MemoryLeakChecker, PerformanceChecker, SafetyChecker};
pub use validator::{QualityValidator, ValidationResult, ValidationRule};
