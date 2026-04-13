//! Capture Engine - Event capture backend
//!
//! This module provides the CaptureEngine which is responsible for
//! capturing memory events from the application and forwarding them
//! to the EventStore. The CaptureEngine does not store events itself;
//! it only captures and forwards them.

pub mod backends;
pub mod engine;
pub mod inference;
pub mod platform;
pub mod system_monitor;
pub mod types;

pub use backends::{CaptureBackend, CaptureBackendType};
pub use engine::CaptureEngine;

// Re-export inference types
pub use inference::{
    Confidence, DataSource, InferenceEngine, InferenceRule, InferredBorrowInfo,
    InferredSmartPointerInfo, SmartPointerType as InferredSmartPointerType,
};

// Re-export common types for convenience
pub use types::{
    AllocationInfo, BorrowInfo, CloneInfo, MemoryStats, SmartPointerInfo, SmartPointerType,
    TrackingError, TrackingResult,
};
