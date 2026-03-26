//! New unified architecture for memory tracking and analysis
//!
//! This module contains the refactored memory tracking system that consolidates
//! the 4 existing tracking systems, 17 analyzers, and 28 export modules into a
//! unified, simplified architecture while preserving all functionality.
//!
//! ## Module Structure
//!
//! - `types` - Core type definitions and unified data structures
//! - `tracker` - Unified tracking system with 4 backend strategies
//! - `analysis` - 5 core analyzers consolidating 17 existing analyzers
//! - `export` - 5 core exporters consolidating 28 existing export modules
//! - `adapters` - Compatibility layer for integrating old modules
//! - `strategies` - Tracking strategy implementations
//! - `async_memory` - Async task memory tracking
//! - `unified` - Unified backend components
//!
//! ## Migration Status
//!
//! - ✅ Phase 1: Core type definitions - COMPLETED
//! - ✅ Phase 2: Unified tracking core - COMPLETED
//! - ✅ Phase 3: Unified analysis interface - COMPLETED
//! - ✅ Phase 4: Unified export system - COMPLETED
//! - ✅ Phase 5: Adapter layer - COMPLETED
//! - ✅ Phase 6: File migration - COMPLETED
//! - ⏳ Phase 7: Module integration - IN PROGRESS
//! - ⏳ Phase 8: Deprecation - PENDING

// Re-export core modules
pub mod adapters;
pub mod analysis;
pub mod export;
pub mod tracker;
pub mod types;

// Re-export strategy modules
pub mod strategies;

// TODO: Async memory modules - to be integrated later
// pub mod async_memory;

// Re-export unified modules
pub mod unified;

// Re-export key components for convenience
pub use types::{
    Allocation, AllocationMeta, Event, MemoryPassport, MemorySource, PassportEventType,
    PassportStatus, SmartPointerInfo, Snapshot, Stats, TrackingError, TrackingResult,
};

pub use types::Event as EventType;

pub use tracker::{
    AllocationContext, AsyncBackend, HybridBackend, SingleThreadBackend, ThreadLocalBackend,
    TrackingAllocator, TrackingBackend, TrackingConfig, TrackingStrategy, UnifiedStorage,
    UnifiedTracker,
};

pub use analysis::{
    Analyzer, CompositeAnalyzer, FragmentationAnalyzer, LeakAnalyzer, LifecycleAnalyzer,
    SafetyAnalyzer, SmartPointerAnalyzer,
};

pub use export::{
    BinaryExporter, CompositeExporter, CsvExporter, ExportBackend, ExportConfig, ExportError,
    ExportFormat, ExportOutput, HtmlGenerator, JsonExporter,
};

pub use adapters::{AnalysisAdapter, ExportAdapter, MemoryTrackerAdapter};
