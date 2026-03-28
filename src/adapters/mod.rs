//! Adapter layer for bridging old and new tracking systems
//!
//! This module provides adapters to maintain backward compatibility while
//! migrating from the old tracking system (core/) to the new unified
//! tracking system (tracker/, data/, render/).
//!
//! # Architecture
//!
//! ```
//! Old System API
//!     │
//!     ▼
//! Adapter Layer (bridges old API to new system)
//!     │
//!     ▼
//! New Unified Tracking System
//! ```

pub mod async_adapter;
pub mod core_adapter;
pub mod lockfree_adapter;

// Re-export public adapter APIs
pub use async_adapter::AsyncAdapter;
pub use core_adapter::CoreAdapter;
pub use lockfree_adapter::LockfreeAdapter;
