//! Variable Registry - Variable metadata management
//!
//! This module provides variable registration and metadata tracking
//! for the MetadataEngine.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Variable information stored in registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableInfo {
    /// User-defined variable name
    pub var_name: String,
    /// Type name of the variable
    pub type_name: String,
    /// Timestamp when variable was registered
    pub timestamp: u64,
    /// Estimated size of the variable
    pub size: usize,
    /// Thread ID that created this variable
    pub thread_id: usize,
    /// Memory usage of this variable
    pub memory_usage: u64,
}

/// Variable Registry - manages variable address to name mappings
#[derive(Debug)]
pub struct VariableRegistry {
    /// Internal storage for variable info
    variables: Arc<Mutex<HashMap<usize, VariableInfo>>>,
}

impl VariableRegistry {
    /// Create a new VariableRegistry
    pub fn new() -> Self {
        Self {
            variables: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a variable with its address and information
    pub fn register_variable(
        &self,
        address: usize,
        var_name: String,
        type_name: String,
        size: usize,
    ) {
        let thread_id = self.hash_thread_id();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let var_info = VariableInfo {
            var_name,
            type_name,
            timestamp,
            size,
            thread_id,
            memory_usage: size as u64,
        };

        if let Ok(mut vars) = self.variables.lock() {
            vars.insert(address, var_info);
        }
    }

    /// Get variable information by address
    pub fn get_variable_info(&self, address: usize) -> Option<VariableInfo> {
        if let Ok(vars) = self.variables.lock() {
            vars.get(&address).cloned()
        } else {
            None
        }
    }

    /// Get all variables
    pub fn get_all_variables(&self) -> Vec<(usize, VariableInfo)> {
        if let Ok(vars) = self.variables.lock() {
            vars.iter().map(|(k, v)| (*k, v.clone())).collect()
        } else {
            Vec::new()
        }
    }

    /// Clear all variables
    pub fn clear(&self) {
        if let Ok(mut vars) = self.variables.lock() {
            vars.clear();
        }
    }

    /// Get the number of registered variables
    pub fn len(&self) -> usize {
        if let Ok(vars) = self.variables.lock() {
            vars.len()
        } else {
            0
        }
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Hash the current thread ID to a usize
    fn hash_thread_id(&self) -> usize {
        use std::hash::{Hash, Hasher};
        let thread_id = std::thread::current().id();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        thread_id.hash(&mut hasher);
        hasher.finish() as usize
    }
}

impl Default for VariableRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_registry_creation() {
        let registry = VariableRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_register_variable() {
        let registry = VariableRegistry::new();
        registry.register_variable(0x1000, "test_var".to_string(), "i32".to_string(), 4);
        assert_eq!(registry.len(), 1);

        let info = registry.get_variable_info(0x1000);
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.var_name, "test_var");
        assert_eq!(info.type_name, "i32");
        assert_eq!(info.size, 4);
    }

    #[test]
    fn test_get_all_variables() {
        let registry = VariableRegistry::new();
        registry.register_variable(0x1000, "var1".to_string(), "i32".to_string(), 4);
        registry.register_variable(0x2000, "var2".to_string(), "String".to_string(), 24);

        let all = registry.get_all_variables();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_clear() {
        let registry = VariableRegistry::new();
        registry.register_variable(0x1000, "test".to_string(), "i32".to_string(), 4);
        assert_eq!(registry.len(), 1);

        registry.clear();
        assert!(registry.is_empty());
    }
}
