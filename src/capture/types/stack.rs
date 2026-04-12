//! Stack allocation tracking types.
//!
//! This module contains types for tracking stack allocations,
//! including stack scope information and frame details.

use serde::{Deserialize, Serialize};

/// Stack allocation tracking information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StackAllocationInfo {
    /// Stack frame identifier.
    pub frame_id: usize,
    /// Variable name on stack.
    pub var_name: String,
    /// Stack offset from frame pointer.
    pub stack_offset: isize,
    /// Size of stack allocation.
    pub size: usize,
    /// Function name where allocated.
    pub function_name: String,
    /// Stack depth level.
    pub stack_depth: usize,
    /// Lifetime scope information.
    pub scope_info: StackScopeInfo,
}

/// Stack scope information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StackScopeInfo {
    /// Scope type (function, block, loop, etc.).
    pub scope_type: ScopeType,
    /// Scope start line number.
    pub start_line: Option<u32>,
    /// Scope end line number.
    pub end_line: Option<u32>,
    /// Parent scope identifier.
    pub parent_scope: Option<usize>,
    /// Nested scope level.
    pub nesting_level: usize,
}

/// Scope type enumeration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScopeType {
    /// Function scope.
    Function,
    /// Block scope.
    Block,
    /// Loop scope.
    Loop,
    /// Conditional scope.
    Conditional,
    /// Match scope.
    Match,
    /// Async scope.
    Async,
    /// Unsafe scope.
    Unsafe,
}

// Implement From trait for converting from core::types to capture::types
impl From<crate::core::types::StackAllocationInfo> for StackAllocationInfo {
    fn from(old: crate::core::types::StackAllocationInfo) -> Self {
        Self {
            frame_id: old.frame_id,
            var_name: old.var_name,
            stack_offset: old.stack_offset,
            size: old.size,
            function_name: old.function_name,
            stack_depth: old.stack_depth,
            scope_info: StackScopeInfo {
                scope_type: match old.scope_info.scope_type {
                    crate::core::types::ScopeType::Function => ScopeType::Function,
                    crate::core::types::ScopeType::Block => ScopeType::Block,
                    crate::core::types::ScopeType::Loop => ScopeType::Loop,
                    crate::core::types::ScopeType::Conditional => ScopeType::Conditional,
                    crate::core::types::ScopeType::Match => ScopeType::Match,
                    crate::core::types::ScopeType::Async => ScopeType::Async,
                    crate::core::types::ScopeType::Unsafe => ScopeType::Unsafe,
                },
                start_line: old.scope_info.start_line,
                end_line: old.scope_info.end_line,
                parent_scope: old.scope_info.parent_scope,
                nesting_level: old.scope_info.nesting_level,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_allocation_info() {
        let info = StackAllocationInfo {
            frame_id: 1,
            var_name: "local_var".to_string(),
            stack_offset: -16,
            size: 8,
            function_name: "main".to_string(),
            stack_depth: 2,
            scope_info: StackScopeInfo {
                scope_type: ScopeType::Function,
                start_line: Some(10),
                end_line: Some(20),
                parent_scope: None,
                nesting_level: 0,
            },
        };

        assert_eq!(info.frame_id, 1);
        assert_eq!(info.var_name, "local_var");
        assert_eq!(info.stack_offset, -16);
    }

    #[test]
    fn test_scope_type_variants() {
        let types = vec![
            ScopeType::Function,
            ScopeType::Block,
            ScopeType::Loop,
            ScopeType::Conditional,
            ScopeType::Match,
            ScopeType::Async,
            ScopeType::Unsafe,
        ];

        for scope_type in types {
            assert!(!format!("{scope_type:?}").is_empty());
        }
    }

    #[test]
    fn test_stack_scope_info_creation() {
        let scope = StackScopeInfo {
            scope_type: ScopeType::Loop,
            start_line: Some(5),
            end_line: Some(15),
            parent_scope: Some(1),
            nesting_level: 2,
        };

        assert_eq!(scope.scope_type, ScopeType::Loop);
        assert_eq!(scope.start_line, Some(5));
        assert_eq!(scope.parent_scope, Some(1));
        assert_eq!(scope.nesting_level, 2);
    }

    #[test]
    fn test_stack_allocation_info_with_block_scope() {
        let info = StackAllocationInfo {
            frame_id: 2,
            var_name: "block_var".to_string(),
            stack_offset: -32,
            size: 16,
            function_name: "process".to_string(),
            stack_depth: 3,
            scope_info: StackScopeInfo {
                scope_type: ScopeType::Block,
                start_line: Some(25),
                end_line: Some(35),
                parent_scope: Some(0),
                nesting_level: 1,
            },
        };

        assert_eq!(info.scope_info.scope_type, ScopeType::Block);
        assert_eq!(info.size, 16);
    }

    #[test]
    fn test_stack_allocation_info_with_match_scope() {
        let info = StackAllocationInfo {
            frame_id: 3,
            var_name: "match_var".to_string(),
            stack_offset: -8,
            size: 4,
            function_name: "handle".to_string(),
            stack_depth: 1,
            scope_info: StackScopeInfo {
                scope_type: ScopeType::Match,
                start_line: Some(100),
                end_line: Some(120),
                parent_scope: None,
                nesting_level: 0,
            },
        };

        assert_eq!(info.scope_info.scope_type, ScopeType::Match);
        assert_eq!(info.stack_depth, 1);
    }

    #[test]
    fn test_stack_allocation_info_with_async_scope() {
        let info = StackAllocationInfo {
            frame_id: 4,
            var_name: "async_var".to_string(),
            stack_offset: -64,
            size: 32,
            function_name: "async_fn".to_string(),
            stack_depth: 5,
            scope_info: StackScopeInfo {
                scope_type: ScopeType::Async,
                start_line: Some(1),
                end_line: Some(50),
                parent_scope: None,
                nesting_level: 0,
            },
        };

        assert_eq!(info.scope_info.scope_type, ScopeType::Async);
        assert_eq!(info.function_name, "async_fn");
    }

    #[test]
    fn test_stack_allocation_info_with_unsafe_scope() {
        let info = StackAllocationInfo {
            frame_id: 5,
            var_name: "unsafe_var".to_string(),
            stack_offset: -128,
            size: 64,
            function_name: "unsafe_fn".to_string(),
            stack_depth: 2,
            scope_info: StackScopeInfo {
                scope_type: ScopeType::Unsafe,
                start_line: Some(10),
                end_line: Some(20),
                parent_scope: Some(1),
                nesting_level: 1,
            },
        };

        assert_eq!(info.scope_info.scope_type, ScopeType::Unsafe);
    }

    #[test]
    fn test_stack_allocation_info_with_conditional_scope() {
        let info = StackAllocationInfo {
            frame_id: 6,
            var_name: "cond_var".to_string(),
            stack_offset: -24,
            size: 8,
            function_name: "check".to_string(),
            stack_depth: 2,
            scope_info: StackScopeInfo {
                scope_type: ScopeType::Conditional,
                start_line: Some(30),
                end_line: Some(40),
                parent_scope: None,
                nesting_level: 0,
            },
        };

        assert_eq!(info.scope_info.scope_type, ScopeType::Conditional);
    }

    #[test]
    fn test_stack_scope_info_no_lines() {
        let scope = StackScopeInfo {
            scope_type: ScopeType::Function,
            start_line: None,
            end_line: None,
            parent_scope: None,
            nesting_level: 0,
        };

        assert!(scope.start_line.is_none());
        assert!(scope.end_line.is_none());
    }

    #[test]
    fn test_stack_allocation_info_serialization() {
        let info = StackAllocationInfo {
            frame_id: 10,
            var_name: "serialized_var".to_string(),
            stack_offset: -48,
            size: 24,
            function_name: "test_serialization".to_string(),
            stack_depth: 4,
            scope_info: StackScopeInfo {
                scope_type: ScopeType::Function,
                start_line: Some(1),
                end_line: Some(10),
                parent_scope: None,
                nesting_level: 0,
            },
        };

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: StackAllocationInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.frame_id, info.frame_id);
        assert_eq!(deserialized.var_name, info.var_name);
    }

    #[test]
    fn test_stack_scope_info_serialization() {
        let scope = StackScopeInfo {
            scope_type: ScopeType::Loop,
            start_line: Some(5),
            end_line: Some(15),
            parent_scope: Some(1),
            nesting_level: 2,
        };

        let json = serde_json::to_string(&scope).unwrap();
        let deserialized: StackScopeInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.scope_type, scope.scope_type);
        assert_eq!(deserialized.nesting_level, scope.nesting_level);
    }

    #[test]
    fn test_scope_type_serialization() {
        let types = vec![
            ScopeType::Function,
            ScopeType::Block,
            ScopeType::Loop,
            ScopeType::Conditional,
            ScopeType::Match,
            ScopeType::Async,
            ScopeType::Unsafe,
        ];

        for scope_type in types {
            let json = serde_json::to_string(&scope_type).unwrap();
            let deserialized: ScopeType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, scope_type);
        }
    }

    #[test]
    fn test_stack_allocation_info_clone() {
        let info = StackAllocationInfo {
            frame_id: 1,
            var_name: "clone_test".to_string(),
            stack_offset: -16,
            size: 8,
            function_name: "test".to_string(),
            stack_depth: 1,
            scope_info: StackScopeInfo {
                scope_type: ScopeType::Function,
                start_line: None,
                end_line: None,
                parent_scope: None,
                nesting_level: 0,
            },
        };

        let cloned = info.clone();
        assert_eq!(cloned.frame_id, info.frame_id);
        assert_eq!(cloned.var_name, info.var_name);
    }

    #[test]
    fn test_stack_scope_info_clone() {
        let scope = StackScopeInfo {
            scope_type: ScopeType::Block,
            start_line: Some(10),
            end_line: Some(20),
            parent_scope: Some(5),
            nesting_level: 3,
        };

        let cloned = scope.clone();
        assert_eq!(cloned.scope_type, scope.scope_type);
        assert_eq!(cloned.nesting_level, scope.nesting_level);
    }

    #[test]
    fn test_stack_allocation_info_debug() {
        let info = StackAllocationInfo {
            frame_id: 1,
            var_name: "debug_test".to_string(),
            stack_offset: -16,
            size: 8,
            function_name: "test".to_string(),
            stack_depth: 1,
            scope_info: StackScopeInfo {
                scope_type: ScopeType::Function,
                start_line: None,
                end_line: None,
                parent_scope: None,
                nesting_level: 0,
            },
        };

        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("StackAllocationInfo"));
        assert!(debug_str.contains("frame_id"));
    }

    #[test]
    fn test_stack_scope_info_debug() {
        let scope = StackScopeInfo {
            scope_type: ScopeType::Loop,
            start_line: Some(1),
            end_line: Some(10),
            parent_scope: None,
            nesting_level: 0,
        };

        let debug_str = format!("{:?}", scope);
        assert!(debug_str.contains("StackScopeInfo"));
        assert!(debug_str.contains("scope_type"));
    }

    #[test]
    fn test_stack_allocation_info_equality() {
        let info1 = StackAllocationInfo {
            frame_id: 1,
            var_name: "test".to_string(),
            stack_offset: -16,
            size: 8,
            function_name: "fn".to_string(),
            stack_depth: 1,
            scope_info: StackScopeInfo {
                scope_type: ScopeType::Function,
                start_line: None,
                end_line: None,
                parent_scope: None,
                nesting_level: 0,
            },
        };

        let info2 = StackAllocationInfo {
            frame_id: 1,
            var_name: "test".to_string(),
            stack_offset: -16,
            size: 8,
            function_name: "fn".to_string(),
            stack_depth: 1,
            scope_info: StackScopeInfo {
                scope_type: ScopeType::Function,
                start_line: None,
                end_line: None,
                parent_scope: None,
                nesting_level: 0,
            },
        };

        assert_eq!(info1, info2);
    }

    #[test]
    fn test_stack_scope_info_equality() {
        let scope1 = StackScopeInfo {
            scope_type: ScopeType::Function,
            start_line: Some(1),
            end_line: Some(10),
            parent_scope: None,
            nesting_level: 0,
        };

        let scope2 = StackScopeInfo {
            scope_type: ScopeType::Function,
            start_line: Some(1),
            end_line: Some(10),
            parent_scope: None,
            nesting_level: 0,
        };

        assert_eq!(scope1, scope2);
    }

    #[test]
    fn test_boundary_values_stack_allocation() {
        let info = StackAllocationInfo {
            frame_id: usize::MAX,
            var_name: String::new(),
            stack_offset: isize::MIN,
            size: usize::MAX,
            function_name: String::new(),
            stack_depth: usize::MAX,
            scope_info: StackScopeInfo {
                scope_type: ScopeType::Function,
                start_line: Some(u32::MAX),
                end_line: Some(u32::MAX),
                parent_scope: Some(usize::MAX),
                nesting_level: usize::MAX,
            },
        };

        assert_eq!(info.frame_id, usize::MAX);
        assert_eq!(info.stack_offset, isize::MIN);
        assert_eq!(info.size, usize::MAX);
    }

    #[test]
    fn test_negative_stack_offset() {
        let info = StackAllocationInfo {
            frame_id: 1,
            var_name: "negative_offset".to_string(),
            stack_offset: -1024,
            size: 64,
            function_name: "test".to_string(),
            stack_depth: 1,
            scope_info: StackScopeInfo {
                scope_type: ScopeType::Function,
                start_line: None,
                end_line: None,
                parent_scope: None,
                nesting_level: 0,
            },
        };

        assert_eq!(info.stack_offset, -1024);
    }

    #[test]
    fn test_positive_stack_offset() {
        let info = StackAllocationInfo {
            frame_id: 1,
            var_name: "positive_offset".to_string(),
            stack_offset: 256,
            size: 32,
            function_name: "test".to_string(),
            stack_depth: 1,
            scope_info: StackScopeInfo {
                scope_type: ScopeType::Function,
                start_line: None,
                end_line: None,
                parent_scope: None,
                nesting_level: 0,
            },
        };

        assert_eq!(info.stack_offset, 256);
    }

    #[test]
    fn test_deep_nesting_level() {
        let scope = StackScopeInfo {
            scope_type: ScopeType::Block,
            start_line: Some(1),
            end_line: Some(10),
            parent_scope: Some(99),
            nesting_level: 100,
        };

        assert_eq!(scope.nesting_level, 100);
    }
}
