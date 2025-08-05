//! Test for backward compatibility with older binary format versions

use memscope_rs::export::binary::{BinaryReader, FileHeader, MAGIC_BYTES, FORMAT_VERSION};
use memscope_rs::export::binary::BinaryExportError;

#[test]
fn test_version_compatibility_check() {
    // Test that version 1 and 2 are considered compatible
    let header_v1 = FileHeader { magic: *MAGIC_BYTES, version: 1, count: 0 };
    let header_v2 = FileHeader { magic: *MAGIC_BYTES, version: 2, count: 0 };
    let header_v3 = FileHeader { magic: *MAGIC_BYTES, version: 999, count: 0 };
    
    assert!(header_v1.is_compatible_version());
    assert!(header_v2.is_compatible_version());
    assert!(!header_v3.is_compatible_version()); // Future version should not be compatible
    
    assert!(header_v1.is_legacy_version());
    assert!(!header_v2.is_legacy_version());
}

#[test]
fn test_format_version_constants() {
    // Test that format version is correctly set
    assert_eq!(FORMAT_VERSION, 2);
    
    // Test magic bytes
    assert_eq!(MAGIC_BYTES, b"MEMSCOPE");
}

#[test]
fn test_version_detection_logic() {
    // Test the version compatibility logic
    let current_header = FileHeader::new(0);
    assert_eq!(current_header.get_version(), FORMAT_VERSION);
    assert!(current_header.is_compatible_version());
    assert!(!current_header.is_legacy_version());
    
    // Test that version 0 is not compatible (too old)
    let old_header = FileHeader { magic: *MAGIC_BYTES, version: 0, count: 0 };
    assert!(!old_header.is_compatible_version());
}

#[test]
fn test_unsupported_version_error() {
    // Test that UnsupportedVersion error can be created and displayed
    let error = BinaryExportError::UnsupportedVersion(999);
    let error_string = error.to_string();
    assert!(error_string.contains("999"));
    assert!(error_string.contains("Unsupported version"));
}

#[test]
fn test_backward_compatibility_range() {
    // Test that versions 1 and 2 are supported, but not 0 or 3+
    for version in 1..=2 {
        let header = FileHeader { magic: *MAGIC_BYTES, version, count: 0 };
        assert!(header.is_compatible_version(), "Version {} should be compatible", version);
    }
    
    // Test unsupported versions
    let unsupported_versions = [0, 3, 4, 999];
    for version in unsupported_versions {
        let header = FileHeader { magic: *MAGIC_BYTES, version, count: 0 };
        assert!(!header.is_compatible_version(), "Version {} should not be compatible", version);
    }
}