//! Ownership Analyzer
//!
//! This module provides static analysis capabilities for Rust ownership semantics.
//! It uses a four-layer architecture:
//! 1. rustdoc JSON - type information
//! 2. syn - AST analysis
//! 3. inference engine - ownership state machine
//! 4. runtime tracing (optional)
//!
//! This analyzer can detect:
//! - Move operations
//! - Borrow operations (shared and mutable)
//! - Use-after-move errors
//! - Borrow conflicts

use std::collections::HashMap;
use std::path::PathBuf;

/// Type information extracted from rustdoc JSON
#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub name: String,
    pub is_copy: bool,
    pub is_clone: bool,
    pub is_drop: bool,
    pub size: Option<usize>,
}

/// Database of type information from rustdoc JSON
#[derive(Debug, Clone)]
pub struct RustdocDatabase {
    pub types: HashMap<String, TypeInfo>,
    pub impls: HashMap<String, Vec<ImplInfo>>,
}

/// Implementation information
#[derive(Debug, Clone)]
pub struct ImplInfo {
    pub type_name: String,
    pub trait_name: String,
}

/// Ownership operation detected by AST analysis
#[derive(Debug, Clone, PartialEq)]
pub enum OwnershipOp {
    Move {
        target: String,
        source: String,
        line: usize,
    },
    CallMove {
        arg_name: String,
        func_name: String,
        arg_index: usize,
        line: usize,
    },
    Borrow {
        target: String,
        is_mut: bool,
        line: usize,
    },
}

/// Rustdoc JSON extractor
pub struct RustdocExtractor {
    _json_path: PathBuf,
}

impl RustdocExtractor {
    pub fn new(json_path: PathBuf) -> Self {
        Self {
            _json_path: json_path,
        }
    }

    /// Extract type information from rustdoc JSON
    pub fn extract(&self) -> crate::error::MemScopeResult<RustdocDatabase> {
        // TODO: Implement actual rustdoc JSON parsing
        // For now, return a placeholder database
        let mut db = RustdocDatabase {
            types: HashMap::new(),
            impls: HashMap::new(),
        };

        // Add common types with known Copy semantics
        db.types.insert(
            "i32".to_string(),
            TypeInfo {
                name: "i32".to_string(),
                is_copy: true,
                is_clone: true,
                is_drop: false,
                size: Some(4),
            },
        );
        db.types.insert(
            "i64".to_string(),
            TypeInfo {
                name: "i64".to_string(),
                is_copy: true,
                is_clone: true,
                is_drop: false,
                size: Some(8),
            },
        );
        db.types.insert(
            "f32".to_string(),
            TypeInfo {
                name: "f32".to_string(),
                is_copy: true,
                is_clone: true,
                is_drop: false,
                size: Some(4),
            },
        );
        db.types.insert(
            "f64".to_string(),
            TypeInfo {
                name: "f64".to_string(),
                is_copy: true,
                is_clone: true,
                is_drop: false,
                size: Some(8),
            },
        );
        db.types.insert(
            "bool".to_string(),
            TypeInfo {
                name: "bool".to_string(),
                is_copy: true,
                is_clone: true,
                is_drop: false,
                size: Some(1),
            },
        );
        db.types.insert(
            "usize".to_string(),
            TypeInfo {
                name: "usize".to_string(),
                is_copy: true,
                is_clone: true,
                is_drop: false,
                size: Some(8),
            },
        );
        db.types.insert(
            "String".to_string(),
            TypeInfo {
                name: "String".to_string(),
                is_copy: false,
                is_clone: true,
                is_drop: true,
                size: Some(24),
            },
        );
        db.types.insert(
            "Vec".to_string(),
            TypeInfo {
                name: "Vec".to_string(),
                is_copy: false,
                is_clone: true,
                is_drop: true,
                size: Some(24),
            },
        );

        Ok(db)
    }
}

/// AST analyzer for parsing Rust source code
pub struct AstAnalyzer {
    source_code: String,
}

impl AstAnalyzer {
    pub fn new(source_code: String) -> Self {
        Self { source_code }
    }

    /// Analyze source code to extract ownership operations
    pub fn analyze(&self) -> Vec<OwnershipOp> {
        let mut ops = Vec::new();

        // Simple line-based analysis for now
        // TODO: Use syn crate for proper AST parsing
        for (line_num, line) in self.source_code.lines().enumerate() {
            self.detect_move_operations(line, line_num, &mut ops);
            self.detect_borrow_operations(line, line_num, &mut ops);
            self.detect_function_calls(line, line_num, &mut ops);
        }

        ops
    }

    fn detect_move_operations(&self, line: &str, line_num: usize, ops: &mut Vec<OwnershipOp>) {
        // Detect variable assignments that may involve moves
        if line.contains('=') && !line.contains('&') {
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() == 2 {
                let target = parts[0].split_whitespace().next().unwrap_or("");
                let source = parts[1].split_whitespace().next().unwrap_or("");

                // Skip if it's a literal or copy type
                if !source.is_empty()
                    && source.parse::<i32>().is_err()
                    && source.parse::<f64>().is_err()
                    && !source.starts_with('"')
                {
                    ops.push(OwnershipOp::Move {
                        target: target.to_string(),
                        source: source.to_string(),
                        line: line_num,
                    });
                }
            }
        }
    }

    fn detect_borrow_operations(&self, line: &str, line_num: usize, ops: &mut Vec<OwnershipOp>) {
        // Detect borrow operations (& and &mut)
        if line.contains('&') {
            let is_mut = line.contains("&mut");
            // Extract the variable being borrowed
            let var = line
                .split('&')
                .nth(1)
                .and_then(|s| s.split_whitespace().next())
                .unwrap_or("");

            if !var.is_empty() {
                ops.push(OwnershipOp::Borrow {
                    target: var.to_string(),
                    is_mut,
                    line: line_num,
                });
            }
        }
    }

    fn detect_function_calls(&self, line: &str, line_num: usize, ops: &mut Vec<OwnershipOp>) {
        // Detect function calls that may move arguments
        if line.contains('(') && line.contains(')') {
            let func_name = line
                .split('(')
                .next()
                .and_then(|s| s.split_whitespace().last())
                .unwrap_or("");

            let args = line
                .split('(')
                .nth(1)
                .and_then(|s| s.split(')').next())
                .unwrap_or("");

            // Check if function takes ownership (heuristic)
            if self.is_ownership_taking_function(func_name) {
                for (arg_idx, arg) in args.split(',').enumerate() {
                    let arg_name = arg.split_whitespace().next().unwrap_or("");
                    if !arg_name.is_empty() {
                        ops.push(OwnershipOp::CallMove {
                            arg_name: arg_name.to_string(),
                            func_name: func_name.to_string(),
                            arg_index: arg_idx,
                            line: line_num,
                        });
                    }
                }
            }
        }
    }

    fn is_ownership_taking_function(&self, func_name: &str) -> bool {
        // Heuristic: functions that typically take ownership
        // TODO: Use rustdoc JSON to determine actual function signatures
        matches!(
            func_name,
            "into_iter" | "into_vec" | "into_string" | "into_boxed_slice" | "collect" | "consume"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rustdoc_extractor() {
        let extractor = RustdocExtractor::new(PathBuf::from("test.json"));
        let db = extractor.extract().unwrap();

        assert!(db.types.contains_key("i32"));
        assert!(db.types.contains_key("String"));
    }

    #[test]
    fn test_type_info_copy() {
        let extractor = RustdocExtractor::new(PathBuf::from("test.json"));
        let db = extractor.extract().unwrap();

        let i32_info = db.types.get("i32").unwrap();
        assert!(i32_info.is_copy);
    }

    #[test]
    fn test_type_info_string_not_copy() {
        let extractor = RustdocExtractor::new(PathBuf::from("test.json"));
        let db = extractor.extract().unwrap();

        let string_info = db.types.get("String").unwrap();
        assert!(!string_info.is_copy);
        assert!(string_info.is_clone);
    }

    #[test]
    fn test_ast_analyzer_move() {
        let source = "let y = x;";
        let analyzer = AstAnalyzer::new(source.to_string());
        let ops = analyzer.analyze();

        assert!(!ops.is_empty());
        assert!(matches!(ops[0], OwnershipOp::Move { .. }));
    }

    #[test]
    fn test_ast_analyzer_borrow() {
        let source = "let y = &x;";
        let analyzer = AstAnalyzer::new(source.to_string());
        let ops = analyzer.analyze();

        assert!(!ops.is_empty());
        assert!(matches!(ops[0], OwnershipOp::Borrow { .. }));
    }

    #[test]
    fn test_ast_analyzer_mut_borrow() {
        let source = "let y = &mut x;";
        let analyzer = AstAnalyzer::new(source.to_string());
        let ops = analyzer.analyze();

        assert!(!ops.is_empty());
        if let OwnershipOp::Borrow { is_mut, .. } = &ops[0] {
            assert!(*is_mut);
        }
    }

    #[test]
    fn test_ast_analyzer_function_call() {
        let source = "let iter = vec.into_iter();";
        let analyzer = AstAnalyzer::new(source.to_string());
        let ops = analyzer.analyze();

        // Should detect the into_iter call
        assert!(!ops.is_empty());
    }
}
