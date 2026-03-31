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
}
