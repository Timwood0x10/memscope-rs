//! Export functionality for memory tracking data
//!
//! This module provides functions to export memory tracking data to various formats
//! such as JSON and SVG.

// use serde::Serialize; // Unused import
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

use svg::node::element::{Group, Line, Rectangle, Text as SvgText, Title as SvgTitle};
use svg::Document;

use crate::tracker::{AllocationInfo, HotspotInfo, MemoryTracker}; // Added HotspotInfo

/// Represents a snapshot of memory usage at a specific point in time.
///
/// This struct is primarily used for JSON export, capturing details about
/// active allocations and overall memory statistics when the snapshot is created.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)] // Added Deserialize
pub struct MemorySnapshot {
    /// ISO 8601 timestamp of when the snapshot was taken
    pub timestamp: String,

    /// Total number of active allocations
    pub total_allocations: usize,

    /// Total number of bytes allocated
    pub total_allocated: usize,

    /// Detailed information about active allocations at the time of the snapshot.
    pub active_allocations: Vec<AllocationInfo>,
    /// Optional list of memory allocation hotspots, if analysis is performed.
    #[serde(skip_serializing_if = "Option::is_none")] // Don't serialize if None
    pub allocation_hotspots: Option<Vec<HotspotInfo>>,
}

/// Export memory usage data to a JSON file
///
/// # Arguments
/// * `tracker` - The memory tracker containing the data to export
/// * `path` - The path where the JSON file should be saved
/// * `enable_sync` - Whether to force sync to disk (can be slow but ensures data is written)
///
/// # Returns
/// `io::Result<()>` indicating success or failure
pub fn export_to_json<P: AsRef<Path>>(
    tracker: &MemoryTracker,
    path: P,
    enable_sync: bool,
) -> io::Result<()> {
    let path = path.as_ref();
    let file_path = path.display().to_string();

    println!("[EXPORT] Starting export to: {}", file_path);
    let _start_time = std::time::Instant::now();

    // Print some debug info about the current state
    println!("[EXPORT] Starting export to: {}", file_path);
    println!(
        "[EXPORT] Number of active allocations: {}",
        tracker.get_active_allocations().len()
    );
    println!(
        "[EXPORT] Number of allocation log entries: {}",
        tracker.get_allocation_log().len()
    );

    // Print first few allocations for debugging
    let active_allocs = tracker.get_active_allocations();
    println!("[EXPORT] First few active allocations (up to 5):");
    for (i, alloc) in active_allocs.iter().enumerate().take(5) {
        println!(
            "  {}. ptr: {:#x}, size: {}, var: {:?}",
            i + 1,
            alloc.ptr,
            alloc.size,
            alloc.var_name.as_deref().unwrap_or("<unnamed>")
        );
    }

    // 1. Create parent directories if needed
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            #[cfg(not(test))]
            tracing::debug!("Creating parent directory: {}", parent.display());
            std::fs::create_dir_all(parent).map_err(|e| {
                let msg = format!("Failed to create directory {}: {}", parent.display(), e);
                #[cfg(not(test))]
                tracing::error!("{}", msg);
                io::Error::new(io::ErrorKind::Other, msg)
            })?;
        }
    }

    // 2. Create snapshot with timeout protection
    println!("[EXPORT] Starting snapshot creation...");
    let snapshot_start = std::time::Instant::now();

    // Get the data we need while holding the lock
    let active_allocations = tracker.get_active_allocations();
    let allocation_log = tracker.get_allocation_log();

    // Create snapshot in the current thread with a timeout
    // We'll use a channel to communicate the result from the thread
    let (tx, rx) = std::sync::mpsc::channel();

    // Spawn a thread to create the snapshot
    let _thread = std::thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            create_snapshot_from_cloned_data(active_allocations, allocation_log)
        }));
        let _ = tx.send(result);
    });

    // Wait for the result with a timeout
    let snapshot = match rx.recv_timeout(std::time::Duration::from_secs(30)) {
        Ok(result) => match result {
            Ok(snapshot) => snapshot,
            Err(panic) => {
                let panic_msg = if let Some(s) = panic.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic during snapshot creation".to_string()
                };
                println!("[EXPORT] Panic creating snapshot: {}", panic_msg);
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Snapshot creation panicked: {}", panic_msg),
                ));
            }
        },
        Err(_) => {
            println!("[EXPORT] Timeout creating snapshot (took >30s)");
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Snapshot creation timed out after 30 seconds",
            ));
        }
    };

    println!(
        "[EXPORT] Snapshot created in {:?}",
        snapshot_start.elapsed()
    );

    println!(
        "[EXPORT] Snapshot contains {} active allocations",
        snapshot.active_allocations.len()
    );

    println!("[EXPORT] Serializing snapshot to JSON...");
    let json_start = std::time::Instant::now();

    // Serialize the snapshot
    let json = match serde_json::to_string_pretty(&snapshot) {
        Ok(json) => {
            let json_size = json.len() as f64 / 1024.0 / 1024.0; // Convert to MB
            println!(
                "[EXPORT] JSON serialized in {:?}, size: {:.2} MB",
                json_start.elapsed(),
                json_size
            );
            json
        }
        Err(e) => {
            let msg = format!("Failed to serialize snapshot: {}", e);
            println!("[EXPORT] {}", msg);
            return Err(io::Error::new(io::ErrorKind::InvalidData, msg));
        }
    };

    // 3. Write to a temporary file first
    let temp_path = format!("{}.tmp", file_path);
    println!("[EXPORT] Writing to temporary file: {}", temp_path);

    {
        let mut file = File::create(&temp_path).map_err(|e| {
            let msg = format!("Failed to create temporary file {}: {}", temp_path, e);
            #[cfg(not(test))]
            tracing::error!("{}", msg);
            io::Error::new(e.kind(), msg)
        })?;

        // Write the JSON data in chunks to show progress for large files
        const CHUNK_SIZE: usize = 8 * 1024; // 8KB chunks
        let mut bytes_written = 0;
        let total_bytes = json.len();

        for chunk in json.as_bytes().chunks(CHUNK_SIZE) {
            file.write_all(chunk).map_err(|e| {
                let _ = std::fs::remove_file(&temp_path);
                let msg = format!("Failed to write to file {}: {}", temp_path, e);
                io::Error::new(e.kind(), msg)
            })?;

            // Update progress
            bytes_written += chunk.len();
            if total_bytes > 0 {
                let percent = (bytes_written as f64 / total_bytes as f64 * 100.0) as u32;
                if percent % 10 == 0 {
                    // Log progress every 10%
                    println!(
                        "[EXPORT] Export progress: {}% ({} bytes)",
                        percent, bytes_written
                    );
                }
            }
        }

        // Optional: Flush and sync to disk
        if enable_sync {
            #[cfg(not(test))]
            tracing::debug!("Syncing file to disk...");

            file.sync_all().map_err(|e| {
                let _ = std::fs::remove_file(&temp_path);
                let msg = format!("Failed to sync file {}: {}", temp_path, e);
                #[cfg(not(test))]
                tracing::error!("{}", msg);
                io::Error::new(e.kind(), msg)
            })?;
        } else {
            // Just flush the buffers without waiting for disk sync
            file.flush().map_err(|e| {
                let _ = std::fs::remove_file(&temp_path);
                let msg = format!("Failed to flush file {}: {}", temp_path, e);
                #[cfg(not(test))]
                tracing::error!("{}", msg);
                io::Error::new(e.kind(), msg)
            })?;
        }
    } // File handle is dropped here, ensuring it's closed

    // 4. Atomically rename the temp file to the final name
    #[cfg(not(test))]
    tracing::debug!("Renaming temporary file to final destination...");

    match std::fs::rename(&temp_path, path) {
        Ok(_) => {
            #[cfg(not(test))]
            tracing::info!("Successfully exported memory snapshot to: {}", file_path);
            Ok(())
        }
        Err(e) => {
            let _ = std::fs::remove_file(&temp_path);
            let msg = format!("Failed to rename temp file to {}: {}", file_path, e);
            #[cfg(not(test))]
            tracing::error!("{}", msg);
            Err(io::Error::new(e.kind(), msg))
        }
    }
}

/// Creates a memory snapshot from cloned allocation data.
///
/// This function is designed to be called from a separate thread where direct access
/// to the `MemoryTracker`'s internal mutex-protected data is not desired. It operates
/// on clones of the necessary data.
///
/// # Arguments
/// * `cloned_active_allocations`: A `Vec<AllocationInfo>` of active allocations.
/// * `cloned_allocation_log`: A `Vec<AllocationInfo>` of the allocation log.
///
/// # Returns
/// A `MemorySnapshot` instance.
fn create_snapshot_from_cloned_data(
    cloned_active_allocations: Vec<AllocationInfo>,
    cloned_allocation_log: Vec<AllocationInfo>,
) -> MemorySnapshot {
    // Calculate stats from cloned active allocations
    let total_allocations = cloned_active_allocations.len();
    let total_allocated = cloned_active_allocations.iter().map(|a| a.size).sum();

    // Perform hotspot analysis using cloned data
    // Note: This requires analyze_hotspots_from_data to be available from crate::tracker
    let hotspots = MemoryTracker::analyze_hotspots_from_data(
        &cloned_active_allocations,
        &cloned_allocation_log,
    );

    MemorySnapshot {
        timestamp: chrono::Local::now().to_rfc3339(),
        total_allocations,
        total_allocated,
        active_allocations: cloned_active_allocations, // Pass on the cloned active allocations
        allocation_hotspots: if hotspots.is_empty() {
            None
        } else {
            Some(hotspots)
        },
    }
}

/// Export memory lifecycle data to an SVG visualization
///
/// # Arguments
/// * `tracker` - The memory tracker containing the data to export
/// * `path` - The path where the SVG file should be saved
/// * `enable_sync` - Whether to force sync to disk (can be slow but ensures data is written)
///
/// # Returns
/// `io::Result<()>` indicating success or failure
pub fn export_to_svg<P: AsRef<Path>>(
    tracker: &MemoryTracker,
    path: P,
    enable_sync: bool,
) -> io::Result<()> {
    let path = path.as_ref();
    let file_path = path.display().to_string();

    #[cfg(not(test))]
    tracing::info!("Exporting memory visualization to: {}", file_path);

    // 1. Create parent directories if needed
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            #[cfg(not(test))]
            tracing::debug!("Creating parent directory: {}", parent.display());
            std::fs::create_dir_all(parent).map_err(|e| {
                let msg = format!("Failed to create directory {}: {}", parent.display(), e);
                #[cfg(not(test))]
                tracing::error!("{}", msg);
                io::Error::new(io::ErrorKind::Other, msg)
            })?;
        }
    }

    // 2. Combine allocation data
    #[cfg(not(test))]
    tracing::debug!("Collecting allocation data...");
    let mut combined_allocations = tracker.get_allocation_log(); // Get deallocated items
    combined_allocations.extend(tracker.get_active_allocations()); // Add active items

    // 3. Create SVG document (this is the potentially time-consuming part)
    tracing::debug!("Generating SVG document (with timeout)...");

    let (tx, rx) = std::sync::mpsc::channel();
    // Clone data that needs to be moved into the thread
    // AllocationInfo is Clone, so this is fine.
    let allocations_clone = combined_allocations.clone();

    let _thread = std::thread::spawn(move || {
        // Catch panics within the thread
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            create_svg_document(&allocations_clone)
        }));
        let _ = tx.send(result); // Send result (Ok(Document) or Err(panic)) back
    });

    // Wait for the result with a timeout (e.g., 60 seconds)
    let doc = match rx.recv_timeout(std::time::Duration::from_secs(60)) {
        Ok(result) => match result {
            // Received something from the thread
            Ok(doc) => doc, // Successfully created document
            Err(panic_err) => {
                // Thread panicked
                let panic_msg = if let Some(s) = panic_err.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic_err.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic during SVG document creation".to_string()
                };
                #[cfg(not(test))]
                tracing::error!("Panic during SVG creation: {}", panic_msg);
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("SVG creation panicked: {}", panic_msg),
                ));
            }
        },
        Err(_) => {
            // Timeout occurred
            #[cfg(not(test))]
            tracing::error!("Timeout during SVG document creation (took >60s)");
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "SVG document creation timed out after 60 seconds",
            ));
        }
    };
    // Write the SVG document to a string buffer
    let mut buffer = Vec::new();
    svg::write(&mut buffer, &doc).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to generate SVG: {}", e),
        )
    })?;

    // 4. Write to a temporary file first
    let temp_path = format!("{}.tmp", file_path);
    #[cfg(not(test))]
    tracing::debug!("Writing to temporary file: {}", temp_path);

    let _write_start = std::time::Instant::now();

    {
        let mut file = File::create(&temp_path).map_err(|e| {
            let msg = format!("Failed to create temporary file {}: {}", temp_path, e);
            #[cfg(not(test))]
            tracing::error!("{}", msg);
            io::Error::new(e.kind(), msg)
        })?;

        // Write the SVG data in chunks to show progress for large files
        const CHUNK_SIZE: usize = 8 * 1024; // 8KB chunks
        let mut written = 0;
        let total = buffer.len();

        for chunk in buffer.chunks(CHUNK_SIZE) {
            file.write_all(chunk).map_err(|e| {
                let _ = std::fs::remove_file(&temp_path);
                let msg = format!("Failed to write to file {}: {}", temp_path, e);
                #[cfg(not(test))]
                tracing::error!("{}", msg);
                io::Error::new(e.kind(), msg)
            })?;

            written += chunk.len();
            #[cfg(not(test))]
            if total > 0 {
                let percent = (written as f64 / total as f64 * 100.0) as u32;
                if percent % 10 == 0 {
                    // Log progress every 10%
                    tracing::debug!("Export progress: {}% ({} bytes)", percent, written);
                }
            }
        }

        // Optional: Flush and sync to disk
        if enable_sync {
            #[cfg(not(test))]
            tracing::debug!("Syncing file to disk...");

            file.sync_all().map_err(|e| {
                let _ = std::fs::remove_file(&temp_path);
                let msg = format!("Failed to sync file {}: {}", temp_path, e);
                #[cfg(not(test))]
                tracing::error!("{}", msg);
                io::Error::new(e.kind(), msg)
            })?;
        } else {
            // Just flush the buffers without waiting for disk sync
            file.flush().map_err(|e| {
                let _ = std::fs::remove_file(&temp_path);
                let msg = format!("Failed to flush file {}: {}", temp_path, e);
                #[cfg(not(test))]
                tracing::error!("{}", msg);
                io::Error::new(e.kind(), msg)
            })?;
        }
    } // File handle is dropped here, ensuring it's closed

    // 7. Atomically rename the temp file to the final name
    #[cfg(not(test))]
    tracing::debug!("Renaming temporary file to final destination...");

    match std::fs::rename(&temp_path, path) {
        Ok(_) => {
            #[cfg(not(test))]
            tracing::info!(
                "Successfully exported memory visualization to: {}",
                file_path
            );
            Ok(())
        }
        Err(e) => {
            let _ = std::fs::remove_file(&temp_path);
            let msg = format!("Failed to rename temp file to {}: {}", file_path, e);
            #[cfg(not(test))]
            tracing::error!("{}", msg);
            Err(io::Error::new(e.kind(), msg))
        }
    }
}

/// Create an SVG document visualizing memory allocation lifecycles
fn create_svg_document(all_allocations: &[AllocationInfo]) -> Document {
    const WIDTH: u32 = 1200;
    const HEIGHT_PER_ROW: u32 = 20;
    const ROW_SPACING: u32 = 5;
    const MARGIN_TOP: u32 = 60;
    const MARGIN_BOTTOM: u32 = 50;
    const MARGIN_LEFT: u32 = 150; // Increased for labels
    const MARGIN_RIGHT: u32 = 50;
    const LABEL_AREA_WIDTH: u32 = MARGIN_LEFT - 10; // Space for var_name/type_name

    let num_allocations = all_allocations.len();
    let chart_height = num_allocations as u32 * (HEIGHT_PER_ROW + ROW_SPACING);
    let total_height = chart_height + MARGIN_TOP + MARGIN_BOTTOM;

    let mut doc = Document::new()
        .set("width", WIDTH)
        .set("height", total_height)
        .set("viewBox", (0, 0, WIDTH, total_height));

    // Add title
    let title_text = SvgText::new("Memory Allocation Lifecycles")
        .set("x", WIDTH / 2)
        .set("y", MARGIN_TOP / 2)
        .set("text-anchor", "middle")
        .set("font-size", 16);
    doc = doc.add(title_text);

    if all_allocations.is_empty() {
        let empty_text = SvgText::new("No allocation data collected.")
            .set("x", WIDTH / 2)
            .set("y", total_height / 2)
            .set("text-anchor", "middle")
            .set("font-size", 14);
        doc = doc.add(empty_text);
        return doc;
    }

    let mut sorted_allocations = all_allocations.to_vec();
    sorted_allocations.sort_by_key(|a| a.timestamp_alloc);

    let min_time = sorted_allocations.first().map_or(0, |a| a.timestamp_alloc);

    // Determine timeline_end_time: the latest point in time any event (allocation or deallocation) occurred.
    // This ensures the timeline extends to cover all activities.
    let timeline_end_time = sorted_allocations
        .iter()
        .flat_map(|a| {
            let mut times = vec![a.timestamp_alloc]; // Always consider allocation time
            if let Some(dealloc_time) = a.timestamp_dealloc {
                times.push(dealloc_time); // Consider deallocation time if it exists
            }
            // For active items, their "duration" effectively goes up to the latest event in the dataset.
            // So, we don't need to push a.timestamp_alloc again here for active items.
            // The max of all alloc and dealloc times will define the end of the timeline.
            times.into_iter()
        })
        .max()
        .unwrap_or(min_time + 1000); // Fallback if no allocations or only one with no dealloc. Add 1s.

    let time_range = if timeline_end_time > min_time {
        timeline_end_time - min_time
    } else {
        1 // Avoid division by zero if all events are at the same millisecond or list is empty
    }; // Avoid division by zero

    // Draw time axis
    let axis_y = MARGIN_TOP + chart_height + 10;
    let axis_line = Line::new()
        .set("x1", MARGIN_LEFT)
        .set("y1", axis_y)
        .set("x2", WIDTH - MARGIN_RIGHT)
        .set("y2", axis_y)
        .set("stroke", "black");
    doc = doc.add(axis_line);

    // Add time labels (simplified: start and end)
    let start_time_text = SvgText::new(format!("{} ms", min_time))
        .set("x", MARGIN_LEFT)
        .set("y", axis_y + 15)
        .set("text-anchor", "middle")
        .set("font-size", 10);
    doc = doc.add(start_time_text);

    let end_time_text = SvgText::new(format!("{} ms", timeline_end_time))
        .set("x", WIDTH - MARGIN_RIGHT)
        .set("y", axis_y + 15)
        .set("text-anchor", "middle")
        .set("font-size", 10);
    doc = doc.add(end_time_text);

    for (i, alloc_info) in sorted_allocations.iter().enumerate() {
        let y_position = MARGIN_TOP + i as u32 * (HEIGHT_PER_ROW + ROW_SPACING);

        let start_x_rel = alloc_info.timestamp_alloc - min_time;
        let start_x = MARGIN_LEFT
            + (start_x_rel as f64 / time_range as f64 * (WIDTH - MARGIN_LEFT - MARGIN_RIGHT) as f64)
                as u32;

        // Active items should extend to the globally determined timeline_end_time
        let end_time_for_bar = alloc_info.timestamp_dealloc.unwrap_or(timeline_end_time);
        let end_x_rel = end_time_for_bar - min_time;
        let end_x = MARGIN_LEFT
            + (end_x_rel as f64 / time_range as f64 * (WIDTH - MARGIN_LEFT - MARGIN_RIGHT) as f64)
                as u32;

        let bar_width = if end_x > start_x { end_x - start_x } else { 1 };

        let color = if alloc_info.timestamp_dealloc.is_some() {
            "steelblue" // Deallocated
        } else {
            "lightcoral" // Active
        };

        let rect = Rectangle::new()
            .set("x", start_x)
            .set("y", y_position)
            .set("width", bar_width)
            .set("height", HEIGHT_PER_ROW)
            .set("fill", color);

        let mut group = Group::new().add(rect);

        // Add label (var_name or type_name) to the left of the timeline area
        let label_text_content = format!(
            "{} ({}B)",
            alloc_info
                .var_name
                .as_deref()
                .or(alloc_info.type_name.as_deref())
                .unwrap_or("?"),
            alloc_info.size
        );
        let item_label = SvgText::new(label_text_content)
            .set("x", MARGIN_LEFT - 5) // Position before the timeline bars
            .set("y", y_position + HEIGHT_PER_ROW / 2 + 4) // Vertically centered with bar
            .set("text-anchor", "end") // Align text to the right of x
            .set("font-size", 10)
            .set("font-family", "monospace");
        group = group.add(item_label);

        // Add tooltip (consolidated and enhanced)
        let tooltip_backtrace_info =
            format!("Backtrace frames: {}", alloc_info.backtrace_ips.len());
        let tooltip_text = format!(
            "Ptr: 0x{:x}\nSize: {}B\nAllocated: {} ms\nDeallocated: {}\nVariable: {}\nType: {}\nThread ID: {}\n{}",
            alloc_info.ptr,
            alloc_info.size,
            alloc_info.timestamp_alloc,
            alloc_info
                .timestamp_dealloc
                .map_or_else(|| "Active".to_string(), |t| format!("{} ms", t)),
            alloc_info.var_name.as_deref().unwrap_or("N/A"),
            alloc_info.type_name.as_deref().unwrap_or("N/A"),
            alloc_info.thread_id, // Added thread_id
            tooltip_backtrace_info // Added backtrace summary
        );
        let title_element = SvgTitle::new(tooltip_text);
        group = group.add(title_element);

        doc = doc.add(group);
    }

    doc
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracker::MemoryTracker;
    use std::fs::{self, File};
    use std::io::Read;
    // Remove unused sleep and Duration imports if no longer needed after test adjustments
    // use std::thread::sleep;
    // use std::time::Duration;

    #[cfg(test)]
    use tempfile::tempdir;

    // Helper to create a populated MemoryTracker for testing export functions
    fn create_populated_tracker_for_svg_test() -> MemoryTracker {
        let tracker = MemoryTracker::new();
        let base_time = 1678886400000; // Some fixed time in milliseconds

        // Clear any existing data that might be there from other tests or setups
        tracker.clear_all_for_test();

        // Allocation 1: Active (will be fetched by get_active_allocations)
        tracker
            .add_allocation_for_test(AllocationInfo {
                ptr: 0x1000,
                size: 100,
                timestamp_alloc: base_time + 100, // Allocates at 100ms
                timestamp_dealloc: None,          // Active
                var_name: Some("varActive".to_string()),
                type_name: Some("TypeActive".to_string()),
                backtrace_ips: vec![1, 2, 3],
                thread_id: 1,
            })
            .unwrap();

        // Allocation 2: Deallocated (will be fetched by get_allocation_log)
        tracker
            .add_deallocated_for_test(AllocationInfo {
                ptr: 0x2000,
                size: 200,
                timestamp_alloc: base_time + 50, // Allocates at 50ms
                timestamp_dealloc: Some(base_time + 150), // Deallocates at 150ms
                var_name: Some("varDeallocated".to_string()),
                type_name: Some("TypeDeallocated".to_string()),
                backtrace_ips: vec![4, 5, 6],
                thread_id: 2,
            })
            .unwrap();

        // Allocation 3: Active, later allocation (will be fetched by get_active_allocations)
        tracker
            .add_allocation_for_test(AllocationInfo {
                ptr: 0x3000,
                size: 50,
                timestamp_alloc: base_time + 200, // Allocates at 200ms
                timestamp_dealloc: None,          // Active
                var_name: Some("varActiveLater".to_string()),
                type_name: Some("TypeActiveLater".to_string()),
                backtrace_ips: vec![7, 8],
                thread_id: 1,
            })
            .unwrap();

        tracker
    }

    // Helper for JSON test, may need different setup if it relies on internal state differently
    fn create_populated_tracker_for_json_test() -> MemoryTracker {
        let tracker = MemoryTracker::new();
        let base_time = 1678886400000;
        tracker.clear_all_for_test();

        // Allocation 1: With var_name, type_name, specific backtrace and thread_id
        tracker
            .add_allocation_for_test(AllocationInfo {
                ptr: 0x1000,
                size: 100,
                timestamp_alloc: base_time + 100,
                timestamp_dealloc: None,
                var_name: Some("varA".to_string()),
                type_name: Some("TypeA".to_string()),
                backtrace_ips: vec![1, 2, 3, 4, 5],
                thread_id: 111,
            })
            .unwrap();

        // Allocation 2: Without var_name/type_name (None), different backtrace and thread_id
        tracker
            .add_allocation_for_test(AllocationInfo {
                ptr: 0x3000,
                size: 300,
                timestamp_alloc: base_time + 400,
                timestamp_dealloc: None,
                var_name: None,  // Test case with None
                type_name: None, // Test case with None
                backtrace_ips: vec![6, 7, 8],
                thread_id: 222,
            })
            .unwrap();
        // JSON export typically shows active allocations, so a deallocated one might not be expected here
        // unless the test specifically checks for it in a different way (e.g. if JSON also included a log)
        // For now, keeping it simple for JSON which focuses on active state.
        tracker
    }

    #[test]
    fn test_export_to_json_valid_data() {
        let tracker = create_populated_tracker_for_json_test(); // Use JSON specific helper
        let dir = tempdir().unwrap();
        let json_path = dir.path().join("test_snapshot.json");

        let result = tracker.export_to_json(&json_path, false); // Disable sync for tests to make them faster
        assert!(
            result.is_ok(),
            "export_to_json should return Ok. Error: {:?}",
            result.err()
        );

        let mut file_content = String::new();
        File::open(&json_path)
            .unwrap()
            .read_to_string(&mut file_content)
            .unwrap();

        let snapshot: Result<MemorySnapshot, _> = serde_json::from_str(&file_content);
        assert!(
            snapshot.is_ok(),
            "Deserializing JSON snapshot should be Ok. Error: {:?}",
            snapshot.err()
        );
        let snapshot_data = snapshot.unwrap();

        // create_snapshot (used by export_to_json) gets *active* allocations.
        // varA (0x1000) and varC (0x3000) are active. varB (0x2000) was deallocated.
        assert_eq!(
            snapshot_data.active_allocations.len(),
            2,
            "Expected 2 active allocations in JSON snapshot"
        ); // Now explicitly populated

        let var_a_info = snapshot_data
            .active_allocations
            .iter()
            .find(|a| a.ptr == 0x1000)
            .expect("Allocation with ptr 0x1000 not found in JSON active allocations");

        assert_eq!(var_a_info.var_name.as_deref(), Some("varA"));
        assert_eq!(var_a_info.type_name.as_deref(), Some("TypeA"));
        assert_eq!(var_a_info.size, 100);
        assert_eq!(var_a_info.backtrace_ips, vec![1, 2, 3, 4, 5]);
        assert_eq!(var_a_info.thread_id, 111);

        let var_c_info = snapshot_data // Renaming to var_b_info or similar might be clearer if ptrs change
            .active_allocations
            .iter()
            .find(|a| a.ptr == 0x3000)
            .expect("Allocation with ptr 0x3000 not found in JSON active allocations");

        assert_eq!(var_c_info.var_name, None); // Expecting None
        assert_eq!(var_c_info.type_name, None); // Expecting None
        assert_eq!(var_c_info.size, 300);
        assert_eq!(var_c_info.backtrace_ips, vec![6, 7, 8]);
        assert_eq!(var_c_info.thread_id, 222);
    }

    #[test]
    fn test_export_to_svg_generates_file() {
        let tracker = create_populated_tracker_for_svg_test(); // Use SVG specific helper
        let dir = tempdir().unwrap();
        let svg_path = dir.path().join("test_visualization.svg");

        let result = tracker.export_to_svg(&svg_path, false); // Disable sync for tests to make them faster
        assert!(
            result.is_ok(),
            "export_to_svg should return Ok. Error: {:?}",
            result.err()
        );

        assert!(svg_path.exists(), "SVG file should exist");
        let svg_content = fs::read_to_string(&svg_path).unwrap();
        assert!(!svg_content.is_empty(), "SVG file should not be empty");

        assert!(
            svg_content.starts_with("<svg"),
            "SVG content should start with <svg"
        );
        assert!(
            svg_content.ends_with("</svg>"),
            "SVG content should end with </svg>"
        );

        // With the new logic, SVG should contain both active and deallocated items.
        // Check for varActive, varDeallocated, and varActiveLater.
        assert!(
            svg_content.contains("varActive"),
            "SVG should contain text for 'varActive'"
        );
        assert!(
            svg_content.contains("varDeallocated"),
            "SVG should contain text for 'varDeallocated'"
        );
        assert!(
            svg_content.contains("varActiveLater"),
            "SVG should contain text for 'varActiveLater'"
        );

        // Check for tooltip content (enhanced)
        assert!(
            svg_content.contains("Ptr: 0x1000"),
            "SVG tooltip for 0x1000 missing"
        );
        assert!(
            svg_content.contains("Size: 100B"),
            "SVG tooltip for 0x1000 missing size"
        );
        assert!(
            svg_content.contains("Deallocated: Active"),
            "SVG tooltip for 0x1000 incorrect dealloc status"
        );
        assert!(
            svg_content.contains("Thread ID: 1"),
            "SVG tooltip for 0x1000 missing thread ID"
        );
        assert!(
            svg_content.contains("Backtrace frames: 3"),
            "SVG tooltip for 0x1000 missing backtrace info"
        );

        assert!(
            svg_content.contains("Ptr: 0x2000"),
            "SVG tooltip for 0x2000 missing"
        );
        assert!(
            svg_content.contains("Size: 200B"),
            "SVG tooltip for 0x2000 missing size"
        );
        assert!(
            svg_content.contains(&format!("Deallocated: {} ms", 1678886400000i64 + 150)),
            "SVG tooltip for 0x2000 incorrect dealloc status"
        );
        assert!(
            svg_content.contains("Thread ID: 2"),
            "SVG tooltip for 0x2000 missing thread ID"
        );

        assert!(
            svg_content.contains("<rect"),
            "SVG should contain <rect> elements for bars"
        );

        // Verify colors - varActive (0x1000) and varActiveLater (0x3000) should be lightcoral, varDeallocated (0x2000) should be steelblue
        // This requires finding the <rect> elements and checking their fill.
        // Example check for varActive (ptr 0x1000, active):
        // <text x="145" y="XY" ...>varActive (100B)</text> ... <rect x="ABC" y="UV" ... fill="lightcoral"><title>Ptr: 0x1000...</title></rect>
        // Example check for varDeallocated (ptr 0x2000, deallocated):
        // <text x="145" y="XY" ...>varDeallocated (200B)</text> ... <rect x="DEF" y="WZ" ... fill="steelblue"><title>Ptr: 0x2000...</title></rect>

        // A simplified check: count occurrences of colors. Expect 2 active, 1 deallocated.
        assert_eq!(
            svg_content.matches("lightcoral").count(),
            2,
            "Expected 2 active items with lightcoral color"
        );
        assert_eq!(
            svg_content.matches("steelblue").count(),
            1,
            "Expected 1 deallocated item with steelblue color"
        );
    }
}
