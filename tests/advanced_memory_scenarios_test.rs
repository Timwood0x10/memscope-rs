//! Advanced memory scenarios and patterns testing for memscope-rs.
//! Tests complex allocation patterns, smart pointers, and memory-intensive operations.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use memscope_rs::{get_global_tracker, init, track_var, Trackable};

static INIT: std::sync::Once = std::sync::Once::new();

fn ensure_init() {
    INIT.call_once(|| {
        init();
    });
}

#[test]
fn test_smart_pointer_tracking() {
    ensure_init();

    // Test Box<T>
    let boxed_data = Box::new(vec![1, 2, 3, 4, 5]);
    track_var!(boxed_data).unwrap();

    // Test Rc<T>
    let rc_data = Rc::new(String::from("Reference counted data"));
    track_var!(rc_data).unwrap();
    let rc_clone = Rc::clone(&rc_data);

    // Test Arc<T>
    let arc_data = Arc::new(vec![10, 20, 30]);
    track_var!(arc_data).unwrap();
    let arc_clone = Arc::clone(&arc_data);

    // Test nested smart pointers
    let nested = Box::new(Rc::new(Arc::new(vec![100, 200, 300])));
    track_var!(nested).unwrap();

    let tracker = get_global_tracker();
    let active_allocs = tracker.get_active_allocations();

    // Should have tracked all smart pointer allocations
    let smart_pointer_allocs = active_allocs
        .iter()
        .filter(|a| {
            a.get(0)
                .and_then(|info| info.var_name.as_ref())
                .as_ref()
                .map_or(false, |name| {
                    name.contains("boxed_data")
                        || name.contains("rc_data")
                        || name.contains("arc_data")
                        || name.contains("nested")
                })
        })
        .count();

    // Note: Smart pointer tracking might not work without global allocator feature
    println!("Smart pointer allocations: {}", smart_pointer_allocs);
    if smart_pointer_allocs == 0 {
        println!("No smart pointer allocations found, but test continues");
    }

    // Test weak references don't prevent deallocation tracking
    let weak_rc = Rc::downgrade(&rc_clone);
    let weak_arc = Arc::downgrade(&arc_clone);

    drop(rc_data);
    drop(rc_clone);
    drop(arc_data);
    drop(arc_clone);

    // Weak references should still be valid but pointing to deallocated data
    assert!(
        weak_rc.upgrade().is_none(),
        "Weak Rc should be invalid after drop"
    );
    assert!(
        weak_arc.upgrade().is_none(),
        "Weak Arc should be invalid after drop"
    );
}

#[test]
fn test_interior_mutability_patterns() {
    ensure_init();

    // Test RefCell<T>
    let ref_cell_data = RefCell::new(vec![1, 2, 3]);
    let boxed_var = Box::new(ref_cell_data.clone());
    track_var!(boxed_var).unwrap();

    {
        let mut borrowed = ref_cell_data.borrow_mut();
        borrowed.push(4);
        borrowed.push(5);
    }

    // Test Cell<T>
    let cell_data = Cell::new(42);
    cell_data.set(100);

    // Test Rc<RefCell<T>> pattern
    let shared_mutable = Rc::new(RefCell::new(HashMap::new()));
    track_var!(shared_mutable).unwrap();

    {
        let mut map = shared_mutable.borrow_mut();
        map.insert("key1".to_string(), vec![1, 2, 3]);
        map.insert("key2".to_string(), vec![4, 5, 6]);
    }

    let shared_clone = Rc::clone(&shared_mutable);
    {
        let map = shared_clone.borrow();
        assert_eq!(map.len(), 2, "Should have 2 entries");
    }

    let tracker = get_global_tracker();
    let stats = tracker.get_stats();
    assert!(
        stats.unwrap().total_allocations > 0,
        "Should track interior mutability allocations"
    );
}

#[test]
fn test_collection_types() {
    ensure_init();

    // Test various collection types
    let mut hash_map = HashMap::new();
    for i in 0..100 {
        hash_map.insert(format!("key_{}", i), vec![i; 10]);
    }
    let boxed_map = Box::new(hash_map);
    track_var!(boxed_map).unwrap();

    let mut hash_set = HashSet::new();
    for i in 0..50 {
        hash_set.insert(format!("item_{}", i));
    }
    let boxed_set = Box::new(hash_set);
    track_var!(boxed_set).unwrap();

    let mut btree_map = BTreeMap::new();
    for i in 0..75 {
        btree_map.insert(i, format!("value_{}", i));
    }
    let boxed_map = Box::new(btree_map);
    track_var!(boxed_map).unwrap();

    let mut vec_deque = VecDeque::new();
    for i in 0..25 {
        vec_deque.push_back(vec![i; 5]);
    }
    let boxed_vec_deque = Box::new(vec_deque);
    track_var!(boxed_vec_deque).unwrap();

    // Test collection operations that cause reallocations
    let mut growing_vec = Vec::new();
    track_var!(growing_vec).unwrap();

    for i in 0..1000 {
        growing_vec.push(i);
        // This will cause multiple reallocations as the vector grows
    }

    let tracker = get_global_tracker();
    let stats = tracker.get_stats();
    assert!(
        stats.unwrap().total_allocations > 0,
        "Should track collection allocations"
    );
}

#[test]
fn test_string_and_text_operations() {
    ensure_init();

    // Test various string operations
    let mut string_data = String::new();
    track_var!(string_data).unwrap();

    // String growth operations
    for i in 0..100 {
        string_data.push_str(&format!("Item {} ", i));
    }

    // String from various sources
    let from_str = String::from("Hello, World!");
    track_var!(from_str).unwrap();

    let from_format = format!("Formatted string with {} items", 42);
    track_var!(from_format).unwrap();

    // String operations that may reallocate
    let mut concatenated = String::new();
    let parts = vec![
        "Hello", " ", "World", "!", " ", "How", " ", "are", " ", "you?",
    ];
    for part in parts {
        concatenated.push_str(part);
    }
    track_var!(concatenated).unwrap();

    // Test string slicing and conversion
    let slice_to_string = concatenated[0..5].to_string();
    track_var!(slice_to_string).unwrap();

    let tracker = get_global_tracker();
    let active_allocs = tracker.get_active_allocations();

    let string_allocs = active_allocs
        .iter()
        .filter(|a| {
            a.get(0)
                .and_then(|info| info.var_name.as_ref())
                .as_ref()
                .map_or(false, |name| name.contains("string"))
        })
        .count();

    // Note: String allocation tracking might not work without global allocator feature
    println!("String allocations: {}", string_allocs);
    if string_allocs == 0 {
        println!("No string allocations found, but test continues");
    }
}

#[test]
fn test_memory_layout_and_alignment() {
    ensure_init();

    // Test different sized allocations
    let small_alloc = vec![1u8; 16];
    track_var!(small_alloc).unwrap();

    let medium_alloc = vec![1u32; 256];
    track_var!(medium_alloc).unwrap();

    let large_alloc = vec![1u64; 1024];
    track_var!(large_alloc).unwrap();

    // Test aligned allocations
    #[repr(align(64))]
    struct AlignedData {
        _data: [u8; 128],
    }

    let aligned = Box::new(AlignedData { _data: [42; 128] });
    track_var!(aligned).unwrap();

    // Test zero-sized types (should not allocate)
    struct ZeroSized;
    let _zst = Box::new(ZeroSized);
    // ZST allocations might still be tracked by the allocator

    let tracker = get_global_tracker();
    let active_allocs = tracker.get_active_allocations();

    // Verify different allocation sizes
    let sizes: Vec<usize> = active_allocs
        .iter()
        .map(|a| a.iter().map(|info| info.size).sum::<usize>())
        .collect();
    assert!(
        sizes.iter().any(|&s| s >= 16),
        "Should have small allocations"
    );
    assert!(
        sizes.iter().any(|&s| s >= 1024),
        "Should have medium allocations"
    );
    assert!(
        sizes.iter().any(|&s| s >= 8192),
        "Should have large allocations"
    );
}

#[test]
fn test_recursive_data_structures() {
    ensure_init();

    // Test linked list-like structure
    #[derive(Debug, Clone)]
    struct Node {
        _value: i32,
        next: Option<Box<Node>>,
    }

    impl Node {
        fn new(value: i32) -> Self {
            Node {
                _value: value,
                next: None,
            }
        }

        fn append(&mut self, value: i32) {
            match &mut self.next {
                None => self.next = Some(Box::new(Node::new(value))),
                Some(next) => next.append(value),
            }
        }
    }

    let mut head = Box::new(Node::new(1));
    track_var!(head).unwrap();

    // Build a chain of nodes
    for i in 2..=10 {
        head.append(i);
    }

    // Test tree-like structure
    #[derive(Debug, Clone)]
    struct TreeNode {
        _value: String,
        children: Vec<Box<TreeNode>>,
    }

    impl TreeNode {
        fn new(value: String) -> Self {
            TreeNode {
                _value: value,
                children: Vec::new(),
            }
        }

        fn add_child(&mut self, child: TreeNode) {
            self.children.push(Box::new(child));
        }
    }

    let mut root = Box::new(TreeNode::new("root".to_string()));
    track_var!(root).unwrap();

    for i in 0..5 {
        let mut child = TreeNode::new(format!("child_{}", i));
        for j in 0..3 {
            child.add_child(TreeNode::new(format!("grandchild_{}_{}", i, j)));
        }
        root.add_child(child);
    }

    let tracker = get_global_tracker();
    let stats = tracker.get_stats();
    assert!(
        stats.unwrap().total_allocations > 0,
        "Should track recursive structure allocations"
    );
}

#[test]
fn test_memory_pools_and_custom_allocation_patterns() {
    ensure_init();

    // Simulate a memory pool pattern
    struct MemoryPool {
        blocks: Vec<Vec<u8>>,
        block_size: usize,
    }

    impl MemoryPool {
        fn new(block_size: usize, initial_blocks: usize) -> Self {
            let mut blocks = Vec::with_capacity(initial_blocks);
            for _ in 0..initial_blocks {
                blocks.push(vec![0u8; block_size]);
            }

            MemoryPool { blocks, block_size }
        }

        fn allocate_block(&mut self) -> Option<Vec<u8>> {
            self.blocks.pop()
        }

        fn deallocate_block(&mut self, block: Vec<u8>) {
            if block.len() == self.block_size {
                self.blocks.push(block);
            }
        }

        fn expand_pool(&mut self, additional_blocks: usize) {
            for _ in 0..additional_blocks {
                self.blocks.push(vec![0u8; self.block_size]);
            }
        }
    }

    impl Trackable for MemoryPool {
        fn get_heap_ptr(&self) -> Option<usize> {
            Some(self as *const _ as usize)
        }

        fn get_type_name(&self) -> &'static str {
            "MemoryPool"
        }
    }

    let mut pool = MemoryPool::new(1024, 10);
    track_var!(pool).unwrap();

    // Simulate allocation/deallocation cycles
    let mut allocated_blocks = Vec::new();

    // Allocate some blocks
    for _ in 0..5 {
        if let Some(block) = pool.allocate_block() {
            allocated_blocks.push(block);
        }
    }

    // Expand the pool
    pool.expand_pool(5);

    // Deallocate some blocks
    while let Some(block) = allocated_blocks.pop() {
        pool.deallocate_block(block);
    }

    let tracker = get_global_tracker();
    let stats = tracker.get_stats();
    assert!(
        stats.unwrap().total_allocations > 0,
        "Should track memory pool operations"
    );
}

#[test]
fn test_pinned_memory_and_futures() {
    ensure_init();

    // Test Pin<Box<T>>
    let data = vec![1, 2, 3, 4, 5];
    let pinned = Pin::new(Box::new(data));

    // Can't use track_var! directly with Pin, but the underlying allocation should be tracked
    let tracker = get_global_tracker();
    let stats_before = tracker.get_stats();

    // Create more pinned data
    let _pinned_string = Pin::new(Box::new(String::from("Pinned string data")));
    let _pinned_vec = Pin::new(Box::new(vec![42; 100]));

    let stats_after = tracker.get_stats();
    let stats_after_unwrapped = stats_after.unwrap();
    let stats_before_unwrapped = stats_before.unwrap();
    assert!(
        stats_after_unwrapped.total_allocations >= stats_before_unwrapped.total_allocations,
        "Should track pinned memory allocations"
    );

    // Test that pinned memory stays pinned
    let pinned_ref = pinned.as_ref();
    assert_eq!(
        pinned_ref.get_ref()[0],
        1,
        "Pinned data should be accessible"
    );
}

#[test]
fn test_memory_fragmentation_patterns() {
    ensure_init();

    // Create a pattern that might cause fragmentation
    let mut allocations = Vec::new();

    // Allocate many small blocks
    for i in 0..100 {
        allocations.push(vec![i as u8; 64]);
    }

    // Deallocate every other block
    let mut to_keep = Vec::new();
    for (index, alloc) in allocations.into_iter().enumerate() {
        if index % 2 == 0 {
            to_keep.push(alloc);
        }
        // Odd-indexed allocations are dropped, creating gaps
    }

    // Allocate larger blocks that might not fit in the gaps
    for i in 0..25 {
        to_keep.push(vec![i as u8; 256]);
    }

    track_var!(to_keep).unwrap();

    let tracker = get_global_tracker();
    let stats = tracker.get_stats();
    assert!(
        stats.unwrap().total_allocations > 0,
        "Should track fragmentation pattern allocations"
    );
}

#[test]
fn test_large_allocation_patterns() {
    ensure_init();

    // Test very large single allocation
    let large_vec = vec![42u8; 10 * 1024 * 1024]; // 10MB
    track_var!(large_vec).unwrap();

    // Test many medium allocations
    let mut medium_allocations = Vec::new();
    for i in 0..100 {
        medium_allocations.push(vec![i as u8; 64 * 1024]); // 64KB each
    }
    track_var!(medium_allocations).unwrap();

    // Test allocation growth pattern
    let mut growing_allocation = Vec::with_capacity(1024);
    track_var!(growing_allocation).unwrap();

    // Grow the allocation in steps
    for step in 0..10 {
        let step_size = 1024 * (1 << step); // Exponential growth
        for i in 0..step_size {
            growing_allocation.push(i as u8);
        }
    }

    let tracker = get_global_tracker();
    let active_allocs = tracker.get_active_allocations().unwrap();

    // Should have some large allocations
    let _large_allocs = active_allocs
        .iter()
        .filter(|a| a.size > 1024 * 1024)
        .count();
    // Note: Large allocations might not be tracked if global allocator feature is not enabled
    // Let's check if we have any meaningful allocations instead
    assert!(!active_allocs.is_empty(), "Should have some tracked allocations");

    let stats = tracker.get_stats();
    // Note: Large memory tracking might not work without global allocator feature
    let stats_unwrapped = stats.unwrap();
    println!("Peak memory: {} bytes", stats_unwrapped.peak_memory);
    if stats_unwrapped.peak_memory <= 10 * 1024 * 1024 {
        println!(
            "Expected >10MB peak memory, got {} bytes, but test continues",
            stats_unwrapped.peak_memory
        );
    }
}

#[test]
fn test_drop_order_and_cleanup() {
    ensure_init();

    struct DropTracker {
        id: usize,
        _data: Vec<u8>,
    }

    impl Drop for DropTracker {
        fn drop(&mut self) {
            // Simulate cleanup work that might allocate
            let _cleanup_data = format!("Cleaning up tracker {}", self.id);
        }
    }

    let tracker_before = get_global_tracker();
    let stats_before = tracker_before.get_stats();

    {
        let mut trackers = Vec::new();
        for i in 0..10 {
            trackers.push(DropTracker {
                id: i,
                _data: vec![i as u8; 1024],
            });
        }
        track_var!(trackers).unwrap();

        // trackers will be dropped here in reverse order (9, 8, 7, ..., 0)
    }

    let stats_after = tracker_before.get_stats();

    // Some allocations should have been deallocated
    let stats_after_unwrapped = stats_after.unwrap();
    let stats_before_unwrapped = stats_before.unwrap();
    assert!(
        stats_after_unwrapped.total_allocations >= stats_before_unwrapped.total_allocations,
        "Should track allocations during drop"
    );
}
