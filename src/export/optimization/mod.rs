//! Export performance optimization.

pub mod compression;
pub mod streaming;
pub mod parallel;
pub mod adaptive_performance;
pub mod config_optimizer;
pub mod system_optimizer;
pub mod streaming_json_writer;
pub mod high_speed_buffered_writer;
pub mod parallel_shard_processor;
pub mod fast_export_coordinator;
pub mod performance_benchmark;
pub mod performance_comparison;

// Re-export optimization types
pub use compression::*;
pub use streaming::*;
pub use parallel::*;
// pub use adaptive_performance::*;  // Commented out to fix unused import warning
// pub use config_optimizer::*;  // Commented out to fix unused import warning
// pub use system_optimizer::*;  // Commented out to fix unused import warning
// pub use streaming_json_writer::*;  // Commented out to fix unused import warning
// pub use high_speed_buffered_writer::*;  // Commented out to fix unused import warning
// pub use parallel_shard_processor::*;  // Commented out to fix unused import warning
// pub use fast_export_coordinator::*;  // Commented out to fix unused import warning
// pub use performance_benchmark::*;  // Commented out to fix unused import warning
// pub use performance_comparison::*;  // Commented out to fix unused import warning