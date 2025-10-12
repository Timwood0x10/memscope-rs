use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeCategory {
    Primitive,
    Collection,
    SmartPointer,
    UserDefined,
    System,
    Async,
}

pub struct TypeClassifier {
    category_map: HashMap<String, TypeCategory>,
}

impl TypeClassifier {
    pub fn new() -> Self {
        let mut classifier = Self {
            category_map: HashMap::new(),
        };
        classifier.initialize_categories();
        classifier
    }

    fn initialize_categories(&mut self) {
        let primitives = [
            "i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128", "f32", "f64",
            "bool", "char", "usize", "isize",
        ];

        for &prim in &primitives {
            self.category_map
                .insert(prim.to_string(), TypeCategory::Primitive);
        }

        let collections = [
            "Vec", "HashMap", "BTreeMap", "HashSet", "BTreeSet", "VecDeque",
        ];
        for &coll in &collections {
            self.category_map
                .insert(coll.to_string(), TypeCategory::Collection);
        }

        let smart_ptrs = ["Box", "Arc", "Rc", "Weak"];
        for &ptr in &smart_ptrs {
            self.category_map
                .insert(ptr.to_string(), TypeCategory::SmartPointer);
        }
    }

    pub fn classify(&self, type_name: &str) -> TypeCategory {
        if let Some(category) = self.category_map.get(type_name) {
            return category.clone();
        }

        if type_name.starts_with("std::") {
            return TypeCategory::System;
        }

        for (prefix, category) in &self.category_map {
            if type_name.starts_with(prefix) && type_name.contains('<') {
                return category.clone();
            }
        }

        if type_name.contains("Future") || type_name.contains("Stream") {
            return TypeCategory::Async;
        }

        TypeCategory::UserDefined
    }

    pub fn is_container(&self, type_name: &str) -> bool {
        matches!(self.classify(type_name), TypeCategory::Collection)
    }

    pub fn is_smart_pointer(&self, type_name: &str) -> bool {
        matches!(self.classify(type_name), TypeCategory::SmartPointer)
    }
}

impl Default for TypeClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_classification() {
        let classifier = TypeClassifier::new();
        assert_eq!(classifier.classify("i32"), TypeCategory::Primitive);
        assert_eq!(classifier.classify("bool"), TypeCategory::Primitive);
    }

    #[test]
    fn test_collection_classification() {
        let classifier = TypeClassifier::new();
        assert_eq!(classifier.classify("Vec<i32>"), TypeCategory::Collection);
        assert_eq!(
            classifier.classify("HashMap<String, i32>"),
            TypeCategory::Collection
        );
    }

    #[test]
    fn test_smart_pointer_classification() {
        let classifier = TypeClassifier::new();
        assert_eq!(classifier.classify("Box<i32>"), TypeCategory::SmartPointer);
        assert_eq!(
            classifier.classify("Arc<String>"),
            TypeCategory::SmartPointer
        );
    }

    #[test]
    fn test_system_classification() {
        let classifier = TypeClassifier::new();
        assert_eq!(
            classifier.classify("std::string::String"),
            TypeCategory::System
        );
        assert_eq!(classifier.classify("std::vec::Vec"), TypeCategory::System);
    }

    #[test]
    fn test_helper_methods() {
        let classifier = TypeClassifier::new();
        assert!(classifier.is_container("Vec<i32>"));
        assert!(classifier.is_smart_pointer("Box<String>"));
        assert!(!classifier.is_container("i32"));
    }
}
