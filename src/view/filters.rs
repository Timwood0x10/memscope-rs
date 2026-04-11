//! View filters for data filtering.

use crate::snapshot::ActiveAllocation;
use std::collections::HashSet;

use super::MemoryView;

/// View filter types.
#[derive(Debug, Clone)]
pub enum ViewFilter {
    /// Filter by thread ID
    ByThread(u64),
    /// Filter by type name (contains)
    ByType(String),
    /// Filter by size range [min, max]
    BySizeRange(usize, usize),
    /// Filter by address range [min, max]
    ByAddressRange(u64, u64),
    /// Filter by minimum size
    ByMinSize(usize),
    /// Filter by maximum size
    ByMaxSize(usize),
}

/// Filter builder for MemoryView.
pub struct FilterBuilder<'a> {
    view: &'a MemoryView,
    filters: Vec<ViewFilter>,
}

impl<'a> FilterBuilder<'a> {
    /// Create new filter builder.
    pub fn new(view: &'a MemoryView) -> Self {
        Self {
            view,
            filters: Vec::new(),
        }
    }

    /// Filter by thread ID.
    pub fn by_thread(mut self, thread_id: u64) -> Self {
        self.filters.push(ViewFilter::ByThread(thread_id));
        self
    }

    /// Filter by type name (contains match).
    pub fn by_type(mut self, type_name: &str) -> Self {
        self.filters.push(ViewFilter::ByType(type_name.to_string()));
        self
    }

    /// Filter by size range [min, max].
    pub fn by_size_range(mut self, min: usize, max: usize) -> Self {
        self.filters.push(ViewFilter::BySizeRange(min, max));
        self
    }

    /// Filter by address range [min, max].
    pub fn by_address_range(mut self, min: u64, max: u64) -> Self {
        self.filters.push(ViewFilter::ByAddressRange(min, max));
        self
    }

    /// Filter by minimum size.
    pub fn by_min_size(mut self, min: usize) -> Self {
        self.filters.push(ViewFilter::ByMinSize(min));
        self
    }

    /// Filter by maximum size.
    pub fn by_max_size(mut self, max: usize) -> Self {
        self.filters.push(ViewFilter::ByMaxSize(max));
        self
    }

    /// Push a filter to the filter chain.
    pub fn push(mut self, filter: ViewFilter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Apply filters and return filtered allocations.
    pub fn apply(&self) -> Vec<&ActiveAllocation> {
        let mut result: Vec<_> = self.view.allocations();

        for filter in &self.filters {
            result = match filter {
                ViewFilter::ByThread(tid) => {
                    result.into_iter().filter(|a| a.thread_id == *tid).collect()
                }
                ViewFilter::ByType(t) => result
                    .into_iter()
                    .filter(|a| a.type_name.as_ref().map(|n| n.contains(t)).unwrap_or(false))
                    .collect(),
                ViewFilter::BySizeRange(min, max) => result
                    .into_iter()
                    .filter(|a| a.size >= *min && a.size <= *max)
                    .collect(),
                ViewFilter::ByAddressRange(min, max) => result
                    .into_iter()
                    .filter(|a| {
                        a.ptr
                            .map(|p| p as u64 >= *min && p as u64 <= *max)
                            .unwrap_or(false)
                    })
                    .collect(),
                ViewFilter::ByMinSize(min) => {
                    result.into_iter().filter(|a| a.size >= *min).collect()
                }
                ViewFilter::ByMaxSize(max) => {
                    result.into_iter().filter(|a| a.size <= *max).collect()
                }
            };
        }

        result
    }

    /// Apply filters and return count.
    pub fn count(&self) -> usize {
        self.apply().len()
    }

    /// Apply filters and return total size.
    pub fn total_size(&self) -> usize {
        self.apply().iter().map(|a| a.size).sum()
    }

    /// Get unique thread IDs after filtering.
    pub fn thread_ids(&self) -> Vec<u64> {
        let allocs = self.apply();
        let ids: HashSet<u64> = allocs.iter().map(|a| a.thread_id).collect();
        ids.into_iter().collect()
    }

    /// Get unique type names after filtering.
    pub fn type_names(&self) -> Vec<String> {
        let allocs = self.apply();
        let types: HashSet<String> = allocs.iter().filter_map(|a| a.type_name.clone()).collect();
        types.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_store::MemoryEvent;

    #[test]
    fn test_filter_by_thread() {
        let events = vec![
            MemoryEvent::allocate(0x1000, 64, 1),
            MemoryEvent::allocate(0x2000, 128, 2),
        ];
        let view = MemoryView::from_events(events);
        let builder = view.filter().by_thread(1);
        let filtered = builder.apply();
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_filter_by_size() {
        let events = vec![
            MemoryEvent::allocate(0x1000, 64, 1),
            MemoryEvent::allocate(0x2000, 128, 1),
            MemoryEvent::allocate(0x3000, 256, 1),
        ];
        let view = MemoryView::from_events(events);
        let builder = view.filter().by_size_range(100, 200);
        let filtered = builder.apply();
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_filter_chain() {
        let events = vec![
            MemoryEvent::allocate(0x1000, 64, 1).with_type_name("Vec<i32>".to_string()),
            MemoryEvent::allocate(0x2000, 128, 1).with_type_name("String".to_string()),
            MemoryEvent::allocate(0x3000, 64, 2).with_type_name("Vec<i32>".to_string()),
        ];
        let view = MemoryView::from_events(events);
        let builder = view.filter().by_thread(1).by_type("Vec");
        let filtered = builder.apply();
        assert_eq!(filtered.len(), 1);
    }
}
