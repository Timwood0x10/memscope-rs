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

    #[test]
    fn test_temporary_usage_pattern_variants() {
        let patterns = vec![
            TemporaryUsagePattern::Immediate,
            TemporaryUsagePattern::FunctionArgument,
            TemporaryUsagePattern::ExpressionChain,
            TemporaryUsagePattern::TemporaryStorage,
            TemporaryUsagePattern::MovedToPermanent,
        ];

        for pattern in patterns {
            let info = TemporaryObjectInfo {
                temp_id: 0,
                created_at: 0,
                destroyed_at: None,
                lifetime_ns: None,
                creation_context: CreationContext {
                    function_name: String::new(),
                    expression_type: ExpressionType::Literal,
                    source_location: None,
                    call_stack: vec![],
                },
                usage_pattern: pattern.clone(),
                location_type: MemoryLocationType::Stack,
            };
            assert_eq!(info.usage_pattern, pattern);
        }
    }

    #[test]
    fn test_memory_location_type_variants() {
        let locations = vec![
            MemoryLocationType::Stack,
            MemoryLocationType::Heap,
            MemoryLocationType::Register,
            MemoryLocationType::Static,
            MemoryLocationType::ThreadLocal,
        ];

        for location in locations {
            let info = TemporaryObjectInfo {
                temp_id: 0,
                created_at: 0,
                destroyed_at: None,
                lifetime_ns: None,
                creation_context: CreationContext {
                    function_name: String::new(),
                    expression_type: ExpressionType::Literal,
                    source_location: None,
                    call_stack: vec![],
                },
                usage_pattern: TemporaryUsagePattern::Immediate,
                location_type: location.clone(),
            };
            assert_eq!(info.location_type, location);
        }
    }

    #[test]
    fn test_expression_type_all_variants() {
        let types = vec![
            ExpressionType::FunctionCall,
            ExpressionType::MethodCall,
            ExpressionType::OperatorOverload,
            ExpressionType::Conversion,
            ExpressionType::Literal,
            ExpressionType::Conditional,
            ExpressionType::Match,
        ];

        for expr_type in types {
            let ctx = CreationContext {
                function_name: String::new(),
                expression_type: expr_type.clone(),
                source_location: None,
                call_stack: vec![],
            };
            assert_eq!(ctx.expression_type, expr_type);
        }
    }

    #[test]
    fn test_creation_context_with_source_location() {
        let ctx = CreationContext {
            function_name: "test_fn".to_string(),
            expression_type: ExpressionType::MethodCall,
            source_location: Some(SourceLocation {
                file: "test.rs".to_string(),
                line: 42,
                column: 10,
            }),
            call_stack: vec!["main".to_string(), "test_fn".to_string()],
        };

        assert!(ctx.source_location.is_some());
        assert_eq!(ctx.call_stack.len(), 2);
    }

    #[test]
    fn test_creation_context_no_source_location() {
        let ctx = CreationContext {
            function_name: "simple".to_string(),
            expression_type: ExpressionType::Literal,
            source_location: None,
            call_stack: vec![],
        };

        assert!(ctx.source_location.is_none());
        assert!(ctx.call_stack.is_empty());
    }

    #[test]
    fn test_temporary_object_info_not_destroyed() {
        let info = TemporaryObjectInfo {
            temp_id: 2,
            created_at: 5000,
            destroyed_at: None,
            lifetime_ns: None,
            creation_context: CreationContext {
                function_name: "async_fn".to_string(),
                expression_type: ExpressionType::FunctionCall,
                source_location: None,
                call_stack: vec![],
            },
            usage_pattern: TemporaryUsagePattern::TemporaryStorage,
            location_type: MemoryLocationType::Heap,
        };

        assert!(info.destroyed_at.is_none());
        assert!(info.lifetime_ns.is_none());
    }

    #[test]
    fn test_temporary_object_info_serialization() {
        let info = TemporaryObjectInfo {
            temp_id: 100,
            created_at: 10000,
            destroyed_at: Some(15000),
            lifetime_ns: Some(5000),
            creation_context: CreationContext {
                function_name: "serialize_test".to_string(),
                expression_type: ExpressionType::Conversion,
                source_location: Some(SourceLocation {
                    file: "lib.rs".to_string(),
                    line: 1,
                    column: 1,
                }),
                call_stack: vec!["main".to_string()],
            },
            usage_pattern: TemporaryUsagePattern::ExpressionChain,
            location_type: MemoryLocationType::Stack,
        };

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: TemporaryObjectInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.temp_id, info.temp_id);
        assert_eq!(deserialized.created_at, info.created_at);
    }

    #[test]
    fn test_creation_context_serialization() {
        let ctx = CreationContext {
            function_name: "test".to_string(),
            expression_type: ExpressionType::OperatorOverload,
            source_location: None,
            call_stack: vec!["a".to_string(), "b".to_string()],
        };

        let json = serde_json::to_string(&ctx).unwrap();
        let deserialized: CreationContext = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.function_name, ctx.function_name);
        assert_eq!(deserialized.call_stack.len(), 2);
    }

    #[test]
    fn test_expression_type_serialization() {
        let types = vec![
            ExpressionType::FunctionCall,
            ExpressionType::MethodCall,
            ExpressionType::OperatorOverload,
            ExpressionType::Conversion,
            ExpressionType::Literal,
            ExpressionType::Conditional,
            ExpressionType::Match,
        ];

        for expr_type in types {
            let json = serde_json::to_string(&expr_type).unwrap();
            let deserialized: ExpressionType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, expr_type);
        }
    }

    #[test]
    fn test_temporary_usage_pattern_serialization() {
        let patterns = vec![
            TemporaryUsagePattern::Immediate,
            TemporaryUsagePattern::FunctionArgument,
            TemporaryUsagePattern::ExpressionChain,
            TemporaryUsagePattern::TemporaryStorage,
            TemporaryUsagePattern::MovedToPermanent,
        ];

        for pattern in patterns {
            let json = serde_json::to_string(&pattern).unwrap();
            let deserialized: TemporaryUsagePattern = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, pattern);
        }
    }

    #[test]
    fn test_memory_location_type_serialization() {
        let locations = vec![
            MemoryLocationType::Stack,
            MemoryLocationType::Heap,
            MemoryLocationType::Register,
            MemoryLocationType::Static,
            MemoryLocationType::ThreadLocal,
        ];

        for location in locations {
            let json = serde_json::to_string(&location).unwrap();
            let deserialized: MemoryLocationType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, location);
        }
    }

    #[test]
    fn test_temporary_object_info_clone() {
        let info = TemporaryObjectInfo {
            temp_id: 1,
            created_at: 0,
            destroyed_at: None,
            lifetime_ns: None,
            creation_context: CreationContext {
                function_name: "clone_test".to_string(),
                expression_type: ExpressionType::Literal,
                source_location: None,
                call_stack: vec![],
            },
            usage_pattern: TemporaryUsagePattern::Immediate,
            location_type: MemoryLocationType::Stack,
        };

        let cloned = info.clone();
        assert_eq!(cloned.temp_id, info.temp_id);
    }

    #[test]
    fn test_creation_context_clone() {
        let ctx = CreationContext {
            function_name: "clone_ctx".to_string(),
            expression_type: ExpressionType::Match,
            source_location: Some(SourceLocation {
                file: "clone.rs".to_string(),
                line: 10,
                column: 5,
            }),
            call_stack: vec!["caller".to_string()],
        };

        let cloned = ctx.clone();
        assert_eq!(cloned.function_name, ctx.function_name);
    }

    #[test]
    fn test_temporary_object_info_debug() {
        let info = TemporaryObjectInfo {
            temp_id: 1,
            created_at: 0,
            destroyed_at: None,
            lifetime_ns: None,
            creation_context: CreationContext {
                function_name: "debug_test".to_string(),
                expression_type: ExpressionType::Literal,
                source_location: None,
                call_stack: vec![],
            },
            usage_pattern: TemporaryUsagePattern::Immediate,
            location_type: MemoryLocationType::Stack,
        };

        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("TemporaryObjectInfo"));
        assert!(debug_str.contains("temp_id"));
    }

    #[test]
    fn test_creation_context_debug() {
        let ctx = CreationContext {
            function_name: "debug_ctx".to_string(),
            expression_type: ExpressionType::Conditional,
            source_location: None,
            call_stack: vec![],
        };

        let debug_str = format!("{:?}", ctx);
        assert!(debug_str.contains("CreationContext"));
        assert!(debug_str.contains("function_name"));
    }

    #[test]
    fn test_temporary_object_info_equality() {
        let info1 = TemporaryObjectInfo {
            temp_id: 1,
            created_at: 100,
            destroyed_at: Some(200),
            lifetime_ns: Some(100),
            creation_context: CreationContext {
                function_name: "test".to_string(),
                expression_type: ExpressionType::Literal,
                source_location: None,
                call_stack: vec![],
            },
            usage_pattern: TemporaryUsagePattern::Immediate,
            location_type: MemoryLocationType::Stack,
        };

        let info2 = TemporaryObjectInfo {
            temp_id: 1,
            created_at: 100,
            destroyed_at: Some(200),
            lifetime_ns: Some(100),
            creation_context: CreationContext {
                function_name: "test".to_string(),
                expression_type: ExpressionType::Literal,
                source_location: None,
                call_stack: vec![],
            },
            usage_pattern: TemporaryUsagePattern::Immediate,
            location_type: MemoryLocationType::Stack,
        };

        assert_eq!(info1, info2);
    }

    #[test]
    fn test_creation_context_equality() {
        let ctx1 = CreationContext {
            function_name: "test".to_string(),
            expression_type: ExpressionType::FunctionCall,
            source_location: None,
            call_stack: vec![],
        };

        let ctx2 = CreationContext {
            function_name: "test".to_string(),
            expression_type: ExpressionType::FunctionCall,
            source_location: None,
            call_stack: vec![],
        };

        assert_eq!(ctx1, ctx2);
    }

    #[test]
    fn test_boundary_values_temporary() {
        let info = TemporaryObjectInfo {
            temp_id: usize::MAX,
            created_at: u64::MAX,
            destroyed_at: Some(u64::MAX),
            lifetime_ns: Some(u64::MAX),
            creation_context: CreationContext {
                function_name: String::new(),
                expression_type: ExpressionType::Literal,
                source_location: None,
                call_stack: vec![],
            },
            usage_pattern: TemporaryUsagePattern::Immediate,
            location_type: MemoryLocationType::Stack,
        };

        assert_eq!(info.temp_id, usize::MAX);
        assert_eq!(info.created_at, u64::MAX);
    }

    #[test]
    fn test_long_call_stack() {
        let call_stack: Vec<String> = (0..100).map(|i| format!("func_{}", i)).collect();

        let ctx = CreationContext {
            function_name: "deep_call".to_string(),
            expression_type: ExpressionType::FunctionCall,
            source_location: None,
            call_stack: call_stack.clone(),
        };

        assert_eq!(ctx.call_stack.len(), 100);
    }

    #[test]
    fn test_zero_lifetime() {
        let info = TemporaryObjectInfo {
            temp_id: 1,
            created_at: 1000,
            destroyed_at: Some(1000),
            lifetime_ns: Some(0),
            creation_context: CreationContext {
                function_name: "instant".to_string(),
                expression_type: ExpressionType::Literal,
                source_location: None,
                call_stack: vec![],
            },
            usage_pattern: TemporaryUsagePattern::Immediate,
            location_type: MemoryLocationType::Register,
        };

        assert_eq!(info.lifetime_ns, Some(0));
    }

    #[test]
    fn test_heap_temporary() {
        let info = TemporaryObjectInfo {
            temp_id: 1,
            created_at: 0,
            destroyed_at: None,
            lifetime_ns: None,
            creation_context: CreationContext {
                function_name: "alloc".to_string(),
                expression_type: ExpressionType::FunctionCall,
                source_location: None,
                call_stack: vec![],
            },
            usage_pattern: TemporaryUsagePattern::MovedToPermanent,
            location_type: MemoryLocationType::Heap,
        };

        assert_eq!(info.location_type, MemoryLocationType::Heap);
    }

    #[test]
    fn test_static_temporary() {
        let info = TemporaryObjectInfo {
            temp_id: 1,
            created_at: 0,
            destroyed_at: None,
            lifetime_ns: None,
            creation_context: CreationContext {
                function_name: "static_fn".to_string(),
                expression_type: ExpressionType::Literal,
                source_location: None,
                call_stack: vec![],
            },
            usage_pattern: TemporaryUsagePattern::TemporaryStorage,
            location_type: MemoryLocationType::Static,
        };

        assert_eq!(info.location_type, MemoryLocationType::Static);
    }

    #[test]
    fn test_thread_local_temporary() {
        let info = TemporaryObjectInfo {
            temp_id: 1,
            created_at: 0,
            destroyed_at: None,
            lifetime_ns: None,
            creation_context: CreationContext {
                function_name: "thread_fn".to_string(),
                expression_type: ExpressionType::FunctionCall,
                source_location: None,
                call_stack: vec![],
            },
            usage_pattern: TemporaryUsagePattern::TemporaryStorage,
            location_type: MemoryLocationType::ThreadLocal,
        };

        assert_eq!(info.location_type, MemoryLocationType::ThreadLocal);
    }
}
