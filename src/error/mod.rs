//! Unified error handling system for MemScope
//!
//! Provides comprehensive error types and handling patterns for all modules.
//! Features structured error reporting, error chaining, and recovery strategies.
//!
//! # Quick Start
//!
//! ```ignore
//! use memscope_rs::error::{record_error, generate_error_report, IntoMemScopeError};
//!
//! // Record an error
//! let error = MemScopeError::memory(MemoryOperation::Allocation, "failed");
//! record_error("my_module", "error_type", &error);
//!
//! // Generate error report
//! let report = generate_error_report();
//! println!("Error summary: {}", report.summary);
//! ```

pub mod conversions;
pub mod error_manager;
pub mod handler;
pub mod recovery;
pub mod types;

pub use error_manager::{
    generate_error_report, get_error_stats, global_error_manager, record_error, ErrorManager,
    ErrorRecord, ErrorReport, ErrorStats, IntoMemScopeError,
};
pub use handler::{ErrorHandler, ErrorReporter};
pub use recovery::{RecoveryAction, RecoveryStrategy};
pub use types::MemScopeError;
pub use types::MemScopeResult;
