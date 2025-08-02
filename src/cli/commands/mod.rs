//! CLI command implementations
//!
//! This module contains the implementation of various CLI commands:
//! - analyze: Memory analysis command
//! - generate_report: Report generation command
//! - test: Testing utilities
//! - convert: Format conversion command
//! - binary_info: Binary file information command
//! - binary_validate: Binary file validation command

pub mod analyze;
pub mod binary_info;
pub mod binary_validate;
pub mod convert;
pub mod generate_report;
pub mod html_from_json;
pub mod test;

// Re-export command functions
pub use analyze::run_analyze;
pub use binary_info::run_binary_info;
pub use binary_validate::run_binary_validate;
pub use convert::run_convert;
pub use generate_report::run_generate_report;
pub use html_from_json::run_html_from_json;
pub use test::run_test;
