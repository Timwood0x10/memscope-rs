//! View Module - Unified read-only access to memory data.
//!
//! This module provides MemoryView which serves as the single source
//! of truth for all analysis modules. It reuses MemorySnapshot to
//! avoid duplicate allocation rebuilding.

mod filters;
mod memory_view;
mod stats;

pub use filters::{FilterBuilder, ViewFilter};
pub use memory_view::MemoryView;
pub use stats::ViewStats;
