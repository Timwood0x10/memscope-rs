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
