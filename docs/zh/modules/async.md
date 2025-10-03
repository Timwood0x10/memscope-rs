# 异步模块：任务中心内存分析

异步模块为 async/await 应用程序提供**任务感知内存跟踪**。与基于线程的跟踪不同，此系统在单个异步任务（Futures）的粒度上跟踪内存。

## 🎯 适用场景

**✅ 完美适用于：**
- async/await 应用程序
- Tokio、async-std、smol 运行时
- 任务级内存分析
- 异步服务监控
- 微服务和异步 Web 服务器

**❌ 使用其他模块：**
- 同步应用程序
- 基于线程池的并发
- 不需要任务级粒度时

## ⚡ 核心 API

### 快速开始

```rust
use memscope_rs::async_memory::{initialize, spawn_tracked, get_memory_snapshot};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化异步内存跟踪
    initialize().await?;
    
    // 创建被跟踪的异步任务
    let task = spawn_tracked(async {
        let data = vec![0u8; 1024 * 1024]; // 1MB 分配
        
        // 模拟异步工作
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // 处理数据
        let processed = data.iter().map(|&x| x as u64).sum::<u64>();
        processed
    });
    
    // 等待结果
    let result = task.await?;
    println!("处理了 {} 字节，总和: {}", 1024 * 1024, result);
    
    // 获取内存快照
    let snapshot = get_memory_snapshot();
    println!("活跃任务: {}", snapshot.active_task_count());
    println!("跟踪的总内存: {} 字节", snapshot.total_memory_bytes());
    
    Ok(())
}
```

### 高级任务跟踪

```rust
use memscope_rs::async_memory::{
    initialize, create_tracked, TaskMemoryProfile, 
    AsyncResourceMonitor, TaskType
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    initialize().await?;
    
    // 为详细分析创建资源监控器
    let mut monitor = AsyncResourceMonitor::new();
    monitor.start_monitoring().await?;
    
    // CPU 密集任务
    let cpu_task = create_tracked(
        async {
            let mut data = vec![0u64; 1_000_000];
            for i in 0..data.len() {
                data[i] = (i as u64).pow(2) % 1000;
            }
            data.iter().sum::<u64>()
        },
        TaskType::CpuIntensive
    );
    
    // I/O 密集任务
    let io_task = create_tracked(
        async {
            let mut results = Vec::new();
            for i in 0..100 {
                // 模拟文件 I/O
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                results.push(format!("Result-{}", i));
            }
            results.len()
        },
        TaskType::IoIntensive
    );
    
    // 网络密集任务
    let network_task = create_tracked(
        async {
            let mut responses = Vec::new();
            for i in 0..50 {
                // 模拟网络请求
                tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
                let response = format!("Response-{}: {}", i, "A".repeat(1024));
                responses.push(response);
            }
            responses.len()
        },
        TaskType::NetworkIntensive
    );
    
    // 等待所有任务
    let (cpu_result, io_result, network_result) = tokio::try_join!(
        cpu_task,
        io_task,
        network_task
    )?;
    
    println!("CPU 任务结果: {}", cpu_result);
    println!("I/O 任务结果: {}", io_result);
    println!("网络任务结果: {}", network_result);
    
    // 停止监控并获取详细分析
    let analysis = monitor.stop_monitoring().await?;
    
    println!("📊 任务分析：");
    println!("   - CPU 密集任务: {}", analysis.cpu_task_count);
    println!("   - I/O 密集任务: {}", analysis.io_task_count);
    println!("   - 网络密集任务: {}", analysis.network_task_count);
    println!("   - 每任务峰值内存: {:.2} MB", analysis.peak_memory_per_task_mb);
    
    Ok(())
}
```

## 🎮 真实世界示例

### 异步 Web 服务器

```rust
use memscope_rs::async_memory::{initialize, spawn_tracked, TaskType};
use tokio::sync::mpsc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[tokio::main]
async fn async_web_server_simulation() -> Result<(), Box<dyn std::error::Error>> {
    initialize().await?;
    
    let request_counter = Arc::new(AtomicUsize::new(0));
    let (tx, mut rx) = mpsc::channel(1000);
    
    // 产生请求处理任务
    let mut handlers = Vec::new();
    for worker_id in 0..10 {
        let tx = tx.clone();
        let counter = Arc::clone(&request_counter);
        
        let handler = spawn_tracked(
            async move {
                for request_id in 0..100 {
                    // 模拟传入请求
                    let request = format!("Request-{}-{}", worker_id, request_id);
                    
                    // 处理请求
                    let response = process_request(request).await;
                    
                    // 发送响应
                    if tx.send(response).await.is_err() {
                        break;
                    }
                    
                    counter.fetch_add(1, Ordering::Relaxed);
                    
                    // 模拟请求间隔
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
            },
            TaskType::NetworkIntensive
        );
        
        handlers.push(handler);
    }
    
    // 响应收集器
    let collector = spawn_tracked(
        async move {
            let mut responses = Vec::new();
            while let Some(response) = rx.recv().await {
                responses.push(response);
                if responses.len() >= 1000 {
                    break;
                }
            }
            responses.len()
        },
        TaskType::IoIntensive
    );
    
    // 等待所有处理器
    for handler in handlers {
        handler.await?;
    }
    
    let response_count = collector.await?;
    let total_requests = request_counter.load(Ordering::Relaxed);
    
    println!("🌐 异步 Web 服务器模拟完成！");
    println!("   - 处理的请求: {}", total_requests);
    println!("   - 收集的响应: {}", response_count);
    
    // 获取最终内存快照
    let snapshot = get_memory_snapshot();
    println!("   - 峰值并发任务: {}", snapshot.peak_concurrent_tasks());
    println!("   - 分配的总内存: {:.2} MB", snapshot.total_memory_bytes() as f64 / (1024.0 * 1024.0));
    
    Ok(())
}

async fn process_request(request: String) -> String {
    // 模拟数据库查询
    tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
    let query_result = vec![request.clone(); 10];
    
    // 模拟业务逻辑
    let processed_data = query_result
        .iter()
        .map(|r| format!("Processed: {}", r))
        .collect::<Vec<_>>();
    
    // 模拟响应序列化
    format!("{{\"status\":\"ok\",\"data\":{:?}}}", processed_data)
}
```

### 微服务通信

```rust
use memscope_rs::async_memory::{
    initialize, create_tracked, TaskType, 
    TaskMemoryProfile, get_memory_snapshot
};
use tokio::sync::oneshot;
use std::collections::HashMap;

#[tokio::main]
async fn microservice_simulation() -> Result<(), Box<dyn std::error::Error>> {
    initialize().await?;
    
    // 服务 A：用户服务
    let user_service = create_tracked(
        async {
            let mut users = Vec::new();
            for i in 0..1000 {
                // 模拟用户数据处理
                let user = format!("{{\"id\":{},\"name\":\"User{}\",\"email\":\"user{}@example.com\"}}", i, i, i);
                users.push(user);
                
                // 模拟数据库写入
                if i % 100 == 0 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
            }
            users.len()
        },
        TaskType::IoIntensive
    );
    
    // 服务 B：支付服务
    let payment_service = create_tracked(
        async {
            let mut transactions = Vec::new();
            for i in 0..500 {
                // 模拟支付处理
                let transaction = format!("{{\"id\":{},\"amount\":{:.2},\"status\":\"completed\"}}", i, i as f64 * 10.50);
                transactions.push(transaction);
                
                // 模拟外部 API 调用
                tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
            }
            transactions.len()
        },
        TaskType::NetworkIntensive
    );
    
    // 服务 C：分析服务
    let analytics_service = create_tracked(
        async {
            let mut metrics = Vec::new();
            for batch in 0..50 {
                // 模拟数据聚合
                let mut batch_data = Vec::new();
                for i in 0..1000 {
                    let value = (batch * 1000 + i) as f64;
                    batch_data.push(value.sin().abs());
                }
                
                // 计算统计
                let sum: f64 = batch_data.iter().sum();
                let avg = sum / batch_data.len() as f64;
                let metric = format!("{{\"batch\":{},\"avg\":{:.4},\"count\":{}}}", batch, avg, batch_data.len());
                metrics.push(metric);
                
                // 模拟计算延迟
                tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
            }
            metrics.len()
        },
        TaskType::CpuIntensive
    );
    
    // API 网关 - 编排所有服务
    let gateway = create_tracked(
        async {
            let mut responses = Vec::new();
            
            // 模拟 100 个复合请求
            for request_id in 0..100 {
                // 模拟并行服务调用
                let user_data = format!("User data for request {}", request_id);
                let payment_data = format!("Payment data for request {}", request_id);
                let analytics_data = format!("Analytics for request {}", request_id);
                
                // 组合响应
                let composite_response = format!(
                    "{{\"request_id\":{},\"user\":\"{}\",\"payment\":\"{}\",\"analytics\":\"{}\"}}",
                    request_id, user_data, payment_data, analytics_data
                );
                responses.push(composite_response);
                
                // 模拟响应时间
                tokio::time::sleep(tokio::time::Duration::from_millis(15)).await;
            }
            
            responses.len()
        },
        TaskType::NetworkIntensive
    );
    
    // 等待所有服务
    let (user_count, payment_count, analytics_count, gateway_count) = tokio::try_join!(
        user_service,
        payment_service,
        analytics_service,
        gateway
    )?;
    
    println!("🏗️  微服务模拟完成！");
    println!("   - 用户服务处理: {} 用户", user_count);
    println!("   - 支付服务处理: {} 交易", payment_count);
    println!("   - 分析服务处理: {} 指标", analytics_count);
    println!("   - 网关处理: {} 复合请求", gateway_count);
    
    // 最终内存分析
    let snapshot = get_memory_snapshot();
    println!("📊 内存分析：");
    println!("   - 峰值并发任务: {}", snapshot.peak_concurrent_tasks());
    println!("   - 跟踪的总分配: {}", snapshot.total_allocations());
    println!("   - 内存效率: {:.1}%", snapshot.memory_efficiency_percent());
    
    Ok(())
}
```

## 📊 性能特征

### 跟踪开销

| 特性 | 开销 | 描述 |
|---------|----------|-------------|
| **任务识别** | < 5ns | 零开销任务 ID 提取 |
| **内存跟踪** | < 0.1% CPU | 无锁事件缓冲 |
| **数据收集** | < 1MB/线程 | 高效环形缓冲区 |

### 任务分析指标

```rust
use memscope_rs::async_memory::{TaskMemoryProfile, TaskPerformanceMetrics};

async fn analyze_task_performance() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = get_memory_snapshot();
    
    // 获取每种任务类型的性能指标
    let cpu_metrics = snapshot.get_task_metrics(TaskType::CpuIntensive);
    let io_metrics = snapshot.get_task_metrics(TaskType::IoIntensive);
    let network_metrics = snapshot.get_task_metrics(TaskType::NetworkIntensive);
    
    println!("🎯 任务性能分析：");
    println!("CPU 任务：");
    println!("   - 平均内存: {:.2} MB", cpu_metrics.avg_memory_mb);
    println!("   - 峰值内存: {:.2} MB", cpu_metrics.peak_memory_mb);
    println!("   - 完成率: {:.1}%", cpu_metrics.completion_rate_percent);
    
    println!("I/O 任务：");
    println!("   - 平均内存: {:.2} MB", io_metrics.avg_memory_mb);
    println!("   - 等待时间: {:.2}ms", io_metrics.avg_wait_time_ms);
    
    println!("网络任务：");
    println!("   - 平均内存: {:.2} MB", network_metrics.avg_memory_mb);
    println!("   - 吞吐量: {:.1} 请求/秒", network_metrics.throughput_per_sec);
    
    Ok(())
}
```

## 🔗 下一步

- **[混合模块](hybrid.md)** - 跨模块综合分析
- **[API 参考](api-reference/analysis-api.md)** - 完整异步 API 文档
- **[示例](examples/async-usage.md)** - 更多异步示例