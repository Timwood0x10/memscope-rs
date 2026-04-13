//! Export engine module.

use crate::core::error::{MemScopeError, MemScopeResult};
use crate::view::MemoryView;
use std::path::Path;

/// Export engine.
///
/// Provides export functionality for analysis results.
pub struct ExportEngine<'a> {
    view: &'a MemoryView,
}

impl<'a> ExportEngine<'a> {
    /// Create new export engine.
    pub fn new(view: &'a MemoryView) -> Self {
        Self { view }
    }

    /// Export to JSON file.
    pub fn json<P: AsRef<Path>>(&self, path: P) -> MemScopeResult<()> {
        let json = self.to_json()?;
        std::fs::write(path, json)
            .map_err(|e| MemScopeError::export("json", format!("Failed to write JSON file: {}", e)))
    }

    /// Export to HTML dashboard.
    pub fn html<P: AsRef<Path>>(&self, path: P) -> MemScopeResult<()> {
        let html = self.to_html()?;
        std::fs::write(path, html)
            .map_err(|e| MemScopeError::export("html", format!("Failed to write HTML file: {}", e)))
    }

    /// Generate JSON string.
    pub fn to_json(&self) -> MemScopeResult<String> {
        let report = self.build_report();
        serde_json::to_string_pretty(&report)
            .map_err(|e| MemScopeError::export("json", format!("Failed to serialize JSON: {}", e)))
    }

    /// Generate HTML string with proper escaping.
    ///
    /// Uses HTML entity encoding to prevent XSS vulnerabilities.
    pub fn to_html(&self) -> MemScopeResult<String> {
        let report = self.build_report();
        let json = serde_json::to_string_pretty(&report).unwrap_or_default();
        let escaped_json = html_escape(&json);

        Ok(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Memory Analysis Report</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 2rem; }}
        h1 {{ color: #333; }}
        pre {{ background: #f5f5f5; padding: 1rem; border-radius: 4px; overflow-x: auto; }}
    </style>
</head>
<body>
    <h1>Memory Analysis Report</h1>
    <pre>{}</pre>
</body>
</html>"#,
            escaped_json
        ))
    }

    fn build_report(&self) -> serde_json::Value {
        serde_json::json!({
            "stats": {
                "allocation_count": self.view.len(),
                "total_bytes": self.view.total_memory(),
            },
            "snapshot": self.view.snapshot(),
        })
    }
}

/// Escape HTML special characters to prevent XSS.
///
/// Converts the following characters to HTML entities:
/// - `&` → `&amp;`
/// - `<` → `&lt;`
/// - `>` → `&gt;`
/// - `"` → `&quot;`
/// - `'` → `&#x27;`
fn html_escape(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&#x27;".to_string(),
            _ => c.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_store::MemoryEvent;

    #[test]
    fn test_export_to_json() {
        let events = vec![MemoryEvent::allocate(0x1000, 64, 1)];
        let view = MemoryView::from_events(events);
        let engine = ExportEngine::new(&view);
        let json = engine.to_json().expect("JSON export should succeed");
        assert!(json.contains("allocation_count"));
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("&"), "&amp;");
        assert_eq!(html_escape("<"), "&lt;");
        assert_eq!(html_escape(">"), "&gt;");
        assert_eq!(html_escape("\""), "&quot;");
        assert_eq!(html_escape("'"), "&#x27;");
        assert_eq!(html_escape("Hello <World>"), "Hello &lt;World&gt;");
    }

    #[test]
    fn test_html_escape_prevents_xss() {
        let malicious = "<script>alert('xss')</script>";
        let escaped = html_escape(malicious);
        assert!(!escaped.contains("<script>"));
        assert!(escaped.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_to_html_escapes_content() {
        let events = vec![MemoryEvent::allocate(0x1000, 64, 1)
            .with_var_name("<script>alert('xss')</script>".to_string())];
        let view = MemoryView::from_events(events);
        let engine = ExportEngine::new(&view);
        let html = engine.to_html().expect("HTML export should succeed");

        // Should not contain unescaped script tags
        assert!(!html.contains("<script>"));
        // Should contain escaped version
        assert!(html.contains("&lt;script&gt;"));
    }
}
