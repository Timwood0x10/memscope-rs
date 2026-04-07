# 错误处理模块 (Error Handling Module)

## 概述

错误模块为整个 memscope-rs 系统提供统一的错误处理。它定义了结构化错误类型、严重级别和恢复策略。

## 组件

### 1. 错误类型

**文件**: `src/error/types.rs`

```rust
pub struct MemScopeError {
    pub kind: ErrorKind,
    pub severity: ErrorSeverity,
    pub context: ErrorContext,
    pub source: Option<Box<dyn std::error::Error>>,
}

pub enum ErrorKind {
    AllocationError,
    TrackingError,
    AnalysisError,
    RenderError,
    ExportError,
}

pub enum ErrorSeverity {
    Warning,
    Error,
    Critical,
}
```

### 2. 错误处理器

**文件**: `src/error/handler.rs`

```rust
pub trait ErrorHandler: Send + Sync {
    fn handle_error(&self, error: &MemScopeError);
    fn handle_warning(&self, warning: &MemScopeError);
}

pub struct ErrorReporter {
    // 收集并报告错误
}
```

### 3. 恢复

**文件**: `src/error/recovery.rs`

```rust
pub enum RecoveryAction {
    Retry,
    Skip,
    Abort,
    Recover,
}

pub trait RecoveryStrategy {
    fn try_recover(&self, error: &MemScopeError) -> RecoveryAction;
}
```

## 设计决策

1. **链式错误**: 保留源错误
2. **严重级别**: 按影响分类
3. **恢复策略**: 可配置的錯誤處理

## 使用

```rust
use memscope_rs::error::{MemScopeError, ErrorKind, ErrorSeverity};

fn handle_error() {
    let error = MemScopeError {
        kind: ErrorKind::AllocationError,
        severity: ErrorSeverity::Error,
        context: ErrorContext::new(),
        source: None,
    };
}
```

## 限制

1. **无异常传播**: 错误是值
2. **恢复有限**: 大多数错误无法恢复
