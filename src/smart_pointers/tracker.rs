use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PointerType {
    Box,
    Rc,
    Arc,
    Weak,
    RefCell,
    Mutex,
    RwLock,
}

#[derive(Debug, Clone)]
pub struct PointerInfo {
    pub ptr_type: PointerType,
    pub allocation_size: usize,
    pub created_at: Instant,
    pub ref_count: Option<usize>,
    pub inner_type: String,
    pub allocation_id: u64,
}

pub struct SmartPointerTracker {
    active_pointers: HashMap<usize, PointerInfo>,
    allocation_counter: AtomicUsize,
    type_stats: HashMap<PointerType, TypeStats>,
}

#[derive(Debug, Default)]
pub struct TypeStats {
    pub total_count: usize,
    pub current_count: usize,
    pub total_size: usize,
    pub current_size: usize,
    pub max_concurrent: usize,
}

impl SmartPointerTracker {
    pub fn new() -> Self {
        Self {
            active_pointers: HashMap::new(),
            allocation_counter: AtomicUsize::new(0),
            type_stats: HashMap::new(),
        }
    }

    pub fn track_allocation(
        &mut self,
        ptr_addr: usize,
        ptr_type: PointerType,
        size: usize,
        inner_type: String,
        ref_count: Option<usize>,
    ) -> u64 {
        let allocation_id = self.allocation_counter.fetch_add(1, Ordering::Relaxed) as u64;

        let info = PointerInfo {
            ptr_type: ptr_type.clone(),
            allocation_size: size,
            created_at: Instant::now(),
            ref_count,
            inner_type,
            allocation_id,
        };

        self.active_pointers.insert(ptr_addr, info);
        self.update_type_stats(&ptr_type, size, true);

        allocation_id
    }

    pub fn track_deallocation(&mut self, ptr_addr: usize) -> Option<PointerInfo> {
        if let Some(info) = self.active_pointers.remove(&ptr_addr) {
            self.update_type_stats(&info.ptr_type, info.allocation_size, false);
            Some(info)
        } else {
            None
        }
    }

    pub fn update_ref_count(&mut self, ptr_addr: usize, new_count: usize) -> bool {
        if let Some(info) = self.active_pointers.get_mut(&ptr_addr) {
            info.ref_count = Some(new_count);
            true
        } else {
            false
        }
    }

    pub fn get_active_count(&self) -> usize {
        self.active_pointers.len()
    }

    pub fn get_active_by_type(&self, ptr_type: &PointerType) -> Vec<&PointerInfo> {
        self.active_pointers
            .values()
            .filter(|info| &info.ptr_type == ptr_type)
            .collect()
    }

    pub fn get_type_stats(&self, ptr_type: &PointerType) -> Option<&TypeStats> {
        self.type_stats.get(ptr_type)
    }

    pub fn get_all_type_stats(&self) -> &HashMap<PointerType, TypeStats> {
        &self.type_stats
    }

    pub fn get_memory_usage_by_type(&self) -> HashMap<PointerType, usize> {
        let mut usage = HashMap::new();

        for (ptr_type, stats) in &self.type_stats {
            usage.insert(ptr_type.clone(), stats.current_size);
        }

        usage
    }

    pub fn find_long_lived_pointers(&self, threshold_secs: u64) -> Vec<&PointerInfo> {
        let threshold = std::time::Duration::from_secs(threshold_secs);
        let now = Instant::now();

        self.active_pointers
            .values()
            .filter(|info| now.duration_since(info.created_at) > threshold)
            .collect()
    }

    pub fn clear(&mut self) {
        self.active_pointers.clear();
        self.type_stats.clear();
        self.allocation_counter.store(0, Ordering::Relaxed);
    }

    fn update_type_stats(&mut self, ptr_type: &PointerType, size: usize, is_allocation: bool) {
        let stats = self.type_stats.entry(ptr_type.clone()).or_default();

        if is_allocation {
            stats.total_count += 1;
            stats.current_count += 1;
            stats.total_size += size;
            stats.current_size += size;
            stats.max_concurrent = stats.max_concurrent.max(stats.current_count);
        } else {
            stats.current_count = stats.current_count.saturating_sub(1);
            stats.current_size = stats.current_size.saturating_sub(size);
        }
    }
}

impl Default for SmartPointerTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl PointerInfo {
    pub fn age(&self) -> std::time::Duration {
        Instant::now().duration_since(self.created_at)
    }

    pub fn age_secs(&self) -> f64 {
        self.age().as_secs_f64()
    }

    pub fn is_reference_counted(&self) -> bool {
        matches!(
            self.ptr_type,
            PointerType::Rc | PointerType::Arc | PointerType::Weak
        )
    }

    pub fn is_synchronized(&self) -> bool {
        matches!(
            self.ptr_type,
            PointerType::Mutex | PointerType::RwLock | PointerType::Arc
        )
    }
}

impl TypeStats {
    pub fn average_size(&self) -> f64 {
        if self.total_count > 0 {
            self.total_size as f64 / self.total_count as f64
        } else {
            0.0
        }
    }

    pub fn current_average_size(&self) -> f64 {
        if self.current_count > 0 {
            self.current_size as f64 / self.current_count as f64
        } else {
            0.0
        }
    }

    pub fn allocation_rate(&self, duration_secs: f64) -> f64 {
        if duration_secs > 0.0 {
            self.total_count as f64 / duration_secs
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tracking() {
        let mut tracker = SmartPointerTracker::new();

        let _id =
            tracker.track_allocation(0x1000, PointerType::Box, 64, "String".to_string(), None);

        assert_eq!(tracker.get_active_count(), 1);
        // ID is unsigned, always non-negative

        let info = tracker.track_deallocation(0x1000);
        assert!(info.is_some());
        assert_eq!(tracker.get_active_count(), 0);
    }

    #[test]
    fn test_type_statistics() {
        let mut tracker = SmartPointerTracker::new();

        tracker.track_allocation(0x1000, PointerType::Arc, 128, "Data".to_string(), Some(1));
        tracker.track_allocation(
            0x2000,
            PointerType::Arc,
            256,
            "Vec<u8>".to_string(),
            Some(1),
        );

        let stats = tracker
            .get_type_stats(&PointerType::Arc)
            .expect("Stats should exist");
        assert_eq!(stats.current_count, 2);
        assert_eq!(stats.current_size, 384);
        assert_eq!(stats.average_size(), 192.0);
    }

    #[test]
    fn test_ref_count_updates() {
        let mut tracker = SmartPointerTracker::new();

        tracker.track_allocation(0x1000, PointerType::Rc, 64, "String".to_string(), Some(1));
        assert!(tracker.update_ref_count(0x1000, 3));

        let active = tracker.get_active_by_type(&PointerType::Rc);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].ref_count, Some(3));
    }

    #[test]
    fn test_long_lived_detection() {
        let mut tracker = SmartPointerTracker::new();

        tracker.track_allocation(0x1000, PointerType::Box, 64, "String".to_string(), None);

        // Should not find any long-lived pointers immediately
        let long_lived = tracker.find_long_lived_pointers(1);
        assert_eq!(long_lived.len(), 0);
    }
}
