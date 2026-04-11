//! Classification analysis module.

use crate::view::MemoryView;
use std::collections::HashMap;

/// Classification analysis module.
///
/// Provides type classification and categorization.
pub struct ClassificationAnalysis {
    view: MemoryView,
}

impl ClassificationAnalysis {
    /// Create from view.
    pub fn from_view(view: &MemoryView) -> Self {
        Self { view: view.clone() }
    }

    /// Classify allocations by type.
    pub fn by_type(&self) -> HashMap<String, TypeClassification> {
        let mut types: HashMap<String, TypeClassification> = HashMap::new();

        for alloc in self.view.allocations() {
            let type_name = alloc
                .type_name
                .clone()
                .unwrap_or_else(|| "unknown".to_string());

            let entry = types.entry(type_name).or_default();
            entry.count += 1;
            entry.total_bytes += alloc.size;
            entry.category = classify_type(&alloc.type_name.clone().unwrap_or_default());
        }

        types
    }

    /// Get classification summary.
    pub fn summary(&self) -> ClassificationSummary {
        let types = self.by_type();
        let mut categories: HashMap<TypeCategory, usize> = HashMap::new();

        for classification in types.values() {
            *categories.entry(classification.category).or_default() += classification.count;
        }

        ClassificationSummary {
            total_types: types.len(),
            categories,
        }
    }
}

/// Classification for a specific type.
#[derive(Debug, Clone, Default)]
pub struct TypeClassification {
    /// Number of allocations
    pub count: usize,
    /// Total bytes
    pub total_bytes: usize,
    /// Type category
    pub category: TypeCategory,
}

/// Type category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TypeCategory {
    /// Unknown type
    #[default]
    Unknown,
    /// Collection type (Vec, HashMap, etc.)
    Collection,
    /// String type
    String,
    /// Smart pointer (Arc, Rc, Box)
    SmartPointer,
    /// Primitive type
    Primitive,
    /// Custom type
    Custom,
}

/// Classification summary.
#[derive(Debug, Clone)]
pub struct ClassificationSummary {
    /// Total unique types
    pub total_types: usize,
    /// Count by category
    pub categories: HashMap<TypeCategory, usize>,
}

/// Classify a type name into a category.
fn classify_type(type_name: &str) -> TypeCategory {
    if type_name.contains("Vec<")
        || type_name.contains("HashMap<")
        || type_name.contains("HashSet<")
        || type_name.contains("BTreeMap<")
        || type_name.contains("BTreeSet<")
        || type_name.contains("LinkedList<")
        || type_name.contains("VecDeque<")
    {
        TypeCategory::Collection
    } else if type_name.contains("String") || type_name.contains("str") {
        TypeCategory::String
    } else if type_name.contains("Arc<")
        || type_name.contains("Rc<")
        || type_name.contains("Box<")
        || type_name.contains("Weak<")
    {
        TypeCategory::SmartPointer
    } else if type_name.contains("i32")
        || type_name.contains("u32")
        || type_name.contains("i64")
        || type_name.contains("u64")
        || type_name.contains("f32")
        || type_name.contains("f64")
        || type_name.contains("bool")
        || type_name.contains("char")
    {
        TypeCategory::Primitive
    } else if !type_name.is_empty() && type_name != "unknown" {
        TypeCategory::Custom
    } else {
        TypeCategory::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_store::MemoryEvent;

    #[test]
    fn test_classification() {
        let events = vec![
            MemoryEvent::allocate(0x1000, 64, 1).with_type_name("Vec<i32>".to_string()),
            MemoryEvent::allocate(0x2000, 32, 1).with_type_name("String".to_string()),
        ];
        let view = MemoryView::from_events(events);
        let analysis = ClassificationAnalysis::from_view(&view);
        let types = analysis.by_type();
        assert_eq!(types.len(), 2);
    }

    #[test]
    fn test_type_category() {
        assert_eq!(classify_type("Vec<i32>"), TypeCategory::Collection);
        assert_eq!(classify_type("String"), TypeCategory::String);
        assert_eq!(classify_type("Arc<Mutex<T>>"), TypeCategory::SmartPointer);
    }
}
