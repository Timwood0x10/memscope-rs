use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub struct BoundedHistory<T> {
    max_entries: usize,
    max_age: Duration,
    entries: VecDeque<TimestampedEntry<T>>,
    total_memory_limit: usize,
    current_memory_usage: usize,
}

struct TimestampedEntry<T> {
    data: T,
    timestamp: Instant,
    memory_footprint: usize,
}

#[derive(Debug, Clone)]
pub struct MemoryUsageStats {
    pub entry_count: usize,
    pub memory_usage_mb: f64,
    pub memory_usage_ratio: f64,
    pub oldest_entry_age_secs: Option<f64>,
    pub average_entry_size: f64,
}

impl<T> BoundedHistory<T> {
    pub fn new(max_entries: usize, max_age: Duration, memory_limit_mb: usize) -> Self {
        Self {
            max_entries,
            max_age,
            entries: VecDeque::new(),
            total_memory_limit: memory_limit_mb * 1024 * 1024,
            current_memory_usage: 0,
        }
    }

    pub fn push(&mut self, data: T) -> bool {
        let memory_footprint =
            std::mem::size_of::<T>() + std::mem::size_of::<TimestampedEntry<T>>();

        if memory_footprint > self.total_memory_limit {
            return false;
        }

        if self.current_memory_usage + memory_footprint > self.total_memory_limit {
            self.evict_oldest_entries(memory_footprint);
        }

        let entry = TimestampedEntry {
            data,
            timestamp: Instant::now(),
            memory_footprint,
        };

        self.entries.push_back(entry);
        self.current_memory_usage += memory_footprint;

        while self.entries.len() > self.max_entries {
            self.remove_oldest();
        }

        self.cleanup_expired();
        true
    }

    pub fn entries(&self) -> impl Iterator<Item = &T> {
        self.entries.iter().map(|entry| &entry.data)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_memory_usage = 0;
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn get_memory_usage_stats(&self) -> MemoryUsageStats {
        let memory_usage_mb = self.current_memory_usage as f64 / (1024.0 * 1024.0);
        let memory_usage_ratio = self.current_memory_usage as f64 / self.total_memory_limit as f64;
        let oldest_entry_age_secs = self
            .entries
            .front()
            .map(|entry| entry.timestamp.elapsed().as_secs_f64());
        let average_entry_size = if self.entries.is_empty() {
            0.0
        } else {
            self.current_memory_usage as f64 / self.entries.len() as f64
        };

        MemoryUsageStats {
            entry_count: self.entries.len(),
            memory_usage_mb,
            memory_usage_ratio,
            oldest_entry_age_secs,
            average_entry_size,
        }
    }

    pub fn cleanup_expired(&mut self) -> usize {
        let cutoff = Instant::now() - self.max_age;
        let mut removed_count = 0;

        while let Some(entry) = self.entries.front() {
            if entry.timestamp < cutoff {
                self.remove_oldest();
                removed_count += 1;
            } else {
                break;
            }
        }
        removed_count
    }

    fn evict_oldest_entries(&mut self, needed_space: usize) {
        let mut freed_space = 0;
        while freed_space < needed_space && !self.entries.is_empty() {
            freed_space += self.remove_oldest();
        }
    }

    fn remove_oldest(&mut self) -> usize {
        if let Some(entry) = self.entries.pop_front() {
            self.current_memory_usage -= entry.memory_footprint;
            entry.memory_footprint
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        let mut history = BoundedHistory::new(3, Duration::from_secs(60), 1);

        assert!(history.push(1));
        assert!(history.push(2));
        assert!(history.push(3));
        assert_eq!(history.len(), 3);

        assert!(history.push(4));
        assert_eq!(history.len(), 3);

        let values: Vec<_> = history.entries().cloned().collect();
        assert_eq!(values, vec![2, 3, 4]);
    }

    #[test]
    fn test_memory_stats() {
        let mut history = BoundedHistory::new(100, Duration::from_secs(60), 10);

        for i in 0..50 {
            history.push(i);
        }

        let stats = history.get_memory_usage_stats();
        assert_eq!(stats.entry_count, 50);
        assert!(stats.memory_usage_mb > 0.0);
        assert!(stats.average_entry_size > 0.0);
    }
}
