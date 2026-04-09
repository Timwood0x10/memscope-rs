//! Merkle Tree Implementation with unsafe code and memscope tracking
//!
//! This example demonstrates a Merkle Tree implementation using unsafe Rust
//! with memscope memory tracking and HTML dashboard export.

use memscope_rs::{global_tracker, init_global_tracking, track, MemScopeResult};
use std::fmt;

/// Hash type (256-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hash(pub [u8; 32]);

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl Hash {
    pub fn zero() -> Self {
        Hash([0u8; 32])
    }

    pub fn from_data(data: &[u8]) -> Self {
        let mut hash = [0u8; 32];
        unsafe {
            let data_ptr = data.as_ptr();
            let data_len = data.len();

            // Simple hash function (not cryptographically secure)
            for (i, byte) in hash.iter_mut().enumerate() {
                let mut byte_sum: u8 = 0;
                let mut multiplier = 1u8;

                for j in 0..data_len {
                    let byte_ptr = data_ptr.add(j);
                    let byte_val = *byte_ptr;
                    byte_sum = byte_sum.wrapping_add(byte_val.wrapping_mul(multiplier));
                    multiplier = multiplier.wrapping_mul(17);
                }

                *byte = byte_sum.wrapping_add(i as u8);
            }
        }
        Hash(hash)
    }
}

/// Merkle Tree node
#[derive(Debug, Clone)]
pub enum MerkleNode {
    Leaf(Hash, Vec<u8>),
    Branch(Hash, Box<MerkleNode>, Box<MerkleNode>),
}

impl MerkleNode {
    pub fn hash(&self) -> Hash {
        match self {
            MerkleNode::Leaf(hash, _) => *hash,
            MerkleNode::Branch(hash, _, _) => *hash,
        }
    }

    pub fn leaf(data: Vec<u8>) -> Self {
        let hash = Hash::from_data(&data);
        MerkleNode::Leaf(hash, data)
    }

    /// # Safety
    ///
    /// This function performs unsafe raw pointer operations to manually combine hash values.
    ///
    /// # Safety Requirements
    ///
    /// - Both `left` and `right` must be valid MerkleNode instances
    /// - The hash arrays in both nodes must be properly initialized
    /// - Memory access patterns must be within bounds
    /// - The function must only be called when direct memory manipulation is required
    pub unsafe fn branch_unsafe(left: MerkleNode, right: MerkleNode) -> Self {
        let left_hash = left.hash();
        let right_hash = right.hash();

        // Manual memory manipulation for hash combination
        let mut combined = [0u8; 64];
        let combined_ptr = combined.as_mut_ptr();
        let left_ptr = left_hash.0.as_ptr();
        let right_ptr = right_hash.0.as_ptr();

        // Copy left hash
        std::ptr::copy_nonoverlapping(left_ptr, combined_ptr, 32);
        // Copy right hash
        std::ptr::copy_nonoverlapping(right_ptr, combined_ptr.add(32), 32);

        // Compute combined hash using raw pointers
        let mut result = [0u8; 32];
        let result_ptr = result.as_mut_ptr();

        for i in 0..32 {
            let combined_byte = *combined_ptr.add(i);
            let combined_byte2 = *combined_ptr.add(i + 32);
            *result_ptr.add(i) = combined_byte.wrapping_add(combined_byte2);
        }

        MerkleNode::Branch(Hash(result), Box::new(left), Box::new(right))
    }

    pub fn branch(left: MerkleNode, right: MerkleNode) -> Self {
        let left_hash = left.hash();
        let right_hash = right.hash();

        let mut combined = [0u8; 64];
        combined[..32].copy_from_slice(&left_hash.0);
        combined[32..(32 + 32)].copy_from_slice(&right_hash.0);

        let hash = Hash::from_data(&combined);
        MerkleNode::Branch(hash, Box::new(left), Box::new(right))
    }
}

/// Merkle Tree
#[derive(Debug)]
pub struct MerkleTree {
    root: Option<MerkleNode>,
    leaves: Vec<Vec<u8>>,
}

impl Default for MerkleTree {
    fn default() -> Self {
        Self::new()
    }
}

impl MerkleTree {
    pub fn new() -> Self {
        MerkleTree {
            root: None,
            leaves: Vec::new(),
        }
    }

    /// # Safety
    ///
    /// This function performs unsafe raw pointer operations to add leaf data to the tree.
    ///
    /// # Safety Requirements
    ///
    /// - `data` must be a valid Vec<u8> with properly allocated memory
    /// - The data pointer must remain valid for the duration of the function call
    /// - The function must only be called when direct memory manipulation is required
    /// - The caller must ensure the data is properly owned and not used elsewhere
    pub unsafe fn add_leaf_unsafe(&mut self, data: Vec<u8>) {
        // Use raw pointer to manipulate leaves vector
        let data_ptr = data.as_ptr();
        let data_len = data.len();

        // Verify data validity through raw pointer access
        if data_len > 0 && !data_ptr.is_null() {
            // Verify first byte
            let first_byte = *data_ptr;
            if first_byte != 0 || data_len == 1 {
                // Add the data
                self.leaves.push(data);
                self.rebuild_tree_unsafe();
            }
        }
    }

    pub fn add_leaf(&mut self, data: Vec<u8>) {
        self.leaves.push(data);
        self.rebuild_tree();
    }

    /// # Safety
    ///
    /// This function performs unsafe raw pointer operations to rebuild the Merkle tree.
    ///
    /// # Safety Requirements
    ///
    /// - `self.leaves` must be a valid vector with properly allocated memory
    /// - All leaf data must remain valid and not be modified during tree construction
    /// - Raw pointer access to `current_level` must stay within bounds
    /// - The function must only be called when direct memory manipulation is required
    /// - The tree structure must be rebuildable from the current leaf data
    pub unsafe fn rebuild_tree_unsafe(&mut self) {
        if self.leaves.is_empty() {
            self.root = None;
            return;
        }

        // Build tree level by level using unsafe operations
        let mut current_level: Vec<MerkleNode> = self
            .leaves
            .iter()
            .map(|data| MerkleNode::leaf(data.clone()))
            .collect();

        while current_level.len() > 1 {
            let mut next_level = Vec::new();

            // Process pairs using raw pointer access
            let level_ptr = current_level.as_ptr();
            let level_len = current_level.len();

            let mut i = 0;
            while i < level_len {
                if i + 1 < level_len {
                    // Read nodes through raw pointers
                    let left_node = &*level_ptr.add(i);
                    let right_node = &*level_ptr.add(i + 1);

                    // Create branch using unsafe method
                    let branch =
                        MerkleNode::branch_unsafe((*left_node).clone(), (*right_node).clone());
                    next_level.push(branch);
                } else {
                    // Odd number of nodes, carry over
                    let node = &*level_ptr.add(i);
                    next_level.push((*node).clone());
                }
                i += 2;
            }

            // Replace current level
            current_level = next_level;
        }

        // Set root
        if !current_level.is_empty() {
            let root_node = current_level.into_iter().next().unwrap();
            self.root = Some(root_node);
        }
    }

    pub fn rebuild_tree(&mut self) {
        if self.leaves.is_empty() {
            self.root = None;
            return;
        }

        let mut current_level: Vec<MerkleNode> = self
            .leaves
            .iter()
            .map(|data| MerkleNode::leaf(data.clone()))
            .collect();

        while current_level.len() > 1 {
            let mut next_level = Vec::new();

            for chunk in current_level.chunks(2) {
                if chunk.len() == 2 {
                    let branch = MerkleNode::branch(chunk[0].clone(), chunk[1].clone());
                    next_level.push(branch);
                } else {
                    next_level.push(chunk[0].clone());
                }
            }

            current_level = next_level;
        }

        self.root = current_level.into_iter().next();
    }

    pub fn root_hash(&self) -> Option<Hash> {
        self.root.as_ref().map(|node| node.hash())
    }

    pub fn leaf_count(&self) -> usize {
        self.leaves.len()
    }

    /// # Safety
    ///
    /// This function performs unsafe raw pointer operations to verify leaf hash.
    ///
    /// # Safety Requirements
    ///
    /// - `index` must be within bounds of `self.leaves` (0 <= index < leaves.len())
    /// - `self.leaves` must be a valid vector with properly allocated memory
    /// - The leaf data at the specified index must remain valid during verification
    /// - Raw pointer access to leaves and hash arrays must stay within bounds
    /// - The function must only be called when direct memory manipulation is required
    pub unsafe fn verify_leaf_unsafe(&self, index: usize, expected_hash: Hash) -> bool {
        if index >= self.leaves.len() {
            return false;
        }

        // Use raw pointer to access leaf data
        let leaves_ptr = self.leaves.as_ptr();
        let leaf_ptr = &*leaves_ptr.add(index);

        let computed_hash = Hash::from_data(leaf_ptr);

        // Compare hashes byte by byte using raw pointers
        let expected_ptr = expected_hash.0.as_ptr();
        let computed_ptr = computed_hash.0.as_ptr();

        for i in 0..32 {
            if *expected_ptr.add(i) != *computed_ptr.add(i) {
                return false;
            }
        }

        true
    }

    pub fn verify_leaf(&self, index: usize, expected_hash: Hash) -> bool {
        if index >= self.leaves.len() {
            return false;
        }

        let computed_hash = Hash::from_data(&self.leaves[index]);
        computed_hash == expected_hash
    }
}

unsafe fn create_large_buffer(size: usize) -> *mut u8 {
    let layout = std::alloc::Layout::from_size_align(size, 8).unwrap();
    let ptr = std::alloc::alloc(layout);
    if ptr.is_null() {
        panic!("Memory allocation failed");
    }

    // Initialize buffer with pattern
    for i in 0..size {
        *ptr.add(i) = (i % 256) as u8;
    }

    // Track unsafe allocation using global tracker
    let tracker = global_tracker().expect("Failed to get tracker");
    let ptr_addr = ptr as usize;

    // First create a passport for this allocation
    let _ = tracker.create_passport(ptr_addr, size, "unsafe_alloc".to_string());

    // Then record the handover to FFI event
    tracker.record_handover(
        ptr_addr,
        "unsafe_alloc".to_string(),
        "create_large_buffer".to_string(),
    );

    ptr
}

unsafe fn free_large_buffer(ptr: *mut u8, size: usize) {
    let layout = std::alloc::Layout::from_size_align(size, 8).unwrap();

    let tracker = global_tracker().expect("Failed to get tracker");
    let ptr_addr = ptr as usize;

    // Track unsafe deallocation with passport
    let _ = tracker.create_passport(ptr_addr, size, "unsafe_dealloc".to_string());
    tracker.record_free(
        ptr_addr,
        "unsafe_dealloc".to_string(),
        "free_large_buffer".to_string(),
    );

    std::alloc::dealloc(ptr, layout);
}

fn main() -> MemScopeResult<()> {
    println!("🌳 Merkle Tree Example with Unsafe Code & Memscope Tracking");
    println!("=========================================================\n");

    // Initialize memscope global tracking
    println!("🚀 Initializing memscope tracking...");
    init_global_tracking()?;
    println!("✓ Memscope tracking initialized\n");

    let tracker = global_tracker()?;
    println!("✓ Tracker initialized\n");

    // Create a new Merkle tree
    let mut tree = MerkleTree::new();

    println!("📝 Adding leaves to the tree...\n");

    // Add some leaves with tracking
    let leaf1 = b"Transaction 1".to_vec();
    let leaf2 = b"Transaction 2".to_vec();
    let leaf3 = b"Transaction 3".to_vec();
    let leaf4 = b"Transaction 4".to_vec();

    // Track the leaf data vectors
    track!(tracker, leaf1);
    track!(tracker, leaf2);
    track!(tracker, leaf3);
    track!(tracker, leaf4);

    tree.add_leaf(leaf1.clone());
    tree.add_leaf(leaf2.clone());
    tree.add_leaf(leaf3.clone());
    tree.add_leaf(leaf4.clone());

    println!("✓ Added {} leaves", tree.leaf_count());
    println!("🔐 Root hash: {}", tree.root_hash().unwrap());
    println!();

    // Test unsafe operations
    println!("🔧 Testing unsafe operations...\n");

    let mut unsafe_tree = MerkleTree::new();

    unsafe {
        // Create large buffer with unsafe allocation
        let large_size = 1024 * 1024; // 1MB
        let large_buffer = create_large_buffer(large_size);
        println!("✓ Allocated {} bytes using unsafe", large_size);

        // Add data from unsafe buffer
        let mut data = Vec::with_capacity(large_size);
        for i in 0..large_size {
            data.push(*large_buffer.add(i));
        }
        unsafe_tree.add_leaf_unsafe(data);

        // Add more leaves using unsafe method
        unsafe_tree.add_leaf_unsafe(b"Unsafe Transaction 1".to_vec());
        unsafe_tree.add_leaf_unsafe(b"Unsafe Transaction 2".to_vec());

        println!("✓ Added {} leaves using unsafe", unsafe_tree.leaf_count());

        // Verify leaf using unsafe method
        let test_hash = unsafe_tree.root_hash().unwrap();
        let verified = unsafe_tree.verify_leaf_unsafe(0, test_hash);
        println!("✓ Leaf verification (unsafe): {}", verified);

        // Free the buffer
        free_large_buffer(large_buffer, large_size);
        println!("✓ Freed unsafe memory");
    }

    println!();
    println!(
        "🔐 Unsafe tree root hash: {}",
        unsafe_tree.root_hash().unwrap()
    );

    // Create more allocations to test memory tracking
    println!("\n📊 Creating additional allocations for testing...\n");

    let mut large_vectors = Vec::new();
    for i in 0..100 {
        let vec: Vec<u8> = (0..1000).map(|j| ((i + j) % 256) as u8).collect();
        track!(tracker, vec); // Track each large vector
        large_vectors.push(vec);
    }

    println!("✓ Created {} large vectors", large_vectors.len());
    println!("✓ Total memory: ~{} KB", large_vectors.len() * 1000 / 1024);

    // Create some more complex data structures
    println!("\n🏗️  Building complex data structures...\n");

    let mut matrix = Vec::new();
    for i in 0..50 {
        let mut row = Vec::new();
        for j in 0..20 {
            let mut inner = Vec::new();
            for k in 0..10 {
                inner.push((i * j * k) as u32);
            }
            track!(tracker, inner); // Track inner vectors
            row.push(inner);
        }
        track!(tracker, row); // Track row vectors
        matrix.push(row);
    }

    println!(
        "✓ Created matrix: {}x{}x{}",
        matrix.len(),
        matrix[0].len(),
        matrix[0][0].len()
    );

    // Simulate some work with the data
    println!("\n⚙️  Processing data...\n");

    let mut results = Vec::new();
    for i in 0..10 {
        let result: Vec<f64> = (0..1000).map(|j| (i as f64 + j as f64).sin()).collect();
        track!(tracker, result); // Track result vectors
        results.push(result);
    }

    println!("✓ Processed {} result sets", results.len());

    println!("\n✅ Merkle Tree example completed!");
    println!("📊 Memory allocations:");
    println!("  - Merkle tree nodes: ~{}", tree.leaf_count() * 2);
    println!("  - Large vectors: {}", large_vectors.len());
    println!(
        "  - Matrix cells: {}",
        matrix.len() * matrix[0].len() * matrix[0][0].len()
    );
    println!(
        "  - Results: {} vectors with 1000 elements each",
        results.len()
    );

    // Get tracking statistics
    let stats = tracker.get_stats();
    println!("\n📈 Memscope Analysis Results:");
    println!("  Total allocations: {}", stats.total_allocations);
    println!("  Active allocations: {}", stats.active_allocations);
    println!(
        "  Peak memory: {} bytes ({:.2} MB)",
        stats.peak_memory_bytes,
        stats.peak_memory_bytes as f64 / 1024.0 / 1024.0
    );

    // Get passport statistics
    println!("\n🔧 Passport Tracking Results:");
    println!("  Total passports created: {}", stats.passport_count);
    println!("  Active passports: {}", stats.active_passports);
    println!("  Leaks detected: {}", stats.leaks_detected);

    // Export HTML dashboard
    println!("\n🎨 Exporting HTML dashboard...");
    let output_path = "MemoryAnalysis/merkle_tree_with_tracking";
    tracker.export_json(output_path)?;
    tracker.export_html(output_path)?;

    println!("✓ Export successful!");
    println!("📁 Results saved to: {}", output_path);
    println!(
        "📄 Open {}/dashboard.html for interactive analysis",
        output_path
    );

    Ok(())
}
