use std::{
    // backtrace::Backtrace, // This specific struct is not used directly, backtrace::trace is.
    collections::{HashMap, HashSet}, 
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, 
        Mutex,
    }, 
    thread, 
    time::{SystemTime, UNIX_EPOCH}
};
use std::fs::File; // Added for export_flamegraph_svg
use std::io::{self, BufWriter}; // Added for export_flamegraph_svg
use std::path::Path; // Added for export_flamegraph_svg
use thiserror::Error;
use inferno::flamegraph::{self, Options as FlamegraphOptions}; // Added for flamegraph
use svg::node::element::{Rectangle, Text as SvgText, Title as SvgTitle, Group}; // Added for treemap
use svg::Document; // Added for treemap
use treemap::{Rect, TreemapLayout}; // Added for treemap

/// Error type for memory tracking operations.
#[derive(Error, Debug)]
pub enum MemoryError {
    /// Indicates that a mutex lock could not be acquired.
    #[error("Failed to acquire lock: {0}")]
    LockError(String),

    /// Indicates an invalid operation or state in memory tracking.
    #[error("Memory tracking error: {0}")]
    TrackingError(String),
}

/// Holds detailed information about a single memory allocation event.
///
/// This struct is used to store data for both active allocations and for
/// entries in the historical allocation log.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AllocationInfo {
    /// The memory address of the allocation.
    pub ptr: usize,
    /// The size of the allocation in bytes.
    pub size: usize,
    /// Timestamp (milliseconds since UNIX_EPOCH) when the allocation occurred.
    pub timestamp_alloc: u128,
    /// Timestamp (milliseconds since UNIX_EPOCH) when deallocation occurred, if applicable.
    pub timestamp_dealloc: Option<u128>,
    /// Optional name of the variable associated with this allocation.
    pub var_name: Option<String>,
    /// Optional type name of the variable associated with this allocation.
    pub type_name: Option<String>,
    /// Instruction pointers from the backtrace captured at allocation time.
    pub backtrace_ips: Vec<usize>,
    /// The ID of the thread that performed the allocation.
    pub thread_id: u64,
}

/// Represents a hotspot for memory allocations, identified by a unique backtrace.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)] // Added Deserialize
pub struct HotspotInfo {
    /// Instruction pointers from the backtrace that define this allocation call site.
    pub backtrace_ips: Vec<usize>,
    /// The number of allocations originating from this call site.
    pub count: usize,
    /// The total size (in bytes) of all allocations from this call site.
    pub total_size: usize,
    // Optional: Add a field for representative type_names if desired later
    // pub representative_types: Vec<String>,
}

/// Represents the total memory usage for a specific type.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TypeMemoryUsage {
    /// The name of the data type.
    pub type_name: String,
    /// The total size (in bytes) of memory allocated for this type across all tracked allocations.
    pub total_size: usize,
}

/// Manages the tracking of memory allocations and deallocations.
///
/// `MemoryTracker` maintains a record of active allocations and a log of
/// deallocated memory events. It provides methods to record allocations,
/// deallocations, associate variable names with allocations, and retrieve
/// statistics and detailed allocation information.
///
/// This tracker is typically accessed via a global instance obtained through
/// [`get_global_tracker()`].
pub struct MemoryTracker {
    active_allocations: Mutex<HashMap<usize, AllocationInfo>>,
    allocation_log: Mutex<Vec<AllocationInfo>>,
    tracking_enabled: AtomicBool,
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self {
            active_allocations: Mutex::new(HashMap::new()),
            allocation_log: Mutex::new(Vec::new()),
            tracking_enabled: AtomicBool::new(true),
        }
    }
}

impl MemoryTracker {
    /// Creates a new, empty `MemoryTracker`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Temporarily disables allocation tracking while executing a closure.
    ///
    /// This method is useful when you need to perform operations that might trigger
    /// allocations (e.g., variable association) without recording those allocations.
    ///
    /// # Arguments
    /// * `f`: A closure that will be executed with allocation tracking disabled.
    ///
    /// # Returns
    /// The result of the closure execution.
    pub fn with_allocations_disabled<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        use std::sync::atomic::Ordering::SeqCst;
        
        let was_enabled = self.tracking_enabled.swap(false, SeqCst);
        tracing::debug!("with_allocations_disabled: was_enabled={}", was_enabled);
        
        // record thread id for debugging
        let thread_id = std::thread::current().id();
        tracing::debug!("[Thread {:?}] Entering with_allocations_disabled", thread_id);
        
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            f()
        }));
        
        // restore tracking_enabled
        if was_enabled {
            self.tracking_enabled.store(true, SeqCst);
            tracing::debug!("[Thread {:?}] Restored tracking_enabled to true", thread_id);
        }
        
        
        match result {
            Ok(r) => {
                tracing::debug!("[Thread {:?}] Exiting with_allocations_disabled normally", thread_id);
                r
            },
            Err(panic) => {
                tracing::error!("[Thread {:?}] Panic in with_allocations_disabled: {:?}", thread_id, panic);
                std::panic::resume_unwind(panic);
            }
        }
    }

    /// Records a new memory allocation.
    ///
    /// This method is typically called by the custom global allocator when memory is allocated.
    ///
    /// # Arguments
    /// * `ptr`: The starting memory address of the allocation.
    /// * `size`: The size of the allocation in bytes.
    /// * `type_name`: An optional string representing the type of the allocated object.
    ///
    /// # Returns
    /// `Ok(())` if successful, or `Err(MemoryError)` if a lock could not be acquired or
    /// if there was an issue generating a timestamp.
    pub fn track_allocation(
        &self,
        ptr: usize,
        size: usize,
        type_name: Option<String>,
    ) -> Result<(), MemoryError> {
        let mut active = self.active_allocations.lock()
            .map_err(|e| MemoryError::LockError(format!("Failed to lock active_allocations: {}", e)))?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| MemoryError::TrackingError("System time before UNIX_EPOCH".to_string()))?
            .as_millis();

        active.insert(
            ptr,
            AllocationInfo {
                ptr,
                size,
                timestamp_alloc: timestamp,
                timestamp_dealloc: None,
                var_name: None,
                type_name,
                backtrace_ips: {
                    #[cfg(feature = "backtrace")]
                    {
                        let mut ips = Vec::new();
                        let mut stack_trace = None;
                        backtrace::trace(|frame| {
                            ips.push(frame.ip() as usize);
                            stack_trace = Some(format!("{:?}", frame));
                            // Continue tracing
                            true
                        });
                        (ips, stack_trace)
                    }
                    #[cfg(not(feature = "backtrace"))]
                    {
                        Vec::new() // Empty vec if backtrace feature is not enabled
                    }
                },
                thread_id: {
                    use std::hash::{Hash, Hasher};
                    use std::collections::hash_map::DefaultHasher;
                    
                    let mut hasher = DefaultHasher::new();
                    thread::current().id().hash(&mut hasher);
                    hasher.finish()
                },
            },
        );
        
        Ok(())
    }

    /// Records the deallocation of a memory block.
    ///
    /// This method is typically called by the custom global allocator when memory is freed.
    /// The allocation information is moved from the active list to the historical log.
    ///
    /// # Arguments
    /// * `ptr`: The memory address of the block being deallocated.
    ///
    /// # Returns
    /// `Ok(())` if successful, or `Err(MemoryError)` if the pointer was not found in
    /// active allocations, a lock could not be acquired, or there was an issue
    /// generating a timestamp.
    pub fn track_deallocation(&self, ptr: usize) -> Result<(), MemoryError> {
        let mut active = self.active_allocations.lock()
            .map_err(|e| MemoryError::LockError(format!("Failed to lock active_allocations: {}", e)))?;
            
        if let Some(mut info) = active.remove(&ptr) {
            info.timestamp_dealloc = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|_| MemoryError::TrackingError("System time before UNIX_EPOCH".to_string()))?
                    .as_millis(),
            );
            
            let mut log = self.allocation_log.lock()
                .map_err(|e| MemoryError::LockError(format!("Failed to lock allocation_log: {}", e)))?;
            log.push(info);
            
            Ok(())
        } else {
            Err(MemoryError::TrackingError(format!("No active allocation found for pointer: 0x{:x}", ptr)))
        }
    }

    /// Associates a variable name and a more specific type name with an active allocation.
    ///
    /// This is used by `track_var!` to link source code variable names to their
    /// underlying memory allocations.
    ///
    /// # Arguments
    /// * `ptr`: The memory address of the allocation.
    /// * `var_name`: The name of the variable.
    /// * `type_name`: The specific type name of the variable.
    ///
    /// # Returns
    /// `Ok(())` if successful, or `Err(MemoryError)` if the pointer was not found
    /// in active allocations or if a lock could not be acquired.
    pub fn associate_var(&self, ptr: usize, var_name: String, type_name: String) -> Result<(), MemoryError> {
        // Skip if tracking is disabled to prevent deadlocks
        if !self.tracking_enabled.load(Ordering::SeqCst) {
            tracing::debug!("Skipping variable association - tracking is disabled");
            return Ok(());
        }

        let mut active = self.active_allocations.lock().map_err(|e| MemoryError::LockError(e.to_string()))?;

        if let Some(alloc) = active.get_mut(&ptr) {
            alloc.var_name = Some(var_name.clone());
            alloc.type_name = Some(type_name.clone());
            
            tracing::info!("Successfully associated variable: name='{}', type='{}', ptr=0x{:x}", 
                         var_name, type_name, ptr);
            Ok(())
        } else {
            let active_ptrs: Vec<String> = active.keys()
                .take(5) // Only show first 5 to avoid log spam
                .map(|p| format!("0x{:x}", p))
                .collect();
                
            let err_msg = format!("No active allocation found for pointer: 0x{:x}. Active pointers (first 5): {:?}", 
                               ptr, active_ptrs);
            tracing::error!("{}", err_msg);
            Err(MemoryError::TrackingError(err_msg))
        }
    }

    /// Retrieves statistics about currently active memory allocations.
    ///
    /// # Returns
    /// A `MemoryStats` struct containing the total number of active allocations
    /// and the total memory they occupy.
    pub fn get_stats(&self) -> MemoryStats {
        let active = match self.active_allocations.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        
        let total_allocations = active.len();
        let total_memory = active.values().map(|a| a.size).sum();
        
        MemoryStats {
            total_allocations,
            total_memory,
        }
    }

    /// Retrieves a list of all currently active allocations.
    ///
    /// # Returns
    /// A `Vec<AllocationInfo>` containing clones of the information for all allocations
    /// that have been tracked but not yet deallocated.
    pub fn get_active_allocations(&self) -> Vec<AllocationInfo> {
        let active = match self.active_allocations.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        
        active.values().cloned().collect()
    }

    /// Retrieves the historical log of all deallocated memory events.
    ///
    /// # Returns
    /// A `Vec<AllocationInfo>` containing clones of the information for all allocations
    /// that have been tracked and subsequently deallocated.
    pub fn get_allocation_log(&self) -> Vec<AllocationInfo> {
        let log = match self.allocation_log.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        log.clone()
    }

    /// Exports the current memory snapshot (active allocations) to a JSON file.
    ///
    /// # Arguments
    /// * `path`: The path to the file where the JSON data will be written.
    ///
    /// # Returns
    /// `Ok(())` if successful, or `std::io::Error` if file I/O or serialization fails.
    pub fn export_to_json<P: AsRef<std::path::Path>>(&self, path: P) -> std::io::Result<()> {
        crate::export::export_to_json(self, path)
    }

    /// Exports the memory allocation lifecycle data (from the deallocation log) to an SVG file.
    ///
    /// # Arguments
    /// * `path`: The path to the file where the SVG data will be written.
    ///
    /// # Returns
    /// `Ok(())` if successful, or `std::io::Error` if file I/O or SVG generation fails.
    pub fn export_to_svg<P: AsRef<std::path::Path>>(&self, path: P) -> std::io::Result<()> {
        crate::export::export_to_svg(self, path)
    }

    /// Analyzes both active and logged allocations to identify memory allocation hotspots.
    ///
    /// Hotspots are determined by common backtraces. For each unique backtrace,
    /// this method calculates the total number of allocations and the total size
    /// of memory allocated from that call site.
    ///
    /// # Returns
    /// A `Vec<HotspotInfo>` sorted by `total_size` (descending) and then by `count` (descending).
    /// Allocations with empty backtraces are ignored.
    pub fn analyze_hotspots(&self) -> Vec<HotspotInfo> {
        let mut hotspots_map: HashMap<Vec<usize>, (usize, usize)> = HashMap::new();

        // Process active allocations
        let active_allocs = self.active_allocations.lock().unwrap_or_else(|e| e.into_inner());
        for alloc_info in active_allocs.values() {
            if !alloc_info.backtrace_ips.is_empty() {
                let entry = hotspots_map.entry(alloc_info.backtrace_ips.clone()).or_insert((0, 0));
                entry.0 += 1; // count
                entry.1 += alloc_info.size; // total_size
            }
        }
        drop(active_allocs); // Release lock early

        // Process logged (deallocated) allocations
        let logged_allocs = self.allocation_log.lock().unwrap_or_else(|e| e.into_inner());
        for alloc_info in logged_allocs.iter() {
            if !alloc_info.backtrace_ips.is_empty() {
                let entry = hotspots_map.entry(alloc_info.backtrace_ips.clone()).or_insert((0, 0));
                entry.0 += 1; // count
                entry.1 += alloc_info.size; // total_size
            }
        }
        drop(logged_allocs); // Release lock early

        let mut hotspots_vec: Vec<HotspotInfo> = hotspots_map.into_iter()
            .map(|(ips, (count, total_size))| HotspotInfo {
                backtrace_ips: ips,
                count,
                total_size,
            })
            .collect();

        // Sort by total_size descending, then by count descending
        hotspots_vec.sort_unstable_by(|a, b| {
            b.total_size.cmp(&a.total_size)
                .then_with(|| b.count.cmp(&a.count))
        });

        hotspots_vec
    }

    /// Exports memory allocation hotspots to an SVG flamegraph.
    ///
    /// This method analyzes memory hotspots (call sites with frequent or large allocations)
    /// and generates an SVG flamegraph visualizing these hotspots. The flamegraph can represent
    /// hotspots based on either the total size of allocations or the count of allocations.
    ///
    /// # Arguments
    /// * `path` - The path where the SVG flamegraph file will be saved.
    /// * `title` - A title for the generated flamegraph.
    /// * `use_total_size_as_value` - If `true`, the flamegraph values will represent the total
    ///   size of memory allocated from each call site. If `false`, values will represent
    ///   the number of allocations (count) from each call site.
    ///
    /// # Returns
    /// An `io::Result<()>` indicating success or failure of the export operation.
    /// Returns an error if no hotspot data is available or if flamegraph generation fails.
    ///
    /// # Notes
    /// - Meaningful symbol names in the flamegraph require debug symbols to be available
    ///   during compilation and for the `backtrace` feature of this crate to be enabled.
    /// - Symbol names are demangled using `rustc_demangle`.
    pub fn export_flamegraph_svg<P: AsRef<Path>>(
        &self,
        path: P,
        title: &str,
        use_total_size_as_value: bool, 
    ) -> io::Result<()> {
        let hotspots = self.analyze_hotspots();
        if hotspots.is_empty() {
            return Err(io::Error::new(io::ErrorKind::Other, "No hotspot data to generate flamegraph"));
        }

        let mut unique_ips = HashSet::new();
        for hotspot in &hotspots {
            for &ip in &hotspot.backtrace_ips {
                unique_ips.insert(ip);
            }
        }

        let resolved_symbols = resolve_ips(&unique_ips); 

        let mut folded_lines = Vec::new();
        for hotspot in hotspots {
            if hotspot.backtrace_ips.is_empty() { // Skip if backtrace is empty
                continue;
            }
            let stack_str: String = hotspot.backtrace_ips.iter()
                .map(|&ip| resolved_symbols.get(&ip)
                    .cloned()
                    .unwrap_or_else(|| format!("{:#x}", ip)))
                .rev() // Reverse to get root;parent;leaf order for flamegraph
                .collect::<Vec<String>>()
                .join(";");

            if stack_str.is_empty() { // Should not happen if backtrace_ips was not empty
                continue;
            }

            let value = if use_total_size_as_value {
                hotspot.total_size
            } else {
                hotspot.count
            };
            folded_lines.push(format!("{} {}", stack_str, value));
        }

        if folded_lines.is_empty() {
            return Err(io::Error::new(io::ErrorKind::Other, "No valid stack data for flamegraph after processing"));
        }
        
        let output_file = File::create(path)?;
        let mut writer = BufWriter::new(output_file);

        let mut options = FlamegraphOptions::default();
        options.title = title.to_string();
        options.count_name = if use_total_size_as_value { "bytes".to_string() } else { "samples".to_string() };
        // options.font_size = 12; 
        // options.flame_chart = true; // For ICicle charts (time based) - not directly applicable here

        flamegraph::from_lines(
            &mut options,
            folded_lines.iter().map(|s| s.as_str()),
            &mut writer,
        ).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to generate flamegraph: {}", e)))?;
        
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn clear_all_for_test(&self) {
        if let Ok(mut active) = self.active_allocations.lock() {
            active.clear();
        }
        if let Ok(mut log) = self.allocation_log.lock() {
            log.clear();
        }
    }

    #[cfg(test)]
    pub(crate) fn add_allocation_for_test(&self, alloc_info: AllocationInfo) -> Result<(), MemoryError> {
        let mut active = self.active_allocations.lock()
            .map_err(|e| MemoryError::LockError(format!("Failed to lock active_allocations for test injection: {}", e)))?;
        active.insert(alloc_info.ptr, alloc_info);
        Ok(())
    }

    /// Aggregates memory usage by type name from active allocations.
    ///
    /// This method iterates over all currently active allocations, groups them by their
    /// `type_name` (or "UnknownType" if not set), and sums the total allocated size
    /// for each type.
    ///
    /// # Returns
    /// A `Vec<TypeMemoryUsage>` where each entry represents a unique type name and the
    /// total memory size it occupies. Only types with a total size greater than 0 are included.
    pub fn aggregate_memory_by_type(&self) -> Vec<TypeMemoryUsage> {
        let mut usage_map: HashMap<String, usize> = HashMap::new();
        let active_allocs = self.get_active_allocations(); // Uses the existing public method

        for alloc_info in active_allocs {
            let type_key = alloc_info.type_name.clone().unwrap_or_else(|| "UnknownType".to_string());
            *usage_map.entry(type_key).or_insert(0) += alloc_info.size;
        }

        usage_map.into_iter()
            .map(|(type_name, total_size)| TypeMemoryUsage { type_name, total_size })
            .filter(|item| item.total_size > 0) // Only include types with size > 0
            .collect()
    }

    /// Exports aggregated memory usage by type to an SVG treemap.
    ///
    /// This method visualizes the total memory allocated for each data type
    /// as a treemap, where the area of each rectangle corresponds to its memory usage.
    ///
    /// # Arguments
    /// * `path` - The path where the SVG treemap file will be saved.
    /// * `title` - A title for the generated treemap.
    /// * `width` - The width of the SVG canvas.
    /// * `height` - The height of the SVG canvas.
    ///
    /// # Returns
    /// An `io::Result<()>` indicating success or failure of the export operation.
    /// Returns an error if no type usage data is available or if SVG generation fails.
    ///
    /// # Notes
    /// - Type names are used as labels. "UnknownType" is used if a type name was not available.
    /// - Colors are cycled for different types for better visual distinction.
    pub fn export_treemap_svg<P: AsRef<Path>>(
        &self,
        path: P,
        title: &str,
        width: u32,
        height: u32,
    ) -> io::Result<()> {
        let type_usages = self.aggregate_memory_by_type();
        if type_usages.is_empty() {
            let document = Document::new()
                .set("width", width)
                .set("height", height)
                .set("viewBox", (0, 0, width, height))
                .add(SvgText::new("No memory usage data to display for treemap.")
                    .set("x", width / 2)
                    .set("y", height / 2)
                    .set("text-anchor", "middle"));
            svg::save(path, &document)?;
            return Ok(());
        }

        #[derive(Clone)]
        struct TreemapDataItem {
            label: String,
            value: f64,
            original_size: usize,
            bounds: treemap::Rect,
        }
        
        impl treemap::Mappable for TreemapDataItem {
            fn size(&self) -> f64 { self.value }
            fn bounds(&self) -> &treemap::Rect { &self.bounds }
            fn set_bounds(&mut self, bounds: treemap::Rect) { self.bounds = bounds; }
        }
        let mut items_to_layout: Vec<TreemapDataItem> = type_usages.into_iter()
            .map(|usage| TreemapDataItem {
                label: usage.type_name,
                value: usage.total_size as f64,
                original_size: usage.total_size,
                bounds: treemap::Rect::new(),
            })
            .collect();
        
        items_to_layout.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap_or(std::cmp::Ordering::Equal));

        let layout = TreemapLayout::new();
        let mut target_rect = Rect::new();
        target_rect.w = width as f64;
        target_rect.h = height as f64;
        
        layout.layout_items(&mut items_to_layout, target_rect);

        let mut document = Document::new()
            .set("width", width)
            .set("height", height)
            .set("viewBox", (0, 0, width, height));

        document = document.add(
            SvgText::new(title)
                .set("x", width / 2)
                .set("y", 20) 
                .set("text-anchor", "middle")
                .set("font-size", 16.0),
        );

        let colors = vec!["#1f77b4", "#ff7f0e", "#2ca02c", "#d62728", "#9467bd", "#8c564b", "#e377c2", "#7f7f7f", "#bcbd22", "#17becf"];
        
        for (i, item) in items_to_layout.iter().enumerate() {
            let bounds = &item.bounds;
            let tooltip = format!("{} ({} bytes)", item.label, item.original_size);
            
            let rect = Rectangle::new()
                .set("x", bounds.x)
                .set("y", bounds.y)
                .set("width", bounds.w)
                .set("height", bounds.h)
                .set("fill", colors[i % colors.len()])
                .set("stroke", "white")
                .set("stroke-width", 1);

            let title = SvgTitle::new(tooltip);
            
            let mut group = Group::new()
                .add(rect)
                .add(title);

            // Add text if there's enough space
            if bounds.w > 50.0 && bounds.h > 15.0 {
                let text = SvgText::new(item.label.clone())
                    .set("x", bounds.x + bounds.w / 2.0)
                    .set("y", bounds.y + bounds.h / 2.0)
                    .set("dy", "0.35em")
                    .set("text-anchor", "middle")
                    .set("font-size", (bounds.h.min(bounds.w) / 5.0).max(8.0).min(12.0))
                    .set("fill", "white");
                group = group.add(text);
            }

            document = document.add(group);
        }
        
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        // svg::save uses io::Write, svg::write is for direct writing without path
        // Since we have a BufWriter, svg::write is appropriate.
        svg::write(&mut writer, &document)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to write SVG treemap: {}", e)))?;

        Ok(())
    }
}

/// Contains summary statistics about memory usage.
#[derive(Debug)]
pub struct MemoryStats {
    /// The total number of currently active allocations.
    pub total_allocations: usize,
    /// The total amount of memory (in bytes) occupied by currently active allocations.
    pub total_memory: usize,
}

/// Returns the `ThreadId` of the current thread.
///
/// Note: This function is not directly related to the core tracking logic of `MemoryTracker`
/// but was part of the original module. Its utility within this specific module might be limited
/// as `MemoryTracker` already captures thread IDs internally using a different method for its `AllocationInfo`.
#[allow(dead_code)] // It's not used within this module, allow dead code for now.
pub fn thread_id() -> std::thread::ThreadId {
    thread::current().id()
}

lazy_static::lazy_static! {
    static ref GLOBAL_TRACKER: Arc<MemoryTracker> = Arc::new(MemoryTracker::new());
}

/// Provides global access to the singleton [`MemoryTracker`] instance.
///
/// This function returns a thread-safe `Arc` pointer to the global tracker,
/// allowing various parts of an application (and the allocator) to interact
/// with the same tracking data.
pub fn get_global_tracker() -> Arc<MemoryTracker> {
    GLOBAL_TRACKER.clone()
}

/// Resolves a set of unique instruction pointer addresses to symbol names.
///
/// This function uses the `backtrace` crate to resolve each instruction pointer.
/// If a symbol name is found, it attempts to demangle it using `rustc_demangle`.
/// The resolved name (or the original hex address if resolution fails) is stored.
///
/// # Arguments
/// * `unique_ips`: A `HashSet` of unique instruction pointer addresses (`usize`).
///
/// # Returns
/// A `HashMap` mapping each instruction pointer address (`usize`) to its resolved
/// and demangled symbol name (`String`).
pub fn resolve_ips(unique_ips: &HashSet<usize>) -> HashMap<usize, String> {
    let mut resolved_map = HashMap::new();
    for &ip_addr in unique_ips {
        let resolved_name = format!("{:#x}", ip_addr); // Fallback to hex address
        let _ip_void = ip_addr as *mut std::ffi::c_void;
        
        // Ensure the backtrace feature is active for symbol resolution
        #[cfg(feature = "backtrace")]
        backtrace::resolve(ip_void, |symbol| {
            if let Some(name) = symbol.name() {
                if let Some(name_str) = name.as_str() {
                     // Basic demangling attempt for Rust symbols
                    let demangled_name = rustc_demangle::try_demangle(name_str)
                        .map(|d| d.to_string())
                        .unwrap_or_else(|_| name_str.to_string());
                    resolved_name = demangled_name;
                }
            }
            // Optionally, append file:line if available
            // if let Some(filename) = symbol.filename() {
            //     if let Some(lineno) = symbol.lineno() {
            //         resolved_name.push_str(&format!(" ({}:{})", filename.to_string_lossy(), lineno));
            //     }
            // }
        });
        resolved_map.insert(ip_addr, resolved_name);
    }
    resolved_map
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;
    // HotspotInfo is in the same module (super), so direct import isn't strictly needed here
    // and can cause unused import warning if not used elsewhere in this specific test file.
    // Let's remove it and rely on `HotspotInfo` being directly accessible.

    #[test]
    fn test_new_tracker() {
        let tracker = MemoryTracker::new();
        assert!(tracker.get_active_allocations().is_empty(), "New tracker should have no active allocations");
        assert!(tracker.get_allocation_log().is_empty(), "New tracker should have an empty allocation log");
        let stats = tracker.get_stats();
        assert_eq!(stats.total_allocations, 0, "New tracker should have 0 total allocations");
        assert_eq!(stats.total_memory, 0, "New tracker should have 0 total memory");
    }

    #[test]
    fn test_track_allocation_and_stats() {
        let tracker = MemoryTracker::new();
        tracker.track_allocation(0x1000, 100, Some("TestType1".to_string())).unwrap();
        tracker.track_allocation(0x2000, 200, Some("TestType2".to_string())).unwrap();

        let active_allocs = tracker.get_active_allocations();
        assert_eq!(active_allocs.len(), 2, "Should have two active allocations");

        let alloc1 = active_allocs.iter().find(|a| a.ptr == 0x1000).expect("Allocation 0x1000 not found");
        assert_eq!(alloc1.size, 100);
        assert_eq!(alloc1.type_name.as_deref(), Some("TestType1"));

        let alloc2 = active_allocs.iter().find(|a| a.ptr == 0x2000).expect("Allocation 0x2000 not found");
        assert_eq!(alloc2.size, 200);
        assert_eq!(alloc2.type_name.as_deref(), Some("TestType2"));

        let stats = tracker.get_stats();
        assert_eq!(stats.total_allocations, 2, "Stats should show 2 total allocations");
        assert_eq!(stats.total_memory, 300, "Stats should show 300 total memory");
    }

    #[test]
    fn test_track_deallocation() {
        let tracker = MemoryTracker::new();
        tracker.track_allocation(0x3000, 50, Some("TestDealloc".to_string())).unwrap();
        
        // Ensure timestamps are different
        sleep(Duration::from_millis(1)); 
        tracker.track_deallocation(0x3000).unwrap();

        assert!(tracker.get_active_allocations().iter().find(|a| a.ptr == 0x3000).is_none(), "Allocation 0x3000 should not be active");
        
        let log = tracker.get_allocation_log();
        assert_eq!(log.len(), 1, "Allocation log should have one entry");
        let logged_alloc = &log[0];
        assert_eq!(logged_alloc.ptr, 0x3000);
        assert!(logged_alloc.timestamp_dealloc.is_some(), "Deallocation timestamp should be set");
        assert_ne!(logged_alloc.timestamp_alloc, logged_alloc.timestamp_dealloc.unwrap(), "Alloc and dealloc timestamps should differ");


        let stats = tracker.get_stats();
        assert_eq!(stats.total_allocations, 0, "Stats should show 0 active allocations after deallocation");
    }

    #[test]
    fn test_associate_var() {
        let tracker = MemoryTracker::new();
        tracker.track_allocation(0x4000, 70, Some("BaseType".to_string())).unwrap();
        tracker.associate_var(0x4000, "my_var".to_string(), "SpecificType".to_string()).unwrap();

        let active_allocs = tracker.get_active_allocations();
        let alloc_info = active_allocs.iter().find(|a| a.ptr == 0x4000).expect("Allocation 0x4000 not found");
        
        assert_eq!(alloc_info.var_name.as_deref(), Some("my_var"), "Variable name should be 'my_var'");
        assert_eq!(alloc_info.type_name.as_deref(), Some("SpecificType"), "Type name should be 'SpecificType'");

        let result = tracker.associate_var(0xBADFFF, "bad_var".to_string(), "BadType".to_string()); // Changed 0xBAD_PTR to 0xBADFFF
        assert!(result.is_err(), "Associating var to a bad pointer should return an error");
        matches!(result, Err(MemoryError::TrackingError(_)));
    }
    
    #[test]
    fn test_deallocation_of_unknown_ptr() {
        let tracker = MemoryTracker::new();
        let result = tracker.track_deallocation(0xDEADBEEF);
        assert!(result.is_err(), "Deallocating an unknown pointer should return an error");
        matches!(result, Err(MemoryError::TrackingError(_)));
    }

    #[test]
    fn test_double_deallocation() {
        let tracker = MemoryTracker::new();
        tracker.track_allocation(0x5000, 10, Some("DoubleDealloc".to_string())).unwrap();
        tracker.track_deallocation(0x5000).unwrap(); // First deallocation should be Ok

        let result = tracker.track_deallocation(0x5000); // Second deallocation
        assert!(result.is_err(), "Double deallocation should return an error");
        matches!(result, Err(MemoryError::TrackingError(_)));
    }

    #[test]
    fn test_analyze_hotspots_empty() {
        let tracker = MemoryTracker::new();
        let hotspots = tracker.analyze_hotspots();
        assert!(hotspots.is_empty(), "Hotspots should be empty for a new tracker");
    }

    #[test]
    fn test_analyze_hotspots_with_data() {
        let tracker = MemoryTracker::new();

        // Mocking AllocationInfo directly for precise control over backtrace_ips
        // Normally, track_allocation would generate these.
        let mut active_allocs = tracker.active_allocations.lock().unwrap();
        active_allocs.insert(0x100, AllocationInfo {
            ptr: 0x100, size: 10, timestamp_alloc: 0, timestamp_dealloc: None,
            var_name: None, type_name: None, backtrace_ips: vec![1, 2, 3], thread_id: 1,
        });
        active_allocs.insert(0x200, AllocationInfo {
            ptr: 0x200, size: 20, timestamp_alloc: 0, timestamp_dealloc: None,
            var_name: None, type_name: None, backtrace_ips: vec![1, 2, 3], thread_id: 1,
        });
        active_allocs.insert(0x300, AllocationInfo {
            ptr: 0x300, size: 30, timestamp_alloc: 0, timestamp_dealloc: None,
            var_name: None, type_name: None, backtrace_ips: vec![4, 5, 6], thread_id: 1,
        });
        active_allocs.insert(0x400, AllocationInfo { // Allocation with empty backtrace
            ptr: 0x400, size: 5, timestamp_alloc: 0, timestamp_dealloc: None,
            var_name: None, type_name: None, backtrace_ips: vec![], thread_id: 1,
        });
        drop(active_allocs); // Release lock

        // Deallocated allocation
        let mut logged_allocs = tracker.allocation_log.lock().unwrap();
        logged_allocs.push(AllocationInfo {
            ptr: 0x500, size: 15, timestamp_alloc: 0, timestamp_dealloc: Some(1),
            var_name: None, type_name: None, backtrace_ips: vec![1, 2, 3], thread_id: 1,
        });
        drop(logged_allocs); // Release lock

        let hotspots = tracker.analyze_hotspots();

        assert_eq!(hotspots.len(), 2, "Expected 2 hotspots");

        // Hotspot 1: backtrace [1,2,3]
        let hotspot1 = hotspots.iter().find(|h| h.backtrace_ips == vec![1, 2, 3]);
        assert!(hotspot1.is_some(), "Hotspot for [1,2,3] not found");
        if let Some(h) = hotspot1 {
            assert_eq!(h.count, 3, "Count for hotspot [1,2,3] should be 3");
            assert_eq!(h.total_size, 45, "Total size for hotspot [1,2,3] should be 45 (10+20+15)");
        }

        // Hotspot 2: backtrace [4,5,6]
        let hotspot2 = hotspots.iter().find(|h| h.backtrace_ips == vec![4, 5, 6]);
        assert!(hotspot2.is_some(), "Hotspot for [4,5,6] not found");
        if let Some(h) = hotspot2 {
            assert_eq!(h.count, 1, "Count for hotspot [4,5,6] should be 1");
            assert_eq!(h.total_size, 30, "Total size for hotspot [4,5,6] should be 30");
        }
        
        // Check order (hotspot1 should be first due to larger total_size)
        assert_eq!(hotspots[0].backtrace_ips, vec![1, 2, 3], "First hotspot should be [1,2,3] due to size");
        assert_eq!(hotspots[1].backtrace_ips, vec![4, 5, 6], "Second hotspot should be [4,5,6]");
    }
}