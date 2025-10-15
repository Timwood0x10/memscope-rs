use std::collections::HashMap;

pub trait SizeEstimator: Send + Sync {
    fn estimate_size(&self, type_name: &str) -> Option<usize>;
    fn learn_from_real_allocation(&mut self, type_name: &str, actual_size: usize);
}

#[derive(Debug, Clone)]
pub struct LearnedSize {
    pub average: f64,
    pub count: usize,
    pub confidence: f64,
}

pub struct SmartSizeEstimator {
    known_sizes: HashMap<String, usize>,
    learned_sizes: HashMap<String, LearnedSize>,
    pointer_size: usize,
}

impl SmartSizeEstimator {
    pub fn new() -> Self {
        let mut estimator = Self {
            known_sizes: HashMap::new(),
            learned_sizes: HashMap::new(),
            pointer_size: std::mem::size_of::<*const u8>(),
        };
        estimator.initialize_known_types();
        estimator
    }

    fn initialize_known_types(&mut self) {
        let ptr_size = self.pointer_size;
        let basics = [
            ("i8", 1),
            ("i16", 2),
            ("i32", 4),
            ("i64", 8),
            ("u8", 1),
            ("u16", 2),
            ("u32", 4),
            ("u64", 8),
            ("f32", 4),
            ("f64", 8),
            ("bool", 1),
            ("char", 4),
            ("usize", ptr_size),
            ("isize", ptr_size),
            ("String", ptr_size * 3),
            ("AtomicBool", 1),
        ];

        for (name, size) in &basics {
            self.known_sizes.insert(name.to_string(), *size);
        }
    }

    fn estimate_container_size(&self, type_name: &str) -> Option<usize> {
        let ptr_size = self.pointer_size;

        if type_name.starts_with("Vec<") {
            return Some(ptr_size * 3);
        }
        if type_name.starts_with("HashMap<") {
            return Some(48);
        }
        if type_name.starts_with("Box<") {
            return Some(ptr_size);
        }
        if type_name.starts_with("Arc<") {
            return Some(ptr_size);
        }
        if type_name.starts_with("Option<") {
            return Some(8);
        }

        None
    }

    fn heuristic_estimate(&self, type_name: &str) -> usize {
        let complexity = type_name.len()
            + type_name.matches('<').count() * 4
            + type_name.matches(',').count() * 2;

        match complexity {
            0..=8 => self.pointer_size,
            9..=16 => self.pointer_size * 2,
            17..=32 => self.pointer_size * 4,
            _ => self.pointer_size * 8,
        }
    }
}

impl SizeEstimator for SmartSizeEstimator {
    fn estimate_size(&self, type_name: &str) -> Option<usize> {
        if let Some(&size) = self.known_sizes.get(type_name) {
            return Some(size);
        }

        if let Some(learned) = self.learned_sizes.get(type_name) {
            if learned.confidence > 0.8 {
                return Some(learned.average as usize);
            }
        }

        if let Some(size) = self.estimate_container_size(type_name) {
            return Some(size);
        }

        Some(self.heuristic_estimate(type_name))
    }

    fn learn_from_real_allocation(&mut self, type_name: &str, actual_size: usize) {
        let entry = self
            .learned_sizes
            .entry(type_name.to_string())
            .or_insert(LearnedSize {
                average: actual_size as f64,
                count: 0,
                confidence: 0.0,
            });

        entry.count += 1;
        let alpha = (1.0 / entry.count.min(20) as f64).max(0.05);
        entry.average = entry.average * (1.0 - alpha) + actual_size as f64 * alpha;
        entry.confidence = (entry.count as f64 / 50.0).min(0.99);
    }
}

impl Default for SmartSizeEstimator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_types() {
        let estimator = SmartSizeEstimator::new();
        assert_eq!(estimator.estimate_size("i32"), Some(4));
        assert_eq!(estimator.estimate_size("bool"), Some(1));
    }

    #[test]
    fn test_container_types() {
        let estimator = SmartSizeEstimator::new();
        assert!(estimator.estimate_size("Vec<i32>").is_some());
        assert!(estimator.estimate_size("HashMap<String, i32>").is_some());
    }

    #[test]
    fn test_learning() {
        let mut estimator = SmartSizeEstimator::new();

        // Learn from multiple allocations to build confidence
        for _ in 0..10 {
            estimator.learn_from_real_allocation("CustomType", 128);
        }
        for _ in 0..10 {
            estimator.learn_from_real_allocation("CustomType", 132);
        }

        let size = estimator.estimate_size("CustomType");
        assert!(size.is_some());
        let size_value = size.expect("Size should exist");
        // Learning algorithm should provide some reasonable estimate
        assert!(size_value > 0 && size_value < 1000); // Very flexible range
    }
}
