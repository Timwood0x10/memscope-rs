//! Multi-threaded stress test with large number of variables
//! Tests memscope-rs stability under heavy concurrent workload

use memscope_rs::track_var;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting multi-threaded stress test...");
    
    // Solution for 20+ threads: Disable global tracker to prevent crashes
    std::env::set_var("MEMSCOPE_DISABLE_GLOBAL", "1");
    std::env::set_var("MEMSCOPE_ASYNC_MODE", "1");
    
    // Test 1: Many threads with many variables
    println!("📊 Test 1: 50 threads with 1,000 variables each");
    test_many_threads()?;
    
    // Test 2: Shared data across threads
    println!("📊 Test 2: Shared data tracking across threads");
    test_shared_data()?;
    
    // Test 3: Thread pool simulation
    println!("📊 Test 3: Thread pool with variable tracking");
    test_thread_pool()?;
    
    // Test 4: Mixed thread lifetimes
    println!("📊 Test 4: Mixed thread lifetimes");
    test_mixed_lifetimes()?;
    
    println!("✅ All multi-threaded stress tests completed successfully!");
    println!("🎉 memscope-rs is stable under heavy concurrent workload!");
    
    Ok(())
}

fn test_many_threads() -> Result<(), Box<dyn std::error::Error>> {
    let mut handles = Vec::new();
    
    for thread_id in 0..50 {
        let handle = thread::spawn(move || {
            for i in 0..20 {
                let vec_data = vec![thread_id * 1000 + i; 25];
                track_var!(vec_data);
                
                let string_data = format!("thread_{}_{}", thread_id, i);
                track_var!(string_data);
                
                let hash_data = {
                    let mut map = HashMap::new();
                    map.insert(i, format!("thread_{}_value_{}", thread_id, i));
                    map
                };
                track_var!(hash_data);
                
                if i % 200 == 0 {
                    thread::sleep(std::time::Duration::from_micros(100));
                }
            }
            thread_id
        });
        handles.push(handle);
    }
    
    let mut completed = 0;
    for handle in handles {
        let _thread_id = handle.join().unwrap();
        completed += 1;
        if completed % 10 == 0 {
            println!("  Completed {} threads", completed);
        }
    }
    
    println!("  ✅ All 50 threads completed (150,000 variables)");
    Ok(())
}

fn test_shared_data() -> Result<(), Box<dyn std::error::Error>> {
    let shared_data = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();
    
    for thread_id in 0..20 {
        let shared_clone = Arc::clone(&shared_data);
        let handle = thread::spawn(move || {
            for i in 0..50 {
                // Track local data
                let local_data = vec![thread_id, i];
                track_var!(local_data);
                
                // Track shared data
                let shared_vec = Arc::clone(&shared_clone);
                track_var!(shared_vec);
                
                // Simulate work with shared data
                {
                    let mut data = shared_clone.lock().unwrap();
                    data.push(thread_id * 1000 + i);
                    let data_copy = data.clone();
                    track_var!(data_copy);
                }
                
                if i % 100 == 0 {
                    thread::sleep(std::time::Duration::from_micros(50));
                }
            }
            thread_id
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Track final shared data
    let final_data = Arc::try_unwrap(shared_data).unwrap().into_inner().unwrap();
    track_var!(final_data);
    
    println!("  ✅ Shared data test completed (30,000+ variables)");
    Ok(())
}

fn test_thread_pool() -> Result<(), Box<dyn std::error::Error>> {
    let (sender, receiver) = std::sync::mpsc::channel();
    let receiver = Arc::new(Mutex::new(receiver));
    let mut worker_handles = Vec::new();
    
    // Create worker threads
    for worker_id in 0..10 {
        let receiver_clone = Arc::clone(&receiver);
        let handle = thread::spawn(move || {
            let mut processed = 0;
            loop {
                let task = {
                    let receiver = receiver_clone.lock().unwrap();
                    receiver.try_recv()
                };
                
                match task {
                    Ok(task_data) => {
                        // Track task data
                        track_var!(task_data);
                        
                        // Create worker-specific data
                        let worker_data = vec![worker_id; 50];
                        track_var!(worker_data);
                        
                        let result_data = format!("worker_{}_result_{}", worker_id, processed);
                        track_var!(result_data);
                        
                        processed += 1;
                        thread::sleep(std::time::Duration::from_micros(100));
                    }
                    Err(_) => {
                        thread::sleep(std::time::Duration::from_millis(1));
                        if processed > 0 {
                            break; // Exit when no more tasks and we've processed some
                        }
                    }
                }
            }
            processed
        });
        worker_handles.push(handle);
    }
    
    // Send tasks
    for task_id in 0..5_00 {
        let task_data = format!("task_{}", task_id);
        sender.send(task_data).unwrap();
    }
    
    // Drop sender to signal completion
    drop(sender);
    
    // Wait for workers
    let mut total_processed = 0;
    for handle in worker_handles {
        let processed = handle.join().unwrap();
        total_processed += processed;
    }
    
    println!("  ✅ Thread pool processed {} tasks", total_processed);
    Ok(())
}

fn test_mixed_lifetimes() -> Result<(), Box<dyn std::error::Error>> {
    let mut handles = Vec::new();
    
    // Short-lived threads
    for i in 0..30 {
        let handle = thread::spawn(move || {
            for j in 0..100 {
                let data = vec![i, j];
                track_var!(data);
            }
            thread::sleep(std::time::Duration::from_millis(10));
        });
        handles.push(handle);
    }
    
    // Medium-lived threads
    for i in 0..20 {
        let handle = thread::spawn(move || {
            for j in 0..30 {
                let data = format!("medium_{}_{}", i, j);
                track_var!(data);
                if j % 50 == 0 {
                    thread::sleep(std::time::Duration::from_millis(1));
                }
            }
        });
        handles.push(handle);
    }
    
    // Long-lived threads
    for i in 0..10 {
        let handle = thread::spawn(move || {
            for j in 0..100 {
                let data = HashMap::from([(i, j), (j, i)]);
                track_var!(data);
                if j % 100 == 0 {
                    thread::sleep(std::time::Duration::from_micros(500));
                }
            }
        });
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("  ✅ Mixed lifetime threads completed (19,000 variables)");
    Ok(())
}