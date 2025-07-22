//! Async type tracking and analysis
//!
//! This module implements async type analysis features from ComplexTypeForRust.md:
//! - Future and Stream state machine analysis
//! - Async task lifecycle tracking
//! - Await point analysis

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Global async analyzer instance
static GLOBAL_ASYNC_ANALYZER: OnceLock<Arc<AsyncAnalyzer>> = OnceLock::new();

/// Get the global async analyzer instance
pub fn get_global_async_analyzer() -> Arc<AsyncAnalyzer> {
    GLOBAL_ASYNC_ANALYZER
        .get_or_init(|| Arc::new(AsyncAnalyzer::new()))
        .clone()
}

/// Async type analysis system
pub struct AsyncAnalyzer {
    /// Active futures tracking
    active_futures: Mutex<HashMap<usize, FutureInfo>>,
    /// Future state transitions
    state_transitions: Mutex<Vec<StateTransition>>,
    /// Await point analysis
    await_points: Mutex<Vec<AwaitPoint>>,
    /// Task lifecycle events
    task_events: Mutex<Vec<TaskEvent>>,
}

impl AsyncAnalyzer {
    /// Create a new async analyzer
    pub fn new() -> Self {
        Self {
            active_futures: Mutex::new(HashMap::new()),
            state_transitions: Mutex::new(Vec::new()),
            await_points: Mutex::new(Vec::new()),
            task_events: Mutex::new(Vec::new()),
        }
    }

    /// Track a new future
    pub fn track_future(&self, ptr: usize, future_type: &str, initial_state: FutureState) {
        let future_info = FutureInfo {
            ptr,
            future_type: future_type.to_string(),
            current_state: initial_state.clone(),
            creation_time: current_timestamp(),
            completion_time: None,
            state_history: vec![initial_state.clone()],
            await_count: 0,
            poll_count: 0,
            thread_id: format!("{:?}", std::thread::current().id()),
        };

        if let Ok(mut futures) = self.active_futures.lock() {
            futures.insert(ptr, future_info);
        }

        // Record task creation event
        let event = TaskEvent {
            ptr,
            event_type: TaskEventType::Created,
            timestamp: current_timestamp(),
            thread_id: std::thread::current().id(),
            details: format!("Future {} created", future_type),
        };

        if let Ok(mut events) = self.task_events.lock() {
            events.push(event);
        }
    }

    /// Record a state transition
    pub fn record_state_transition(&self, ptr: usize, from_state: FutureState, to_state: FutureState) {
        let transition = StateTransition {
            ptr,
            from_state: from_state.clone(),
            to_state: to_state.clone(),
            timestamp: current_timestamp(),
            thread_id: std::thread::current().id(),
        };

        if let Ok(mut transitions) = self.state_transitions.lock() {
            transitions.push(transition);
        }

        // Update future info
        if let Ok(mut futures) = self.active_futures.lock() {
            if let Some(future_info) = futures.get_mut(&ptr) {
                future_info.current_state = to_state.clone();
                future_info.state_history.push(to_state.clone());
                
                if matches!(to_state, FutureState::Pending) {
                    future_info.poll_count += 1;
                }
            }
        }
    }

    /// Record an await point
    pub fn record_await_point(&self, ptr: usize, location: &str, await_type: AwaitType) {
        let await_point = AwaitPoint {
            ptr,
            location: location.to_string(),
            await_type,
            timestamp: current_timestamp(),
            thread_id: std::thread::current().id(),
            duration: None, // Will be filled when await completes
        };

        if let Ok(mut awaits) = self.await_points.lock() {
            awaits.push(await_point);
        }

        // Update await count
        if let Ok(mut futures) = self.active_futures.lock() {
            if let Some(future_info) = futures.get_mut(&ptr) {
                future_info.await_count += 1;
            }
        }
    }

    /// Complete an await point
    pub fn complete_await_point(&self, ptr: usize, location: &str) {
        let completion_time = current_timestamp();
        
        if let Ok(mut awaits) = self.await_points.lock() {
            // Find the most recent await point for this location
            for await_point in awaits.iter_mut().rev() {
                if await_point.ptr == ptr && await_point.location == location && await_point.duration.is_none() {
                    await_point.duration = Some(completion_time - await_point.timestamp);
                    break;
                }
            }
        }
    }

    /// Mark a future as completed
    pub fn complete_future(&self, ptr: usize, result: FutureResult) {
        let completion_time = current_timestamp();

        if let Ok(mut futures) = self.active_futures.lock() {
            if let Some(future_info) = futures.get_mut(&ptr) {
                future_info.completion_time = Some(completion_time);
                future_info.current_state = match result {
                    FutureResult::Ready => FutureState::Ready,
                    FutureResult::Cancelled => FutureState::Cancelled,
                    FutureResult::Panicked => FutureState::Panicked,
                };
            }
        }

        // Record completion event
        let event = TaskEvent {
            ptr,
            event_type: TaskEventType::Completed,
            timestamp: completion_time,
            thread_id: std::thread::current().id(),
            details: format!("Future completed with result: {:?}", result),
        };

        if let Ok(mut events) = self.task_events.lock() {
            events.push(event);
        }
    }

    /// Get async statistics
    pub fn get_async_statistics(&self) -> AsyncStatistics {
        let futures = self.active_futures.lock().unwrap();
        let transitions = self.state_transitions.lock().unwrap();
        let awaits = self.await_points.lock().unwrap();
        let _events = self.task_events.lock().unwrap();

        let total_futures = futures.len();
        let completed_futures = futures.values().filter(|f| f.completion_time.is_some()).count();
        let active_futures = total_futures - completed_futures;

        // Calculate average completion time
        let completion_times: Vec<u64> = futures.values()
            .filter_map(|f| {
                if let (Some(completion), creation) = (f.completion_time, f.creation_time) {
                    Some(completion - creation)
                } else {
                    None
                }
            })
            .collect();

        let avg_completion_time = if !completion_times.is_empty() {
            completion_times.iter().sum::<u64>() / completion_times.len() as u64
        } else {
            0
        };

        // Calculate await statistics
        let total_awaits = awaits.len();
        let completed_awaits = awaits.iter().filter(|a| a.duration.is_some()).count();
        
        let await_durations: Vec<u64> = awaits.iter()
            .filter_map(|a| a.duration)
            .collect();

        let avg_await_duration = if !await_durations.is_empty() {
            await_durations.iter().sum::<u64>() / await_durations.len() as u64
        } else {
            0
        };

        // Count by future type
        let mut by_type = HashMap::new();
        for future in futures.values() {
            *by_type.entry(future.future_type.clone()).or_insert(0) += 1;
        }

        AsyncStatistics {
            total_futures,
            active_futures,
            completed_futures,
            total_state_transitions: transitions.len(),
            total_awaits,
            completed_awaits,
            avg_completion_time,
            avg_await_duration,
            by_type,
        }
    }

    /// Analyze async patterns
    pub fn analyze_async_patterns(&self) -> AsyncPatternAnalysis {
        let futures = self.active_futures.lock().unwrap();
        let awaits = self.await_points.lock().unwrap();
        
        let mut patterns = Vec::new();

        // Pattern: Long-running futures
        let long_running_threshold = 1_000_000_000; // 1 second in nanoseconds
        let long_running_count = futures.values()
            .filter(|f| {
                if let Some(completion) = f.completion_time {
                    completion - f.creation_time > long_running_threshold
                } else {
                    current_timestamp() - f.creation_time > long_running_threshold
                }
            })
            .count();

        if long_running_count > 0 {
            patterns.push(AsyncPattern {
                pattern_type: AsyncPatternType::LongRunningFutures,
                description: format!("{} futures running longer than 1 second", long_running_count),
                severity: AsyncPatternSeverity::Warning,
                suggestion: "Consider breaking down long-running operations or adding timeouts".to_string(),
            });
        }

        // Pattern: Excessive polling
        let high_poll_threshold = 100;
        let high_poll_count = futures.values()
            .filter(|f| f.poll_count > high_poll_threshold)
            .count();

        if high_poll_count > 0 {
            patterns.push(AsyncPattern {
                pattern_type: AsyncPatternType::ExcessivePolling,
                description: format!("{} futures polled more than {} times", high_poll_count, high_poll_threshold),
                severity: AsyncPatternSeverity::Warning,
                suggestion: "High poll count may indicate inefficient async design".to_string(),
            });
        }

        // Pattern: Many concurrent futures
        let high_concurrency_threshold = 50;
        if futures.len() > high_concurrency_threshold {
            patterns.push(AsyncPattern {
                pattern_type: AsyncPatternType::HighConcurrency,
                description: format!("{} concurrent futures detected", futures.len()),
                severity: AsyncPatternSeverity::Info,
                suggestion: "High concurrency - ensure this is intentional and resources are managed".to_string(),
            });
        }

        // Pattern: Slow await points
        let slow_await_threshold = 100_000_000; // 100ms in nanoseconds
        let slow_awaits = awaits.iter()
            .filter(|a| a.duration.map_or(false, |d| d > slow_await_threshold))
            .count();

        if slow_awaits > 0 {
            patterns.push(AsyncPattern {
                pattern_type: AsyncPatternType::SlowAwaitPoints,
                description: format!("{} await points took longer than 100ms", slow_awaits),
                severity: AsyncPatternSeverity::Warning,
                suggestion: "Slow await points may indicate blocking operations in async code".to_string(),
            });
        }

        AsyncPatternAnalysis {
            patterns,
            total_futures_analyzed: futures.len(),
            analysis_timestamp: current_timestamp(),
        }
    }

    /// Get future information
    pub fn get_future_info(&self, ptr: usize) -> Option<FutureInfo> {
        self.active_futures.lock().unwrap().get(&ptr).cloned()
    }
}

/// Information about a tracked future
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FutureInfo {
    /// Memory pointer
    pub ptr: usize,
    /// Type of future
    pub future_type: String,
    /// Current state
    pub current_state: FutureState,
    /// Creation timestamp
    pub creation_time: u64,
    /// Completion timestamp (if completed)
    pub completion_time: Option<u64>,
    /// History of state changes
    pub state_history: Vec<FutureState>,
    /// Number of await points
    pub await_count: usize,
    /// Number of times polled
    pub poll_count: usize,
    /// Thread where created
    pub thread_id: String,
}

/// Future states
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FutureState {
    /// Future is pending
    Pending,
    /// Future is ready with a value
    Ready,
    /// Future was cancelled
    Cancelled,
    /// Future panicked
    Panicked,
    /// Initial state
    Created,
}

/// State transition information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// Future pointer
    pub ptr: usize,
    /// Previous state
    pub from_state: FutureState,
    /// New state
    pub to_state: FutureState,
    /// Transition timestamp
    pub timestamp: u64,
    /// Thread where transition occurred
    pub thread_id: String,
}

/// Await point information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwaitPoint {
    /// Future pointer
    pub ptr: usize,
    /// Location in code
    pub location: String,
    /// Type of await
    pub await_type: AwaitType,
    /// Await start timestamp
    pub timestamp: u64,
    /// Thread where await occurred
    pub thread_id: String,
    /// Duration of await (if completed)
    pub duration: Option<u64>,
}

/// Types of await operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AwaitType {
    /// Regular await
    Regular,
    /// Timeout await
    Timeout,
    /// Select await
    Select,
    /// Join await
    Join,
}

/// Task lifecycle events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEvent {
    /// Future pointer
    pub ptr: usize,
    /// Event type
    pub event_type: TaskEventType,
    /// Event timestamp
    pub timestamp: u64,
    /// Thread where event occurred
    pub thread_id: std::thread::ThreadId,
    /// Additional details
    pub details: String,
}

/// Types of task events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskEventType {
    /// Task created
    Created,
    /// Task started
    Started,
    /// Task suspended
    Suspended,
    /// Task resumed
    Resumed,
    /// Task completed
    Completed,
    /// Task cancelled
    Cancelled,
}

/// Future completion results
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FutureResult {
    /// Future completed successfully
    Ready,
    /// Future was cancelled
    Cancelled,
    /// Future panicked
    Panicked,
}

/// Async statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncStatistics {
    /// Total futures tracked
    pub total_futures: usize,
    /// Currently active futures
    pub active_futures: usize,
    /// Completed futures
    pub completed_futures: usize,
    /// Total state transitions
    pub total_state_transitions: usize,
    /// Total await points
    pub total_awaits: usize,
    /// Completed await points
    pub completed_awaits: usize,
    /// Average completion time in nanoseconds
    pub avg_completion_time: u64,
    /// Average await duration in nanoseconds
    pub avg_await_duration: u64,
    /// Count by future type
    pub by_type: HashMap<String, usize>,
}

/// Async pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncPatternAnalysis {
    /// Detected patterns
    pub patterns: Vec<AsyncPattern>,
    /// Total futures analyzed
    pub total_futures_analyzed: usize,
    /// Analysis timestamp
    pub analysis_timestamp: u64,
}

/// Detected async pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncPattern {
    /// Type of pattern
    pub pattern_type: AsyncPatternType,
    /// Description of the pattern
    pub description: String,
    /// Severity level
    pub severity: AsyncPatternSeverity,
    /// Suggested action
    pub suggestion: String,
}

/// Types of async patterns
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AsyncPatternType {
    /// Long-running futures
    LongRunningFutures,
    /// Excessive polling
    ExcessivePolling,
    /// High concurrency
    HighConcurrency,
    /// Slow await points
    SlowAwaitPoints,
    /// Memory leaks in futures
    FutureMemoryLeaks,
}

/// Pattern severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AsyncPatternSeverity {
    /// Informational
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
}

/// Get current timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_future_tracking() {
        let analyzer = AsyncAnalyzer::new();
        
        // Track a future
        analyzer.track_future(0x1000, "async_fn", FutureState::Created);
        
        // Check it's tracked
        let info = analyzer.get_future_info(0x1000);
        assert!(info.is_some());
        assert_eq!(info.unwrap().future_type, "async_fn");
        
        // Record state transition
        analyzer.record_state_transition(0x1000, FutureState::Created, FutureState::Pending);
        
        // Check state updated
        let info = analyzer.get_future_info(0x1000);
        assert_eq!(info.unwrap().current_state, FutureState::Pending);
    }

    #[test]
    fn test_await_tracking() {
        let analyzer = AsyncAnalyzer::new();
        
        // Track future and await
        analyzer.track_future(0x1000, "async_fn", FutureState::Created);
        analyzer.record_await_point(0x1000, "line_42", AwaitType::Regular);
        
        // Complete await
        analyzer.complete_await_point(0x1000, "line_42");
        
        // Check statistics
        let stats = analyzer.get_async_statistics();
        assert_eq!(stats.total_awaits, 1);
        assert_eq!(stats.completed_awaits, 1);
    }
}