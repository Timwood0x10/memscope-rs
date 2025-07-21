//! Simple HTML Export Test
//! A minimal test to verify HTML export functionality

use memscope_rs::export_interactive_html;
use memscope_rs::{get_global_tracker, get_global_unsafe_ffi_tracker, init, track_var};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the memory tracking system
    init();

    println!("🔍 MemScope-RS Simple HTML Export Test");

    // Create a few simple allocations
    let numbers = vec![1, 2, 3, 4, 5];
    track_var!(numbers);

    let text = String::from("Hello, MemScope!");
    track_var!(text);

    let boxed_data = Box::new([0u8; 1024]);
    track_var!(boxed_data);

    // Get trackers
    let tracker = get_global_tracker();
    let unsafe_ffi_tracker = get_global_unsafe_ffi_tracker();

    // Export HTML report
    let html_path = "simple_memory_report.html";
    println!("📊 Exporting HTML report to: {html_path}");

    export_interactive_html(&tracker, Some(&unsafe_ffi_tracker), html_path)?;

    println!("✅ HTML report generated successfully!");
    println!("📂 Open '{html_path}' in your browser to view the report");

    Ok(())
}
