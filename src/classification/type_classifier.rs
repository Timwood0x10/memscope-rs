use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Type categories for memory analysis
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeCategory {
    Primitive,
    Collection,
    SmartPointer,
    UserDefined,
    System,
    Async,
    String,
    Option,
    Result,
    Tuple,
    Array,
    Custom(String),
}

impl TypeCategory {
    /// Get display name for the category
    pub fn display_name(&self) -> &str {
        match self {
            TypeCategory::Primitive => "Primitive Types",
            TypeCategory::Collection => "Collections",
            TypeCategory::SmartPointer => "Smart Pointers",
            TypeCategory::UserDefined => "User Defined",
            TypeCategory::System => "System Types",
            TypeCategory::Async => "Async Types",
            TypeCategory::String => "String Types",
            TypeCategory::Option => "Option Types",
            TypeCategory::Result => "Result Types",
            TypeCategory::Tuple => "Tuples",
            TypeCategory::Array => "Arrays",
            TypeCategory::Custom(name) => name,
        }
    }

    /// Get CSS class for visualization
    pub fn css_class(&self) -> &str {
        match self {
            TypeCategory::Primitive => "type-primitive",
            TypeCategory::Collection => "type-collection",
            TypeCategory::SmartPointer => "type-smart-pointer",
            TypeCategory::UserDefined => "type-user-defined",
            TypeCategory::System => "type-system",
            TypeCategory::Async => "type-async",
            TypeCategory::String => "type-string",
            TypeCategory::Option => "type-option",
            TypeCategory::Result => "type-result",
            TypeCategory::Tuple => "type-tuple",
            TypeCategory::Array => "type-array",
            TypeCategory::Custom(_) => "type-custom",
        }
    }

    /// Get color for visualization
    pub fn color(&self) -> &str {
        match self {
            TypeCategory::Primitive => "#4CAF50",    // Green
            TypeCategory::Collection => "#2196F3",   // Blue
            TypeCategory::SmartPointer => "#FF9800", // Orange
            TypeCategory::UserDefined => "#9C27B0",  // Purple
            TypeCategory::System => "#607D8B",       // Blue Grey
            TypeCategory::Async => "#E91E63",        // Pink
            TypeCategory::String => "#795548",       // Brown
            TypeCategory::Option => "#00BCD4",       // Cyan
            TypeCategory::Result => "#CDDC39",       // Lime
            TypeCategory::Tuple => "#FFC107",        // Amber
            TypeCategory::Array => "#3F51B5",        // Indigo
            TypeCategory::Custom(_) => "#9E9E9E",    // Grey
        }
    }
}

/// Classification rule with pattern and priority
pub struct ClassificationRule {
    pattern: Regex,
    category: TypeCategory,
    priority: u8, // Lower number = higher priority
    description: String,
}

impl ClassificationRule {
    pub fn new(
        pattern: &str,
        category: TypeCategory,
        priority: u8,
        description: &str,
    ) -> Result<Self, regex::Error> {
        Ok(Self {
            pattern: Regex::new(pattern)?,
            category,
            priority,
            description: description.to_string(),
        })
    }

    pub fn matches(&self, type_name: &str) -> bool {
        self.pattern.is_match(type_name)
    }

    pub fn category(&self) -> &TypeCategory {
        &self.category
    }

    pub fn priority(&self) -> u8 {
        self.priority
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

/// Centralized type classifier
pub struct TypeClassifier {
    rules: Vec<ClassificationRule>,
    cache: std::sync::Mutex<HashMap<String, TypeCategory>>,
}

impl std::fmt::Debug for TypeClassifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypeClassifier")
            .field("rules_count", &self.rules.len())
            .finish()
    }
}

static GLOBAL_CLASSIFIER: OnceLock<TypeClassifier> = OnceLock::new();

impl TypeClassifier {
    /// Get the global type classifier instance
    pub fn global() -> &'static TypeClassifier {
        GLOBAL_CLASSIFIER.get_or_init(|| Self::new().expect("Failed to initialize type classifier"))
    }

    /// Create a new type classifier with default rules
    pub fn new() -> Result<Self, regex::Error> {
        let mut classifier = Self {
            rules: Vec::new(),
            cache: std::sync::Mutex::new(HashMap::new()),
        };
        classifier.initialize_default_rules()?;
        Ok(classifier)
    }

    /// Initialize the default classification rules
    fn initialize_default_rules(&mut self) -> Result<(), regex::Error> {
        // Primitive types (highest priority)
        self.add_rule(
            r"^(i8|i16|i32|i64|i128|isize|u8|u16|u32|u64|u128|usize|f32|f64|bool|char)$",
            TypeCategory::Primitive,
            1,
            "Basic primitive types",
        )?;

        // String types
        self.add_rule(
            r"^(String|&str|str|std::string::String)$",
            TypeCategory::String,
            1,
            "String types",
        )?;

        // Option and Result types
        self.add_rule(
            r"^(Option|std::option::Option)<",
            TypeCategory::Option,
            2,
            "Option types",
        )?;
        self.add_rule(
            r"^(Result|std::result::Result)<",
            TypeCategory::Result,
            2,
            "Result types",
        )?;

        // Smart pointers
        self.add_rule(
            r"^(Box|std::boxed::Box)<",
            TypeCategory::SmartPointer,
            2,
            "Box smart pointer",
        )?;
        self.add_rule(
            r"^(Arc|std::sync::Arc)<",
            TypeCategory::SmartPointer,
            2,
            "Arc smart pointer",
        )?;
        self.add_rule(
            r"^(Rc|std::rc::Rc)<",
            TypeCategory::SmartPointer,
            2,
            "Rc smart pointer",
        )?;
        self.add_rule(
            r"^(Weak|std::sync::Weak|std::rc::Weak)<",
            TypeCategory::SmartPointer,
            2,
            "Weak smart pointer",
        )?;

        // Collections
        self.add_rule(
            r"^(Vec|std::vec::Vec)<",
            TypeCategory::Collection,
            2,
            "Vector collection",
        )?;
        self.add_rule(
            r"^(HashMap|std::collections::HashMap)<",
            TypeCategory::Collection,
            2,
            "HashMap collection",
        )?;
        self.add_rule(
            r"^(BTreeMap|std::collections::BTreeMap)<",
            TypeCategory::Collection,
            2,
            "BTreeMap collection",
        )?;
        self.add_rule(
            r"^(HashSet|std::collections::HashSet)<",
            TypeCategory::Collection,
            2,
            "HashSet collection",
        )?;
        self.add_rule(
            r"^(BTreeSet|std::collections::BTreeSet)<",
            TypeCategory::Collection,
            2,
            "BTreeSet collection",
        )?;
        self.add_rule(
            r"^(VecDeque|std::collections::VecDeque)<",
            TypeCategory::Collection,
            2,
            "VecDeque collection",
        )?;
        self.add_rule(
            r"^(LinkedList|std::collections::LinkedList)<",
            TypeCategory::Collection,
            2,
            "LinkedList collection",
        )?;

        // Arrays and tuples
        self.add_rule(
            r"^\[.*;\s*\d+\]$",
            TypeCategory::Array,
            2,
            "Fixed-size arrays",
        )?;
        self.add_rule(r"^\(.*,.*\)$", TypeCategory::Tuple, 2, "Tuple types")?;

        // Async types
        self.add_rule(
            r"^(Future|std::future::Future)<",
            TypeCategory::Async,
            2,
            "Future types",
        )?;
        self.add_rule(
            r"^(Stream|futures::stream::Stream)<",
            TypeCategory::Async,
            2,
            "Stream types",
        )?;
        self.add_rule(
            r"^(Sink|futures::sink::Sink)<",
            TypeCategory::Async,
            2,
            "Sink types",
        )?;

        // System types (lower priority)
        self.add_rule(r"^std::", TypeCategory::System, 3, "Standard library types")?;
        self.add_rule(r"^core::", TypeCategory::System, 3, "Core library types")?;

        Ok(())
    }

    /// Add a new classification rule
    pub fn add_rule(
        &mut self,
        pattern: &str,
        category: TypeCategory,
        priority: u8,
        description: &str,
    ) -> Result<(), regex::Error> {
        let rule = ClassificationRule::new(pattern, category, priority, description)?;
        self.rules.push(rule);

        // Sort rules by priority after adding
        self.rules.sort_by_key(|rule| rule.priority);

        // Clear cache since rules changed
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }

        Ok(())
    }

    /// Classify a type name
    pub fn classify(&self, type_name: &str) -> TypeCategory {
        // Check cache first
        if let Ok(cache) = self.cache.lock() {
            if let Some(category) = cache.get(type_name) {
                return category.clone();
            }
        }

        // Find matching rules and get the highest priority one
        let mut matched_rules: Vec<_> = self
            .rules
            .iter()
            .filter(|rule| rule.matches(type_name))
            .collect();

        matched_rules.sort_by_key(|rule| rule.priority());

        let category = matched_rules
            .first()
            .map(|rule| rule.category().clone())
            .unwrap_or(TypeCategory::UserDefined);

        // Cache the result
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(type_name.to_string(), category.clone());
        }

        category
    }

    /// Get statistics about type classification
    pub fn get_stats(&self) -> ClassificationStats {
        let cache = self.cache.lock().unwrap();
        let mut category_counts: HashMap<TypeCategory, usize> = HashMap::new();

        for category in cache.values() {
            *category_counts.entry(category.clone()).or_insert(0) += 1;
        }

        ClassificationStats {
            total_types_classified: cache.len(),
            category_counts,
            cache_hit_ratio: 1.0, // Since we always cache results
        }
    }

    /// Clear the classification cache
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }

    /// Get all available rules
    pub fn get_rules(&self) -> &[ClassificationRule] {
        &self.rules
    }
}

impl Default for TypeClassifier {
    fn default() -> Self {
        Self::new().expect("Failed to create default type classifier")
    }
}

/// Statistics about type classification
#[derive(Debug, Clone)]
pub struct ClassificationStats {
    pub total_types_classified: usize,
    pub category_counts: HashMap<TypeCategory, usize>,
    pub cache_hit_ratio: f64,
}

impl ClassificationStats {
    /// Generate a human-readable report
    pub fn report(&self) -> String {
        let mut report = "Type Classification Statistics:\n".to_string();
        report.push_str(&format!(
            "  Total types classified: {}\n",
            self.total_types_classified
        ));
        report.push_str(&format!(
            "  Cache hit ratio: {:.1}%\n",
            self.cache_hit_ratio * 100.0
        ));
        report.push_str("  Category breakdown:\n");

        let mut sorted_categories: Vec<_> = self.category_counts.iter().collect();
        sorted_categories.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending

        for (category, count) in sorted_categories {
            let percentage = (*count as f64 / self.total_types_classified as f64) * 100.0;
            report.push_str(&format!(
                "    {}: {} ({:.1}%)\n",
                category.display_name(),
                count,
                percentage
            ));
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_types() {
        let classifier = TypeClassifier::new().unwrap();

        assert_eq!(classifier.classify("i32"), TypeCategory::Primitive);
        assert_eq!(classifier.classify("u64"), TypeCategory::Primitive);
        assert_eq!(classifier.classify("f64"), TypeCategory::Primitive);
        assert_eq!(classifier.classify("bool"), TypeCategory::Primitive);
        assert_eq!(classifier.classify("char"), TypeCategory::Primitive);
    }

    #[test]
    fn test_string_types() {
        let classifier = TypeClassifier::new().unwrap();

        assert_eq!(classifier.classify("String"), TypeCategory::String);
        assert_eq!(classifier.classify("&str"), TypeCategory::String);
        assert_eq!(classifier.classify("str"), TypeCategory::String);
    }

    #[test]
    fn test_smart_pointers() {
        let classifier = TypeClassifier::new().unwrap();

        assert_eq!(classifier.classify("Box<i32>"), TypeCategory::SmartPointer);
        assert_eq!(
            classifier.classify("Arc<String>"),
            TypeCategory::SmartPointer
        );
        assert_eq!(
            classifier.classify("Rc<Vec<u8>>"),
            TypeCategory::SmartPointer
        );
    }

    #[test]
    fn test_collections() {
        let classifier = TypeClassifier::new().unwrap();

        assert_eq!(classifier.classify("Vec<i32>"), TypeCategory::Collection);
        assert_eq!(
            classifier.classify("HashMap<String, i32>"),
            TypeCategory::Collection
        );
        assert_eq!(
            classifier.classify("BTreeSet<u64>"),
            TypeCategory::Collection
        );
    }

    #[test]
    fn test_option_result() {
        let classifier = TypeClassifier::new().unwrap();

        assert_eq!(classifier.classify("Option<i32>"), TypeCategory::Option);
        assert_eq!(
            classifier.classify("Result<String, Error>"),
            TypeCategory::Result
        );
    }

    #[test]
    fn test_arrays_tuples() {
        let classifier = TypeClassifier::new().unwrap();

        assert_eq!(classifier.classify("[i32; 10]"), TypeCategory::Array);
        assert_eq!(classifier.classify("(i32, String)"), TypeCategory::Tuple);
        assert_eq!(
            classifier.classify("(i32, String, bool)"),
            TypeCategory::Tuple
        );
    }

    #[test]
    fn test_user_defined() {
        let classifier = TypeClassifier::new().unwrap();

        assert_eq!(classifier.classify("MyStruct"), TypeCategory::UserDefined);
        assert_eq!(
            classifier.classify("custom::MyType"),
            TypeCategory::UserDefined
        );
    }

    #[test]
    fn test_system_types() {
        let classifier = TypeClassifier::new().unwrap();

        assert_eq!(
            classifier.classify("std::thread::Thread"),
            TypeCategory::System
        );
        assert_eq!(
            classifier.classify("core::ptr::NonNull<u8>"),
            TypeCategory::System
        );
    }

    #[test]
    fn test_async_types() {
        let classifier = TypeClassifier::new().unwrap();

        assert_eq!(
            classifier.classify("Future<Output = i32>"),
            TypeCategory::Async
        );
        assert_eq!(
            classifier.classify("Stream<Item = String>"),
            TypeCategory::Async
        );
    }

    #[test]
    fn test_priority_system() {
        let classifier = TypeClassifier::new().unwrap();

        // std::string::String should be classified as String, not System
        // even though it matches both patterns
        assert_eq!(
            classifier.classify("std::string::String"),
            TypeCategory::String
        );
    }

    #[test]
    fn test_caching() {
        let classifier = TypeClassifier::new().unwrap();

        // First classification
        let category1 = classifier.classify("Vec<i32>");

        // Second classification should use cache
        let category2 = classifier.classify("Vec<i32>");

        assert_eq!(category1, category2);
        assert_eq!(category1, TypeCategory::Collection);
    }

    #[test]
    fn test_global_classifier() {
        let classifier1 = TypeClassifier::global();
        let classifier2 = TypeClassifier::global();

        // Should be the same instance
        assert!(std::ptr::eq(classifier1, classifier2));
    }

    #[test]
    fn test_stats() {
        let classifier = TypeClassifier::new().unwrap();

        // Classify some types
        classifier.classify("i32");
        classifier.classify("String");
        classifier.classify("Vec<u8>");
        classifier.classify("Option<i32>");

        let stats = classifier.get_stats();
        assert_eq!(stats.total_types_classified, 4);
        assert!(stats.category_counts.contains_key(&TypeCategory::Primitive));
        assert!(stats.category_counts.contains_key(&TypeCategory::String));
        assert!(stats
            .category_counts
            .contains_key(&TypeCategory::Collection));
        assert!(stats.category_counts.contains_key(&TypeCategory::Option));
    }
}
