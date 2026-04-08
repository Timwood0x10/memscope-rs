//! Memory View - Safe Memory Access Layer
//!
//! Provides safe, bounds-checked access to memory content for type inference.
//! All memory access goes through this layer - no raw pointer dereferencing.
//!
//! # ValidRegions
//!
//! Uses dynamic mmap region detection with static fallback:
//! - Linux: Reads `/proc/self/maps` for precise regions
//! - Other platforms: Uses static address range bounds

use std::sync::RwLock;

// Static fallback bounds for different platforms
#[cfg(all(target_pointer_width = "64", target_os = "linux"))]
const MAX_USER_ADDR: usize = 0x0000_7fff_ffff_f000;

#[cfg(all(target_pointer_width = "64", target_os = "macos"))]
const MAX_USER_ADDR: usize = 0x0000_7fff_ffff_f000;

#[cfg(all(target_pointer_width = "64", target_os = "windows"))]
const MAX_USER_ADDR: usize = 0x0000_7fff_ffff_0000;

#[cfg(all(
    target_pointer_width = "64",
    not(any(target_os = "linux", target_os = "macos", target_os = "windows"))
))]
const MAX_USER_ADDR: usize = 0x0000_7fff_ffff_ffff;

#[cfg(target_pointer_width = "32")]
const MAX_USER_ADDR: usize = 0x7fff_ffff;

const MIN_VALID_ADDR: usize = 0x1000;

/// Represents a valid memory region from process memory map.
#[derive(Clone, Debug)]
pub struct MemoryRegion {
    pub start: usize,
    pub end: usize,
}

/// Valid memory regions for pointer validation.
///
/// Uses dynamic detection on supported platforms with static fallback.
#[derive(Clone, Debug, Default)]
pub struct ValidRegions {
    regions: Vec<MemoryRegion>,
    is_dynamic: bool,
}

impl ValidRegions {
    /// Create empty regions (will use static bounds).
    pub fn empty() -> Self {
        Self {
            regions: Vec::new(),
            is_dynamic: false,
        }
    }

    /// Create from a list of memory regions.
    pub fn from_regions(regions: Vec<MemoryRegion>) -> Self {
        Self {
            regions,
            is_dynamic: true,
        }
    }

    /// Check if an address falls within valid regions.
    ///
    /// If dynamic regions are available, uses precise checking.
    /// Otherwise, falls back to static bounds.
    pub fn contains(&self, addr: usize) -> bool {
        if addr <= MIN_VALID_ADDR {
            return false;
        }

        if self.is_dynamic && !self.regions.is_empty() {
            // Use partition_point to find the first region where start > addr
            // Then check if the previous region contains addr
            let idx = self.regions.partition_point(|region| region.start <= addr);

            if idx > 0 {
                let region = &self.regions[idx - 1];
                return addr < region.end;
            }
            false
        } else {
            // Static fallback
            addr < MAX_USER_ADDR
        }
    }

    /// Get the number of regions.
    pub fn len(&self) -> usize {
        self.regions.len()
    }

    /// Check if regions are empty.
    pub fn is_empty(&self) -> bool {
        self.regions.is_empty()
    }

    /// Check if using dynamic detection.
    pub fn is_dynamic(&self) -> bool {
        self.is_dynamic
    }
}

/// Global cached valid regions.
static VALID_REGIONS: RwLock<Option<ValidRegions>> = RwLock::new(None);

/// Merge overlapping or adjacent memory regions.
#[cfg(target_os = "linux")]
fn merge_regions(regions: Vec<MemoryRegion>) -> Vec<MemoryRegion> {
    if regions.is_empty() {
        return regions;
    }

    let mut merged: Vec<MemoryRegion> = Vec::with_capacity(regions.len());
    let mut current = regions[0].clone();

    for region in regions.into_iter().skip(1) {
        if region.start < current.end {
            current.end = current.end.max(region.end);
        } else {
            merged.push(current);
            current = region;
        }
    }
    merged.push(current);

    merged
}

/// Get valid memory regions for the current process.
///
/// Platform-specific implementation:
/// - Linux: Reads `/proc/self/maps`
/// - Other: Returns empty (uses static bounds)
#[cfg(target_os = "linux")]
fn get_valid_regions_impl() -> ValidRegions {
    use std::fs;

    let content = match fs::read_to_string("/proc/self/maps") {
        Ok(c) => c,
        Err(_) => return ValidRegions::empty(),
    };

    let mut regions: Vec<MemoryRegion> = content
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                return None;
            }

            let range: Vec<&str> = parts[0].split('-').collect();
            if range.len() != 2 {
                return None;
            }

            let start = usize::from_str_radix(range[0], 16).ok()?;
            let end = usize::from_str_radix(range[1], 16).ok()?;

            // Filter to readable regions only (r-- or r-x or rw-)
            if parts.len() < 2 {
                return None;
            }
            let perms = parts[1];
            if !perms.starts_with('r') {
                return None;
            }

            Some(MemoryRegion { start, end })
        })
        .collect();

    // Sort by start address for binary search
    regions.sort_by_key(|r| r.start);

    // Merge overlapping/adjacent regions
    regions = merge_regions(regions);

    ValidRegions::from_regions(regions)
}

/// Get valid memory regions for the current process (non-Linux).
#[cfg(not(target_os = "linux"))]
fn get_valid_regions_impl() -> ValidRegions {
    ValidRegions::empty()
}

/// Get cached valid regions, initializing if needed.
pub fn get_valid_regions() -> ValidRegions {
    // Fast path: check if already initialized
    {
        let read_guard = match VALID_REGIONS.read() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        if read_guard.is_some() {
            return read_guard
                .as_ref()
                .cloned()
                .expect("VALID_REGIONS should be Some after is_some() check (read path)");
        }
    }

    // Slow path: need to initialize
    // Use write lock and double-check to prevent TOCTOU race
    let mut write_guard = match VALID_REGIONS.write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    // Double-check after acquiring write lock
    if write_guard.is_some() {
        return write_guard
            .as_ref()
            .cloned()
            .expect("VALID_REGIONS should be Some after is_some() check (write path)");
    }

    // Initialize while holding the write lock
    let regions = get_valid_regions_impl();
    *write_guard = Some(regions.clone());
    regions
}

/// Check if a pointer value is valid using dynamic regions with static fallback.
pub fn is_valid_ptr(p: usize) -> bool {
    get_valid_regions().contains(p)
}

/// Check if a pointer value is valid using only static bounds.
pub fn is_valid_ptr_static(p: usize) -> bool {
    p > MIN_VALID_ADDR && p < MAX_USER_ADDR
}

/// Memory view for safe memory access.
pub struct MemoryView<'a> {
    data: &'a [u8],
}

/// Owned memory view that owns its data.
///
/// This is a non-reference version of `MemoryView` that owns the underlying
/// buffer. Useful when the memory view needs to outlive the original scope.
///
/// # When to Use OwnedMemoryView vs MemoryView
///
/// | Scenario | Use |
/// |----------|-----|
/// | Temporary analysis within a function | `MemoryView<&[u8]>` |
/// | Storing memory data for later use | `OwnedMemoryView` |
/// | Returning memory data from a function | `OwnedMemoryView` |
/// | Zero-copy analysis | `MemoryView<&[u8]>` |
///
/// # Lifetime Management
///
/// `OwnedMemoryView` owns its data via `Vec<u8>`, so it has no lifetime parameter.
/// This means:
/// - The data remains valid as long as the `OwnedMemoryView` exists
/// - No need to worry about the underlying data being dropped
/// - Slightly higher memory overhead due to ownership
///
/// # Example
///
/// ```rust
/// use memscope_rs::analysis::unsafe_inference::OwnedMemoryView;
///
/// // Create from a vector (takes ownership)
/// let data = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
/// let view = OwnedMemoryView::new(data);
///
/// // Read values safely
/// if let Some(value) = view.read_usize(0) {
///     println!("First usize: {}", value);
/// }
///
/// // Check bounds
/// if let Some(byte) = view.read_u8(10) {
///     println!("Byte at offset 10: {}", byte);
/// } else {
///     println!("Offset 10 out of bounds");
/// }
///
/// // Access raw slice when needed
/// let slice = view.as_slice();
/// println!("Total bytes: {}", slice.len());
/// ```
///
/// # Memory Safety
///
/// All read methods perform bounds checking and return `Option` types.
/// This ensures safe access even with invalid offsets:
///
/// ```rust
/// let view = OwnedMemoryView::new(vec![0u8; 4]);
///
/// // This returns None (out of bounds)
/// assert!(view.read_usize(0).is_none());
///
/// // This returns None (offset + size > len)
/// assert!(view.read_usize(1).is_none());
/// ```
pub struct OwnedMemoryView {
    data: Vec<u8>,
}

impl OwnedMemoryView {
    /// Create a new `OwnedMemoryView` from a `Vec<u8>`.
    ///
    /// This takes ownership of the vector, so no copying occurs.
    ///
    /// # Example
    ///
    /// ```rust
    /// let view = OwnedMemoryView::new(vec![1, 2, 3, 4]);
    /// assert_eq!(view.len(), 4);
    /// ```
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Returns the length of the underlying data.
    ///
    /// # Example
    ///
    /// ```rust
    /// let view = OwnedMemoryView::new(vec![1, 2, 3]);
    /// assert_eq!(view.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the underlying data is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// let view = OwnedMemoryView::new(vec![]);
    /// assert!(view.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Read a `usize` value from the specified offset.
    ///
    /// Reads `std::mem::size_of::<usize>()` bytes starting at `offset`
    /// and interprets them as a little-endian `usize`.
    ///
    /// Returns `None` if the read would exceed the buffer bounds.
    ///
    /// # Example
    ///
    /// ```rust
    /// let data = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
    /// let view = OwnedMemoryView::new(data);
    ///
    /// if let Some(value) = view.read_usize(0) {
    ///     println!("Value: 0x{:x}", value);
    /// }
    /// ```
    pub fn read_usize(&self, offset: usize) -> Option<usize> {
        let size = std::mem::size_of::<usize>();
        if offset.saturating_add(size) > self.data.len() {
            return None;
        }
        let mut buf = [0u8; 8];
        buf[..size].copy_from_slice(&self.data[offset..offset + size]);
        Some(usize::from_le_bytes(buf))
    }

    /// Read a single byte from the specified offset.
    ///
    /// Returns `None` if the offset is out of bounds.
    ///
    /// # Example
    ///
    /// ```rust
    /// let view = OwnedMemoryView::new(vec![0x10, 0x20, 0x30]);
    ///
    /// assert_eq!(view.read_u8(0), Some(0x10));
    /// assert_eq!(view.read_u8(2), Some(0x30));
    /// assert_eq!(view.read_u8(3), None); // out of bounds
    /// ```
    pub fn read_u8(&self, offset: usize) -> Option<u8> {
        self.data.get(offset).copied()
    }

    /// Returns a slice of the underlying data.
    ///
    /// This provides direct access to the bytes without copying.
    ///
    /// # Example
    ///
    /// ```rust
    /// let view = OwnedMemoryView::new(vec![1, 2, 3, 4, 5]);
    /// let slice = view.as_slice();
    /// assert_eq!(slice, &[1, 2, 3, 4, 5]);
    /// ```
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Returns an iterator over chunks of the underlying data.
    ///
    /// Each chunk has at most `chunk_size` elements.
    ///
    /// # Example
    ///
    /// ```rust
    /// let view = OwnedMemoryView::new(vec![1, 2, 3, 4, 5, 6]);
    /// let chunks: Vec<_> = view.chunks(2).collect();
    /// assert_eq!(chunks, vec![&[1, 2][..], &[3, 4], &[5, 6]]);
    /// ```
    pub fn chunks(&self, chunk_size: usize) -> impl Iterator<Item = &[u8]> {
        self.data.chunks(chunk_size)
    }
}

impl<'a> MemoryView<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn read_usize(&self, offset: usize) -> Option<usize> {
        let size = std::mem::size_of::<usize>();
        if offset.saturating_add(size) > self.data.len() {
            return None;
        }
        let mut buf = [0u8; 8];
        buf[..size].copy_from_slice(&self.data[offset..offset + size]);
        Some(usize::from_le_bytes(buf))
    }

    pub fn read_u8(&self, offset: usize) -> Option<u8> {
        self.data.get(offset).copied()
    }

    pub fn last_byte(&self) -> Option<u8> {
        self.data.last().copied()
    }

    pub fn as_slice(&self) -> &'a [u8] {
        self.data
    }

    pub fn chunks(&self, chunk_size: usize) -> impl Iterator<Item = &'a [u8]> {
        self.data.chunks(chunk_size)
    }
}

/// Count valid pointers in a memory view.
pub fn count_valid_pointers(view: &MemoryView) -> usize {
    let ptr_size = std::mem::size_of::<usize>();
    let mut count = 0;
    for chunk in view.chunks(ptr_size) {
        if chunk.len() < ptr_size {
            break;
        }
        // Use a buffer sized for the platform's pointer size
        let mut buf = [0u8; 16]; // Max pointer size is 16 bytes (128-bit)
        buf[..ptr_size].copy_from_slice(chunk);
        let v = if ptr_size == 8 {
            usize::from_le_bytes(buf[..8].try_into().unwrap())
        } else {
            usize::from_le_bytes({
                let mut arr = [0u8; 8];
                arr[..ptr_size].copy_from_slice(&buf[..ptr_size]);
                arr
            })
        };
        if is_valid_ptr(v) {
            count += 1;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_view_read_usize() {
        let data: [u8; 16] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f,
        ];
        let view = MemoryView::new(&data);

        let val0 = view.read_usize(0).unwrap();
        let val8 = view.read_usize(8).unwrap();

        assert_eq!(val0, 0x0706050403020100);
        assert_eq!(val8, 0x0f0e0d0c0b0a0908);
    }

    #[test]
    fn test_memory_view_bounds_check() {
        let data = [0u8; 8];
        let view = MemoryView::new(&data);

        assert!(view.read_usize(0).is_some());
        assert!(view.read_usize(1).is_none());
        assert!(view.read_usize(8).is_none());
    }

    #[test]
    fn test_is_valid_ptr_static() {
        assert!(!is_valid_ptr_static(0));
        assert!(!is_valid_ptr_static(0x1000));
        assert!(is_valid_ptr_static(0x10000));
        assert!(is_valid_ptr_static(0x7fff_ffff_0000));
        assert!(!is_valid_ptr_static(0xffff_ffff_ffff_ffff));
    }

    #[test]
    fn test_is_valid_ptr() {
        // Should work with either dynamic or static
        assert!(!is_valid_ptr(0));
        assert!(!is_valid_ptr(0x1000));
        // These should pass with static fallback
        assert!(is_valid_ptr(0x10000));
    }

    #[test]
    fn test_count_valid_pointers() {
        let mut data = [0u8; 24];
        let valid_ptr: usize = 0x10000;
        data[..8].copy_from_slice(&valid_ptr.to_le_bytes());

        let view = MemoryView::new(&data);
        assert_eq!(count_valid_pointers(&view), 1);
    }

    #[test]
    fn test_valid_regions_contains() {
        let regions = ValidRegions::empty();
        // Empty regions should use static bounds
        assert!(regions.contains(0x10000));
        assert!(!regions.contains(0));
    }

    #[test]
    fn test_valid_regions_from_regions() {
        let regions = ValidRegions::from_regions(vec![
            MemoryRegion {
                start: 0x1000,
                end: 0x2000,
            },
            MemoryRegion {
                start: 0x3000,
                end: 0x4000,
            },
        ]);

        assert!(regions.is_dynamic());
        assert!(regions.contains(0x1500));
        assert!(regions.contains(0x3500));
        assert!(!regions.contains(0x2500));
        assert!(!regions.contains(0x5000));
    }

    #[test]
    fn test_get_valid_regions() {
        let regions = get_valid_regions();
        // Should return something (dynamic or static)
        // Just verify it doesn't panic
        let _ = regions.contains(0x10000);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux_proc_maps_parsing() {
        // On Linux, should successfully parse /proc/self/maps
        let regions = get_valid_regions_impl();
        // Should have at least some regions (code, stack, heap, etc.)
        // The exact number depends on the process state
        if regions.is_dynamic() {
            assert!(!regions.is_empty());
        }
    }
}
