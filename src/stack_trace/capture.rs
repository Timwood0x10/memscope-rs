use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// Represents a single frame in a stack trace
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Raw instruction pointer address
    pub instruction_pointer: usize,
    /// Symbol name if resolved
    pub symbol_name: Option<String>,
    /// Source filename if available
    pub filename: Option<String>,
    /// Line number in source file
    pub line_number: Option<u32>,
    /// Function or method name
    pub function_name: Option<String>,
}

/// Configuration for stack trace capture behavior
#[derive(Debug, Clone)]
pub struct CaptureConfig {
    /// Maximum number of frames to capture
    pub max_depth: usize,
    /// Number of top frames to skip (e.g., allocator internals)
    pub skip_frames: usize,
    /// Whether to resolve symbol information
    pub enable_symbols: bool,
    /// Whether to cache resolved symbols for performance
    pub cache_symbols: bool,
    /// Whether to filter out system/library frames
    pub filter_system_frames: bool,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            max_depth: 32,
            skip_frames: 2,
            enable_symbols: true,
            cache_symbols: true,
            filter_system_frames: true,
        }
    }
}

/// High-performance stack trace capture engine
pub struct StackTraceCapture {
    /// Capture configuration settings
    config: CaptureConfig,
    /// Whether capture is currently enabled
    enabled: AtomicBool,
    /// Total number of captures performed
    capture_count: AtomicUsize,
    /// Cache of resolved stack frames for performance
    frame_cache: HashMap<usize, StackFrame>,
}

impl StackTraceCapture {
    /// Create new stack trace capture instance with given configuration
    pub fn new(config: CaptureConfig) -> Self {
        Self {
            config,
            enabled: AtomicBool::new(true),
            capture_count: AtomicUsize::new(0),
            frame_cache: HashMap::new(),
        }
    }

    /// Capture full stack trace with symbol resolution
    /// Returns None if capture is disabled
    pub fn capture(&mut self) -> Option<Vec<StackFrame>> {
        if !self.enabled.load(Ordering::Relaxed) {
            return None;
        }

        self.capture_count.fetch_add(1, Ordering::Relaxed);

        let mut frames = Vec::with_capacity(self.config.max_depth);
        let mut frame_count = 0;
        let mut skip_count = 0;

        // Simulate stack walking (simplified implementation)
        let mut current_ip = self.get_current_instruction_pointer();

        while frame_count < self.config.max_depth {
            if skip_count < self.config.skip_frames {
                skip_count += 1;
                current_ip = self.walk_stack_frame(current_ip)?;
                continue;
            }

            let frame = if let Some(cached_frame) = self.frame_cache.get(&current_ip) {
                cached_frame.clone()
            } else {
                let new_frame = self.create_frame(current_ip);
                if self.config.cache_symbols {
                    self.frame_cache.insert(current_ip, new_frame.clone());
                }
                new_frame
            };

            if self.should_include_frame(&frame) {
                frames.push(frame);
                frame_count += 1;
            }

            current_ip = self.walk_stack_frame(current_ip)?;
        }

        Some(frames)
    }

    /// Capture lightweight stack trace (instruction pointers only)
    /// Much faster than full capture, suitable for hot paths
    pub fn capture_lightweight(&self) -> Option<Vec<usize>> {
        if !self.enabled.load(Ordering::Relaxed) {
            return None;
        }

        let mut instruction_pointers = Vec::with_capacity(self.config.max_depth);
        let mut current_ip = self.get_current_instruction_pointer();
        let mut skip_count = 0;

        for _ in 0..self.config.max_depth {
            if skip_count < self.config.skip_frames {
                skip_count += 1;
                current_ip = self.walk_stack_frame(current_ip)?;
                continue;
            }

            instruction_pointers.push(current_ip);
            current_ip = self.walk_stack_frame(current_ip)?;
        }

        Some(instruction_pointers)
    }

    /// Enable stack trace capture
    pub fn enable(&self) {
        self.enabled.store(true, Ordering::Relaxed);
    }

    /// Disable stack trace capture for performance
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Relaxed);
    }

    /// Check if capture is currently enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Get total number of captures performed
    pub fn get_capture_count(&self) -> usize {
        self.capture_count.load(Ordering::Relaxed)
    }

    /// Clear the symbol resolution cache
    pub fn clear_cache(&mut self) {
        self.frame_cache.clear();
    }

    /// Get current size of symbol cache
    pub fn cache_size(&self) -> usize {
        self.frame_cache.len()
    }

    fn get_current_instruction_pointer(&self) -> usize {
        // Platform-specific implementation would go here
        // For now, return a mock value
        0x7fff_0000_0000
    }

    fn walk_stack_frame(&self, current_ip: usize) -> Option<usize> {
        // Platform-specific stack walking implementation
        // For now, simulate by decrementing
        if current_ip > 0x1000_0000 {
            Some(current_ip - 0x1000)
        } else {
            None
        }
    }

    fn create_frame(&self, ip: usize) -> StackFrame {
        let mut frame = StackFrame {
            instruction_pointer: ip,
            symbol_name: None,
            filename: None,
            line_number: None,
            function_name: None,
        };

        if self.config.enable_symbols {
            // Symbol resolution would happen here
            frame.function_name = self.resolve_function_name(ip);
            frame.filename = self.resolve_filename(ip);
            frame.line_number = self.resolve_line_number(ip);
        }

        frame
    }

    fn should_include_frame(&self, frame: &StackFrame) -> bool {
        if !self.config.filter_system_frames {
            return true;
        }

        // Filter out system/library frames
        if let Some(filename) = &frame.filename {
            if filename.contains("/usr/lib") || filename.contains("/lib64") {
                return false;
            }
        }

        if let Some(function_name) = &frame.function_name {
            if function_name.starts_with("__libc_") || function_name.starts_with("_start") {
                return false;
            }
        }

        true
    }

    fn resolve_function_name(&self, ip: usize) -> Option<String> {
        // Mock implementation - real version would use debug symbols
        match ip % 5 {
            0 => Some("main".to_string()),
            1 => Some("allocation_function".to_string()),
            2 => Some("process_data".to_string()),
            3 => Some("handle_request".to_string()),
            _ => Some(format!("function_{:x}", ip)),
        }
    }

    fn resolve_filename(&self, ip: usize) -> Option<String> {
        // Mock implementation
        match ip % 3 {
            0 => Some("src/main.rs".to_string()),
            1 => Some("src/lib.rs".to_string()),
            _ => Some("src/utils.rs".to_string()),
        }
    }

    fn resolve_line_number(&self, ip: usize) -> Option<u32> {
        // Mock implementation
        Some((ip % 1000) as u32 + 1)
    }
}

impl Default for StackTraceCapture {
    fn default() -> Self {
        Self::new(CaptureConfig::default())
    }
}

impl StackFrame {
    pub fn new(ip: usize) -> Self {
        Self {
            instruction_pointer: ip,
            symbol_name: None,
            filename: None,
            line_number: None,
            function_name: None,
        }
    }

    pub fn with_symbols(
        ip: usize,
        function_name: Option<String>,
        filename: Option<String>,
        line_number: Option<u32>,
    ) -> Self {
        Self {
            instruction_pointer: ip,
            symbol_name: function_name.clone(),
            filename,
            line_number,
            function_name,
        }
    }

    pub fn is_resolved(&self) -> bool {
        self.function_name.is_some() || self.filename.is_some()
    }

    pub fn display_name(&self) -> String {
        if let Some(func) = &self.function_name {
            if let (Some(file), Some(line)) = (&self.filename, self.line_number) {
                format!("{}() at {}:{}", func, file, line)
            } else {
                format!("{}()", func)
            }
        } else {
            format!("0x{:x}", self.instruction_pointer)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_capture() {
        let mut capture = StackTraceCapture::default();

        assert!(capture.is_enabled());

        let frames = capture.capture();
        assert!(frames.is_some());

        let frames = frames.expect("Should have frames");
        assert!(!frames.is_empty());
        assert!(frames.len() <= 32);
    }

    #[test]
    fn test_lightweight_capture() {
        let capture = StackTraceCapture::default();

        let ips = capture.capture_lightweight();
        assert!(ips.is_some());

        let ips = ips.expect("Should have IPs");
        assert!(!ips.is_empty());
    }

    #[test]
    fn test_enable_disable() {
        let capture = StackTraceCapture::default();

        assert!(capture.is_enabled());

        capture.disable();
        assert!(!capture.is_enabled());

        capture.enable();
        assert!(capture.is_enabled());
    }

    #[test]
    fn test_frame_creation() {
        let frame = StackFrame::new(0x1234);
        assert_eq!(frame.instruction_pointer, 0x1234);
        assert!(!frame.is_resolved());

        let resolved_frame = StackFrame::with_symbols(
            0x5678,
            Some("test_func".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );
        assert!(resolved_frame.is_resolved());
        assert_eq!(resolved_frame.display_name(), "test_func() at test.rs:42");
    }

    #[test]
    fn test_capture_count() {
        let mut capture = StackTraceCapture::default();

        assert_eq!(capture.get_capture_count(), 0);

        capture.capture();
        assert_eq!(capture.get_capture_count(), 1);

        capture.capture();
        assert_eq!(capture.get_capture_count(), 2);
    }

    #[test]
    fn test_custom_config() {
        let config = CaptureConfig {
            max_depth: 10,
            skip_frames: 1,
            enable_symbols: false,
            cache_symbols: false,
            filter_system_frames: false,
        };

        let mut capture = StackTraceCapture::new(config);

        if let Some(frames) = capture.capture() {
            assert!(frames.len() <= 10);
            // With symbols disabled, frames should not be resolved
            for frame in &frames {
                assert!(frame.function_name.is_none() || !frame.is_resolved());
            }
        }
    }
}
