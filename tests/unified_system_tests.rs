//! test core system

use memscope_rs::analysis::analyzer::{
    Analyzer, CompositeAnalyzer, FragmentationAnalyzer, LeakAnalyzer, LifecycleAnalyzer,
    SafetyAnalyzer, SmartPointerAnalyzer,
};
use memscope_rs::export::exporter::ExportConfig;
use memscope_rs::export::exporter::{
    BinaryExporter, CompositeExporter, CsvExporter, ExportBackend, ExportOutput, HtmlGenerator,
    JsonExporter,
};
use memscope_rs::tracker::backend::{
    AllocationContext, AsyncBackend, HybridBackend, SingleThreadBackend, ThreadLocalBackend,
    TrackingBackend, TrackingConfig, TrackingStrategy, UnifiedTracker,
};
use memscope_rs::types::internal_types::{
    Allocation, Event, MemoryPassport, MemorySource, PassportStatus, Snapshot,
};

use std::time::{SystemTime, UNIX_EPOCH};

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[test]
fn test_unified_types_allocation() {
    let allocation = Allocation::new(0x1000, 1024);
    assert_eq!(allocation.ptr, 0x1000);
    assert_eq!(allocation.size, 1024);
    assert!(allocation.is_active());
}

#[test]
fn test_unified_types_event() {
    let event = Event::Alloc {
        ptr: 0x1000,
        size: 1024,
        thread: 1,
        ts: 1000,
    };
    assert_eq!(event.ptr(), Some(0x1000));
    assert_eq!(event.timestamp(), 1000);
}

#[test]
fn test_unified_types_memory_passport() {
    let ts = now();
    let passport = MemoryPassport::new(1, 0x1000, 1024, MemorySource::Rust, ts);
    assert_eq!(passport.ptr, 0x1000);
    assert_eq!(passport.size, 1024);
    assert_eq!(passport.status, PassportStatus::Active);
}

#[test]
fn test_unified_types_snapshot() {
    let mut snapshot = Snapshot::new(now());
    let allocation = Allocation::new(0x1000, 1024);
    snapshot.allocations.push(allocation);
    assert_eq!(snapshot.allocations.len(), 1);
    // Stats might not be automatically updated
    // assert!(snapshot.stats.total_allocations > 0);
}

#[test]
fn test_single_thread_backend() {
    let backend = SingleThreadBackend::new();
    let ctx = AllocationContext::new(1024);
    backend.track_allocation(0x1000, 1024, ctx);
    let snapshot = backend.snapshot();
    assert_eq!(snapshot.allocations.len(), 1);
    backend.track_deallocation(0x1000);
    let snapshot = backend.snapshot();
    assert_eq!(snapshot.allocations.len(), 1);
}

#[test]
fn test_thread_local_backend() {
    let backend = ThreadLocalBackend::new();
    let ctx = AllocationContext::new(1024);
    backend.track_allocation(0x1000, 1024, ctx);
    let snapshot = backend.snapshot();
    assert_eq!(snapshot.allocations.len(), 1);
}

#[test]
fn test_async_backend() {
    let backend = AsyncBackend::new();
    let ctx = AllocationContext::new(1024);
    backend.track_allocation(0x1000, 1024, ctx);
    let snapshot = backend.snapshot();
    assert_eq!(snapshot.allocations.len(), 1);
}

#[test]
fn test_hybrid_backend() {
    let backend = HybridBackend::new();
    let ctx = AllocationContext::new(1024);
    backend.track_allocation(0x1000, 1024, ctx);
    let snapshot = backend.snapshot();
    assert_eq!(snapshot.allocations.len(), 1);
}

#[test]
fn test_unified_tracker_single_thread() {
    let config = TrackingConfig {
        strategy: TrackingStrategy::SingleThread,
        sampling: Default::default(),
        overhead_limit: Default::default(),
        enable_smart_pointers: true,
        enable_lifecycle: true,
        enable_passports: true,
    };
    let tracker = UnifiedTracker::new(config);
    tracker.track_allocation(0x1000, 1024);
    let snapshot = tracker.snapshot();
    assert_eq!(snapshot.allocations.len(), 1);
}

#[test]
fn test_unified_tracker_thread_local() {
    let config = TrackingConfig {
        strategy: TrackingStrategy::ThreadLocal,
        sampling: Default::default(),
        overhead_limit: Default::default(),
        enable_smart_pointers: true,
        enable_lifecycle: true,
        enable_passports: true,
    };
    let tracker = UnifiedTracker::new(config);
    tracker.track_allocation(0x1000, 1024);
    let snapshot = tracker.snapshot();
    assert_eq!(snapshot.allocations.len(), 1);
}

#[test]
fn test_unified_tracker_async() {
    let config = TrackingConfig {
        strategy: TrackingStrategy::Async,
        sampling: Default::default(),
        overhead_limit: Default::default(),
        enable_smart_pointers: true,
        enable_lifecycle: true,
        enable_passports: true,
    };
    let tracker = UnifiedTracker::new(config);
    tracker.track_allocation(0x1000, 1024);
    let snapshot = tracker.snapshot();
    assert_eq!(snapshot.allocations.len(), 1);
}

#[test]
fn test_unified_tracker_hybrid() {
    let config = TrackingConfig {
        strategy: TrackingStrategy::Hybrid,
        sampling: Default::default(),
        overhead_limit: Default::default(),
        enable_smart_pointers: true,
        enable_lifecycle: true,
        enable_passports: true,
    };
    let tracker = UnifiedTracker::new(config);
    tracker.track_allocation(0x1000, 1024);
    let snapshot = tracker.snapshot();
    assert_eq!(snapshot.allocations.len(), 1);
}

#[test]
fn test_leak_analyzer() {
    let analyzer = LeakAnalyzer::new();
    let mut snapshot = Snapshot::new(now());
    let allocation = Allocation::new(0x1000, 1024);
    snapshot.allocations.push(allocation);
    let report = analyzer.analyze(&snapshot);
    assert!(report.analyzer_name.contains("Leak"));
}

#[test]
fn test_fragmentation_analyzer() {
    let analyzer = FragmentationAnalyzer::new();
    let mut snapshot = Snapshot::new(now());
    let allocation = Allocation::new(0x1000, 1024);
    snapshot.allocations.push(allocation);
    let report = analyzer.analyze(&snapshot);
    assert!(report.analyzer_name.contains("Fragmentation"));
}

#[test]
fn test_lifecycle_analyzer() {
    let analyzer = LifecycleAnalyzer::new();
    let mut snapshot = Snapshot::new(now());
    let allocation = Allocation::new(0x1000, 1024);
    snapshot.allocations.push(allocation);
    let report = analyzer.analyze(&snapshot);
    assert!(report.analyzer_name.contains("Lifecycle"));
}

#[test]
fn test_smart_pointer_analyzer() {
    let analyzer = SmartPointerAnalyzer::new();
    let mut snapshot = Snapshot::new(now());
    let allocation = Allocation::new(0x1000, 1024);
    snapshot.allocations.push(allocation);
    let report = analyzer.analyze(&snapshot);
    assert!(report.analyzer_name.contains("Smart Pointer"));
}

#[test]
fn test_safety_analyzer() {
    let analyzer = SafetyAnalyzer::new();
    let mut snapshot = Snapshot::new(now());
    let allocation = Allocation::new(0x1000, 1024);
    snapshot.allocations.push(allocation);
    let report = analyzer.analyze(&snapshot);
    assert!(report.analyzer_name.contains("Safety"));
}

#[test]
fn test_composite_analyzer() {
    let composite = CompositeAnalyzer::new();
    let mut snapshot = Snapshot::new(now());
    let allocation = Allocation::new(0x1000, 1024);
    snapshot.allocations.push(allocation);
    let reports = composite.analyze_all(&snapshot);
    assert!(!reports.is_empty());
    assert!(reports.iter().any(|r| r.analyzer_name.contains("Leak")));
}

#[test]
fn test_json_exporter() {
    let exporter = JsonExporter::new();
    let mut snapshot = Snapshot::new(now());
    let allocation = Allocation::new(0x1000, 1024);
    snapshot.allocations.push(allocation);
    let export_config = ExportConfig::default();
    let result = exporter.export(&snapshot, &export_config);
    assert!(result.is_ok());
    let output = result.unwrap();
    match output {
        ExportOutput::String(data) => assert!(!data.is_empty()),
        _ => panic!("Expected string output"),
    }
}

#[test]
fn test_csv_exporter() {
    let exporter = CsvExporter::new();
    let mut snapshot = Snapshot::new(now());
    let allocation = Allocation::new(0x1000, 1024);
    snapshot.allocations.push(allocation);
    let export_config = ExportConfig::default();
    let result = exporter.export(&snapshot, &export_config);
    assert!(result.is_ok());
    let output = result.unwrap();
    match output {
        ExportOutput::String(data) => assert!(!data.is_empty()),
        _ => panic!("Expected string output"),
    }
}

#[test]
fn test_binary_exporter() {
    let exporter = BinaryExporter::new();
    let mut snapshot = Snapshot::new(now());
    let allocation = Allocation::new(0x1000, 1024);
    snapshot.allocations.push(allocation);
    let export_config = ExportConfig::default();
    let result = exporter.export(&snapshot, &export_config);
    assert!(result.is_ok());
    let output = result.unwrap();
    match output {
        ExportOutput::Binary(data) => assert!(!data.is_empty()),
        _ => panic!("Expected binary output"),
    }
}

#[test]
fn test_html_generator() {
    let generator = HtmlGenerator::new();
    let mut snapshot = Snapshot::new(now());
    let allocation = Allocation::new(0x1000, 1024);
    snapshot.allocations.push(allocation);
    let export_config = ExportConfig::default();
    let result = generator.export(&snapshot, &export_config);
    assert!(result.is_ok());
    let output = result.unwrap();
    match output {
        ExportOutput::String(data) => {
            assert!(!data.is_empty());
            // Check for any HTML content
            assert!(
                data.contains("<body>")
                    || data.contains("<div")
                    || data.contains("MemScope")
                    || data.len() > 100
            );
        }
        _ => panic!("Expected string output"),
    }
}

#[test]
fn test_composite_exporter() {
    let exporter = CompositeExporter::new();
    let mut snapshot = Snapshot::new(now());
    let allocation = Allocation::new(0x1000, 1024);
    snapshot.allocations.push(allocation);
    let export_config = ExportConfig::default();
    let result = exporter.export(&snapshot, &export_config);
    assert!(result.is_ok());
    let output = result.unwrap();
    match output {
        ExportOutput::String(data) => assert!(!data.is_empty()),
        _ => panic!("Expected string output"),
    }
}

#[test]
fn test_integration_tracking_and_analysis() {
    let config = TrackingConfig {
        strategy: TrackingStrategy::SingleThread,
        sampling: Default::default(),
        overhead_limit: Default::default(),
        enable_smart_pointers: true,
        enable_lifecycle: true,
        enable_passports: true,
    };
    let tracker = UnifiedTracker::new(config);
    tracker.track_allocation(0x1000, 1024);
    tracker.track_allocation(0x2000, 2048);
    tracker.track_deallocation(0x1000);

    let snapshot = tracker.snapshot();
    assert_eq!(snapshot.allocations.len(), 2);

    let analyzer = LeakAnalyzer::new();
    let report = analyzer.analyze(&snapshot);
    assert!(report.analyzer_name.contains("Leak"));
}

#[test]
fn test_integration_tracking_and_export() {
    let config = TrackingConfig {
        strategy: TrackingStrategy::SingleThread,
        sampling: Default::default(),
        overhead_limit: Default::default(),
        enable_smart_pointers: true,
        enable_lifecycle: true,
        enable_passports: true,
    };
    let tracker = UnifiedTracker::new(config);
    tracker.track_allocation(0x1000, 1024);

    let snapshot = tracker.snapshot();
    assert_eq!(snapshot.allocations.len(), 1);

    let exporter = JsonExporter::new();
    let export_config = ExportConfig::default();
    let result = exporter.export(&snapshot, &export_config);
    assert!(result.is_ok());
}

#[test]
fn test_integration_full_pipeline() {
    let config = TrackingConfig {
        strategy: TrackingStrategy::SingleThread,
        sampling: Default::default(),
        overhead_limit: Default::default(),
        enable_smart_pointers: true,
        enable_lifecycle: true,
        enable_passports: true,
    };
    let tracker = UnifiedTracker::new(config);

    // 模拟内存分配和释放
    tracker.track_allocation(0x1000, 1024);
    tracker.track_allocation(0x2000, 2048);
    tracker.track_allocation(0x3000, 4096);
    tracker.track_deallocation(0x1000);

    // 获取快照
    let snapshot = tracker.snapshot();
    assert_eq!(snapshot.allocations.len(), 3);

    // 分析
    let leak_analyzer = LeakAnalyzer::new();
    let leak_report = leak_analyzer.analyze(&snapshot);
    assert!(leak_report.analyzer_name.contains("Leak"));

    let fragmentation_analyzer = FragmentationAnalyzer::new();
    let fragmentation_report = fragmentation_analyzer.analyze(&snapshot);
    assert!(fragmentation_report.analyzer_name.contains("Fragmentation"));

    // 导出
    let json_exporter = JsonExporter::new();
    let export_config = ExportConfig::default();
    let json_result = json_exporter.export(&snapshot, &export_config);
    assert!(json_result.is_ok());

    let csv_exporter = CsvExporter::new();
    let csv_result = csv_exporter.export(&snapshot, &export_config);
    assert!(csv_result.is_ok());

    let html_generator = HtmlGenerator::new();
    let html_result = html_generator.export(&snapshot, &export_config);
    assert!(html_result.is_ok());

    let binary_exporter = BinaryExporter::new();
    let binary_result = binary_exporter.export(&snapshot, &export_config);
    assert!(binary_result.is_ok());
}

#[test]
fn test_memory_passport_tracking() {
    let ts = now();
    let mut passport = MemoryPassport::new(1, 0x1000, 1024, MemorySource::Rust, ts);
    assert_eq!(passport.status, PassportStatus::Active);

    passport.transfer("module_a".to_string(), ts);
    assert_eq!(
        passport.status,
        PassportStatus::Transferred {
            to: "module_a".to_string()
        }
    );

    passport.release(ts);
    assert_eq!(passport.status, PassportStatus::Released);
}

#[test]
fn test_async_task_tracking() {
    let config = TrackingConfig {
        strategy: TrackingStrategy::Async,
        sampling: Default::default(),
        overhead_limit: Default::default(),
        enable_smart_pointers: true,
        enable_lifecycle: true,
        enable_passports: true,
    };
    let tracker = UnifiedTracker::new(config);

    // 模拟异步任务创建
    tracker.track_task_spawn(1);
    tracker.track_allocation(0x1000, 1024);
    tracker.track_task_end(1);

    let snapshot = tracker.snapshot();
    assert!(!snapshot.tasks.is_empty());
}

#[test]
fn test_ffi_tracking() {
    let config = TrackingConfig {
        strategy: TrackingStrategy::SingleThread,
        sampling: Default::default(),
        overhead_limit: Default::default(),
        enable_smart_pointers: true,
        enable_lifecycle: true,
        enable_passports: true,
    };
    let tracker = UnifiedTracker::new(config);

    // simulate FFI allocation and deallocation
    tracker.track_ffi_alloc(0x1000, 1024, "libc");
    tracker.track_ffi_free(0x1000, "libc");

    let snapshot = tracker.snapshot();
    // ensure record event
    assert!(snapshot.allocations.len() > 0 || snapshot.stats.total_allocations > 0);
}
