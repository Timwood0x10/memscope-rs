# 集成示例

将 memscope-rs 集成到现有项目中的实用示例。

## 🎯 目标

- Web 服务器集成
- CLI 工具集成
- 测试框架集成
- CI/CD 集成

## 🌐 Web 服务器集成

```rust
use memscope_rs::{init, track_var};
use warp::Filter;

#[tokio::main]
async fn main() {
    init();
    
    let routes = warp::path("api")
        .and(warp::path("data"))
        .map(|| {
            let response_data = vec![1, 2, 3, 4, 5];
            track_var!(response_data);
            warp::reply::json(&response_data)
        });
    
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
```

## 🔧 CLI 工具集成

```rust
use clap::Parser;
use memscope_rs::{init, track_var};

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    input: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init();
    
    let args = Args::parse();
    let input_data = std::fs::read_to_string(&args.input)?;
    track_var!(input_data);
    
    // 处理数据...
    
    Ok(())
}
```

## 🧪 测试集成

```rust
#[cfg(test)]
mod tests {
    use memscope_rs::{init, track_var, get_global_tracker};
    
    #[test]
    fn test_memory_usage() {
        init();
        
        let test_data = vec![1; 1000];
        track_var!(test_data);
        
        let tracker = get_global_tracker();
        let stats = tracker.get_stats().unwrap();
        assert!(stats.active_allocations > 0);
    }
}
```

## 🔄 CI/CD 集成

```yaml
# .github/workflows/memory-check.yml
name: Memory Analysis
on: [push, pull_request]
jobs:
  memory-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run memory analysis
        run: |
          cargo test --features memory-analysis
          memscope analyze cargo test
```

## 🎉 总结

memscope-rs 可以轻松集成到：
- Web 应用
- CLI 工具  
- 测试套件
- CI/CD 流程