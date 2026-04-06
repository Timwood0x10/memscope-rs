//! Lockfree memory tracker implementation.
//!
//! This module contains the ThreadLocalTracker for thread-local memory tracking
//! using lock-free data structures for optimal concurrent performance.

use std::{
    fs::File,
    io::Write,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
        Arc, OnceLock,
    },
    thread::ThreadId,
};

use crossbeam::queue::SegQueue;
use dashmap::DashMap;

use super::lockfree_types::{Event, EventType, MemoryStats};

static TRACKING_ENABLED: AtomicBool = AtomicBool::new(false);
static OUTPUT_DIRECTORY: OnceLock<std::path::PathBuf> = OnceLock::new();

/// Thread-local memory tracker using lock-free data structures.
///
/// This tracker uses SegQueue for events and DashMap for active allocations,
/// ensuring no event loss under high contention.
pub struct ThreadLocalTracker {
    thread_id: ThreadId,
    events: Arc<SegQueue<Event>>,
    active_allocations: Arc<DashMap<usize, usize>>,
    total_allocations: AtomicU64,
    total_allocated: AtomicU64,
    total_deallocations: AtomicU64,
    total_deallocated: AtomicU64,
    active_memory: AtomicU64,
    peak_memory: AtomicU64,
    output_file: PathBuf,
    sample_rate: f64,
    total_seen: AtomicUsize,
    total_tracked: AtomicUsize,
}

impl ThreadLocalTracker {
    pub fn new(thread_id: ThreadId, output_file: PathBuf, sample_rate: f64) -> Self {
        Self {
            thread_id,
            events: Arc::new(SegQueue::new()),
            active_allocations: Arc::new(DashMap::new()),
            total_allocations: AtomicU64::new(0),
            total_allocated: AtomicU64::new(0),
            total_deallocations: AtomicU64::new(0),
            total_deallocated: AtomicU64::new(0),
            active_memory: AtomicU64::new(0),
            peak_memory: AtomicU64::new(0),
            output_file,
            sample_rate: sample_rate.clamp(0.0, 1.0),
            total_seen: AtomicUsize::new(0),
            total_tracked: AtomicUsize::new(0),
        }
    }

    pub fn track_allocation(&self, ptr: usize, size: usize, call_stack_hash: u64) {
        self.total_seen.fetch_add(1, Ordering::Relaxed);

        if self.sample_rate < 1.0 {
            let sample_decision = rand::random::<f64>();
            if sample_decision >= self.sample_rate {
                return;
            }
        }

        self.total_tracked.fetch_add(1, Ordering::Relaxed);

        let event = Event::allocation(ptr, size, call_stack_hash, self.thread_id);
        self.events.push(event);

        self.active_allocations.insert(ptr, size);

        self.total_allocations.fetch_add(1, Ordering::Relaxed);
        self.total_allocated
            .fetch_add(size as u64, Ordering::Relaxed);

        let new_active = self.active_memory.fetch_add(size as u64, Ordering::Relaxed) + size as u64;

        let mut current_peak = self.peak_memory.load(Ordering::Relaxed);
        while new_active > current_peak {
            match self.peak_memory.compare_exchange_weak(
                current_peak,
                new_active,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_peak = actual,
            }
        }
    }

    pub fn track_deallocation(&self, ptr: usize, call_stack_hash: u64) {
        let size = self
            .active_allocations
            .remove(&ptr)
            .map(|(_, v)| v)
            .unwrap_or(0);

        let event = Event::deallocation(ptr, size, call_stack_hash, self.thread_id);
        self.events.push(event);

        self.total_deallocations.fetch_add(1, Ordering::Relaxed);
        self.total_deallocated
            .fetch_add(size as u64, Ordering::Relaxed);
        self.active_memory.fetch_sub(size as u64, Ordering::Relaxed);
    }

    pub fn get_stats(&self) -> MemoryStats {
        MemoryStats {
            total_allocations: self.total_allocations.load(Ordering::Relaxed) as usize,
            total_allocated: self.total_allocated.load(Ordering::Relaxed) as usize,
            total_deallocations: self.total_deallocations.load(Ordering::Relaxed) as usize,
            total_deallocated: self.total_deallocated.load(Ordering::Relaxed) as usize,
            active_memory: self.active_memory.load(Ordering::Relaxed) as usize,
            peak_memory: self.peak_memory.load(Ordering::Relaxed) as usize,
        }
    }

    pub fn get_sampling_stats(&self) -> (usize, usize) {
        (
            self.total_seen.load(Ordering::Relaxed),
            self.total_tracked.load(Ordering::Relaxed),
        )
    }

    pub fn finalize(&self) -> std::io::Result<()> {
        let mut events = Vec::new();
        while let Some(event) = self.events.pop() {
            events.push(event);
        }

        if events.is_empty() {
            return Ok(());
        }

        if let Some(parent) = self.output_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = File::create(&self.output_file)?;

        let header = "MEMSCOPE_LOCKFREE";
        file.write_all(header.as_bytes())?;

        for event in events {
            self.write_event(&mut file, &event)?;
        }

        file.flush()?;
        Ok(())
    }

    fn write_event(&self, file: &mut File, event: &Event) -> std::io::Result<()> {
        let event_type_byte = match event.event_type {
            EventType::Allocation => 1u8,
            EventType::Deallocation => 2u8,
        };
        file.write_all(&event_type_byte.to_le_bytes())?;
        file.write_all(&event.timestamp.to_le_bytes())?;
        file.write_all(&event.ptr.to_le_bytes())?;
        file.write_all(&event.size.to_le_bytes())?;
        file.write_all(&event.call_stack_hash.to_le_bytes())?;
        Ok(())
    }

    pub fn thread_id(&self) -> ThreadId {
        self.thread_id
    }

    pub fn output_file(&self) -> &PathBuf {
        &self.output_file
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    pub fn clear_events(&self) {
        while self.events.pop().is_some() {}
    }
}

impl Drop for ThreadLocalTracker {
    fn drop(&mut self) {
        if let Err(e) = self.finalize() {
            tracing::warn!("Failed to finalize thread-local tracker: {}", e);
        }
    }
}

pub fn calculate_call_stack_hash(call_stack: &[usize]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    for addr in call_stack {
        addr.hash(&mut hasher);
    }
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_local_tracker_creation() {
        let thread_id = std::thread::current().id();
        let output_file = PathBuf::from("/tmp/test_tracker.bin");
        let tracker = ThreadLocalTracker::new(thread_id, output_file, 1.0);

        assert_eq!(tracker.thread_id(), thread_id);
        assert_eq!(tracker.event_count(), 0);
    }

    #[test]
    fn test_allocation_tracking() {
        let thread_id = std::thread::current().id();
        let output_file = PathBuf::from("/tmp/test_tracker2.bin");
        let tracker = ThreadLocalTracker::new(thread_id, output_file, 1.0);

        tracker.track_allocation(0x1000, 1024, 12345);

        let stats = tracker.get_stats();
        assert_eq!(stats.total_allocations, 1);
        assert_eq!(stats.total_allocated, 1024);
        assert_eq!(stats.active_memory, 1024);
        assert_eq!(tracker.event_count(), 1);
    }

    #[test]
    fn test_deallocation_tracking() {
        let thread_id = std::thread::current().id();
        let output_file = PathBuf::from("/tmp/test_tracker3.bin");
        let tracker = ThreadLocalTracker::new(thread_id, output_file, 1.0);

        tracker.track_allocation(0x1000, 1024, 12345);
        tracker.track_deallocation(0x1000, 12345);

        let stats = tracker.get_stats();
        assert_eq!(stats.total_allocations, 1);
        assert_eq!(stats.total_deallocations, 1);
        assert_eq!(stats.active_memory, 0);
    }

    #[test]
    fn test_call_stack_hash() {
        let call_stack = vec![0x1000, 0x2000, 0x3000];
        let hash1 = calculate_call_stack_hash(&call_stack);
        let hash2 = calculate_call_stack_hash(&call_stack);

        assert_eq!(hash1, hash2);

        let different_stack = vec![0x1000, 0x2000, 0x4000];
        let hash3 = calculate_call_stack_hash(&different_stack);
        assert_ne!(hash1, hash3);
    }
}

thread_local! {
    static THREAD_TRACKER: std::cell::RefCell<Option<ThreadLocalTracker>> = const { std::cell::RefCell::new(None) };
}

fn get_thread_id() -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let thread_id = std::thread::current().id();
    let mut hasher = DefaultHasher::new();
    thread_id.hash(&mut hasher);
    hasher.finish()
}

pub fn init_thread_tracker(
    output_dir: &std::path::Path,
    sample_rate: Option<f64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let sample_rate = sample_rate.unwrap_or(1.0);
    let thread_id = std::thread::current().id();
    let output_file = output_dir.join(format!("memscope_thread_{}.bin", get_thread_id()));

    let tracker = ThreadLocalTracker::new(thread_id, output_file, sample_rate);

    THREAD_TRACKER.with(|thread_tracker| {
        *thread_tracker.borrow_mut() = Some(tracker);
    });

    Ok(())
}

pub fn track_allocation_lockfree(
    ptr: usize,
    size: usize,
    call_stack_hash: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    THREAD_TRACKER.with(|thread_tracker| {
        if let Some(ref tracker) = *thread_tracker.borrow() {
            tracker.track_allocation(ptr, size, call_stack_hash);
            Ok(())
        } else {
            Err("Thread tracker not initialized. Call init_thread_tracker() first.".into())
        }
    })
}

pub fn track_deallocation_lockfree(
    ptr: usize,
    call_stack_hash: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    THREAD_TRACKER.with(|thread_tracker| {
        if let Some(ref tracker) = *thread_tracker.borrow() {
            tracker.track_deallocation(ptr, call_stack_hash);
            Ok(())
        } else {
            Err("Thread tracker not initialized. Call init_thread_tracker() first.".into())
        }
    })
}

pub fn finalize_thread_tracker() -> Result<(), Box<dyn std::error::Error>> {
    THREAD_TRACKER.with(|thread_tracker| {
        let mut tracker_ref = thread_tracker.borrow_mut();
        if let Some(ref mut tracker) = *tracker_ref {
            tracker
                .finalize()
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        } else {
            Ok(())
        }
    })
}

pub fn get_current_tracker() -> Option<ThreadLocalTracker> {
    THREAD_TRACKER.with(|thread_tracker| {
        thread_tracker
            .borrow()
            .as_ref()
            .map(|tracker| ThreadLocalTracker {
                thread_id: tracker.thread_id,
                events: Arc::clone(&tracker.events),
                active_allocations: Arc::clone(&tracker.active_allocations),
                total_allocations: AtomicU64::new(
                    tracker.total_allocations.load(Ordering::Relaxed),
                ),
                total_allocated: AtomicU64::new(tracker.total_allocated.load(Ordering::Relaxed)),
                total_deallocations: AtomicU64::new(
                    tracker.total_deallocations.load(Ordering::Relaxed),
                ),
                total_deallocated: AtomicU64::new(
                    tracker.total_deallocated.load(Ordering::Relaxed),
                ),
                active_memory: AtomicU64::new(tracker.active_memory.load(Ordering::Relaxed)),
                peak_memory: AtomicU64::new(tracker.peak_memory.load(Ordering::Relaxed)),
                output_file: tracker.output_file.clone(),
                sample_rate: tracker.sample_rate,
                total_seen: AtomicUsize::new(tracker.total_seen.load(Ordering::Relaxed)),
                total_tracked: AtomicUsize::new(tracker.total_tracked.load(Ordering::Relaxed)),
            })
    })
}

pub fn trace_all<P: AsRef<std::path::Path>>(
    output_dir: &P,
) -> Result<(), Box<dyn std::error::Error>> {
    let output_path = output_dir.as_ref().to_path_buf();

    let _ = OUTPUT_DIRECTORY.set(output_path.clone());

    if output_path.exists() {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let backup_name = format!(
            "{}.backup.{}",
            output_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            timestamp
        );
        let backup_path = output_path.with_file_name(backup_name);
        std::fs::rename(&output_path, &backup_path)?;
        tracing::info!("Existing directory backed up to: {}", backup_path.display());
    }
    std::fs::create_dir_all(&output_path)?;

    TRACKING_ENABLED.store(true, Ordering::SeqCst);

    tracing::info!("Lockfree tracking started: {}", output_path.display());

    Ok(())
}

pub fn trace_thread<P: AsRef<std::path::Path>>(
    output_dir: &P,
) -> Result<(), Box<dyn std::error::Error>> {
    let output_path = output_dir.as_ref().to_path_buf();

    if !output_path.exists() {
        std::fs::create_dir_all(&output_path)?;
    }

    init_thread_tracker(&output_path, Some(1.0))?;

    Ok(())
}

pub fn stop_tracing() -> Result<(), Box<dyn std::error::Error>> {
    if !TRACKING_ENABLED.load(Ordering::SeqCst) {
        return Ok(());
    }

    let _ = finalize_thread_tracker();

    TRACKING_ENABLED.store(false, Ordering::SeqCst);

    Ok(())
}

pub fn is_tracking() -> bool {
    TRACKING_ENABLED.load(Ordering::SeqCst)
}

pub fn memory_snapshot() -> super::lockfree_types::MemorySnapshot {
    use super::lockfree_types::MemorySnapshot;

    let (current_mb, peak_mb, allocations, deallocations) = THREAD_TRACKER.with(|thread_tracker| {
        if let Some(tracker) = thread_tracker.borrow().as_ref() {
            let stats = tracker.get_stats();
            (
                stats.active_memory as f64 / (1024.0 * 1024.0),
                stats.peak_memory as f64 / (1024.0 * 1024.0),
                stats.total_allocations,
                stats.total_deallocations,
            )
        } else {
            (0.0, 0.0, 0, 0)
        }
    });

    MemorySnapshot {
        current_mb,
        peak_mb,
        allocations: allocations as u64,
        deallocations: deallocations as u64,
        active_threads: if TRACKING_ENABLED.load(Ordering::SeqCst) {
            1
        } else {
            0
        },
    }
}

pub fn quick_trace<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let temp_dir = std::env::temp_dir().join("memscope_lockfree_quick");

    if trace_all(&temp_dir).is_err() {
        return f();
    }

    let result = f();

    let _ = stop_tracing();

    tracing::info!("Quick trace completed - check {}", temp_dir.display());

    result
}

#[cfg(test)]
mod global_api_tests {
    use super::*;

    #[test]
    fn test_init_thread_tracker() {
        let temp_dir = std::env::temp_dir().join("memscope_global_test");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let result = init_thread_tracker(&temp_dir, Some(1.0));
        assert!(result.is_ok(), "Should successfully initialize tracker");

        let result2 = init_thread_tracker(&temp_dir, Some(0.5));
        assert!(result2.is_ok(), "Should handle duplicate initialization");
    }

    #[test]
    fn test_track_without_init() {
        THREAD_TRACKER.with(|t| {
            *t.borrow_mut() = None;
        });

        let result = track_allocation_lockfree(0x1000, 1024, 12345);
        assert!(result.is_err(), "Should fail without initialization");
    }

    #[test]
    fn test_finalize_without_init() {
        THREAD_TRACKER.with(|t| {
            *t.borrow_mut() = None;
        });

        let result = finalize_thread_tracker();
        assert!(
            result.is_ok(),
            "Should handle finalization without initialization"
        );
    }

    #[test]
    fn test_global_api_workflow() {
        let temp_dir = std::env::temp_dir().join("memscope_workflow_test");
        std::fs::create_dir_all(&temp_dir).unwrap();

        init_thread_tracker(&temp_dir, Some(1.0)).unwrap();

        track_allocation_lockfree(0x1000, 1024, 12345).unwrap();
        track_deallocation_lockfree(0x1000, 12345).unwrap();

        let tracker = get_current_tracker();
        assert!(tracker.is_some(), "Should have active tracker");

        if let Some(t) = tracker {
            let stats = t.get_stats();
            assert_eq!(stats.total_allocations, 1);
            assert_eq!(stats.total_deallocations, 1);
        }

        finalize_thread_tracker().unwrap();
    }
}
