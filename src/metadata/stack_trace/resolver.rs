use super::StackFrame;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Fully resolved stack frame with symbol information
#[derive(Debug, Clone)]
pub struct ResolvedFrame {
    /// Original instruction pointer address
    pub instruction_pointer: usize,
    /// Raw symbol name from debug info
    pub symbol_name: String,
    /// Human-readable demangled symbol name
    pub demangled_name: Option<String>,
    /// Source code filename
    pub filename: Option<String>,
    /// Line number in source file
    pub line_number: Option<u32>,
    /// Column number in source line
    pub column: Option<u32>,
    /// Name of module/library containing this symbol
    pub module_name: Option<String>,
    /// Offset from symbol start
    pub offset: Option<usize>,
}

/// High-performance symbol resolver with caching
pub struct SymbolResolver {
    /// Cache of resolved symbols by instruction pointer
    symbol_cache: HashMap<usize, ResolvedFrame>,
    /// Total number of resolution attempts
    resolution_count: AtomicUsize,
    /// Number of cache hits for performance tracking
    cache_hits: AtomicUsize,
    /// Whether to perform symbol demangling
    enable_demangling: bool,
    /// Whether to resolve line number information
    enable_line_info: bool,
}

impl SymbolResolver {
    /// Create new symbol resolver with default settings
    pub fn new() -> Self {
        Self {
            symbol_cache: HashMap::new(),
            resolution_count: AtomicUsize::new(0),
            cache_hits: AtomicUsize::new(0),
            enable_demangling: true,
            enable_line_info: true,
        }
    }

    /// Create symbol resolver with custom options
    pub fn with_options(enable_demangling: bool, enable_line_info: bool) -> Self {
        Self {
            symbol_cache: HashMap::new(),
            resolution_count: AtomicUsize::new(0),
            cache_hits: AtomicUsize::new(0),
            enable_demangling,
            enable_line_info,
        }
    }

    /// Resolve single stack frame to symbol information
    /// Uses cache for performance, falls back to symbol lookup
    pub fn resolve_frame(&mut self, frame: &StackFrame) -> Option<ResolvedFrame> {
        self.resolution_count.fetch_add(1, Ordering::Relaxed);

        // Check cache first
        if let Some(cached) = self.symbol_cache.get(&frame.instruction_pointer) {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
            return Some(cached.clone());
        }

        // Resolve symbol information
        let resolved = self.perform_resolution(frame.instruction_pointer)?;

        // Cache the result
        self.symbol_cache
            .insert(frame.instruction_pointer, resolved.clone());

        Some(resolved)
    }

    pub fn resolve_batch(&mut self, frames: &[StackFrame]) -> Vec<Option<ResolvedFrame>> {
        frames
            .iter()
            .map(|frame| self.resolve_frame(frame))
            .collect()
    }

    pub fn resolve_addresses(&mut self, addresses: &[usize]) -> Vec<Option<ResolvedFrame>> {
        addresses
            .iter()
            .map(|&addr| {
                let frame = StackFrame::new(addr);
                self.resolve_frame(&frame)
            })
            .collect()
    }

    pub fn get_cache_stats(&self) -> (usize, usize, f64) {
        let resolutions = self.resolution_count.load(Ordering::Relaxed);
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let hit_ratio = if resolutions > 0 {
            hits as f64 / resolutions as f64
        } else {
            0.0
        };
        (resolutions, hits, hit_ratio)
    }

    pub fn clear_cache(&mut self) {
        self.symbol_cache.clear();
        self.resolution_count.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
    }

    pub fn cache_size(&self) -> usize {
        self.symbol_cache.len()
    }

    pub fn preload_symbols(&mut self, addresses: &[usize]) {
        for &addr in addresses {
            if !self.symbol_cache.contains_key(&addr) {
                if let Some(resolved) = self.perform_resolution(addr) {
                    self.symbol_cache.insert(addr, resolved);
                }
            }
        }
    }

    fn perform_resolution(&self, address: usize) -> Option<ResolvedFrame> {
        // This would be platform-specific symbol resolution
        // For now, provide a mock implementation

        let symbol_name = self.lookup_symbol_name(address)?;
        let demangled_name = if self.enable_demangling {
            self.demangle_symbol(&symbol_name)
        } else {
            None
        };

        let (filename, line_number, column) = if self.enable_line_info {
            self.lookup_line_info(address)
        } else {
            (None, None, None)
        };

        let module_name = self.lookup_module_name(address);
        let offset = self.calculate_offset(address, &symbol_name);

        Some(ResolvedFrame {
            instruction_pointer: address,
            symbol_name,
            demangled_name,
            filename,
            line_number,
            column,
            module_name,
            offset,
        })
    }

    fn lookup_symbol_name(&self, _address: usize) -> Option<String> {
        // Real implementation would use debug info (DWARF, PDB, dSYM)
        // This feature is not yet implemented
        None
    }

    fn demangle_symbol(&self, _mangled: &str) -> Option<String> {
        // Real implementation should use rustc-demangle crate
        // This feature is not yet implemented
        None
    }

    fn lookup_line_info(&self, _address: usize) -> (Option<String>, Option<u32>, Option<u32>) {
        // Real implementation would use addr2line or debug info
        // This feature is not yet implemented
        (None, None, None)
    }

    fn lookup_module_name(&self, _address: usize) -> Option<String> {
        // Real implementation would use dladdr or equivalent
        // This feature is not yet implemented
        None
    }

    fn calculate_offset(&self, _address: usize, _symbol_name: &str) -> Option<usize> {
        // Real implementation would calculate offset within symbol based on debug info
        // This feature is not yet implemented
        None
    }
}

impl Default for SymbolResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl ResolvedFrame {
    pub fn display_name(&self) -> String {
        let name = self.demangled_name.as_ref().unwrap_or(&self.symbol_name);

        if let (Some(file), Some(line)) = (&self.filename, self.line_number) {
            if let Some(col) = self.column {
                format!("{} at {}:{}:{}", name, file, line, col)
            } else {
                format!("{} at {}:{}", name, file, line)
            }
        } else {
            name.clone()
        }
    }

    pub fn short_display(&self) -> String {
        self.demangled_name
            .as_ref()
            .unwrap_or(&self.symbol_name)
            .clone()
    }

    pub fn has_line_info(&self) -> bool {
        self.filename.is_some() && self.line_number.is_some()
    }

    pub fn is_rust_symbol(&self) -> bool {
        self.symbol_name.starts_with("_ZN")
            || self.demangled_name.as_ref().is_some_and(|name| {
                name.contains("::") || name.starts_with("std::") || name.starts_with("core::")
            })
    }

    pub fn is_system_symbol(&self) -> bool {
        self.module_name.as_ref().is_some_and(|module| {
            module.contains("libc") || module.contains("libpthread") || module.contains("ld-")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "linux")]
    fn test_basic_resolution() {
        let mut resolver = SymbolResolver::new();
        let frame = StackFrame::new(0x12345678);

        let resolved = resolver.resolve_frame(&frame);
        assert!(resolved.is_some());

        let resolved = resolved.expect("Should resolve");
        assert_eq!(resolved.instruction_pointer, 0x12345678);
        assert!(!resolved.symbol_name.is_empty());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_cache_functionality() {
        let mut resolver = SymbolResolver::new();
        let frame = StackFrame::new(0x12345678);

        let resolved1 = resolver.resolve_frame(&frame);
        assert!(resolved1.is_some());

        let resolved2 = resolver.resolve_frame(&frame);
        assert!(resolved2.is_some());

        let (resolutions, hits, hit_ratio) = resolver.get_cache_stats();
        assert_eq!(resolutions, 2);
        assert_eq!(hits, 1);
        assert!((hit_ratio - 0.5).abs() < 0.01);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_batch_resolution() {
        let mut resolver = SymbolResolver::new();
        let frames = vec![
            StackFrame::new(0x1000),
            StackFrame::new(0x2000),
            StackFrame::new(0x3000),
        ];

        let resolved = resolver.resolve_batch(&frames);
        assert_eq!(resolved.len(), 3);

        for result in resolved {
            assert!(result.is_some());
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_demangling() {
        let resolver = SymbolResolver::new();

        let demangled = resolver.demangle_symbol("_ZN4main17h1234567890abcdefE");
        assert_eq!(demangled, Some("main".to_string()));

        let not_mangled = resolver.demangle_symbol("regular_function");
        assert_eq!(not_mangled, None);
    }

    #[test]
    fn test_resolved_frame_display() {
        let frame = ResolvedFrame {
            instruction_pointer: 0x1234,
            symbol_name: "_ZN4main17h1234567890abcdefE".to_string(),
            demangled_name: Some("main".to_string()),
            filename: Some("src/main.rs".to_string()),
            line_number: Some(42),
            column: Some(10),
            module_name: Some("main_executable".to_string()),
            offset: Some(16),
        };

        assert_eq!(frame.display_name(), "main at src/main.rs:42:10");
        assert_eq!(frame.short_display(), "main");
        assert!(frame.has_line_info());
        assert!(frame.is_rust_symbol());
        assert!(!frame.is_system_symbol());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_preload_symbols() {
        let mut resolver = SymbolResolver::new();
        let addresses = vec![0x1000, 0x2000, 0x3000];

        assert_eq!(resolver.cache_size(), 0);

        resolver.preload_symbols(&addresses);
        assert_eq!(resolver.cache_size(), 3);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_clear_cache() {
        let mut resolver = SymbolResolver::new();
        let frame = StackFrame::new(0x12345678);

        resolver.resolve_frame(&frame);
        assert!(resolver.cache_size() > 0);

        resolver.clear_cache();
        assert_eq!(resolver.cache_size(), 0);

        let (resolutions, hits, _) = resolver.get_cache_stats();
        assert_eq!(resolutions, 0);
        assert_eq!(hits, 0);
    }
}
