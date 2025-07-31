//! Binary export format implementation.
//!
//! This module provides a compatibility layer for the old binary export
//! functionality while redirecting to the new unified binary export system.

// Re-export existing binary export functionality for compatibility
pub use super::binary_export::*;

/// Legacy binary export interface (deprecated)
/// 
/// This struct is maintained for backward compatibility.
/// New code should use `crate::export::binary::BinaryExporter` instead.
#[deprecated(note = "Use crate::export::binary::BinaryExporter instead")]
pub struct BinaryExporter {
    // Simple placeholder to avoid recursive type issues
}

impl BinaryExporter {
    /// Create a new binary exporter
    #[deprecated(note = "Use crate::export::binary::BinaryExporter::new() instead")]
    pub fn new() -> Self {
        Self {}
    }
    
    /// Export in simple binary format
    #[deprecated(note = "Use crate::export::binary::BinaryExport::export_default() instead")]
    pub fn export_simple<P: AsRef<std::path::Path>>(
        &self,
        _path: P,
        _data: &[crate::core::types::AllocationInfo],
    ) -> crate::core::types::TrackingResult<()> {
        // Placeholder implementation - redirect to new binary export system
        // TODO: Implement proper data conversion and use new binary export
        Ok(())
    }
    
    /// Export with complex type support
    #[deprecated(note = "Use crate::export::binary::BinaryExport::export_with_config() instead")]
    pub fn export_complex<P: AsRef<std::path::Path>>(
        &self,
        _path: P,
        _data: &[crate::core::types::AllocationInfo],
    ) -> crate::core::types::TrackingResult<()> {
        // Placeholder implementation - redirect to new binary export system
        // TODO: Implement proper data conversion and use new binary export
        Ok(())
    }
}