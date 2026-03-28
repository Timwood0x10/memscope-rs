//! Query Engine - Unified query interface
//!
//! This module provides the QueryEngine which is responsible for
//! querying snapshot data.

pub mod engine;
pub mod types;

pub use engine::{QueryEngine, SharedQueryEngine};
pub use types::{
    AllocationQueryResult, QueryResult, SummaryQueryResult, ThreadQueryResult,
};