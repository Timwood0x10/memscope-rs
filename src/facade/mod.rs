//! Facade API - Unified user interface
//!
//! This module provides the MemScope facade which unifies all engines
//! into a simple, easy-to-use interface.

pub mod compat;
pub mod facade;
pub mod macros;

pub use compat::{
    clear_all, export_json, get_global_memscope, get_memory_summary, get_top_allocations,
    get_variable_info, register_variable,
};
pub use facade::MemScope;