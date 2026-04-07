# Error Handling Module

## Overview

The error module provides unified error handling for the entire memscope-rs system. It defines structured error types, severity levels, and recovery strategies.

## Components

### 1. Error Types

**File**: `src/error/types.rs`

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

### 2. Error Handler

**File**: `src/error/handler.rs`

```rust
pub trait ErrorHandler: Send + Sync {
    fn handle_error(&self, error: &MemScopeError);
    fn handle_warning(&self, warning: &MemScopeError);
}

pub struct ErrorReporter {
    // Collects and reports errors
}
```

### 3. Recovery

**File**: `src/error/recovery.rs`

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

## Design Decisions

1. **Chained errors**: Source error preserved
2. **Severity levels**: Categorized by impact
3. **Recovery strategies**: Configurable error handling

## Usage

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

## Limitations

1. **No exception propagation**: Errors are values
2. **Limited recovery**: Most errors are unrecoverable
