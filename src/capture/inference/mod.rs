//! Inference Engine for memscope-rs
//!
//! This module provides inference capabilities for data that cannot be
//! captured at runtime. All inferred data is clearly marked with its
//! source and confidence level.
//!
//! # Important Warning
//!
//! ⚠️ Inferred data may be WRONG. Use with caution.
//!
//! # Design Principles
//!
//! 1. All inferred data is marked with `_source: "inferred"`
//! 2. Confidence level is provided for each inference
//! 3. Inference rules are documented and configurable

use serde::{Deserialize, Serialize};

/// Source of data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataSource {
    /// Real data captured at runtime
    Captured,
    /// Inferred data (may be wrong)
    Inferred,
    /// User-provided data
    UserProvided,
}

/// Confidence level for inferred data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Confidence {
    /// High confidence (>80% accuracy expected)
    High,
    /// Medium confidence (50-80% accuracy expected)
    Medium,
    /// Low confidence (<50% accuracy expected)
    Low,
}

/// Inferred borrow information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferredBorrowInfo {
    pub immutable_borrows: usize,
    pub mutable_borrows: usize,
    pub max_concurrent_borrows: usize,

    /// Source of this data
    pub _source: DataSource,
    /// Confidence level
    pub _confidence: Confidence,
    /// Inference rule used
    pub _rule: &'static str,
}

/// Inferred smart pointer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferredSmartPointerInfo {
    pub pointer_type: SmartPointerType,
    pub ref_count: Option<usize>,
    pub is_shared: bool,

    /// Source of this data
    pub _source: DataSource,
    /// Confidence level
    pub _confidence: Confidence,
    /// Inference rule used
    pub _rule: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SmartPointerType {
    Rc,
    Arc,
    Box,
    Unknown,
}

/// Inference Engine
pub struct InferenceEngine {
    rules: Vec<InferenceRule>,
}

#[derive(Debug, Clone)]
pub struct InferenceRule {
    pub name: &'static str,
    pub description: &'static str,
    pub confidence: Confidence,
}

impl InferenceEngine {
    pub fn new() -> Self {
        Self {
            rules: vec![
                InferenceRule {
                    name: "type_name_pattern",
                    description: "Infer from type name patterns (e.g., 'Rc<T>' → smart pointer)",
                    confidence: Confidence::Medium,
                },
                InferenceRule {
                    name: "size_heuristic",
                    description: "Infer from allocation size (e.g., 24 bytes → likely String)",
                    confidence: Confidence::Low,
                },
                InferenceRule {
                    name: "usage_pattern",
                    description: "Infer from usage patterns (e.g., multiple clones → shared)",
                    confidence: Confidence::Medium,
                },
            ],
        }
    }

    /// Infer borrow information from type name
    pub fn infer_borrow_info(&self, type_name: Option<&str>) -> InferredBorrowInfo {
        let (immutable, mutable, max_concurrent, confidence) = match type_name {
            Some(name) if name.contains("Rc<") || name.contains("Arc<") => {
                (5, 0, 5, Confidence::Low)
            }
            Some(name) if name.contains("Vec<") || name.contains("String") => {
                (4, 2, 3, Confidence::Low)
            }
            Some(name) if name.contains("Box<") => (2, 1, 1, Confidence::Low),
            _ => (0, 0, 0, Confidence::Low),
        };

        InferredBorrowInfo {
            immutable_borrows: immutable,
            mutable_borrows: mutable,
            max_concurrent_borrows: max_concurrent,
            _source: DataSource::Inferred,
            _confidence: confidence,
            _rule: "type_name_pattern",
        }
    }

    /// Infer smart pointer information from type name
    pub fn infer_smart_pointer(&self, type_name: Option<&str>) -> InferredSmartPointerInfo {
        let (ptr_type, is_shared, confidence) = match type_name {
            Some(name) if name.contains("Rc<") => (SmartPointerType::Rc, true, Confidence::Medium),
            Some(name) if name.contains("Arc<") => {
                (SmartPointerType::Arc, true, Confidence::Medium)
            }
            Some(name) if name.contains("Box<") => {
                (SmartPointerType::Box, false, Confidence::Medium)
            }
            _ => (SmartPointerType::Unknown, false, Confidence::Low),
        };

        InferredSmartPointerInfo {
            pointer_type: ptr_type,
            ref_count: if is_shared { Some(2) } else { None },
            is_shared,
            _source: DataSource::Inferred,
            _confidence: confidence,
            _rule: "type_name_pattern",
        }
    }

    /// Get all inference rules
    pub fn rules(&self) -> &[InferenceRule] {
        &self.rules
    }
}

impl Default for InferenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_borrow_info_rc() {
        let engine = InferenceEngine::new();
        let info = engine.infer_borrow_info(Some("std::rc::Rc<String>"));

        assert_eq!(info._source, DataSource::Inferred);
        assert_eq!(info._confidence, Confidence::Low);
        assert!(info.immutable_borrows > 0);
    }

    #[test]
    fn test_infer_smart_pointer_arc() {
        let engine = InferenceEngine::new();
        let info = engine.infer_smart_pointer(Some("std::sync::Arc<Vec<u8>>"));

        assert_eq!(info.pointer_type, SmartPointerType::Arc);
        assert!(info.is_shared);
        assert_eq!(info._source, DataSource::Inferred);
    }

    #[test]
    fn test_infer_unknown_type() {
        let engine = InferenceEngine::new();
        let info = engine.infer_borrow_info(None);

        assert_eq!(info.immutable_borrows, 0);
        assert_eq!(info._confidence, Confidence::Low);
    }
}
