//! Unified error handling system for MemScope
//!
//! Provides comprehensive error types and handling patterns for all modules.
//! Features structured error reporting, error chaining, and recovery strategies.

pub mod handler;
pub mod recovery;
pub mod types;

pub use handler::{ErrorHandler, ErrorReporter};
pub use recovery::{RecoveryAction, RecoveryStrategy};
pub use types::{ErrorContext, ErrorKind, ErrorSeverity, MemScopeError};
