//! Type and source-location inference helpers for the dashboard.
//!
//! These functions bridge the gap between raw allocation metadata
//! (size, pointer value) and human-readable type/source information
//! when explicit type names or stack traces are unavailable.

/// Infer a plausible type name from allocation size alone.
///
/// This is a heuristic fallback when no explicit type name was recorded.
/// It matches common Rust type sizes to likely candidates.
pub fn infer_type_from_size(size: usize) -> String {
    match size {
        8 => "*mut c_void (30%)".to_string(),
        16 => "&[T] (25%)".to_string(),
        24 => "Vec<_>/String (15%)".to_string(),
        32 | 48 | 64 => "CStruct (10%)".to_string(),
        n if n.is_power_of_two() && n >= 64 => {
            format!("Vec<_>/[u8] ({}%)", 10 + n.trailing_zeros() as u8)
        }
        n if (32..=256).contains(&n) => "[u8] (10%)".to_string(),
        _ => "unknown".to_string(),
    }
}

/// Get inferred type name, preferring an explicit name over size-based inference.
pub fn get_inferred_type_name(type_name: &str, size: usize) -> String {
    if type_name != "unknown" && type_name != "-" && !type_name.is_empty() {
        return type_name.to_string();
    }
    infer_type_from_size(size)
}

/// Extract the first user source file from a stack trace.
///
/// Filters out standard library frames, rustc internal paths,
/// and memscope-internal frames to find the user's code location.
#[allow(dead_code)]
pub fn extract_user_source_file(stack_trace: &Option<Vec<String>>) -> Option<String> {
    if let Some(ref frames) = stack_trace {
        for frame in frames {
            let frame_lower = frame.to_lowercase();
            if !frame_lower.contains("/rustc/")
                && !frame_lower.contains("/library/")
                && !frame_lower.contains("memscope")
                && !frame_lower.contains(".cargo/registry")
                && !frame_lower.contains("/src/core/")
                && !frame_lower.contains("/src/capture/")
                && !frame_lower.contains("/src/unified/")
                && !frame_lower.contains("/src/tracker")
            {
                if let Some(file_part) = frame.split(':').next() {
                    let file_name = file_part.split('/').next_back().unwrap_or(file_part);
                    if !file_name.starts_with('<') && file_name.contains(".rs") {
                        return Some(file_part.to_string());
                    }
                }
            }
        }
    }
    None
}

/// Extract the first user source line number from a stack trace.
#[allow(dead_code)]
pub fn extract_user_source_line(stack_trace: &Option<Vec<String>>) -> Option<u32> {
    if let Some(ref frames) = stack_trace {
        for frame in frames {
            let frame_lower = frame.to_lowercase();
            if !frame_lower.contains("/rustc/")
                && !frame_lower.contains("/library/")
                && !frame_lower.contains("memscope")
                && !frame_lower.contains(".cargo/registry")
                && !frame_lower.contains("/src/core/")
                && !frame_lower.contains("/src/capture/")
                && !frame_lower.contains("/src/unified/")
                && !frame_lower.contains("/src/tracker")
            {
                if let Some(line_part) = frame.rsplit(':').next() {
                    if let Ok(line) = line_part.parse::<u32>() {
                        return Some(line);
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Verify that infer_type_from_size returns correct types
    /// for common sizes.
    /// Invariants: Size 8 -> pointer, 16 -> slice, 24 -> Vec/String,
    /// 32/48/64 -> CStruct, power-of-two >= 64 -> Vec, 32..=256 non-power -> [u8].
    #[test]
    fn test_infer_type_from_size() {
        assert!(
            infer_type_from_size(8).contains("c_void"),
            "Size 8 should be pointer type"
        );
        assert!(
            infer_type_from_size(16).contains("[T]"),
            "Size 16 should be slice type"
        );
        assert!(
            infer_type_from_size(24).contains("Vec"),
            "Size 24 should be Vec/String type"
        );
        assert!(
            infer_type_from_size(32).contains("CStruct"),
            "Size 32 should be CStruct"
        );
        assert!(
            infer_type_from_size(128).contains("Vec"),
            "Size 128 should be Vec type"
        );
        assert!(
            infer_type_from_size(100).contains("[u8]"),
            "Size 100 should be byte array"
        );
        assert_eq!(
            infer_type_from_size(17),
            "unknown",
            "Size 17 should be unknown"
        );
    }

    /// Objective: Verify that get_inferred_type_name prefers explicit type names.
    /// Invariants: Non-empty, non-unknown type names are returned as-is.
    #[test]
    fn test_get_inferred_type_name_explicit() {
        let result = get_inferred_type_name("MyType", 100);
        assert_eq!(result, "MyType", "Should return explicit type name");
    }

    /// Objective: Verify that get_inferred_type_name falls back to inference
    /// for unknown type names.
    /// Invariants: "unknown", "-", and "" all trigger size-based inference.
    #[test]
    fn test_get_inferred_type_name_unknown() {
        let result = get_inferred_type_name("unknown", 24);
        assert!(result.contains("Vec"), "Should infer Vec type for size 24");
        let result = get_inferred_type_name("-", 8);
        assert!(
            result.contains("c_void"),
            "Should infer pointer type for size 8"
        );
        let result = get_inferred_type_name("", 16);
        assert!(
            result.contains("[T]"),
            "Should infer slice type for size 16"
        );
    }

    /// Objective: Verify extract_user_source_file filters correctly.
    /// Invariants: None for std/rustc/memscope frames, Some for user frames.
    #[test]
    fn test_extract_user_source_file() {
        let std_frames = Some(vec![
            "/rustc/library/alloc/src/vec.rs:100".to_string(),
            "/library/std/src/thread.rs:50".to_string(),
        ]);
        assert!(extract_user_source_file(&std_frames).is_none());

        let user_frames = Some(vec![
            "/rustc/library/alloc/src/vec.rs:100".to_string(),
            "/Users/test/my_project/src/main.rs:42".to_string(),
        ]);
        let result = extract_user_source_file(&user_frames);
        assert!(result.is_some());
        assert!(result.unwrap().contains("main.rs"));
    }

    /// Objective: Verify extract_user_source_line extracts correct line number.
    /// Invariants: Returns correct line number for user frames, None for std.
    #[test]
    fn test_extract_user_source_line() {
        let std_frames = Some(vec!["/rustc/library/alloc/src/vec.rs:100".to_string()]);
        assert!(extract_user_source_line(&std_frames).is_none());

        let user_frames = Some(vec![
            "/rustc/library/alloc/src/vec.rs:100".to_string(),
            "/Users/test/my_project/src/main.rs:42".to_string(),
        ]);
        let result = extract_user_source_line(&user_frames);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 42);
    }

    /// Objective: Verify extract_user_source_file handles None input.
    /// Invariants: Should return None.
    #[test]
    fn test_extract_user_source_file_none() {
        assert!(extract_user_source_file(&None).is_none());
    }

    /// Objective: Verify extract_user_source_line handles None input.
    /// Invariants: Should return None.
    #[test]
    fn test_extract_user_source_line_none() {
        assert!(extract_user_source_line(&None).is_none());
    }

    /// Objective: Verify extract_user_source_file filters memscope frames.
    /// Invariants: Should return None for memscope internal paths.
    #[test]
    fn test_extract_user_source_file_memscope() {
        let frames = Some(vec!["/Users/test/memscope/src/tracker.rs:100".to_string()]);
        assert!(extract_user_source_file(&frames).is_none());
    }

    /// Objective: Verify extract_user_source_line filters memscope frames.
    /// Invariants: Should return None for memscope internal paths.
    #[test]
    fn test_extract_user_source_line_memscope() {
        let frames = Some(vec!["/Users/test/memscope/src/tracker.rs:100".to_string()]);
        assert!(extract_user_source_line(&frames).is_none());
    }
}
