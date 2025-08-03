//! Binary export error types and handling

use std::fmt;

/// Binary export error types
#[derive(Debug)]
pub enum BinaryExportError {
    /// File I/O operation failed
    Io(std::io::Error),
    
    /// Invalid file format detected
    InvalidFormat,
    
    /// Unsupported file version
    UnsupportedVersion(u32),
    
    /// Data corruption detected
    CorruptedData(String),
    
    /// Invalid magic bytes in header
    InvalidMagic { expected: String, actual: String },
}

impl fmt::Display for BinaryExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryExportError::Io(err) => write!(f, "File I/O error: {}", err),
            BinaryExportError::InvalidFormat => write!(f, "Invalid file format"),
            BinaryExportError::UnsupportedVersion(version) => {
                write!(f, "Unsupported version: {}", version)
            }
            BinaryExportError::CorruptedData(reason) => {
                write!(f, "Data corruption: {}", reason)
            }
            BinaryExportError::InvalidMagic { expected, actual } => {
                write!(f, "Invalid magic bytes: expected '{}', got '{}'", expected, actual)
            }
        }
    }
}

impl std::error::Error for BinaryExportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BinaryExportError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for BinaryExportError {
    fn from(err: std::io::Error) -> Self {
        BinaryExportError::Io(err)
    }
}

/// Convert to TrackingError for compatibility with existing error handling
impl From<BinaryExportError> for crate::core::types::TrackingError {
    fn from(err: BinaryExportError) -> Self {
        crate::core::types::TrackingError::ExportError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_display() {
        let err = BinaryExportError::InvalidFormat;
        assert_eq!(err.to_string(), "Invalid file format");
        
        let err = BinaryExportError::UnsupportedVersion(2);
        assert_eq!(err.to_string(), "Unsupported version: 2");
    }
    
    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let binary_err = BinaryExportError::from(io_err);
        
        match binary_err {
            BinaryExportError::Io(_) => (),
            _ => panic!("Expected Io error"),
        }
    }
}