# Integration Examples

Practical examples for integrating memscope-rs into existing projects.

## 🎯 Objectives

- Web server integration
- CLI tool integration
- Test framework integration
- CI/CD integration

## 🌐 Web Server Integration

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

## 🔧 CLI Tool Integration

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
    
    // Process data...
    
    Ok(())
}
```

## 🧪 Test Integration

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

## 🔄 CI/CD Integration

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

## 🎉 Summary

memscope-rs can be easily integrated into:
- Web applications
- CLI tools
- Test suites
- CI/CD pipelines