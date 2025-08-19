
use crate::core::tracker::memory_tracker::MemoryTracker;
use crate::core::types::TrackingResult;
use std::path::Path;
use std::sync::Arc;

pub struct MemScopeExporter {
    tracker: Arc<MemoryTracker>,
}

#[derive(Debug, Clone)]
pub struct ExportConfig {
    pub include_system: bool,
    pub compress: bool,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            include_system: false,  
            compress: true, 
        }
    }
}

impl MemScopeExporter {
    pub fn new(tracker: Arc<MemoryTracker>) -> Self {
        Self { tracker }
    }

    pub fn export_json<P: AsRef<Path>>(&self, path: P) -> TrackingResult<()> {
        let allocations = self.tracker.get_active_allocations()?;
        let stats = self.tracker.get_stats()?;
        
        crate::export::unified_export_api::export_user_variables_json(
            allocations, stats, path
        )?;
        Ok(())
    }

    pub fn export_binary<P: AsRef<Path>>(&self, path: P) -> TrackingResult<()> {
        self.tracker.export_user_binary(path)
    }

    pub fn export_html<P: AsRef<Path>>(&self, path: P) -> TrackingResult<()> {
        self.tracker.export_memory_analysis(path)
    }

    pub fn export_auto<P: AsRef<Path>>(&self, path: P) -> TrackingResult<()> {
        let allocations = self.tracker.get_active_allocations()?;
        let count = allocations.len();
        
        if count < 1000 {
            self.export_json(path) 
        } else {
            self.export_binary(path) 
        }
    }
}

pub fn export_json<P: AsRef<Path>>(tracker: Arc<MemoryTracker>, path: P) -> TrackingResult<()> {
    MemScopeExporter::new(tracker).export_json(path)
}
pub fn export_binary<P: AsRef<Path>>(tracker: Arc<MemoryTracker>, path: P) -> TrackingResult<()> {
    MemScopeExporter::new(tracker).export_binary(path)
}
pub fn export_html<P: AsRef<Path>>(tracker: Arc<MemoryTracker>, path: P) -> TrackingResult<()> {
    MemScopeExporter::new(tracker).export_html(path)
}
pub fn export_auto<P: AsRef<Path>>(tracker: Arc<MemoryTracker>, path: P) -> TrackingResult<()> {
    MemScopeExporter::new(tracker).export_auto(path)
}