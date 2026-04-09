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
}
