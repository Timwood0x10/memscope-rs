//! Web server module for MemScope-RS
//!
//! This module provides a web server for interactive memory analysis,
//! similar to pprof but specialized for Rust memory tracking.

pub mod server;
pub mod handlers;
pub mod api;

pub use server::MemScopeServer;