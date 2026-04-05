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

/// Merge overlapping or adjacent memory regions.
fn merge_regions(regions: Vec<MemoryRegion>) -> Vec<MemoryRegion> {
    if regions.is_empty() {
        return regions;
    }

    let mut merged = Vec::with_capacity(regions.len());
    let mut current = regions[0].clone();

    for region in regions.into_iter().skip(1) {
        // Check if regions overlap or are adjacent
        if region.start <= current.end {
            // Merge: extend current region
            current.end = current.end.max(region.end);
        } else {
            // No overlap: push current and start new
            merged.push(current);
            current = region;
        }
    }
    merged.push(current);

    merged
}

/// Get cached valid regions, initializing if needed.
pub fn get_valid_regions() -> ValidRegions {
    {
        let read_guard = VALID_REGIONS.read().unwrap();
        if read_guard.is_some() {
            return read_guard.as_ref().unwrap().clone();
        }
    }

    // Need to initialize
    let regions = get_valid_regions_impl();
    {
        let mut write_guard = VALID_REGIONS.write().unwrap();
        *write_guard = Some(regions.clone());
    }
    regions
}

/// Refresh valid regions (call after significant memory changes).
pub fn refresh_valid_regions() {
    let mut write_guard = VALID_REGIONS.write().unwrap();
    *write_guard = None;
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
    let mut count = 0;
    for chunk in view.chunks(8) {
        if chunk.len() < 8 {
            break;
        }
        let mut buf = [0u8; 8];
        buf.copy_from_slice(chunk);
        let v = usize::from_le_bytes(buf);
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
    fn test_merge_regions() {
        let regions = vec![
            MemoryRegion {
                start: 0x1000,
                end: 0x2000,
            },
            MemoryRegion {
                start: 0x1800,
                end: 0x3000,
            }, // Overlapping
            MemoryRegion {
                start: 0x4000,
                end: 0x5000,
            },
            MemoryRegion {
                start: 0x5000,
                end: 0x6000,
            }, // Adjacent
        ];

        let merged = merge_regions(regions);

        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].start, 0x1000);
        assert_eq!(merged[0].end, 0x3000);
        assert_eq!(merged[1].start, 0x4000);
        assert_eq!(merged[1].end, 0x6000);
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
