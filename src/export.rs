// Export module - Consolidated from export_enhanced.rs, html_export.rs, optimized_html_export.rs, and report_generator.rs
// This module provides all export functionality including JSON, HTML, SVG, and enhanced reporting

use crate::types::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Main export interface - consolidates all export functionality
pub struct ExportManager {
    // This will contain the consolidated export functionality
}

impl ExportManager {
    /// Create a new export manager instance
    pub fn new() -> Self {
        Self {}
    }
    
    /// Export to JSON format (standard)
    pub fn export_json<P: AsRef<Path>>(
        &self,
        tracker: &crate::tracker::MemoryTracker,
        path: P,
    ) -> TrackingResult<()> {
        // Use the existing export_to_json method from MemoryTracker
        tracker.export_to_json(path)
    }
    
    /// Export to enhanced JSON format (with complete data)
    pub fn export_enhanced_json<P: AsRef<Path>>(
        &self,
        tracker: &crate::tracker::MemoryTracker,
        path: P,
    ) -> TrackingResult<()> {
        // Use the enhanced JSON export method we just added
        tracker.export_enhanced_json(path)
    }
    
    /// Export HTML dashboard
    pub fn export_html_dashboard<P: AsRef<Path>>(
        &self,
        tracker: &crate::tracker::MemoryTracker,
        path: P,
    ) -> TrackingResult<()> {
        // Use the existing HTML export functionality
        tracker.export_interactive_dashboard(path)
    }
    
    /// Export memory analysis SVG
    pub fn export_memory_analysis<P: AsRef<Path>>(
        &self,
        tracker: &crate::tracker::MemoryTracker,
        path: P,
    ) -> TrackingResult<()> {
        // Use the existing SVG export functionality
        tracker.export_memory_analysis(path)
    }
    
    /// Export lifecycle timeline SVG
    pub fn export_lifecycle_timeline<P: AsRef<Path>>(
        &self,
        tracker: &crate::tracker::MemoryTracker,
        path: P,
    ) -> TrackingResult<()> {
        // Use the existing lifecycle timeline export
        tracker.export_lifecycle_timeline(path)
    }
}

impl Default for ExportManager {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export all the existing export functions for backward compatibility
// This ensures that existing code continues to work without changes

/// Export to JSON format - backward compatibility function
pub fn export_to_json<P: AsRef<Path>>(path: P) -> TrackingResult<()> {
    let tracker = crate::get_global_tracker();
    let manager = ExportManager::new();
    manager.export_json(&tracker, path)
}

/// Export enhanced JSON - backward compatibility function
pub fn export_enhanced_json<P: AsRef<Path>>(path: P) -> TrackingResult<()> {
    let tracker = crate::get_global_tracker();
    let manager = ExportManager::new();
    manager.export_enhanced_json(&tracker, path)
}

/// Export HTML dashboard - backward compatibility function
pub fn export_html_dashboard<P: AsRef<Path>>(path: P) -> TrackingResult<()> {
    let tracker = crate::get_global_tracker();
    let manager = ExportManager::new();
    manager.export_html_dashboard(&tracker, path)
}

/// Export memory analysis SVG - backward compatibility function
pub fn export_memory_analysis<P: AsRef<Path>>(path: P) -> TrackingResult<()> {
    let tracker = crate::get_global_tracker();
    let manager = ExportManager::new();
    manager.export_memory_analysis(&tracker, path)
}

/// Export lifecycle timeline SVG - backward compatibility function
pub fn export_lifecycle_timeline<P: AsRef<Path>>(path: P) -> TrackingResult<()> {
    let tracker = crate::get_global_tracker();
    let manager = ExportManager::new();
    manager.export_lifecycle_timeline(&tracker, path)
}

// TODO: Gradually move the actual implementation from the individual files to this module
// For now, we're just creating the interface and delegating to the existing implementations