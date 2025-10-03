# 混合模块：跨模块综合分析

混合模块**结合所有跟踪策略**到统一分析框架中。它自动检测并适应不同的运行时模式，提供跨单线程、多线程和异步组件的综合洞察。

## 🎯 适用场景

**✅ 完美适用于：**
- 混合模式的复杂应用程序
- 全栈应用程序（Web 服务器 + 异步任务 + 线程）
- 综合系统分析
- 复杂系统的生产监控
- 跨组件性能分析

**❌ 过度使用于：**
- 简单的单模式应用程序
- 开发/调试（使用特定模块）
- 资源受限环境

## 🔄 核心 API

### 统一跟踪

```rust
use memscope_rs::export::fixed_hybrid_template::{FixedHybridTemplate, RenderMode};
use memscope_rs::unified::{
    UnifiedBackend, EnvironmentDetector, TrackingStrategy
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化统一后端 - 自动检测环境
    let mut backend = UnifiedBackend::new();
    backend.auto_configure()?;
    
    // 后端将自动选择最优策略：
    // - 主线程使用单线程跟踪
    // - 线程池使用多线程跟踪
    // - tokio 任务使用异步跟踪
    
    // 你的混合应用程序代码
    let data = vec![1, 2, 3, 4, 5];
    memscope_rs::track_var!(data);  // 单线程跟踪
    
    // 多线程工作
    let handles: Vec<_> = (0..10).map(|i| {
        std::thread::spawn(move || {
            let thread_data = vec![i; 1000];
            // 自动使用 lockfree 跟踪
            thread_data.len()
        })
    }).collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    // 生成综合混合仪表板
    let template = FixedHybridTemplate::new(RenderMode::Complete);
    let hybrid_data = backend.collect_comprehensive_data()?;
    let dashboard = template.generate_hybrid_dashboard(&hybrid_data)?;
    
    std::fs::write("comprehensive_dashboard.html", dashboard)?;
    
    println!("🎯 综合分析完成！");
    println!("📊 仪表板: comprehensive_dashboard.html");
    
    Ok(())
}
```

### 高级混合分析

```rust
use memscope_rs::unified::{
    UnifiedBackend, TrackingDispatcher, HybridStrategy
};
use memscope_rs::export::fixed_hybrid_template::*;

#[tokio::main]
async fn comprehensive_hybrid_analysis() -> Result<(), Box<dyn std::error::Error>> {
    // 配置高级混合跟踪
    let mut backend = UnifiedBackend::new();
    
    // 设置自定义策略组合
    let hybrid_strategy = HybridStrategy {
        single_thread_threshold: 5,     // < 5 线程使用单线程
        async_task_threshold: 10,       // > 10 任务切换到异步跟踪
        resource_monitoring: true,      // 启用系统资源监控
        cross_boundary_tracking: true,  // 跟踪组件间数据流
    };
    
    backend.configure_hybrid_strategy(hybrid_strategy)?;
    
    // 复杂应用程序模拟
    
    // 1. 单线程初始化
    let config_data = vec!["config1", "config2", "config3"];
    memscope_rs::track_var!(config_data);
    
    // 2. 多线程数据处理
    let processing_handles: Vec<_> = (0..20).map(|worker_id| {
        std::thread::spawn(move || {
            // 重计算
            let mut results = Vec::new();
            for batch in 0..100 {
                let batch_data = vec![worker_id * 1000 + batch; 1000];
                let processed: Vec<_> = batch_data
                    .iter()
                    .map(|&x| x * x)
                    .collect();
                results.extend(processed);
            }
            results.len()
        })
    }).collect();
    
    // 3. 并发异步任务
    let async_tasks = (0..50).map(|task_id| {
        tokio::spawn(async move {
            // 模拟异步 I/O 工作
            let mut data_chunks = Vec::new();
            for chunk in 0..20 {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                let chunk_data = format!("Task-{}-Chunk-{}-{}", task_id, chunk, "X".repeat(100));
                data_chunks.push(chunk_data);
            }
            data_chunks.len()
        })
    });
    
    // 4. 等待所有工作完成
    for handle in processing_handles {
        handle.join().unwrap();
    }
    
    let async_results: Vec<_> = futures::future::join_all(async_tasks).await;
    let total_async_chunks: usize = async_results.into_iter().map(|r| r.unwrap()).sum();
    
    println!("异步任务处理了 {} 数据块", total_async_chunks);
    
    // 5. 生成综合分析
    let comprehensive_data = backend.finalize_and_collect()?;
    
    // 创建详细混合仪表板
    let template = FixedHybridTemplate::new(RenderMode::Complete);
    let dashboard = template.generate_hybrid_dashboard(&comprehensive_data)?;
    
    // 导出多种格式用于不同用例
    std::fs::write("hybrid_comprehensive.html", dashboard)?;
    
    // 生成性能对比报告
    let comparison_data = comprehensive_data.generate_performance_comparison();
    let comparison_report = template.generate_comparison_report(&comparison_data)?;
    std::fs::write("performance_comparison.html", comparison_report)?;
    
    // 生成资源利用率报告
    let resource_data = comprehensive_data.get_resource_utilization();
    let resource_report = template.generate_resource_report(&resource_data)?;
    std::fs::write("resource_utilization.html", resource_report)?;
    
    println!("📊 综合混合分析完成！");
    println!("   📁 主仪表板: hybrid_comprehensive.html");
    println!("   ⚡ 性能对比: performance_comparison.html");
    println!("   🖥️  资源利用率: resource_utilization.html");
    
    // 打印汇总统计
    println!("📈 汇总：");
    println!("   - 单线程分配: {}", comprehensive_data.single_thread_stats.allocation_count);
    println!("   - 多线程分配: {}", comprehensive_data.multi_thread_stats.allocation_count);
    println!("   - 异步任务分配: {}", comprehensive_data.async_stats.allocation_count);
    println!("   - 跨边界数据传输: {}", comprehensive_data.cross_boundary_transfers);
    println!("   - 峰值内存使用: {:.2} MB", comprehensive_data.peak_memory_usage_mb);
    
    Ok(())
}
```

## 🏗️ 真实世界示例：全栈 Web 应用程序

```rust
use memscope_rs::unified::UnifiedBackend;
use memscope_rs::export::fixed_hybrid_template::FixedHybridTemplate;
use tokio::sync::mpsc;
use std::sync::Arc;
use std::collections::HashMap;

#[tokio::main]
async fn fullstack_web_application() -> Result<(), Box<dyn std::error::Error>> {
    // 为全栈应用初始化混合跟踪
    let mut backend = UnifiedBackend::new();
    backend.auto_configure()?;
    
    // 1. 应用程序配置（单线程）
    let app_config = HashMap::from([
        ("database_url", "postgresql://localhost:5432/app"),
        ("redis_url", "redis://localhost:6379"),
        ("port", "8080"),
    ]);
    memscope_rs::track_var!(app_config);
    
    println!("🚀 启动全栈应用程序...");
    
    // 2. 数据库连接池（多线程）
    let db_pool = Arc::new(DatabasePool::new(10));
    let pool_handles: Vec<_> = (0..10).map(|worker_id| {
        let pool = Arc::clone(&db_pool);
        std::thread::spawn(move || {
            // 模拟数据库操作
            for query_id in 0..100 {
                let connection = pool.get_connection();
                let query_result = execute_query(&connection, &format!("SELECT * FROM users WHERE id = {}", query_id));
                let processed_result = process_database_result(query_result);
                pool.return_connection(connection);
                
                // 模拟处理时间
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            worker_id
        })
    }).collect();
    
    // 3. Web 服务器（异步任务）
    let (request_tx, mut request_rx) = mpsc::channel(1000);
    
    // 请求处理器
    let handler_tasks = (0..20).map(|handler_id| {
        let tx = request_tx.clone();
        tokio::spawn(async move {
            for request_id in 0..50 {
                // 模拟传入 HTTP 请求
                let request = HttpRequest {
                    id: format!("{}-{}", handler_id, request_id),
                    path: format!("/api/users/{}", request_id),
                    method: "GET".to_string(),
                    headers: vec![("Content-Type", "application/json")],
                    body: vec![0u8; 1024], // 1KB 请求体
                };
                
                // 处理请求
                let response = process_http_request(request).await;
                
                // 发送到响应处理器
                if tx.send(response).await.is_err() {
                    break;
                }
                
                // 模拟请求间隔
                tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
            }
        })
    });
    
    // 响应处理器
    let response_processor = tokio::spawn(async move {
        let mut responses = Vec::new();
        while let Some(response) = request_rx.recv().await {
            // 模拟响应处理（日志、指标等）
            let processed_response = format!(
                "{{\"status\":{},\"size\":{},\"timestamp\":{}}}",
                response.status_code,
                response.body.len(),
                chrono::Utc::now().timestamp()
            );
            responses.push(processed_response);
            
            if responses.len() >= 1000 {
                break;
            }
        }
        responses.len()
    });
    
    // 4. 后台工作者（混合异步 + 线程）
    let background_workers = (0..5).map(|worker_id| {
        tokio::spawn(async move {
            // 邮件服务模拟
            for email_batch in 0..20 {
                let mut emails = Vec::new();
                for email_id in 0..50 {
                    let email = Email {
                        id: format!("email-{}-{}-{}", worker_id, email_batch, email_id),
                        to: format!("user{}@example.com", email_id),
                        subject: "重要通知".to_string(),
                        body: "A".repeat(2048), // 2KB 邮件正文
                    };
                    emails.push(email);
                }
                
                // 模拟邮件发送
                send_email_batch(emails).await;
                
                // 批次间等待
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            worker_id
        })
    });
    
    // 5. 等待所有组件完成
    
    // 数据库工作者
    for handle in pool_handles {
        handle.join().unwrap();
    }
    
    // Web 服务器任务
    for task in handler_tasks {
        task.await?;
    }
    
    let total_responses = response_processor.await?;
    println!("📡 处理了 {} HTTP 响应", total_responses);
    
    // 后台工作者
    for worker in background_workers {
        let worker_id = worker.await?;
        println!("📧 后台工作者 {} 完成", worker_id);
    }
    
    // 6. 生成综合分析
    let comprehensive_data = backend.finalize_and_collect()?;
    
    // 创建详细报告
    let template = FixedHybridTemplate::new(RenderMode::Complete);
    
    // 主仪表板
    let main_dashboard = template.generate_hybrid_dashboard(&comprehensive_data)?;
    std::fs::write("fullstack_analysis.html", main_dashboard)?;
    
    // 组件特定报告
    let db_report = template.generate_component_report(&comprehensive_data, "database")?;
    std::fs::write("database_analysis.html", db_report)?;
    
    let web_report = template.generate_component_report(&comprehensive_data, "web_server")?;
    std::fs::write("web_server_analysis.html", web_report)?;
    
    let background_report = template.generate_component_report(&comprehensive_data, "background_workers")?;
    std::fs::write("background_workers_analysis.html", background_report)?;
    
    println!("🎯 全栈分析完成！");
    println!("📊 生成的报告：");
    println!("   - 主报告: fullstack_analysis.html");
    println!("   - 数据库: database_analysis.html");
    println!("   - Web 服务器: web_server_analysis.html");
    println!("   - 后台工作者: background_workers_analysis.html");
    
    // 性能摘要
    println!("📈 性能摘要：");
    println!("   - 数据库吞吐量: {:.1} 查询/秒", comprehensive_data.database_metrics.queries_per_second);
    println!("   - Web 服务器吞吐量: {:.1} 请求/秒", comprehensive_data.web_metrics.requests_per_second);
    println!("   - 后台处理: {:.1} 作业/秒", comprehensive_data.background_metrics.jobs_per_second);
    println!("   - 峰值内存使用: {:.2} GB", comprehensive_data.peak_memory_usage_mb / 1024.0);
    println!("   - 内存效率: {:.1}%", comprehensive_data.memory_efficiency_percent);
    
    Ok(())
}

// 辅助结构和函数
struct DatabasePool {
    connections: Vec<String>,
}

impl DatabasePool {
    fn new(size: usize) -> Self {
        let connections = (0..size).map(|i| format!("connection-{}", i)).collect();
        DatabasePool { connections }
    }
    
    fn get_connection(&self) -> &str {
        &self.connections[0] // 简化
    }
    
    fn return_connection(&self, _conn: &str) {
        // 简化
    }
}

struct HttpRequest {
    id: String,
    path: String,
    method: String,
    headers: Vec<(&'static str, &'static str)>,
    body: Vec<u8>,
}

struct HttpResponse {
    status_code: u16,
    headers: Vec<(&'static str, &'static str)>,
    body: Vec<u8>,
}

struct Email {
    id: String,
    to: String,
    subject: String,
    body: String,
}

async fn process_http_request(request: HttpRequest) -> HttpResponse {
    // 模拟请求处理
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    
    let response_body = format!(
        "{{\"request_id\":\"{}\",\"path\":\"{}\",\"processed\":true}}",
        request.id, request.path
    );
    
    HttpResponse {
        status_code: 200,
        headers: vec![("Content-Type", "application/json")],
        body: response_body.into_bytes(),
    }
}

fn execute_query(_connection: &str, query: &str) -> Vec<String> {
    // 模拟数据库查询
    vec![format!("result for: {}", query)]
}

fn process_database_result(result: Vec<String>) -> Vec<String> {
    result.into_iter().map(|r| format!("processed: {}", r)).collect()
}

async fn send_email_batch(emails: Vec<Email>) {
    // 模拟邮件发送
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
}
```

## 📊 混合分析特性

### 跨组件数据流

混合模块跟踪不同组件间的数据移动：

```rust
use memscope_rs::unified::CrossBoundaryTracker;

fn track_data_flow() -> Result<(), Box<dyn std::error::Error>> {
    let mut tracker = CrossBoundaryTracker::new();
    
    // 跟踪从单线程到多线程的数据移动
    let data = vec![1, 2, 3, 4, 5];
    let data_id = tracker.register_data(&data, "main_thread_data")?;
    
    let handle = std::thread::spawn(move || {
        // 数据跨越线程边界
        tracker.track_boundary_cross(data_id, "thread_worker")?;
        
        // 在工作线程中处理
        let processed = data.into_iter().map(|x| x * 2).collect::<Vec<_>>();
        Ok(processed)
    });
    
    let result = handle.join().unwrap()?;
    tracker.track_completion(data_id, result.len())?;
    
    Ok(())
}
```

### 性能对比仪表板

混合模块生成跨所有跟踪模式的对比分析：

```rust
// 生成的仪表板包括：
// - 内存使用对比（单线程 vs 多线程 vs 异步）
// - 性能瓶颈识别
// - 跨组件资源利用率
// - 跨边界传输效率
// - 可扩展性分析建议
```

## 🔗 下一步

- **[API 参考](api-reference/)** - 完整 API 文档
- **[示例](examples/integration-examples.md)** - 完整集成示例
- **[性能优化](advanced/performance-optimization.md)** - 优化技巧