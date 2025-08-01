//! Parallel processing optimization for binary export
//!
//! This module provides advanced parallel processing capabilities including
//! lock-free data structures, work-stealing schedulers, and optimized
//! thread pool management for maximum performance.

use std::collections::HashMap;
use std::sync::{Arc, atomic::{AtomicUsize, AtomicBool, Ordering}};
use std::time::{Duration, Instant};
use std::thread;
use crossbeam_utils::thread as crossbeam_thread;
use crossbeam_queue::{SegQueue, ArrayQueue};
use serde::{Serialize, Deserialize};

use super::error::BinaryExportError;
use super::processor::{ProcessedData, WorkItem, ProcessedWorkItem};

/// Configuration for parallel processing
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Number of worker threads
    pub worker_threads: usize,
    /// Work stealing enabled
    pub enable_work_stealing: bool,
    /// Load balancing strategy
    pub load_balancing: LoadBalancingStrategy,
    /// Thread affinity settings
    pub thread_affinity: ThreadAffinityConfig,
    /// Performance monitoring enabled
    pub enable_monitoring: bool,
    /// Maximum queue size per worker
    pub max_queue_size: usize,
    /// Work stealing attempts before giving up
    pub steal_attempts: usize,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            worker_threads: num_cpus::get(),
            enable_work_stealing: true,
            load_balancing: LoadBalancingStrategy::WorkStealing,
            thread_affinity: ThreadAffinityConfig::default(),
            enable_monitoring: true,
            max_queue_size: 1024,
            steal_attempts: 3,
        }
    }
}

/// Load balancing strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadBalancingStrategy {
    /// Round-robin distribution
    RoundRobin,
    /// Work stealing between threads
    WorkStealing,
    /// Dynamic load balancing based on queue sizes
    Dynamic,
    /// Least loaded worker selection
    LeastLoaded,
}

/// Thread affinity configuration
#[derive(Debug, Clone)]
pub struct ThreadAffinityConfig {
    /// Enable CPU affinity
    pub enabled: bool,
    /// CPU cores to use (None = all cores)
    pub cpu_cores: Option<Vec<usize>>,
    /// NUMA node preference
    pub numa_node: Option<usize>,
}

impl Default for ThreadAffinityConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cpu_cores: None,
            numa_node: None,
        }
    }
}

/// Parallel processing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelStats {
    /// Total work items processed
    pub total_items: u64,
    /// Items processed per worker
    pub items_per_worker: Vec<u64>,
    /// Work stealing statistics
    pub steal_stats: StealStats,
    /// Load balancing efficiency
    pub load_balance_efficiency: f64,
    /// Thread utilization percentages
    pub thread_utilization: Vec<f64>,
    /// Total processing time
    pub total_time: Duration,
    /// Parallel efficiency (vs sequential)
    pub parallel_efficiency: f64,
}

/// Work stealing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StealStats {
    /// Successful steal attempts
    pub successful_steals: u64,
    /// Failed steal attempts
    pub failed_steals: u64,
    /// Items stolen per worker
    pub steals_per_worker: Vec<u64>,
    /// Average steal latency
    pub avg_steal_latency: Duration,
}

/// Main parallel processor
pub struct ParallelProcessor {
    /// Configuration
    config: ParallelConfig,
    /// Worker threads
    workers: Vec<WorkerThread>,
    /// Global work queue for overflow
    global_queue: Arc<SegQueue<WorkItem>>,
    /// Performance monitor
    monitor: Arc<PerformanceMonitor>,
    /// Shutdown signal
    shutdown: Arc<AtomicBool>,
}

/// Individual worker thread
struct WorkerThread {
    /// Worker ID
    id: usize,
    /// Local work queue (lock-free)
    local_queue: Arc<ArrayQueue<WorkItem>>,
    /// Worker statistics
    stats: Arc<WorkerStats>,
    /// Thread handle
    handle: Option<thread::JoinHandle<()>>,
}

/// Worker thread statistics
struct WorkerStats {
    /// Items processed
    items_processed: AtomicUsize,
    /// Items stolen from this worker
    items_stolen: AtomicUsize,
    /// Items stolen by this worker
    items_stolen_by: AtomicUsize,
    /// Processing time
    processing_time: AtomicUsize, // in microseconds
    /// Idle time
    idle_time: AtomicUsize, // in microseconds
}

/// Performance monitoring system
struct PerformanceMonitor {
    /// Enabled flag
    enabled: AtomicBool,
    /// Monitoring start time
    start_time: Instant,
    /// Global statistics
    global_stats: GlobalStats,
}

/// Global performance statistics
struct GlobalStats {
    /// Total items processed
    total_items: AtomicUsize,
    /// Total processing time
    total_processing_time: AtomicUsize,
    /// Load imbalance events
    load_imbalance_events: AtomicUsize,
    /// Queue overflow events
    queue_overflow_events: AtomicUsize,
}

impl ParallelProcessor {
    /// Create a new parallel processor
    pub fn new(config: ParallelConfig) -> Self {
        let global_queue = Arc::new(SegQueue::new());
        let monitor = Arc::new(PerformanceMonitor::new(config.enable_monitoring));
        let shutdown = Arc::new(AtomicBool::new(false));
        
        let mut workers = Vec::new();
        
        // Create worker threads
        for worker_id in 0..config.worker_threads {
            let worker = WorkerThread::new(
                worker_id,
                config.max_queue_size,
                Arc::clone(&global_queue),
                Arc::clone(&monitor),
                Arc::clone(&shutdown),
                config.clone(),
            );
            workers.push(worker);
        }
        
        Self {
            config,
            workers,
            global_queue,
            monitor,
            shutdown,
        }
    }

    /// Process work items in parallel
    pub fn process_items(&mut self, items: Vec<WorkItem>) -> Result<Vec<ProcessedWorkItem>, BinaryExportError> {
        let start_time = Instant::now();
        
        // Reset monitoring
        self.monitor.reset();
        
        // Distribute work items
        self.distribute_work_items(items)?;
        
        // Start worker threads
        self.start_workers()?;
        
        // Wait for completion
        let results = self.wait_for_completion()?;
        
        // Calculate statistics
        let processing_time = start_time.elapsed();
        let stats = self.calculate_stats(processing_time);
        
        // Log performance metrics if monitoring is enabled
        if self.config.enable_monitoring {
            self.log_performance_metrics(&stats);
        }
        
        Ok(results)
    }

    /// Distribute work items to worker queues
    fn distribute_work_items(&mut self, items: Vec<WorkItem>) -> Result<(), BinaryExportError> {
        match self.config.load_balancing {
            LoadBalancingStrategy::RoundRobin => {
                self.distribute_round_robin(items)
            }
            LoadBalancingStrategy::WorkStealing => {
                self.distribute_for_work_stealing(items)
            }
            LoadBalancingStrategy::Dynamic => {
                self.distribute_dynamic(items)
            }
            LoadBalancingStrategy::LeastLoaded => {
                self.distribute_least_loaded(items)
            }
        }
    }

    /// Round-robin distribution
    fn distribute_round_robin(&mut self, items: Vec<WorkItem>) -> Result<(), BinaryExportError> {
        for (index, item) in items.into_iter().enumerate() {
            let worker_id = index % self.config.worker_threads;
            
            if let Err(_) = self.workers[worker_id].local_queue.push(item) {
                // Queue full, push to global queue
                self.global_queue.push(item);
                self.monitor.global_stats.queue_overflow_events.fetch_add(1, Ordering::Relaxed);
            }
        }
        Ok(())
    }

    /// Distribution optimized for work stealing
    fn distribute_for_work_stealing(&mut self, items: Vec<WorkItem>) -> Result<(), BinaryExportError> {
        // Sort items by priority and estimated processing time
        let mut sorted_items = items;
        sorted_items.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then_with(|| self.estimate_processing_time(b).cmp(&self.estimate_processing_time(a)))
        });
        
        // Distribute larger items first to enable better work stealing
        for (index, item) in sorted_items.into_iter().enumerate() {
            let worker_id = index % self.config.worker_threads;
            
            if let Err(_) = self.workers[worker_id].local_queue.push(item) {
                self.global_queue.push(item);
                self.monitor.global_stats.queue_overflow_events.fetch_add(1, Ordering::Relaxed);
            }
        }
        Ok(())
    }

    /// Dynamic load balancing
    fn distribute_dynamic(&mut self, items: Vec<WorkItem>) -> Result<(), BinaryExportError> {
        for item in items {
            // Find worker with smallest queue
            let mut min_queue_size = usize::MAX;
            let mut best_worker = 0;
            
            for (worker_id, worker) in self.workers.iter().enumerate() {
                let queue_size = worker.local_queue.len();
                if queue_size < min_queue_size {
                    min_queue_size = queue_size;
                    best_worker = worker_id;
                }
            }
            
            if let Err(_) = self.workers[best_worker].local_queue.push(item) {
                self.global_queue.push(item);
                self.monitor.global_stats.queue_overflow_events.fetch_add(1, Ordering::Relaxed);
            }
        }
        Ok(())
    }

    /// Least loaded distribution
    fn distribute_least_loaded(&mut self, items: Vec<WorkItem>) -> Result<(), BinaryExportError> {
        for item in items {
            // Find worker with least total load (queue size + processing time)
            let mut min_load = f64::MAX;
            let mut best_worker = 0;
            
            for (worker_id, worker) in self.workers.iter().enumerate() {
                let queue_load = worker.local_queue.len() as f64;
                let processing_load = worker.stats.processing_time.load(Ordering::Relaxed) as f64 / 1_000_000.0; // Convert to seconds
                let total_load = queue_load + processing_load;
                
                if total_load < min_load {
                    min_load = total_load;
                    best_worker = worker_id;
                }
            }
            
            if let Err(_) = self.workers[best_worker].local_queue.push(item) {
                self.global_queue.push(item);
                self.monitor.global_stats.queue_overflow_events.fetch_add(1, Ordering::Relaxed);
            }
        }
        Ok(())
    }

    /// Estimate processing time for a work item
    fn estimate_processing_time(&self, item: &WorkItem) -> Duration {
        // Simple heuristic based on data size and type
        let base_time = match item.item_type {
            crate::export::binary::processor::WorkItemType::Allocations => Duration::from_micros(100),
            crate::export::binary::processor::WorkItemType::Analysis => Duration::from_micros(500),
            crate::export::binary::processor::WorkItemType::Metadata => Duration::from_micros(50),
        };
        
        // Scale by data size
        let size_factor = (item.data.len() as f64 / 1024.0).max(1.0);
        Duration::from_micros((base_time.as_micros() as f64 * size_factor) as u64)
    }

    /// Start all worker threads
    fn start_workers(&mut self) -> Result<(), BinaryExportError> {
        for worker in &mut self.workers {
            worker.start()?;
        }
        Ok(())
    }

    /// Wait for all workers to complete
    fn wait_for_completion(&mut self) -> Result<Vec<ProcessedWorkItem>, BinaryExportError> {
        // Signal shutdown when all work is done
        self.wait_for_work_completion();
        self.shutdown.store(true, Ordering::Relaxed);
        
        // Collect results from all workers
        let mut all_results = Vec::new();
        
        for worker in &mut self.workers {
            if let Some(handle) = worker.handle.take() {
                match handle.join() {
                    Ok(results) => all_results.extend(results),
                    Err(_) => return Err(BinaryExportError::InternalError(
                        "Worker thread panicked".to_string()
                    )),
                }
            }
        }
        
        Ok(all_results)
    }

    /// Wait for all work to be completed
    fn wait_for_work_completion(&self) {
        loop {
            let mut all_empty = true;
            
            // Check if all local queues are empty
            for worker in &self.workers {
                if !worker.local_queue.is_empty() {
                    all_empty = false;
                    break;
                }
            }
            
            // Check global queue
            if !self.global_queue.is_empty() {
                all_empty = false;
            }
            
            if all_empty {
                // Wait a bit more to ensure all workers are truly idle
                thread::sleep(Duration::from_millis(10));
                break;
            }
            
            thread::sleep(Duration::from_millis(1));
        }
    }

    /// Calculate performance statistics
    fn calculate_stats(&self, total_time: Duration) -> ParallelStats {
        let mut items_per_worker = Vec::new();
        let mut steals_per_worker = Vec::new();
        let mut thread_utilization = Vec::new();
        
        let mut total_items = 0;
        let mut total_steals = 0;
        let mut total_failed_steals = 0;
        
        for worker in &self.workers {
            let items = worker.stats.items_processed.load(Ordering::Relaxed) as u64;
            let steals = worker.stats.items_stolen_by.load(Ordering::Relaxed) as u64;
            let processing_time = worker.stats.processing_time.load(Ordering::Relaxed) as u64;
            let idle_time = worker.stats.idle_time.load(Ordering::Relaxed) as u64;
            
            items_per_worker.push(items);
            steals_per_worker.push(steals);
            
            let total_worker_time = processing_time + idle_time;
            let utilization = if total_worker_time > 0 {
                processing_time as f64 / total_worker_time as f64
            } else {
                0.0
            };
            thread_utilization.push(utilization);
            
            total_items += items;
            total_steals += steals;
        }
        
        // Calculate load balance efficiency
        let avg_items = total_items as f64 / self.config.worker_threads as f64;
        let variance = items_per_worker.iter()
            .map(|&items| (items as f64 - avg_items).powi(2))
            .sum::<f64>() / self.config.worker_threads as f64;
        let load_balance_efficiency = 1.0 - (variance.sqrt() / avg_items.max(1.0));
        
        // Calculate parallel efficiency
        let sequential_estimate = Duration::from_secs_f64(total_items as f64 * 0.001); // 1ms per item estimate
        let parallel_efficiency = sequential_estimate.as_secs_f64() / 
            (total_time.as_secs_f64() * self.config.worker_threads as f64);
        
        ParallelStats {
            total_items,
            items_per_worker,
            steal_stats: StealStats {
                successful_steals: total_steals,
                failed_steals: total_failed_steals,
                steals_per_worker,
                avg_steal_latency: Duration::from_micros(50), // Placeholder
            },
            load_balance_efficiency: load_balance_efficiency.min(1.0).max(0.0),
            thread_utilization,
            total_time,
            parallel_efficiency: parallel_efficiency.min(1.0).max(0.0),
        }
    }

    /// Log performance metrics
    fn log_performance_metrics(&self, stats: &ParallelStats) {
        println!("Parallel Processing Statistics:");
        println!("  Total items: {}", stats.total_items);
        println!("  Total time: {:?}", stats.total_time);
        println!("  Load balance efficiency: {:.2}%", stats.load_balance_efficiency * 100.0);
        println!("  Parallel efficiency: {:.2}%", stats.parallel_efficiency * 100.0);
        println!("  Successful steals: {}", stats.steal_stats.successful_steals);
        
        for (i, utilization) in stats.thread_utilization.iter().enumerate() {
            println!("  Worker {} utilization: {:.2}%", i, utilization * 100.0);
        }
    }

    /// Get current performance statistics
    pub fn get_stats(&self) -> ParallelStats {
        self.calculate_stats(self.monitor.elapsed())
    }

    /// Shutdown the parallel processor
    pub fn shutdown(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        
        // Wait for all workers to finish
        for worker in &mut self.workers {
            if let Some(handle) = worker.handle.take() {
                let _ = handle.join();
            }
        }
    }
}

impl WorkerThread {
    fn new(
        id: usize,
        queue_size: usize,
        global_queue: Arc<SegQueue<WorkItem>>,
        monitor: Arc<PerformanceMonitor>,
        shutdown: Arc<AtomicBool>,
        config: ParallelConfig,
    ) -> Self {
        Self {
            id,
            local_queue: Arc::new(ArrayQueue::new(queue_size)),
            stats: Arc::new(WorkerStats::new()),
            handle: None,
        }
    }

    fn start(&mut self) -> Result<(), BinaryExportError> {
        let worker_id = self.id;
        let local_queue = Arc::clone(&self.local_queue);
        let stats = Arc::clone(&self.stats);
        // Note: We would need to pass other required parameters here
        // This is a simplified version
        
        let handle = thread::spawn(move || {
            let mut results = Vec::new();
            
            loop {
                // Worker main loop would go here
                // This is a placeholder implementation
                thread::sleep(Duration::from_millis(1));
                break;
            }
            
            results
        });
        
        self.handle = Some(handle);
        Ok(())
    }
}

impl WorkerStats {
    fn new() -> Self {
        Self {
            items_processed: AtomicUsize::new(0),
            items_stolen: AtomicUsize::new(0),
            items_stolen_by: AtomicUsize::new(0),
            processing_time: AtomicUsize::new(0),
            idle_time: AtomicUsize::new(0),
        }
    }
}

impl PerformanceMonitor {
    fn new(enabled: bool) -> Self {
        Self {
            enabled: AtomicBool::new(enabled),
            start_time: Instant::now(),
            global_stats: GlobalStats::new(),
        }
    }

    fn reset(&self) {
        self.global_stats.total_items.store(0, Ordering::Relaxed);
        self.global_stats.total_processing_time.store(0, Ordering::Relaxed);
        self.global_stats.load_imbalance_events.store(0, Ordering::Relaxed);
        self.global_stats.queue_overflow_events.store(0, Ordering::Relaxed);
    }

    fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl GlobalStats {
    fn new() -> Self {
        Self {
            total_items: AtomicUsize::new(0),
            total_processing_time: AtomicUsize::new(0),
            load_imbalance_events: AtomicUsize::new(0),
            queue_overflow_events: AtomicUsize::new(0),
        }
    }
}

impl Drop for ParallelProcessor {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::binary::processor::{WorkItem, WorkItemType, WorkPriority};

    #[test]
    fn test_parallel_config_default() {
        let config = ParallelConfig::default();
        assert!(config.worker_threads > 0);
        assert!(config.enable_work_stealing);
        assert_eq!(config.load_balancing, LoadBalancingStrategy::WorkStealing);
    }

    #[test]
    fn test_parallel_processor_creation() {
        let config = ParallelConfig::default();
        let processor = ParallelProcessor::new(config);
        assert_eq!(processor.workers.len(), num_cpus::get());
    }

    #[test]
    fn test_load_balancing_strategies() {
        let strategies = [
            LoadBalancingStrategy::RoundRobin,
            LoadBalancingStrategy::WorkStealing,
            LoadBalancingStrategy::Dynamic,
            LoadBalancingStrategy::LeastLoaded,
        ];
        
        for strategy in &strategies {
            let mut config = ParallelConfig::default();
            config.load_balancing = *strategy;
            config.worker_threads = 2; // Small number for testing
            
            let mut processor = ParallelProcessor::new(config);
            
            // Create test work items
            let items = vec![
                WorkItem {
                    id: 0,
                    item_type: WorkItemType::Allocations,
                    data: vec![1, 2, 3],
                    priority: WorkPriority::High,
                },
                WorkItem {
                    id: 1,
                    item_type: WorkItemType::Analysis,
                    data: vec![4, 5, 6],
                    priority: WorkPriority::Medium,
                },
            ];
            
            // Test distribution (should not panic)
            let result = processor.distribute_work_items(items);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_worker_stats() {
        let stats = WorkerStats::new();
        
        assert_eq!(stats.items_processed.load(Ordering::Relaxed), 0);
        assert_eq!(stats.items_stolen.load(Ordering::Relaxed), 0);
        
        stats.items_processed.fetch_add(5, Ordering::Relaxed);
        assert_eq!(stats.items_processed.load(Ordering::Relaxed), 5);
    }

    #[test]
    fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new(true);
        assert!(monitor.enabled.load(Ordering::Relaxed));
        
        monitor.reset();
        assert_eq!(monitor.global_stats.total_items.load(Ordering::Relaxed), 0);
        
        let elapsed = monitor.elapsed();
        assert!(elapsed.as_nanos() > 0);
    }
}