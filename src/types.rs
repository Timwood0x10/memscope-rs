//! Core types and error handling for the trace_tools library.
//!
//! This module defines the core data structures and error types used throughout
//! the trace_tools library.

use std::collections::HashMap;
use std::sync::{Mutex, Arc};
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use crossbeam::queue::SegQueue;
use serde::{Serialize, Deserialize};

/// Error type for memory tracking operations
#[derive(Debug, thiserror::Error)]
pub enum TrackingError {
    #[error("Failed to acquire allocation lock: {0}")]
    LockError(String),
    
    #[error("Invalid pointer association: {ptr:?}")]
    InvalidPointer { ptr: usize },
    
    #[error("Allocation tracking disabled")]
    TrackingDisabled,
    
    #[error("Memory corruption detected")]
    MemoryCorruption,
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type TrackingResult<T> = Result<T, TrackingError>;

/// Allocation information for a tracked memory allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationInfo {
    /// Memory address of the allocation
    pub ptr: usize,
    /// Size of the allocation in bytes
    pub size: usize,
    /// Timestamp when the allocation occurred (in milliseconds since UNIX_EPOCH)
    pub timestamp_alloc: u128,
    pub timestamp_dealloc: Option<u128>,
    pub var_name: Option<String>,
    pub fn_context: Option<String>,
    pub type_name: Option<String>,
    pub stack_trace: Option<String>,
    pub thread_id: Option<String>,
}

impl From<crate::tracker::AllocationInfo> for AllocationInfo {
    fn from(info: crate::tracker::AllocationInfo) -> Self {
        Self {
            ptr: info.ptr,
            size: info.size,
            timestamp_alloc: info.timestamp_alloc,
            timestamp_dealloc: info.timestamp_dealloc,
            var_name: info.var_name,
            type_name: info.type_name,
            fn_context: None,
            stack_trace: Some(format!("{:?}", info.backtrace_ips)),
            thread_id: Some(info.thread_id.to_string()),
        }
    }
}

impl Default for AllocationInfo {
    fn default() -> Self {
        Self {
            ptr: 0,
            size: 0,
            timestamp_alloc: 0,
            timestamp_dealloc: None,
            var_name: None,
            fn_context: None,
            type_name: None,
            stack_trace: None,
            thread_id: None,
        }
    }
}

/// Event type for memory allocation tracking
#[derive(Debug, Clone)]
pub enum AllocationEvent {
    /// Allocation event
    Alloc { 
        ptr: usize, 
        size: usize, 
        timestamp: u128,
        thread_id: String,
    },
    Dealloc { 
        ptr: usize, 
        timestamp: u128,
        thread_id: String,
    },
    Associate { 
        ptr: usize, 
        var_name: String, 
        type_name: String,
        fn_context: Option<String>,
        stack_trace: Option<String>,
    },
}

lazy_static::lazy_static! {
    pub static ref ACTIVE_ALLOCATIONS: Mutex<HashMap<usize, AllocationInfo>> = Mutex::new(HashMap::new());
    pub static ref ALLOCATION_LOG: Mutex<Vec<AllocationInfo>> = Mutex::new(Vec::new());
    pub static ref ALLOC_COUNTER: AtomicUsize = AtomicUsize::new(0);
    pub static ref TRACKING_ENABLED: AtomicBool = AtomicBool::new(true);
    
    // Lock-free event queue for high-performance allocation tracking
    pub static ref EVENT_QUEUE: SegQueue<AllocationEvent> = SegQueue::new();
    
    // Event processor handle
    pub static ref EVENT_PROCESSOR: Arc<AllocationProcessor> = Arc::new(AllocationProcessor::new());
}

/// Event Processing System
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct AllocationProcessor {
    sender: Sender<AllocationEvent>,
    _handle: JoinHandle<()>,
}

impl AllocationProcessor {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        let handle = thread::spawn(move || {
            Self::process_events(receiver);
        });
        
        Self {
            sender,
            _handle: handle,
        }
    }
    
    pub fn send_event(&self, event: AllocationEvent) -> TrackingResult<()> {
        if !TRACKING_ENABLED.load(Ordering::Relaxed) {
            return Err(TrackingError::TrackingDisabled);
        }
        
        self.sender.send(event)
            .map_err(|e| TrackingError::LockError(format!("Failed to send event: {}", e)))
    }
    
    fn process_events(receiver: Receiver<AllocationEvent>) {
        while let Ok(event) = receiver.recv() {
            if let Err(e) = Self::handle_event(event) {
                eprintln!("Error processing allocation event: {}", e);
            }
        }
    }
    
    fn handle_event(event: AllocationEvent) -> TrackingResult<()> {
        match event {
            AllocationEvent::Alloc { ptr, size, timestamp, thread_id } => {
                let info = AllocationInfo {
                    ptr,
                    size,
                    timestamp_alloc: timestamp,
                    timestamp_dealloc: None,
                    var_name: None,
                    fn_context: None,
                    type_name: None,
                    stack_trace: None,
                    thread_id: Some(thread_id),
                };
                
                ACTIVE_ALLOCATIONS.lock().unwrap().insert(ptr, info);
            },
            
            AllocationEvent::Dealloc { ptr, timestamp, thread_id: _ } => {
                if let Ok(mut active) = ACTIVE_ALLOCATIONS.lock() {
                    if let Some(mut info) = active.remove(&ptr) {
                        info.timestamp_dealloc = Some(timestamp);
                        if let Ok(mut log) = ALLOCATION_LOG.lock() {
                            log.push(info);
                        }
                    }
                }
            },
            
            AllocationEvent::Associate { ptr, var_name, type_name, fn_context, stack_trace } => {
                if let Ok(mut active) = ACTIVE_ALLOCATIONS.lock() {
                    if let Some(info) = active.get_mut(&ptr) {
                        info.var_name = Some(var_name);
                        info.type_name = Some(type_name);
                        info.fn_context = fn_context;
                        info.stack_trace = stack_trace;
                    }
                }
            }
        }
        Ok(())
    }
}

// --- Memory Leak Detection ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLeak {
    pub allocation_info: AllocationInfo,
    pub suspected_leak_duration: u128,
    pub severity: LeakSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LeakSeverity {
    Low,
    Medium,
    High,
    Critical,
}

pub struct LeakDetector {
    leak_threshold_ms: u128,
    critical_threshold_ms: u128,
}

impl LeakDetector {
    pub fn new() -> Self {
        Self {
            leak_threshold_ms: 30_000, // 30 seconds
            critical_threshold_ms: 300_000, // 5 minutes
        }
    }
    
    pub fn analyze_leaks(&self) -> TrackingResult<Vec<MemoryLeak>> {
        let current_time = current_timestamp();
        let mut leaks = Vec::new();
        
        let active = ACTIVE_ALLOCATIONS.lock()
            .map_err(|e| TrackingError::LockError(format!("Failed to lock active allocations: {}", e)))?;
        
        for (_, info) in active.iter() {
            let duration = current_time - info.timestamp_alloc;
            
            if duration > self.leak_threshold_ms {
                let severity = if duration > self.critical_threshold_ms {
                    LeakSeverity::Critical
                } else if duration > self.leak_threshold_ms * 3 {
                    LeakSeverity::High
                } else if duration > self.leak_threshold_ms * 2 {
                    LeakSeverity::Medium
                } else {
                    LeakSeverity::Low
                };
                
                leaks.push(MemoryLeak {
                    allocation_info: info.clone(),
                    suspected_leak_duration: duration,
                    severity,
                });
            }
        }
        
        Ok(leaks)
    }
}

// --- Visualization Data Structures ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    pub timestamp: u128,
    pub active_allocations: Vec<AllocationVisualization>,
    pub total_allocated: usize,
    pub total_deallocated: usize,
    pub allocation_count: usize,
    pub deallocation_count: usize,
    pub peak_memory_usage: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationVisualization {
    pub var_name: String,
    pub type_name: String,
    pub size: usize,
    pub lifetime_ms: Option<u128>,
    pub function_context: Option<String>,
    pub stack_trace: Option<String>,
    pub thread_id: Option<String>,
}

// --- Enhanced Utility Functions ---
pub fn current_timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

pub fn get_thread_id() -> String {
    format!("{:?}", thread::current().id())
}

#[cfg(feature = "backtrace")]
pub fn capture_stack_trace() -> Option<String> {
    use backtrace::Backtrace;
    let bt = Backtrace::new();
    Some(format!("{:?}", bt))
}

#[cfg(not(feature = "backtrace"))]
pub fn capture_stack_trace() -> Option<String> {
    None
}

pub fn associate_variable_with_ptr(ptr: usize, var_name: &str, type_name_str: &str) -> TrackingResult<()> {
    let event = AllocationEvent::Associate {
        ptr,
        var_name: var_name.to_string(),
        type_name: type_name_str.to_string(),
        fn_context: None,
        stack_trace: capture_stack_trace(),
    };
    
    EVENT_PROCESSOR.send_event(event)
}

pub fn generate_memory_timeline() -> TrackingResult<Vec<MemorySnapshot>> {
    let log = ALLOCATION_LOG.lock()
        .map_err(|e| TrackingError::LockError(format!("Failed to lock allocation log: {}", e)))?;
    
    let active = ACTIVE_ALLOCATIONS.lock()
        .map_err(|e| TrackingError::LockError(format!("Failed to lock active allocations: {}", e)))?;
    
    let mut snapshots = Vec::new();
    let current_time = current_timestamp();
    
    // Create current snapshot
    let active_visualizations: Vec<AllocationVisualization> = active.iter().map(|(_, info)| {
        AllocationVisualization {
            var_name: info.var_name.clone().unwrap_or_else(|| "unknown".to_string()),
            type_name: info.type_name.clone().unwrap_or_else(|| "unknown".to_string()),
            size: info.size,
            lifetime_ms: Some(current_time - info.timestamp_alloc),
            function_context: info.fn_context.clone(),
            stack_trace: info.stack_trace.clone(),
            thread_id: info.thread_id.clone(),
        }
    }).collect();
    
    let total_allocated: usize = active.values().map(|info| info.size).sum();
    let total_deallocated: usize = log.iter().map(|info| info.size).sum();
    
    snapshots.push(MemorySnapshot {
        timestamp: current_time,
        active_allocations: active_visualizations,
        total_allocated,
        total_deallocated,
        allocation_count: active.len(),
        deallocation_count: log.len(),
        peak_memory_usage: total_allocated,
    });
    
    Ok(snapshots)
}

// --- Enhanced TrackedVec Implementation ---
pub struct TrackedVec<T> {
    inner: Vec<T>,
    var_name: String,
    type_name_str: String,
}

impl<T> TrackedVec<T> {
    pub fn new(vec: Vec<T>, var_name: &str) -> Self {
        let type_name_str = std::any::type_name::<Vec<T>>().to_string();
        let s = Self {
            inner: vec,
            var_name: var_name.to_string(),
            type_name_str,
        };
        
        s.associate_if_allocated();
        s
    }
    
    pub fn with_capacity(capacity: usize, var_name: &str) -> Self {
        Self::new(Vec::with_capacity(capacity), var_name)
    }
    
    fn associate_if_allocated(&self) {
        if let Some(ptr) = self.get_pointer_if_allocated() {
            let _ = associate_variable_with_ptr(ptr, &self.var_name, &self.type_name_str);
        }
    }
    
    fn get_pointer_if_allocated(&self) -> Option<usize> {
        if self.inner.capacity() > 0 {
            Some(self.inner.as_ptr() as usize)
        } else {
            None
        }
    }
    
    fn update_ptr_after_reallocation(&self, old_ptr_option: Option<usize>) {
        if let Some(new_ptr) = self.get_pointer_if_allocated() {
            if old_ptr_option != Some(new_ptr) {
                let _ = associate_variable_with_ptr(new_ptr, &self.var_name, &self.type_name_str);
            }
        }
    }
    
    // Enhanced Vec methods with proper tracking
    pub fn push(&mut self, value: T) {
        let old_ptr = self.get_pointer_if_allocated();
        self.inner.push(value);
        self.update_ptr_after_reallocation(old_ptr);
    }
    
    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop()
    }
    
    pub fn reserve(&mut self, additional: usize) {
        let old_ptr = self.get_pointer_if_allocated();
        self.inner.reserve(additional);
        self.update_ptr_after_reallocation(old_ptr);
    }
    
    pub fn shrink_to_fit(&mut self) {
        let old_ptr = self.get_pointer_if_allocated();
        self.inner.shrink_to_fit();
        self.update_ptr_after_reallocation(old_ptr);
    }
    
    pub fn insert(&mut self, index: usize, element: T) {
        let old_ptr = self.get_pointer_if_allocated();
        self.inner.insert(index, element);
        self.update_ptr_after_reallocation(old_ptr);
    }
    
    pub fn remove(&mut self, index: usize) -> T {
        self.inner.remove(index)
    }
    
    pub fn clear(&mut self) {
        self.inner.clear();
    }
    
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }
    
    pub fn as_slice(&self) -> &[T] {
        self.inner.as_slice()
    }
    
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.inner.as_mut_slice()
    }
}

impl<T> std::ops::Deref for TrackedVec<T> {
    type Target = [T];
    
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> std::ops::DerefMut for TrackedVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> std::ops::Index<usize> for TrackedVec<T> {
    type Output = T;
    
    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl<T> std::ops::IndexMut<usize> for TrackedVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for TrackedVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrackedVec")
            .field("var_name", &self.var_name)
            .field("type_name", &self.type_name_str)
            .field("inner", &self.inner)
            .finish()
    }
}