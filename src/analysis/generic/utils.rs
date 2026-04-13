use super::types::{ConstraintType, GenericConstraint};

pub fn extract_constraints(type_name: &str) -> Vec<GenericConstraint> {
    let mut constraints = Vec::new();

    if is_collection_type(type_name) {
        constraints.push(GenericConstraint {
            parameter_name: "T".to_string(),
            constraint_type: ConstraintType::Sized,
            description: "Type must be Sized for standard collections".to_string(),
        });
    }

    if is_smart_pointer_type(type_name) {
        constraints.push(GenericConstraint {
            parameter_name: "T".to_string(),
            constraint_type: ConstraintType::Sized,
            description: "Type must be Sized for smart pointers".to_string(),
        });
    }

    if is_thread_safe_type(type_name) {
        constraints.push(GenericConstraint {
            parameter_name: "T".to_string(),
            constraint_type: ConstraintType::Send,
            description: "Type must be Send for thread-safe containers".to_string(),
        });
    }

    if is_sync_required_type(type_name) {
        constraints.push(GenericConstraint {
            parameter_name: "T".to_string(),
            constraint_type: ConstraintType::Sync,
            description: "Type must be Sync for shared concurrent access".to_string(),
        });
    }

    constraints
}

fn is_collection_type(type_name: &str) -> bool {
    let collection_patterns = [
        r"\bVec<",
        r"\bVecDeque<",
        r"\bLinkedList<",
        r"\bHashMap<",
        r"\bBTreeMap<",
        r"\bHashSet<",
        r"\bBTreeSet<",
        r"\bBinaryHeap<",
    ];

    collection_patterns.iter().any(|pattern| {
        regex::Regex::new(pattern)
            .map(|re| re.is_match(type_name))
            .unwrap_or(false)
    })
}

fn is_smart_pointer_type(type_name: &str) -> bool {
    let smart_pointer_patterns = [r"\bBox<", r"\bRc<", r"\bArc<", r"\bWeak<"];

    smart_pointer_patterns.iter().any(|pattern| {
        regex::Regex::new(pattern)
            .map(|re| re.is_match(type_name))
            .unwrap_or(false)
    })
}

fn is_thread_safe_type(type_name: &str) -> bool {
    let thread_safe_patterns = [
        r"\bMutex<",
        r"\bRwLock<",
        r"\bmpsc::",
        r"\bSender<",
        r"\bReceiver<",
    ];

    thread_safe_patterns.iter().any(|pattern| {
        regex::Regex::new(pattern)
            .map(|re| re.is_match(type_name))
            .unwrap_or(false)
    })
}

fn is_sync_required_type(type_name: &str) -> bool {
    let sync_required_patterns = [r"\bArc<", r"&\s*Mutex<", r"&\s*RwLock<"];

    sync_required_patterns.iter().any(|pattern| {
        regex::Regex::new(pattern)
            .map(|re| re.is_match(type_name))
            .unwrap_or(false)
    })
}

pub fn parse_generic_parameters(type_name: &str) -> (String, Vec<String>) {
    if let Some(start) = type_name.find('<') {
        if let Some(end) = type_name.rfind('>') {
            let base_type = type_name[..start].to_string();
            let params_str = &type_name[start + 1..end];

            let params: Vec<String> = params_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            return (base_type, params);
        }
    }

    (type_name.to_string(), Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Verify extract_constraints identifies Vec as collection type
    /// Invariants: Vec<T> should have Sized constraint
    #[test]
    fn test_extract_constraints_vec() {
        let constraints = extract_constraints("Vec<i32>");
        assert!(!constraints.is_empty(), "Vec should have constraints");
        assert!(
            constraints
                .iter()
                .any(|c| matches!(c.constraint_type, ConstraintType::Sized)),
            "Vec should have Sized constraint"
        );
    }

    /// Objective: Verify extract_constraints identifies HashMap as collection type
    /// Invariants: HashMap<K, V> should have Sized constraint
    #[test]
    fn test_extract_constraints_hashmap() {
        let constraints = extract_constraints("HashMap<String, i32>");
        assert!(!constraints.is_empty(), "HashMap should have constraints");
    }

    /// Objective: Verify extract_constraints identifies Box as smart pointer
    /// Invariants: Box<T> should have Sized constraint
    #[test]
    fn test_extract_constraints_box() {
        let constraints = extract_constraints("Box<MyStruct>");
        assert!(!constraints.is_empty(), "Box should have constraints");
        assert!(
            constraints
                .iter()
                .any(|c| matches!(c.constraint_type, ConstraintType::Sized)),
            "Box should have Sized constraint"
        );
    }

    /// Objective: Verify extract_constraints identifies Arc as thread-safe
    /// Invariants: Arc<T> should have Send and Sync constraints
    #[test]
    fn test_extract_constraints_arc() {
        let constraints = extract_constraints("Arc<MyStruct>");
        assert!(!constraints.is_empty(), "Arc should have constraints");
        assert!(
            constraints
                .iter()
                .any(|c| matches!(c.constraint_type, ConstraintType::Sync)),
            "Arc should have Sync constraint"
        );
    }

    /// Objective: Verify extract_constraints identifies Mutex as thread-safe
    /// Invariants: Mutex<T> should have Send constraint
    #[test]
    fn test_extract_constraints_mutex() {
        let constraints = extract_constraints("Mutex<MyData>");
        assert!(!constraints.is_empty(), "Mutex should have constraints");
        assert!(
            constraints
                .iter()
                .any(|c| matches!(c.constraint_type, ConstraintType::Send)),
            "Mutex should have Send constraint"
        );
    }

    /// Objective: Verify extract_constraints handles non-generic types
    /// Invariants: Non-generic types should have no constraints
    #[test]
    fn test_extract_constraints_non_generic() {
        let constraints = extract_constraints("i32");
        assert!(
            constraints.is_empty(),
            "i32 should have no generic constraints"
        );
    }

    /// Objective: Verify extract_constraints handles empty string
    /// Invariants: Empty string should return empty constraints
    #[test]
    fn test_extract_constraints_empty() {
        let constraints = extract_constraints("");
        assert!(
            constraints.is_empty(),
            "Empty string should have no constraints"
        );
    }

    /// Objective: Verify parse_generic_parameters parses Vec<i32>
    /// Invariants: Should return base type and single parameter
    #[test]
    fn test_parse_generic_parameters_vec() {
        let (base, params) = parse_generic_parameters("Vec<i32>");
        assert_eq!(base, "Vec", "Base type should be Vec");
        assert_eq!(params, vec!["i32"], "Should have one parameter");
    }

    /// Objective: Verify parse_generic_parameters parses HashMap<K, V>
    /// Invariants: Should return base type and two parameters
    #[test]
    fn test_parse_generic_parameters_hashmap() {
        let (base, params) = parse_generic_parameters("HashMap<String, i32>");
        assert_eq!(base, "HashMap", "Base type should be HashMap");
        assert_eq!(params, vec!["String", "i32"], "Should have two parameters");
    }

    /// Objective: Verify parse_generic_parameters handles nested generics
    /// Invariants: Should extract outer type and inner as single param
    #[test]
    fn test_parse_generic_parameters_nested() {
        let (base, params) = parse_generic_parameters("Vec<Vec<i32>>");
        assert_eq!(base, "Vec", "Base type should be Vec");
        assert_eq!(params.len(), 1, "Should have one parameter");
        assert!(params[0].contains("Vec"), "Parameter should contain Vec");
    }

    /// Objective: Verify parse_generic_parameters handles non-generic types
    /// Invariants: Should return original string and empty params
    #[test]
    fn test_parse_generic_parameters_non_generic() {
        let (base, params) = parse_generic_parameters("String");
        assert_eq!(base, "String", "Base should be original string");
        assert!(params.is_empty(), "Should have no parameters");
    }

    /// Objective: Verify parse_generic_parameters handles empty string
    /// Invariants: Should return empty string and empty params
    #[test]
    fn test_parse_generic_parameters_empty() {
        let (base, params) = parse_generic_parameters("");
        assert_eq!(base, "", "Base should be empty");
        assert!(params.is_empty(), "Should have no parameters");
    }

    /// Objective: Verify parse_generic_parameters handles malformed input
    /// Invariants: Malformed input should return original string
    #[test]
    fn test_parse_generic_parameters_malformed() {
        let (base, params) = parse_generic_parameters("Vec<i32");
        assert_eq!(base, "Vec<i32", "Malformed should return original");
        assert!(params.is_empty(), "Should have no parameters for malformed");
    }

    /// Objective: Verify parse_generic_parameters handles whitespace
    /// Invariants: Whitespace should be trimmed from parameters
    #[test]
    fn test_parse_generic_parameters_whitespace() {
        let (base, params) = parse_generic_parameters("HashMap< String , i32 >");
        assert_eq!(base, "HashMap", "Base should be HashMap");
        assert_eq!(
            params,
            vec!["String", "i32"],
            "Parameters should be trimmed"
        );
    }

    /// Objective: Verify extract_constraints handles Rc
    /// Invariants: Rc<T> should have Sized constraint
    #[test]
    fn test_extract_constraints_rc() {
        let constraints = extract_constraints("Rc<MyStruct>");
        assert!(!constraints.is_empty(), "Rc should have constraints");
    }

    /// Objective: Verify extract_constraints handles RwLock
    /// Invariants: RwLock<T> should have Send constraint
    #[test]
    fn test_extract_constraints_rwlock() {
        let constraints = extract_constraints("RwLock<Data>");
        assert!(!constraints.is_empty(), "RwLock should have constraints");
    }

    /// Objective: Verify extract_constraints handles complex nested types
    /// Invariants: Complex types should be detected correctly
    #[test]
    fn test_extract_constraints_complex_nested() {
        let constraints = extract_constraints("Arc<Mutex<Vec<HashMap<String, i32>>>>");
        assert!(
            !constraints.is_empty(),
            "Complex nested type should have constraints"
        );
        assert!(
            constraints
                .iter()
                .any(|c| matches!(c.constraint_type, ConstraintType::Sync)),
            "Arc should contribute Sync constraint"
        );
    }

    /// Objective: Verify extract_constraints handles VecDeque
    /// Invariants: VecDeque<T> should have Sized constraint
    #[test]
    fn test_extract_constraints_vecdeque() {
        let constraints = extract_constraints("VecDeque<i32>");
        assert!(!constraints.is_empty(), "VecDeque should have constraints");
    }

    /// Objective: Verify extract_constraints handles BTreeMap
    /// Invariants: BTreeMap<K, V> should have Sized constraint
    #[test]
    fn test_extract_constraints_btreemap() {
        let constraints = extract_constraints("BTreeMap<String, i32>");
        assert!(!constraints.is_empty(), "BTreeMap should have constraints");
    }

    /// Objective: Verify extract_constraints handles BinaryHeap
    /// Invariants: BinaryHeap<T> should have Sized constraint
    #[test]
    fn test_extract_constraints_binaryheap() {
        let constraints = extract_constraints("BinaryHeap<i32>");
        assert!(
            !constraints.is_empty(),
            "BinaryHeap should have constraints"
        );
    }

    /// Objective: Verify extract_constraints handles LinkedList
    /// Invariants: LinkedList<T> should have Sized constraint
    #[test]
    fn test_extract_constraints_linkedlist() {
        let constraints = extract_constraints("LinkedList<i32>");
        assert!(
            !constraints.is_empty(),
            "LinkedList should have constraints"
        );
    }

    /// Objective: Verify extract_constraints handles Weak
    /// Invariants: Weak<T> should have Sized constraint
    #[test]
    fn test_extract_constraints_weak() {
        let constraints = extract_constraints("Weak<MyStruct>");
        assert!(!constraints.is_empty(), "Weak should have constraints");
    }

    /// Objective: Verify extract_constraints handles mpsc channel types
    /// Invariants: Sender<T> and Receiver<T> should have Send constraint
    #[test]
    fn test_extract_constraints_channel() {
        let sender_constraints = extract_constraints("mpsc::Sender<Message>");
        assert!(
            !sender_constraints.is_empty(),
            "Sender should have constraints"
        );

        let receiver_constraints = extract_constraints("mpsc::Receiver<Message>");
        assert!(
            !receiver_constraints.is_empty(),
            "Receiver should have constraints"
        );
    }
}
