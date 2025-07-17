// Types module - Refactored from the original monolithic types.rs
// This module organizes types into logical groups for better maintainability

// For now, re-export everything from the original types.rs to maintain compatibility
// We'll gradually move types into separate modules

// Re-export all types from the main types file for backward compatibility
pub use crate::types_original::*;

// TODO: Gradually move types to these modules:
// pub mod core;
// pub mod allocation; 
// pub mod visualization;
// pub mod analysis;