//! Dashboard HTML rendering module
//!
//! This module provides HTML dashboard rendering functionality using Handlebars templates.

pub mod renderer;

pub use renderer::{rebuild_allocations_from_events, DashboardContext, DashboardRenderer};
