//! Temporary object tracking types.
//!
//! This module contains types for tracking temporary objects,
//! their creation context, and usage patterns.

use serde::{Deserialize, Serialize};

use super::generic::SourceLocation;

/// Temporary object tracking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporaryObjectInfo {
    /// Temporary object identifier.
    pub temp_id: usize,
    /// Creation timestamp.
    pub created_at: u64,
    /// Destruction timestamp.
    pub destroyed_at: Option<u64>,
    /// Lifetime in nanoseconds.
    pub lifetime_ns: Option<u64>,
    /// Creation context.
    pub creation_context: CreationContext,
    /// Usage pattern.
    pub usage_pattern: TemporaryUsagePattern,
    /// Memory location type.
    pub location_type: MemoryLocationType,
}

/// Creation context for temporary objects.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreationContext {
    /// Function where created.
    pub function_name: String,
    /// Expression type that created the temporary.
    pub expression_type: ExpressionType,
    /// Source location.
    pub source_location: Option<SourceLocation>,
    /// Call stack at creation.
    pub call_stack: Vec<String>,
}

/// Expression type that creates temporaries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExpressionType {
    /// Function call expression.
    FunctionCall,
    /// Method call expression.
    MethodCall,
    /// Operator overload expression.
    OperatorOverload,
    /// Conversion expression.
    Conversion,
    /// Literal expression.
    Literal,
    /// Conditional expression.
    Conditional,
    /// Match expression.
    Match,
}

/// Temporary usage pattern.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TemporaryUsagePattern {
    /// Used immediately and discarded.
    Immediate,
    /// Passed to function.
    FunctionArgument,
    /// Used in expression chain.
    ExpressionChain,
    /// Stored temporarily.
    TemporaryStorage,
    /// Moved to permanent location.
    MovedToPermanent,
}

/// Memory location type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryLocationType {
    /// Stack memory location.
    Stack,
    /// Heap memory location.
    Heap,
    /// Register memory location.
    Register,
    /// Static memory location.
    Static,
    /// Thread-local memory location.
    ThreadLocal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temporary_object_info() {
        let info = TemporaryObjectInfo {
            temp_id: 1,
            created_at: 1000,
            destroyed_at: Some(2000),
            lifetime_ns: Some(1000),
            creation_context: CreationContext {
                function_name: "test_func".to_string(),
                expression_type: ExpressionType::FunctionCall,
                source_location: None,
                call_stack: vec![],
            },
            usage_pattern: TemporaryUsagePattern::Immediate,
            location_type: MemoryLocationType::Stack,
        };

        assert_eq!(info.temp_id, 1);
        assert!(info.destroyed_at.is_some());
    }

    #[test]
    fn test_expression_type() {
        let types = vec![
            ExpressionType::FunctionCall,
            ExpressionType::MethodCall,
            ExpressionType::OperatorOverload,
            ExpressionType::Conversion,
            ExpressionType::Literal,
        ];

        for expr_type in types {
            assert!(!format!("{expr_type:?}").is_empty());
        }
    }
}

// Implement From trait for converting from core::types to capture::types
impl From<crate::core::types::TemporaryObjectInfo> for TemporaryObjectInfo {
    fn from(old: crate::core::types::TemporaryObjectInfo) -> Self {
        Self {
            temp_id: old.temp_id,
            created_at: old.created_at,
            destroyed_at: old.destroyed_at,
            lifetime_ns: old.lifetime_ns,
            creation_context: CreationContext {
                function_name: old.creation_context.function_name,
                expression_type: match old.creation_context.expression_type {
                    crate::core::types::ExpressionType::FunctionCall => {
                        ExpressionType::FunctionCall
                    }
                    crate::core::types::ExpressionType::MethodCall => ExpressionType::MethodCall,
                    crate::core::types::ExpressionType::OperatorOverload => {
                        ExpressionType::OperatorOverload
                    }
                    crate::core::types::ExpressionType::Conversion => ExpressionType::Conversion,
                    crate::core::types::ExpressionType::Literal => ExpressionType::Literal,
                    crate::core::types::ExpressionType::Conditional => ExpressionType::Conditional,
                    crate::core::types::ExpressionType::Match => ExpressionType::Match,
                },
                source_location: old.creation_context.source_location.map(|loc| {
                    super::generic::SourceLocation {
                        file: loc.file,
                        line: loc.line,
                        column: loc.column,
                    }
                }),
                call_stack: old.creation_context.call_stack,
            },
            usage_pattern: match old.usage_pattern {
                crate::core::types::TemporaryUsagePattern::Immediate => {
                    TemporaryUsagePattern::Immediate
                }
                crate::core::types::TemporaryUsagePattern::FunctionArgument => {
                    TemporaryUsagePattern::FunctionArgument
                }
                crate::core::types::TemporaryUsagePattern::ExpressionChain => {
                    TemporaryUsagePattern::ExpressionChain
                }
                crate::core::types::TemporaryUsagePattern::TemporaryStorage => {
                    TemporaryUsagePattern::TemporaryStorage
                }
                crate::core::types::TemporaryUsagePattern::MovedToPermanent => {
                    TemporaryUsagePattern::MovedToPermanent
                }
            },
            location_type: match old.location_type {
                crate::core::types::MemoryLocationType::Stack => MemoryLocationType::Stack,
                crate::core::types::MemoryLocationType::Heap => MemoryLocationType::Heap,
                crate::core::types::MemoryLocationType::Register => MemoryLocationType::Register,
                crate::core::types::MemoryLocationType::Static => MemoryLocationType::Static,
                crate::core::types::MemoryLocationType::ThreadLocal => {
                    MemoryLocationType::ThreadLocal
                }
            },
        }
    }
}
