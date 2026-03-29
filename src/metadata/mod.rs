//! Metadata Engine - Centralized metadata management
//!
//! This module provides the MetadataEngine which is responsible for
//! managing all metadata including variables, scopes, threads, types,
//! and pointers across the memscope system.

pub mod engine;
pub mod registry;
pub mod scope;
pub mod smart_pointers;
pub mod stack_trace;
pub mod thread;

pub use engine::MetadataEngine;
