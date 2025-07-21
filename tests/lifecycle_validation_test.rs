use memscope_rs::{get_global_tracker, init, track_var};
use std::{fs, path::Path, thread, time::Duration};

// A simple struct to ensure we are tracking something with a known layout
struct TrackedData {
    _data: u64, // Ensure it has some size
}

fn main() {
    // Initialize the tracker
    println!("Initializing tracker...");
    init(); // This sets up the global allocator and tracing subscriber

    // --- Scenario Setup ---
    println!("Setting up scenario...");

    // Variable 1: Stays active
    let active_var = Box::new(TrackedData { _data: 1 });
    let _tracked_active_var = track_var!(active_var);
    println!("Tracked active_var");
    thread::sleep(Duration::from_millis(50)); // Ensure distinct timestamps

    // Variable 2: Allocated and then deallocated
    {
        let dealloc_var = String::from("This string will be deallocated");
        let _tracked_dealloc_var = track_var!(dealloc_var);
        println!("Tracked dealloc_var");
        thread::sleep(Duration::from_millis(50)); // Ensure distinct timestamps
                                                  // dealloc_var goes out of scope here and is dropped
        println!("dealloc_var going out of scope.");
    }
    thread::sleep(Duration::from_millis(50)); // Allow time for deallocation to be processed

    // Variable 3: Another active variable, different type
    let another_active_vec = vec![10, 20, 30, 40, 50];
    let _tracked_another_active_vec = track_var!(another_active_vec);
    println!("Tracked another_active_vec");
    thread::sleep(Duration::from_millis(50));

    // --- Export Data ---
    println!("Exporting data...");
    let tracker = get_global_tracker();
    let json_path = Path::new("test_output.json");
    let svg_path = Path::new("test_output.svg");

    if let Err(e) = tracker.export_to_json(json_path) {
        // Enable sync for reliable file writing
        eprintln!("Failed to export JSON: {e}");
        std::process::exit(1);
    }
    println!("Exported JSON to test_output.json");

    if let Err(e) = tracker.export_memory_analysis(svg_path) {
        // Enable sync for reliable file writing
        eprintln!("Failed to export SVG: {e}");
        std::process::exit(1);
    }
    println!("Exported SVG to test_output.svg");

    // --- Validation ---
    println!("Validating outputs...");

    // Validate JSON
    println!("Validating JSON: {}", json_path.display());
    let json_content = fs::read_to_string(json_path).expect("Failed to read test_output.json");
    assert!(!json_content.is_empty(), "JSON output is empty.");
    println!("JSON content length: {}", json_content.len());

    // Basic check for variable names (adjust based on actual naming in tracker)
    // Note: `track_var!` uses the variable identifier as the name.
    assert!(
        json_content.contains("active_var"),
        "JSON missing 'active_var'"
    );
    assert!(
        json_content.contains("another_active_vec"),
        "JSON missing 'another_active_vec'"
    );
    // `dealloc_var` should not be in active_allocations in JSON if JSON shows snapshot of active
    assert!(!json_content.contains("dealloc_var"), "JSON should not list 'dealloc_var' as active. Check if it's in the log instead if JSON format includes it.");
    println!("JSON basic content validated.");

    // Validate SVG
    println!("Validating SVG: {}", svg_path.display());
    let svg_content = fs::read_to_string(svg_path).expect("Failed to read test_output.svg");
    assert!(!svg_content.is_empty(), "SVG output is empty.");
    println!("SVG content length: {}", svg_content.len());

    assert!(svg_content.contains("<svg"), "SVG missing <svg> tag.");
    assert!(svg_content.contains("</svg>"), "SVG missing </svg> tag.");

    // Check for variable names (these should be present as text elements or in tooltips)
    assert!(
        svg_content.contains("active_var"),
        "SVG missing 'active_var'"
    );
    assert!(
        svg_content.contains("dealloc_var"),
        "SVG missing 'dealloc_var'"
    ); // Should be in SVG lifecycle
    assert!(
        svg_content.contains("another_active_vec"),
        "SVG missing 'another_active_vec'"
    );

    // Check for <rect> elements, indicating 그려진 bars
    assert!(
        svg_content.contains("<rect"),
        "SVG missing <rect> elements for lifecycle bars."
    );
    println!("SVG basic content validated.");

    println!("--- Test Program Completed Successfully ---");
}
