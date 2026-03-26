// BinaryExporter fix for bincode 2.x compatibility

use crate::types::internal_types::Snapshot;
use super::{ExportError, ExportFormat, ExportOutput, ExportMetadata, ExportBackend};

/// Helper function to export snapshot with bincode fallback to JSON
pub fn export_snapshot_with_fallback(snapshot: &Snapshot) -> Result<Vec<u8>, ExportError> {
    // Try bincode 2.x API first
    match bincode::encode_to_vec(snapshot, bincode::config::standard()) {
        Ok(binary) => Ok(binary),
        Err(_) => {
            // Fallback to JSON if bincode fails
            serde_json::to_vec(snapshot)
                .map_err(|e| ExportError::SerializationError(format!("JSON fallback failed: {e}")))
        }
    }
}