//! Query Engine - Unified query interface
//!
//! This module provides the QueryEngine which is responsible for
//! querying snapshot data.

pub mod engine;
pub mod types;
pub mod presets;

pub use engine::{QueryEngine, SharedQueryEngine};
pub use presets::QueryPresets;
pub use types::{
    AllocationQueryResult, Query, QueryResult, SummaryQueryResult, ThreadQueryResult,
};