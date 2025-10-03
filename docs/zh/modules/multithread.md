# 多线程模块：无锁高并发跟踪

多线程（lockfree）模块专为**高并发应用程序**设计，支持 20+ 线程。它使用基于采样的跟踪，零共享状态，实现最大性能。

## 🎯 适用场景

**✅ 完美适用于：**

- 高并发应用程序（30+ 线程）
- 生产监控系统
- 性能关键应用程序
- 近似数据可接受的场景
- Web 服务器、数据库、高吞吐量系统

**❌ 使用单线程模块：**

- 开发和调试
- 线程数 < 10 的应用程序
- 需要精确精度的场景

## 🔀 核心 API

### 快速开始 - 简单跟踪

```rust
use memscope_rs::lockfree::{trace_all, stop_tracing, export_comprehensive_analysis};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 开始跟踪所有线程
    trace_all("./MemoryAnalysis")?;
  
    // 你的多线程应用程序代码
    let handles: Vec<_> = (0..30).map(|i| {
        std::thread::spawn(move || {
            // 线程本地跟踪自动发生
            let data = vec![0u8; 1024 * 1024]; // 1MB 分配
          
            // 模拟工作
            std::thread::sleep(std::time::Duration::from_millis(100));
          
            println!("线程 {} 完成", i);
        })
    }).collect();
  
    // 等待所有线程
    for handle in handles {
        handle.join().unwrap();
    }
  
    // 停止跟踪并导出
    stop_tracing()?;
    export_comprehensive_analysis("./MemoryAnalysis", "multi_thread_analysis")?;
  
    println!("🎯 多线程分析完成！");
    Ok(())
}
```

### 高级配置

```rust
use memscope_rs::lockfree::{
    SamplingConfig, PlatformResourceCollector, 
    comprehensive_profile_execution
};

fn advanced_multi_threaded_tracking() -> Result<(), Box<dyn std::error::Error>> {
    // 配置采样以获得最佳性能
    let sampling_config = SamplingConfig {
        sample_rate: 0.01,        // 1% 采样率
        min_allocation_size: 1024, // 只跟踪 > 1KB 的分配
        buffer_size: 1024 * 1024, // 每线程 1MB 缓冲区
    };
  
    // 开始综合性能分析
    let mut session = comprehensive_profile_execution(
        "./HighConcurrencyAnalysis",
        Some(sampling_config)
    )?;
  
    // 你的高并发工作负载
    let handles: Vec<_> = (0..100).map(|thread_id| {
        std::thread::spawn(move || {
            for iteration in 0..1000 {
                // 重内存工作负载
                let data = vec![thread_id; 10000];
              
                // CPU 密集工作
                let sum: usize = data.iter().sum();
              
                // I/O 模拟
                if iteration % 100 == 0 {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
              
                // 让数据超出作用域
                drop(data);
            }
        })
    }).collect();
  
    // 等待完成
    for handle in handles {
        handle.join().unwrap();
    }
  
    // 完成并导出综合分析
    let analysis_result = session.finalize()?;
  
    println!("📊 综合分析结果：");
    println!("   - 分析的线程: {}", analysis_result.thread_count);
    println!("   - 总分配数: {}", analysis_result.total_allocations);
    println!("   - 峰值内存使用: {:.2} MB", analysis_result.peak_memory_mb);
    println!("   - 性能瓶颈: {}", analysis_result.bottlenecks.len());
  
    Ok(())
}
```

## 📊 平台资源监控

lockfree 模块包含综合系统资源跟踪：

```rust
use memscope_rs::lockfree::{
    PlatformResourceCollector, ThreadResourceMetrics,
    CpuResourceMetrics, IoResourceMetrics
};

fn monitor_system_resources() -> Result<(), Box<dyn std::error::Error>> {
    let mut collector = PlatformResourceCollector::new()?;
  
    // 开始监控
    collector.start_monitoring()?;
  
    // 在监控时运行你的工作负载
    let handles: Vec<_> = (0..50).map(|i| {
        std::thread::spawn(move || {
            // CPU 密集任务
            let mut data = vec![0u64; 100000];
            for j in 0..data.len() {
                data[j] = (i as u64 * j as u64) % 1000;
            }
          
            // 内存密集任务
            let large_data = vec![data; 10];
          
            // I/O 模拟
            std::thread::sleep(std::time::Duration::from_millis(50));
          
            large_data.len()
        })
    }).collect();
  
    for handle in handles {
        handle.join().unwrap();
    }
  
    // 停止监控并获取结果
    let metrics = collector.stop_monitoring()?;
  
    println!("🖥️  系统资源使用：");
    println!("   - 峰值 CPU 使用: {:.1}%", metrics.cpu_metrics.peak_usage_percent);
    println!("   - 峰值内存使用: {:.2} GB", metrics.memory_metrics.peak_usage_gb);
    println!("   - 总 I/O 操作: {}", metrics.io_metrics.total_operations);
    println!("   - 线程效率: {:.2}%", metrics.thread_metrics.efficiency_percent);
  
    Ok(())
}
```

## ⚡ 性能特征

### 跟踪开销

| 配置             | CPU 开销 | 内存开销     | 精度        |
| ---------------- | -------- | ------------ | ----------- |
| **默认**   | < 0.5%   | < 1MB/线程   | ~95% 准确性 |
| **高采样** | < 2%     | < 5MB/线程   | ~99% 准确性 |
| **低采样** | < 0.1%   | < 512KB/线程 | ~85% 准确性 |

### 可扩展性

| 线程数             | 导出时间 | 分析时间 | 文件大小 |
| ------------------ | -------- | -------- | -------- |
| **30 线程**  | 211ms    | 150ms    | 480KB    |
| **100 线程** | 450ms    | 300ms    | 1.2MB    |
| **500 线程** | 1.1s     | 800ms    | 4.8MB    |

## 🎮 真实世界示例

### Web 服务器监控

```rust
use memscope_rs::lockfree::{trace_all, stop_tracing, export_comprehensive_analysis};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

fn simulate_web_server() -> Result<(), Box<dyn std::error::Error>> {
    // 开始综合跟踪
    trace_all("./WebServerAnalysis")?;
  
    let request_counter = Arc::new(AtomicUsize::new(0));
  
    // 模拟带多个工作线程的 Web 服务器
    let handles: Vec<_> = (0..20).map(|worker_id| {
        let counter = Arc::clone(&request_counter);
      
        std::thread::spawn(move || {
            for request_id in 0..1000 {
                // 模拟请求处理
                let request_data = format!("Request-{}-{}", worker_id, request_id);
                let response_buffer = vec![0u8; 4096]; // 4KB 响应
              
                // 模拟数据库查询
                let query_result = vec![request_data.as_bytes(); 10];
              
                // 模拟 JSON 序列化
                let json_response = format!(
                    "{{\"worker\":{},\"request\":{},\"data\":{:?}}}",
                    worker_id, request_id, query_result.len()
                );
              
                // 更新指标
                counter.fetch_add(1, Ordering::Relaxed);
              
                // 模拟响应时间
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        })
    }).collect();
  
    // 等待所有工作线程完成
    for handle in handles {
        handle.join().unwrap();
    }
  
    // 停止跟踪并分析
    stop_tracing()?;
    export_comprehensive_analysis("./WebServerAnalysis", "web_server_performance")?;
  
    let total_requests = request_counter.load(Ordering::Relaxed);
    println!("🌐 Web 服务器模拟完成！");
    println!("   - 处理的总请求数: {}", total_requests);
    println!("   - 分析导出到: web_server_performance.html");
  
    Ok(())
}
```

### 数据库连接池

```rust
use memscope_rs::lockfree::{comprehensive_profile_execution, SamplingConfig};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

struct ConnectionPool {
    connections: Arc<Mutex<VecDeque<String>>>,
}

impl ConnectionPool {
    fn new(size: usize) -> Self {
        let mut connections = VecDeque::new();
        for i in 0..size {
            connections.push_back(format!("Connection-{}", i));
        }
      
        ConnectionPool {
            connections: Arc::new(Mutex::new(connections)),
        }
    }
  
    fn get_connection(&self) -> Option<String> {
        self.connections.lock().unwrap().pop_front()
    }
  
    fn return_connection(&self, conn: String) {
        self.connections.lock().unwrap().push_back(conn);
    }
}

fn database_workload_simulation() -> Result<(), Box<dyn std::error::Error>> {
    // 为数据库类工作负载配置
    let config = SamplingConfig {
        sample_rate: 0.05,        // 数据库操作 5% 采样
        min_allocation_size: 512, // 跟踪 > 512 字节的分配
        buffer_size: 2 * 1024 * 1024, // 高频操作 2MB 缓冲区
    };
  
    let mut session = comprehensive_profile_execution(
        "./DatabaseAnalysis",
        Some(config)
    )?;
  
    let pool = Arc::new(ConnectionPool::new(10));
  
    // 模拟并发数据库操作
    let handles: Vec<_> = (0..50).map(|thread_id| {
        let pool = Arc::clone(&pool);
      
        std::thread::spawn(move || {
            for query_id in 0..200 {
                // 从池中获取连接
                let connection = loop {
                    if let Some(conn) = pool.get_connection() {
                        break conn;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(1));
                };
              
                // 模拟查询执行
                let query = format!("SELECT * FROM table WHERE id = {}", query_id);
                let result_set = vec![format!("Row-{}-{}", thread_id, query_id); 100];
              
                // 模拟结果处理
                let processed_data: Vec<String> = result_set
                    .iter()
                    .map(|row| format!("Processed: {}", row))
                    .collect();
              
                // 模拟序列化
                let serialized = format!("{:?}", processed_data);
              
                // 将连接返回到池
                pool.return_connection(connection);
              
                // 模拟网络延迟
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
        })
    }).collect();
  
    for handle in handles {
        handle.join().unwrap();
    }
  
    let analysis = session.finalize()?;
  
    println!("🗄️  数据库模拟完成！");
    println!("   - 连接池效率: {:.1}%", analysis.resource_efficiency);
    println!("   - 检测到的内存热点: {}", analysis.bottlenecks.len());
  
    Ok(())
}
```

## 🔗 下一步

- **[异步模块](async.md)** - 任务中心内存分析
- **[混合模块](hybrid.md)** - 跨模块综合分析
- **[示例](examples/concurrent-analysis.md)** - 更多多线程示例
