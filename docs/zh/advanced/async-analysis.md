# 异步内存分析

分析异步 Rust 程序中的内存使用模式，包括 Future、async/await 和异步运行时。

## 🎯 目标

- 跟踪异步任务内存
- 分析 Future 状态机
- 监控异步运行时开销
- 检测异步内存泄漏

## 🔮 基础异步跟踪

### 简单异步函数

```rust
use memscope_rs::{init, track_var};
use tokio;

#[tokio::main]
async fn main() {
    init();
    
    // 跟踪异步任务中的数据
    let result = async_computation().await;
    track_var!(result);
    
    println!("异步计算完成: {:?}", result);
}

async fn async_computation() -> Vec<i32> {
    let data = vec![1, 2, 3, 4, 5];
    track_var!(data);
    
    // 模拟异步工作
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    data.into_iter().map(|x| x * 2).collect()
}
```

### Future 状态机分析

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

struct TrackedFuture<T> {
    inner: Pin<Box<dyn Future<Output = T>>>,
    state_data: Vec<u8>,
}

impl<T> TrackedFuture<T> {
    fn new<F>(future: F, state_size: usize) -> Self 
    where 
        F: Future<Output = T> + 'static 
    {
        let state_data = vec![0; state_size];
        track_var!(state_data);
        
        Self {
            inner: Box::pin(future),
            state_data,
        }
    }
}

impl<T> Future for TrackedFuture<T> {
    type Output = T;
    
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 跟踪 Future 轮询
        println!("轮询 Future，状态大小: {} bytes", self.state_data.len());
        self.inner.as_mut().poll(cx)
    }
}
```

## 🏃‍♂️ 异步运行时分析

### Tokio 运行时跟踪

```rust
use tokio::runtime::Runtime;
use memscope_rs::analysis::AsyncAnalyzer;

fn analyze_tokio_runtime() {
    init();
    
    let rt = Runtime::new().unwrap();
    let analyzer = AsyncAnalyzer::new();
    
    rt.block_on(async {
        // 创建多个异步任务
        let mut handles = vec![];
        
        for i in 0..10 {
            let handle = tokio::spawn(async move {
                let task_data = vec![i; 1000];
                track_var!(task_data);
                
                tokio::time::sleep(
                    tokio::time::Duration::from_millis(i * 10)
                ).await;
                
                task_data.len()
            });
            handles.push(handle);
        }
        
        // 等待所有任务完成
        for handle in handles {
            let result = handle.await.unwrap();
            println!("任务完成，处理了 {} 个元素", result);
        }
        
        // 分析异步内存使用
        let tracker = memscope_rs::get_global_tracker();
        let allocations = tracker.get_active_allocations().unwrap();
        let async_report = analyzer.analyze_future_states(&allocations);
        
        println!("异步分析报告:");
        println!("  异步分配数: {}", async_report.async_allocation_count);
        println!("  Future 状态开销: {} bytes", async_report.future_state_overhead);
    });
}
```

### 异步流处理

```rust
use tokio_stream::{self as stream, StreamExt};

async fn analyze_async_streams() {
    init();
    
    // 创建异步流
    let stream_data = stream::iter(0..1000)
        .map(|i| {
            let item_data = vec![i; 100];
            track_var!(item_data);
            item_data
        })
        .collect::<Vec<_>>()
        .await;
    
    track_var!(stream_data);
    
    println!("流处理完成，处理了 {} 个项目", stream_data.len());
}
```

## 📊 异步内存模式

### 任务生命周期分析

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

async fn analyze_task_lifecycles() {
    init();
    
    let shared_state = Arc::new(Mutex::new(Vec::new()));
    track_var!(shared_state);
    
    // 短期任务
    let short_tasks: Vec<_> = (0..5).map(|i| {
        let state = Arc::clone(&shared_state);
        tokio::spawn(async move {
            let local_data = vec![i; 100];
            track_var!(local_data);
            
            let mut guard = state.lock().await;
            guard.extend_from_slice(&local_data);
        })
    }).collect();
    
    // 长期任务
    let long_task = {
        let state = Arc::clone(&shared_state);
        tokio::spawn(async move {
            let persistent_data = vec![0; 10000];
            track_var!(persistent_data);
            
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                
                let guard = state.lock().await;
                if guard.len() > 1000 {
                    break;
                }
            }
        })
    };
    
    // 等待短期任务完成
    for task in short_tasks {
        task.await.unwrap();
    }
    
    // 取消长期任务
    long_task.abort();
    
    println!("任务生命周期分析完成");
}
```

### 异步内存泄漏检测

```rust
use std::collections::HashMap;
use tokio::sync::mpsc;

async fn detect_async_memory_leaks() {
    init();
    
    let (tx, mut rx) = mpsc::channel(100);
    let mut task_registry = HashMap::new();
    
    // 创建可能泄漏的任务
    for i in 0..10 {
        let tx_clone = tx.clone();
        let handle = tokio::spawn(async move {
            let task_data = vec![i; 1000];
            track_var!(task_data);
            
            // 模拟可能永远不完成的任务
            if i % 3 == 0 {
                // 这些任务会很快完成
                let _ = tx_clone.send(format!("Task {} completed", i)).await;
            } else {
                // 这些任务可能永远不完成（内存泄漏）
                tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                let _ = tx_clone.send(format!("Task {} completed", i)).await;
            }
        });
        
        task_registry.insert(i, handle);
    }
    
    // 等待一些任务完成
    tokio::time::timeout(
        tokio::time::Duration::from_secs(1),
        async {
            while let Some(msg) = rx.recv().await {
                println!("收到消息: {}", msg);
            }
        }
    ).await.ok();
    
    // 检查未完成的任务（潜在泄漏）
    let mut leaked_tasks = 0;
    for (id, handle) in task_registry {
        if !handle.is_finished() {
            println!("任务 {} 可能泄漏", id);
            handle.abort();
            leaked_tasks += 1;
        }
    }
    
    println!("检测到 {} 个潜在的异步内存泄漏", leaked_tasks);
}
```

## 🔍 高级异步分析

### 自定义异步分析器

```rust
use memscope_rs::analysis::AsyncAnalyzer;

struct CustomAsyncAnalyzer {
    task_count: usize,
    future_sizes: Vec<usize>,
}

impl CustomAsyncAnalyzer {
    fn new() -> Self {
        Self {
            task_count: 0,
            future_sizes: Vec::new(),
        }
    }
    
    fn analyze_async_allocation(&mut self, size: usize, type_name: &str) {
        if type_name.contains("Future") || type_name.contains("async") {
            self.task_count += 1;
            self.future_sizes.push(size);
        }
    }
    
    fn generate_report(&self) -> AsyncAnalysisReport {
        let total_future_memory: usize = self.future_sizes.iter().sum();
        let avg_future_size = if self.task_count > 0 {
            total_future_memory / self.task_count
        } else {
            0
        };
        
        AsyncAnalysisReport {
            async_allocation_count: self.task_count,
            future_state_overhead: total_future_memory,
            average_future_size: avg_future_size,
            largest_future_size: self.future_sizes.iter().max().copied().unwrap_or(0),
        }
    }
}

#[derive(Debug)]
struct AsyncAnalysisReport {
    async_allocation_count: usize,
    future_state_overhead: usize,
    average_future_size: usize,
    largest_future_size: usize,
}
```

## 📈 性能优化

### 异步内存优化策略

```rust
// ❌ 低效：每个任务都分配大量内存
async fn inefficient_async_processing() {
    let tasks: Vec<_> = (0..1000).map(|i| {
        tokio::spawn(async move {
            let large_buffer = vec![0; 10000]; // 每个任务 10KB
            track_var!(large_buffer);
            
            // 处理数据...
            large_buffer.len()
        })
    }).collect();
    
    for task in tasks {
        task.await.unwrap();
    }
}

// ✅ 高效：使用共享内存池
use std::sync::Arc;

async fn efficient_async_processing() {
    let shared_pool = Arc::new(Mutex::new(Vec::new()));
    track_var!(shared_pool);
    
    let tasks: Vec<_> = (0..1000).map(|i| {
        let pool = Arc::clone(&shared_pool);
        tokio::spawn(async move {
            // 从池中获取缓冲区
            let buffer = {
                let mut pool_guard = pool.lock().await;
                pool_guard.pop().unwrap_or_else(|| vec![0; 10000])
            };
            
            // 处理数据...
            let result = buffer.len();
            
            // 归还缓冲区到池中
            {
                let mut pool_guard = pool.lock().await;
                pool_guard.push(buffer);
            }
            
            result
        })
    }).collect();
    
    for task in tasks {
        task.await.unwrap();
    }
}
```

## 🎉 总结

异步内存分析帮助你：
- 理解异步任务的内存使用
- 检测异步内存泄漏
- 优化 Future 状态机大小
- 改善异步程序性能