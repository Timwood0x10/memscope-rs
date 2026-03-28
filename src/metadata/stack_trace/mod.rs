//! Enhanced stack trace capture and analysis
//!
//! Provides high-performance stack trace collection with:
//! - Configurable depth control
//! - Symbol resolution caching
//! - Hot path optimization
//! - Thread-local caching

pub mod cache;
pub mod capture;
pub mod resolver;

pub use cache::{CacheStats, StackTraceCache};
pub use capture::{CaptureConfig, StackFrame, StackTraceCapture};
pub use resolver::{ResolvedFrame, SymbolResolver};
