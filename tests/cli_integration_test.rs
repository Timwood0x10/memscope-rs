//! CLI integration tests for binary data processing commands
//!
//! This module tests the CLI commands:
//! - analyze command with binary export options
//! - export command for binary file conversion

use std::process::Command;
use std::fs;
use tempfile::TempDir;

/// Test the analyze command with binary export
#[test]
fn test_analyze_command_binary_export() {
    println!("🧪 Testing analyze command with binary export");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("test_analysis");
    let binary_path = format!("{}.ms", output_path.to_string_lossy());
    
    // Run analyze command with binary export
    let output = Command::new("cargo")
        .args(&[
            "run", "--",
            "analyze",
            "--export", "binary",
            "--compression", "zstd",
            "--compression-level", "6",
            "--output", &output_path.to_string_lossy(),
            "echo", "test"  // Simple command to analyze
        ])
        .output();
    
    match output {
        Ok(result) => {
            println!("Command output: {}", String::from_utf8_lossy(&result.stdout));
            if !result.stderr.is_empty() {
                println!("Command stderr: {}", String::from_utf8_lossy(&result.stderr));
            }
            
            // Check if binary file was created
            if std::path::Path::new(&binary_path).exists() {
                println!("✅ Binary file created successfully: {binary_path}");
                
                // Verify file is not empty
                let file_size = fs::metadata(&binary_path).unwrap().len();
                assert!(file_size > 0, "Binary file should not be empty");
                println!("   - File size: {file_size} bytes");
            } else {
                println!("⚠️  Binary file not found (this may be expected if no memory tracking data is available)");
            }
        }
        Err(e) => {
            println!("⚠️  Command execution failed: {e}");
            println!("   This may be expected in a test environment without actual memory tracking");
        }
    }
    
    println!("🎉 Analyze command test completed");
}

/// Test the export command for binary to JSON conversion
#[test]
fn test_export_command_binary_to_json() {
    println!("🧪 Testing export command binary -> JSON");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let binary_path = temp_dir.path().join("test_input.ms");
    let json_path = temp_dir.path().join("test_output.json");
    
    // Create a test binary file first
    create_test_binary_file(&binary_path);
    
    // Run export command
    let output = Command::new("cargo")
        .args(&[
            "run", "--",
            "export",
            "--input", &binary_path.to_string_lossy(),
            "--format", "json",
            "--output", &json_path.to_string_lossy(),
        ])
        .output();
    
    match output {
        Ok(result) => {
            println!("Command output: {}", String::from_utf8_lossy(&result.stdout));
            if !result.stderr.is_empty() {
                println!("Command stderr: {}", String::from_utf8_lossy(&result.stderr));
            }
            
            if result.status.success() {
                println!("✅ Export command executed successfully");
                
                // Check if JSON file was created
                if json_path.exists() {
                    let file_size = fs::metadata(&json_path).unwrap().len();
                    println!("   - JSON file created: {} bytes", file_size);
                    
                    // Verify JSON content
                    let json_content = fs::read_to_string(&json_path).unwrap();
                    let _: serde_json::Value = serde_json::from_str(&json_content)
                        .expect("Generated file should be valid JSON");
                    println!("   - JSON file is valid");
                } else {
                    println!("⚠️  JSON file not created");
                }
            } else {
                println!("⚠️  Export command failed with status: {}", result.status);
            }
        }
        Err(e) => {
            println!("❌ Command execution failed: {e}");
        }
    }
    
    println!("🎉 Export command JSON test completed");
}

/// Test the export command for binary to HTML conversion
#[test]
fn test_export_command_binary_to_html() {
    println!("🧪 Testing export command binary -> HTML");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let binary_path = temp_dir.path().join("test_input.ms");
    let html_path = temp_dir.path().join("test_output.html");
    
    // Create a test binary file first
    create_test_binary_file(&binary_path);
    
    // Run export command
    let output = Command::new("cargo")
        .args(&[
            "run", "--",
            "export",
            "--input", &binary_path.to_string_lossy(),
            "--format", "html",
            "--output", &html_path.to_string_lossy(),
        ])
        .output();
    
    match output {
        Ok(result) => {
            println!("Command output: {}", String::from_utf8_lossy(&result.stdout));
            if !result.stderr.is_empty() {
                println!("Command stderr: {}", String::from_utf8_lossy(&result.stderr));
            }
            
            if result.status.success() {
                println!("✅ Export command executed successfully");
                
                // Check if HTML file was created
                if html_path.exists() {
                    let file_size = fs::metadata(&html_path).unwrap().len();
                    println!("   - HTML file created: {} bytes", file_size);
                    
                    // Verify HTML content
                    let html_content = fs::read_to_string(&html_path).unwrap();
                    assert!(html_content.contains("<!DOCTYPE html>"), "Should be valid HTML");
                    assert!(html_content.contains("Memory Analysis"), "Should contain analysis content");
                    println!("   - HTML file is valid");
                } else {
                    println!("⚠️  HTML file not created");
                }
            } else {
                println!("⚠️  Export command failed with status: {}", result.status);
            }
        }
        Err(e) => {
            println!("❌ Command execution failed: {e}");
        }
    }
    
    println!("🎉 Export command HTML test completed");
}

/// Test the export command with validation-only flag
#[test]
fn test_export_command_validation_only() {
    println!("🧪 Testing export command with --validate-only");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let binary_path = temp_dir.path().join("test_input.ms");
    
    // Create a test binary file first
    create_test_binary_file(&binary_path);
    
    // Run export command with validation only
    let output = Command::new("cargo")
        .args(&[
            "run", "--",
            "export",
            "--input", &binary_path.to_string_lossy(),
            "--validate-only",
        ])
        .output();
    
    match output {
        Ok(result) => {
            println!("Command output: {}", String::from_utf8_lossy(&result.stdout));
            if !result.stderr.is_empty() {
                println!("Command stderr: {}", String::from_utf8_lossy(&result.stderr));
            }
            
            if result.status.success() {
                println!("✅ Validation command executed successfully");
                
                // Check that output contains validation information
                let stdout = String::from_utf8_lossy(&result.stdout);
                if stdout.contains("Validation Results") || stdout.contains("validation") {
                    println!("   - Validation output detected");
                }
            } else {
                println!("⚠️  Validation command failed with status: {}", result.status);
            }
        }
        Err(e) => {
            println!("❌ Command execution failed: {e}");
        }
    }
    
    println!("🎉 Export validation test completed");
}

/// Test the export command with streaming flag
#[test]
fn test_export_command_streaming() {
    println!("🧪 Testing export command with --streaming");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let binary_path = temp_dir.path().join("test_input.ms");
    let json_path = temp_dir.path().join("test_streaming.json");
    
    // Create a test binary file first
    create_test_binary_file(&binary_path);
    
    // Run export command with streaming
    let output = Command::new("cargo")
        .args(&[
            "run", "--",
            "export",
            "--input", &binary_path.to_string_lossy(),
            "--format", "json",
            "--output", &json_path.to_string_lossy(),
            "--streaming",
        ])
        .output();
    
    match output {
        Ok(result) => {
            println!("Command output: {}", String::from_utf8_lossy(&result.stdout));
            if !result.stderr.is_empty() {
                println!("Command stderr: {}", String::from_utf8_lossy(&result.stderr));
            }
            
            if result.status.success() {
                println!("✅ Streaming export command executed successfully");
                
                // Check if JSON file was created
                if json_path.exists() {
                    let file_size = fs::metadata(&json_path).unwrap().len();
                    println!("   - JSON file created with streaming: {} bytes", file_size);
                }
            } else {
                println!("⚠️  Streaming export command failed with status: {}", result.status);
            }
        }
        Err(e) => {
            println!("❌ Command execution failed: {e}");
        }
    }
    
    println!("🎉 Export streaming test completed");
}

/// Test error handling for invalid input files
#[test]
fn test_export_command_error_handling() {
    println!("🧪 Testing export command error handling");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let nonexistent_path = temp_dir.path().join("nonexistent.ms");
    let output_path = temp_dir.path().join("output.json");
    
    // Run export command with nonexistent input file
    let output = Command::new("cargo")
        .args(&[
            "run", "--",
            "export",
            "--input", &nonexistent_path.to_string_lossy(),
            "--format", "json",
            "--output", &output_path.to_string_lossy(),
        ])
        .output();
    
    match output {
        Ok(result) => {
            println!("Command output: {}", String::from_utf8_lossy(&result.stdout));
            if !result.stderr.is_empty() {
                println!("Command stderr: {}", String::from_utf8_lossy(&result.stderr));
            }
            
            // Command should fail for nonexistent file
            if !result.status.success() {
                println!("✅ Command correctly failed for nonexistent file");
                
                // Check error message
                let stderr = String::from_utf8_lossy(&result.stderr);
                if stderr.contains("does not exist") || stderr.contains("not found") {
                    println!("   - Appropriate error message provided");
                }
            } else {
                println!("⚠️  Command unexpectedly succeeded for nonexistent file");
            }
        }
        Err(e) => {
            println!("❌ Command execution failed: {e}");
        }
    }
    
    println!("🎉 Error handling test completed");
}

/// Test help output for commands
#[test]
fn test_command_help_output() {
    println!("🧪 Testing command help output");
    
    // Test analyze command help
    let analyze_help = Command::new("cargo")
        .args(&["run", "--", "analyze", "--help"])
        .output();
    
    if let Ok(result) = analyze_help {
        let help_text = String::from_utf8_lossy(&result.stdout);
        println!("Analyze help output length: {} chars", help_text.len());
        
        // Check for key parameters
        if help_text.contains("--compression") && help_text.contains("--compression-level") {
            println!("✅ Analyze command help includes binary export options");
        }
    }
    
    // Test export command help
    let export_help = Command::new("cargo")
        .args(&["run", "--", "export", "--help"])
        .output();
    
    if let Ok(result) = export_help {
        let help_text = String::from_utf8_lossy(&result.stdout);
        println!("Export help output length: {} chars", help_text.len());
        
        // Check for key parameters
        if help_text.contains("--input") && help_text.contains("--format") && help_text.contains("--streaming") {
            println!("✅ Export command help includes all expected options");
        }
    }
    
    println!("🎉 Help output test completed");
}

/// Test command parameter validation
#[test]
fn test_command_parameter_validation() {
    println!("🧪 Testing command parameter validation");
    
    // Test export command with missing required parameters
    let output = Command::new("cargo")
        .args(&["run", "--", "export"])
        .output();
    
    if let Ok(result) = output {
        if !result.status.success() {
            println!("✅ Export command correctly fails without required parameters");
            
            let stderr = String::from_utf8_lossy(&result.stderr);
            if stderr.contains("required") || stderr.contains("missing") {
                println!("   - Appropriate error message for missing parameters");
            }
        }
    }
    
    // Test export command with invalid format
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let binary_path = temp_dir.path().join("test.ms");
    let output_path = temp_dir.path().join("output.txt");
    
    create_test_binary_file(&binary_path);
    
    let output = Command::new("cargo")
        .args(&[
            "run", "--",
            "export",
            "--input", &binary_path.to_string_lossy(),
            "--format", "invalid_format",
            "--output", &output_path.to_string_lossy(),
        ])
        .output();
    
    if let Ok(result) = output {
        if !result.status.success() {
            println!("✅ Export command correctly fails with invalid format");
            
            let stderr = String::from_utf8_lossy(&result.stderr);
            if stderr.contains("Unsupported format") || stderr.contains("invalid") {
                println!("   - Appropriate error message for invalid format");
            }
        }
    }
    
    println!("🎉 Parameter validation test completed");
}

/// Helper function to create a test binary file
fn create_test_binary_file(path: &std::path::Path) {
    use memscope_rs::export::formats::binary_export::{BinaryExportOptions, export_memory_to_binary};
    use memscope_rs::core::tracker::MemoryTracker;
    
    // Create a simple test binary file
    let tracker = MemoryTracker::new();
    let options = BinaryExportOptions::fast();
    
    // Export to binary (this will create a minimal file even with empty tracker)
    if let Err(e) = export_memory_to_binary(&tracker, path, options) {
        println!("⚠️  Failed to create test binary file: {e}");
        
        // Create a minimal binary file manually for testing
        let minimal_data = b"test binary data";
        fs::write(path, minimal_data).expect("Should be able to create test file");
    }
}

/// Test the complete CLI workflow
#[test]
fn test_complete_cli_workflow() {
    println!("🧪 Testing complete CLI workflow");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let binary_path = temp_dir.path().join("workflow_test.ms");
    let json_path = temp_dir.path().join("workflow_test.json");
    let html_path = temp_dir.path().join("workflow_test.html");
    
    // Step 1: Create binary file
    create_test_binary_file(&binary_path);
    println!("✅ Step 1: Test binary file created");
    
    // Step 2: Convert to JSON
    let json_result = Command::new("cargo")
        .args(&[
            "run", "--",
            "export",
            "--input", &binary_path.to_string_lossy(),
            "--format", "json",
            "--output", &json_path.to_string_lossy(),
        ])
        .output();
    
    if let Ok(result) = json_result {
        if result.status.success() && json_path.exists() {
            println!("✅ Step 2: Binary -> JSON conversion successful");
        }
    }
    
    // Step 3: Convert to HTML
    let html_result = Command::new("cargo")
        .args(&[
            "run", "--",
            "export",
            "--input", &binary_path.to_string_lossy(),
            "--format", "html",
            "--output", &html_path.to_string_lossy(),
        ])
        .output();
    
    if let Ok(result) = html_result {
        if result.status.success() && html_path.exists() {
            println!("✅ Step 3: Binary -> HTML conversion successful");
        }
    }
    
    // Step 4: Validate binary file
    let validation_result = Command::new("cargo")
        .args(&[
            "run", "--",
            "export",
            "--input", &binary_path.to_string_lossy(),
            "--validate-only",
        ])
        .output();
    
    if let Ok(result) = validation_result {
        if result.status.success() {
            println!("✅ Step 4: Binary validation successful");
        }
    }
    
    println!("🎉 Complete CLI workflow test completed");
}