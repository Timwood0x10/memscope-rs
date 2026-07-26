//! Dashboard helper functions for Handlebars templates.

use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};

/// Format thread ID from raw format to user-friendly format.
pub fn format_thread_id(raw: &str) -> String {
    // Check for "ThreadId(n)" format with sufficient length
    // "ThreadId(" is 9 chars, ")" is 1 char, so minimum length is 11 (e.g., "ThreadId(0)")
    if raw.starts_with("ThreadId(") && raw.ends_with(')') && raw.len() > 10 {
        let num = &raw[9..raw.len() - 1];
        format!("Thread-{}", num)
    } else {
        raw.to_string()
    }
}

/// Format bytes to human-readable string.
pub fn format_bytes(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = KB * 1024;
    const GB: usize = MB * 1024;
    const TB: usize = GB * 1024;
    const PB: usize = TB * 1024;

    if bytes >= PB {
        format!("{:.2} PB", bytes as f64 / PB as f64)
    } else if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Register all Handlebars helpers.
pub fn register_helpers(handlebars: &mut Handlebars<'static>) {
    handlebars.register_helper("format_bytes", Box::new(format_bytes_helper));
    handlebars.register_helper("greater_than", Box::new(greater_than_helper));
    handlebars.register_helper("contains", Box::new(contains_helper));
    handlebars.register_helper("json", Box::new(json_helper));
    handlebars.register_helper("eq", Box::new(eq_helper));
    handlebars.register_helper("risk_class", Box::new(risk_class_helper));
    handlebars.register_helper("risk_label", Box::new(risk_label_helper));
    handlebars.register_helper("len", Box::new(len_helper));
}

/// Handlebars helper: format bytes to human-readable string.
fn format_bytes_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).unwrap().value();
    if let Some(bytes) = param.as_u64() {
        let formatted = format_bytes(bytes as usize);
        out.write(&formatted)?;
    }
    Ok(())
}

/// Handlebars helper: check if first value is greater than second.
fn greater_than_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param1 = h.param(0).unwrap().value();
    let param2 = h.param(1).unwrap().value();

    if let (Some(v1), Some(v2)) = (param1.as_u64(), param2.as_u64()) {
        if v1 > v2 {
            out.write("true")?;
        }
    }
    Ok(())
}

/// Handlebars helper: check if haystack contains needle.
fn contains_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let haystack = h.param(0).unwrap().value();
    let needle = h.param(1).unwrap().value();

    if let (Some(h_str), Some(n_str)) = (haystack.as_str(), needle.as_str()) {
        if h_str.contains(n_str) {
            out.write("true")?;
        }
    }
    Ok(())
}

/// Handlebars helper: serialize value to JSON string.
fn json_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).unwrap().value();
    let json_string = serde_json::to_string(param).map_err(|e| {
        handlebars::RenderErrorReason::Other(format!("Failed to serialize to JSON: {}", e))
    })?;
    out.write(&json_string)?;
    Ok(())
}

/// Handlebars helper: strict equality check between two values.
/// Writes "true" when both params are equal strings/numbers, otherwise writes
/// nothing (empty string is falsy under Handlebars `{{#if}}`).
fn eq_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let p1 = h.param(0).and_then(|p| p.value().as_str());
    let p2 = h.param(1).and_then(|p| p.value().as_str());
    if let (Some(a), Some(b)) = (p1, p2) {
        if a == b {
            out.write("true")?;
        }
    } else {
        let n1 = h.param(0).and_then(|p| p.value().as_u64());
        let n2 = h.param(1).and_then(|p| p.value().as_u64());
        if let (Some(a), Some(b)) = (n1, n2) {
            if a == b {
                out.write("true")?;
            }
        }
    }
    Ok(())
}

/// Handlebars helper: map a risk level string to a Tailwind color class set.
/// Usage: class="{{risk_class risk_level}}" → e.g. "bg-error/20 text-error"
fn risk_class_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let level = h
        .param(0)
        .and_then(|p| p.value().as_str())
        .unwrap_or("")
        .to_lowercase();
    let class = match level.as_str() {
        "high" | "critical" => "bg-error/20 text-error border-error/40",
        "medium" | "moderate" => "bg-warning/20 text-warning border-warning/40",
        "low" | "info" => "bg-success/20 text-success border-success/40",
        _ => "bg-surface-container-highest text-on-surface-variant border-outline-variant",
    };
    out.write(class)?;
    Ok(())
}

/// Handlebars helper: map a status string to a Tailwind color class set.
/// Useful for passport status, thread status, task status badges.
fn risk_label_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let label = h
        .param(0)
        .and_then(|p| p.value().as_str())
        .unwrap_or("")
        .to_lowercase();
    let class = if label.contains("leak")
        || label.contains("error")
        || label.contains("fail")
        || label.contains("foreign")
        || label.contains("zombie")
    {
        "bg-error/20 text-error border-error/40"
    } else if label.contains("active")
        || label.contains("running")
        || label.contains("process")
        || label.contains("wait")
        || label.contains("pending")
    {
        "bg-warning/20 text-warning border-warning/40"
    } else if label.contains("clean")
        || label.contains("ok")
        || label.contains("success")
        || label.contains("complete")
        || label.contains("done")
        || label.contains("idle")
    {
        "bg-success/20 text-success border-success/40"
    } else {
        "bg-surface-container-highest text-on-surface-variant border-outline-variant"
    };
    out.write(class)?;
    Ok(())
}

/// Handlebars helper: return the length of an array value.
/// Usage: {{len thread_policies}} → "3"
/// Works on top-level and nested arrays (e.g. {{len lifecycle_events}}).
fn len_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(param) = h.param(0) {
        let value = param.value();
        if let Some(arr) = value.as_array() {
            out.write(&arr.len().to_string())?;
            return Ok(());
        }
        // Objects with a known length-like field fall back to 0.
    }
    out.write("0")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_thread_id_valid() {
        assert_eq!(format_thread_id("ThreadId(1)"), "Thread-1");
        assert_eq!(format_thread_id("ThreadId(123)"), "Thread-123");
        assert_eq!(format_thread_id("ThreadId(0)"), "Thread-0");
    }

    #[test]
    fn test_format_thread_id_invalid() {
        assert_eq!(format_thread_id("ThreadId()"), "ThreadId()");
        assert_eq!(format_thread_id("ThreadId(1"), "ThreadId(1");
        assert_eq!(format_thread_id("ThreadId1)"), "ThreadId1)");
        assert_eq!(format_thread_id("Thread(1)"), "Thread(1)");
        assert_eq!(format_thread_id("custom_name"), "custom_name");
        assert_eq!(format_thread_id(""), "");
    }

    #[test]
    fn test_format_bytes_bytes() {
        assert_eq!(format_bytes(0), "0 bytes");
        assert_eq!(format_bytes(100), "100 bytes");
        assert_eq!(format_bytes(1023), "1023 bytes");
    }

    #[test]
    fn test_format_bytes_kb() {
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(10240), "10.00 KB");
    }

    #[test]
    fn test_format_bytes_mb() {
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 2), "2.00 MB");
        assert_eq!(format_bytes(1536 * 1024), "1.50 MB");
    }

    #[test]
    fn test_format_bytes_gb() {
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_bytes(1024 * 1024 * 1024 * 5), "5.00 GB");
    }

    #[test]
    fn test_format_bytes_tb() {
        assert_eq!(format_bytes(1024usize.pow(4)), "1.00 TB");
        assert_eq!(format_bytes(1024usize.pow(4) * 3), "3.00 TB");
    }

    #[test]
    fn test_format_bytes_pb() {
        assert_eq!(format_bytes(1024usize.pow(5)), "1.00 PB");
        assert_eq!(format_bytes(1024usize.pow(5) * 10), "10.00 PB");
    }

    #[test]
    fn test_register_helpers() {
        let mut handlebars = Handlebars::new();
        register_helpers(&mut handlebars);
    }
}
