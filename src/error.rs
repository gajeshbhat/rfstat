//! Error handling for the rfstat application.
//!
//! This module provides comprehensive error types and handling for all
//! operations that can fail during file system analysis.

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for rfstat operations.
///
/// This enum covers all possible error conditions that can occur
/// during file system scanning and analysis.
#[derive(Error, Debug)]
pub enum RfstatError {
    /// I/O error occurred during file system operations
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Permission denied when accessing a file or directory
    #[error("Permission denied accessing: {path}")]
    PermissionDenied { path: PathBuf },

    /// File or directory not found
    #[error("Path not found: {path}")]
    PathNotFound { path: PathBuf },

    /// Invalid path provided
    #[error("Invalid path: {path}")]
    InvalidPath { path: PathBuf },

    /// Error during serialization (JSON, CSV output)
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// CSV writing error
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    /// Configuration error
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// Generic error with custom message
    #[error("Error: {message}")]
    Generic { message: String },
}

/// Result type alias for rfstat operations.
pub type Result<T> = std::result::Result<T, RfstatError>;

impl RfstatError {
    /// Creates a new permission denied error.
    pub fn permission_denied<P: Into<PathBuf>>(path: P) -> Self {
        Self::PermissionDenied { path: path.into() }
    }

    /// Creates a new path not found error.
    pub fn path_not_found<P: Into<PathBuf>>(path: P) -> Self {
        Self::PathNotFound { path: path.into() }
    }

    /// Creates a new invalid path error.
    pub fn invalid_path<P: Into<PathBuf>>(path: P) -> Self {
        Self::InvalidPath { path: path.into() }
    }

    /// Creates a new configuration error.
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Creates a new generic error.
    pub fn generic<S: Into<String>>(message: S) -> Self {
        Self::Generic {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_error_creation() {
        let path = Path::new("/nonexistent");
        let error = RfstatError::path_not_found(path);

        match error {
            RfstatError::PathNotFound { path: p } => {
                assert_eq!(p, Path::new("/nonexistent"));
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_error_display() {
        let error = RfstatError::generic("Test error message");
        assert_eq!(error.to_string(), "Error: Test error message");
    }
}
