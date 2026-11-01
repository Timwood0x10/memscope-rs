//! Each allocator regression runs in a fresh process: deadlocks must fail with a
//! deadline, and allocator aborts must not take down the parent test suite.

use memscope_rs::task_registry::{global_registry, TaskIdRegistry};
use memscope_rs::tracker::Tracker;
use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

#[cfg(not(feature = "tracking-allocator"))]
#[global_allocator]
static ALLOCATOR: memscope_rs::TrackingAllocator = memscope_rs::TrackingAllocator::new();

const CASE_ENV: &str = "MEMSCOPE_ALLOCATOR_REENTRANCY_CASE";
const DIR_ENV: &str = "MEMSCOPE_ALLOCATOR_REENTRANCY_DIR";

fn run_child(case: &str) {
    let dir = tempfile::tempdir().expect("create child artifacts directory");
    let log_path = dir.path().join("child.log");
    let log = std::fs::File::create(&log_path).expect("create child output");
    let mut child = Command::new(std::env::current_exe().expect("locate test binary"))
        .args([
            "--ignored",
            "--exact",
            "allocator_child",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(CASE_ENV, case)
        .env(DIR_ENV, dir.path())
        .stdin(Stdio::null())
        .stdout(log.try_clone().expect("clone log handle"))
        .stderr(log)
        .spawn()
        .expect("spawn allocator child");

    let status = match child.wait_timeout(Duration::from_secs(15)) {
        Ok(Some(status)) => status,
        result => {
            // Always reap, including errors; do not leave a deadlocked test behind.
            let _ = child.kill();
            let _ = child.wait();
            let output = std::fs::read_to_string(&log_path).unwrap_or_default();
            panic!("{case}: child did not finish: {result:?}\n{output}");
        }
    };
    let output = std::fs::read_to_string(&log_path).expect("read child output");
    assert!(
        status.success(),
        "{case}: child exited with {status}\n{output}"
    );
    assert!(
        dir.path().join("completed").is_file(),
        "{case}: child workload did not run (check --exact filter)\n{output}"
    );
}

#[test]
fn registry_query_is_allocator_safe() {
    run_child("registry_query");
}

#[test]
fn auto_export_configuration_is_allocator_safe() {
    run_child("auto_export");
}

#[test]
fn thread_local_strategy_is_allocator_safe() {
    run_child("thread_local");
}

#[test]
fn cross_thread_release_is_published() {
    run_child("cross_thread");
}

#[test]
#[ignore = "entry point launched by the parent regression tests"]
fn allocator_child() {
    let Ok(case) = std::env::var(CASE_ENV) else {
        return;
    };
    let dir =
        std::path::PathBuf::from(std::env::var_os(DIR_ENV).expect("child artifacts directory"));
    match case.as_str() {
        "registry_query" => {
            let registry = global_registry();
            registry.clear();
            let _scope = registry.task_scope("active_task");
            let id = TaskIdRegistry::current_task_id().expect("task context");
            let data = vec![1u8; 64];
            let meta = registry.get_task(id).expect("query under an active task");
            assert!(meta.memory_usage >= 64);
            std::hint::black_box(data);
        }
        "auto_export" => {
            let path = dir.join("export");
            std::fs::create_dir(&path).expect("create export directory");
            let tracker = Tracker::global().with_auto_export(path.to_str().unwrap());
            let data = vec![1u8; 64];
            memscope_rs::track!(tracker, data);
            drop(data);
            assert!(
                !path.join("memory_analysis.json").exists(),
                "allocator callbacks must not auto-export"
            );
            drop(tracker);
            assert!(path.join("memory_analysis.json").exists());
        }
        "thread_local" => {
            memscope_rs::configure_tracking_strategy(true);
            std::thread::spawn(|| {
                std::hint::black_box(vec![1u8; 64]);
            })
            .join()
            .expect("worker allocation must survive");
            memscope_rs::configure_tracking_strategy(false);
        }
        "cross_thread" => {
            use memscope_rs::event_store::MemoryEventType;
            memscope_rs::configure_tracking_strategy(true);
            let tracker = Tracker::global();
            let data = vec![1u8; 64];
            let ptr = data.as_ptr() as usize;
            let serial = tracker
                .events()
                .iter()
                .rev()
                .find(|event| event.ptr == ptr && event.event_type == MemoryEventType::Allocate)
                .expect("allocator must publish the block")
                .binding_serial;
            std::thread::spawn(move || drop(data))
                .join()
                .expect("cross-thread drop");
            memscope_rs::configure_tracking_strategy(false);
            let releases = tracker
                .events()
                .iter()
                .filter(|event| {
                    event.ptr == ptr
                        && event.binding_serial == serial
                        && event.event_type == MemoryEventType::Deallocate
                })
                .count();
            assert_eq!(releases, 1);
        }
        other => panic!("unknown child case: {other}"),
    }
    std::fs::write(dir.join("completed"), case).expect("publish child completion marker");
}
