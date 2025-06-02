use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;

// Extension trait to add timeout functionality to JoinHandle
pub trait JoinHandleExt<T> {
    fn join_timeout(self, timeout: Duration) -> Result<T, Box<dyn std::any::Any + Send + 'static>>;
}

impl<T: Send + 'static> JoinHandleExt<T> for thread::JoinHandle<T> {
    fn join_timeout(self, timeout: Duration) -> Result<T, Box<dyn std::any::Any + Send + 'static>> {
        // If the thread is already finished, just join it
        if self.is_finished() {
            return self.join().map_err(|e| Box::new(e) as _);
        }

        // Create a flag to track if the thread completed
        let completed = Arc::new(AtomicBool::new(false));
        let completed_clone = completed.clone();
        
        // Spawn a thread that will wait for the original thread
        let handle = thread::spawn(move || {
            let result = self.join();
            completed_clone.store(true, Ordering::SeqCst);
            result
        });
        
        // Wait for the timeout or until the thread completes
        let start = std::time::Instant::now();
        while !completed.load(Ordering::SeqCst) && start.elapsed() < timeout {
            thread::sleep(Duration::from_millis(10));
        }
        
        if completed.load(Ordering::SeqCst) {
            // Thread completed within timeout
            match handle.join() {
                Ok(result) => result.map_err(|e| Box::new(e) as _),
                Err(_) => Err(Box::new("Watcher thread panicked") as _),
            }
        } else {
            // Timeout reached and thread is still running
            // Return an error indicating timeout
            Err(Box::new("Thread join timed out") as _)
        }
    }
}
