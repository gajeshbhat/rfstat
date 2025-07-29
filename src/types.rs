//! Core data structures and type definitions for rfstat.
//!
//! This module contains the fundamental types used throughout the application,
//! including file statistics, configuration options, and output formats.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Represents comprehensive statistics for a single file or directory.
///
/// # Examples
///
/// ```rust
/// use rfstat::FileEntry;
/// use std::path::PathBuf;
///
/// let entry = FileEntry {
///     path: PathBuf::from("/home/user/document.txt"),
///     size: 1024,
///     is_dir: false,
///     modified: chrono::Utc::now(),
///     permissions: 0o644,
///     file_type: Some("txt".to_string()),
/// };
///
/// assert_eq!(entry.size_human(), "1.02 kB");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// Full path to the file or directory
    pub path: PathBuf,
    /// Size in bytes
    pub size: u64,
    /// Whether this entry is a directory
    pub is_dir: bool,
    /// Last modified timestamp
    pub modified: DateTime<Utc>,
    /// File permissions (Unix-style)
    pub permissions: u32,
    /// File extension/type (if applicable)
    pub file_type: Option<String>,
}

impl FileEntry {
    /// Returns the file size in human-readable format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rfstat::FileEntry;
    /// use std::path::PathBuf;
    /// use chrono::Utc;
    ///
    /// let entry = FileEntry {
    ///     path: PathBuf::from("test.txt"),
    ///     size: 2048,
    ///     is_dir: false,
    ///     modified: Utc::now(),
    ///     permissions: 0o644,
    ///     file_type: Some("txt".to_string()),
    /// };
    ///
    /// assert_eq!(entry.size_human(), "2.05 kB");
    /// ```
    pub fn size_human(&self) -> String {
        humansize::format_size(self.size, humansize::DECIMAL)
    }

    /// Returns the file name without the full path.
    pub fn name(&self) -> String {
        self.path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }
}

/// Aggregated statistics for a collection of files and directories.
///
/// This structure provides comprehensive analysis of file system data,
/// including size distributions, file type breakdowns, and summary statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStats {
    /// Total number of files (excluding directories)
    pub total_files: u64,
    /// Total number of directories
    pub total_dirs: u64,
    /// Total size of all files in bytes
    pub total_size: u64,
    /// Average file size in bytes
    pub avg_file_size: u64,
    /// Largest file size in bytes
    pub max_file_size: u64,
    /// Smallest file size in bytes
    pub min_file_size: u64,
    /// Breakdown by file extension
    pub file_types: HashMap<String, TypeStats>,
    /// Size distribution buckets
    pub size_distribution: SizeDistribution,
    /// Individual file entries
    pub entries: Vec<FileEntry>,
}

impl FileStats {
    /// Creates a new empty FileStats instance.
    pub fn new() -> Self {
        Self {
            total_files: 0,
            total_dirs: 0,
            total_size: 0,
            avg_file_size: 0,
            max_file_size: 0,
            min_file_size: u64::MAX,
            file_types: HashMap::new(),
            size_distribution: SizeDistribution::new(),
            entries: Vec::new(),
        }
    }

    /// Returns the total size in human-readable format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rfstat::FileStats;
    ///
    /// let mut stats = FileStats::new();
    /// stats.total_size = 1048576; // 1 MB
    /// assert_eq!(stats.total_size_human(), "1.05 MB");
    /// ```
    pub fn total_size_human(&self) -> String {
        humansize::format_size(self.total_size, humansize::DECIMAL)
    }

    /// Returns the average file size in human-readable format.
    pub fn avg_file_size_human(&self) -> String {
        humansize::format_size(self.avg_file_size, humansize::DECIMAL)
    }
}

impl Default for FileStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for a specific file type/extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeStats {
    /// Number of files of this type
    pub count: u64,
    /// Total size of all files of this type
    pub total_size: u64,
    /// Average size for this file type
    pub avg_size: u64,
}

impl TypeStats {
    /// Creates a new TypeStats instance.
    pub fn new() -> Self {
        Self {
            count: 0,
            total_size: 0,
            avg_size: 0,
        }
    }

    /// Returns the total size in human-readable format.
    pub fn total_size_human(&self) -> String {
        humansize::format_size(self.total_size, humansize::DECIMAL)
    }
}

impl Default for TypeStats {
    fn default() -> Self {
        Self::new()
    }
}

/// File size distribution across different size buckets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeDistribution {
    /// Files smaller than 1 KB
    pub tiny: u64,      // < 1 KB
    /// Files between 1 KB and 1 MB
    pub small: u64,     // 1 KB - 1 MB
    /// Files between 1 MB and 100 MB
    pub medium: u64,    // 1 MB - 100 MB
    /// Files between 100 MB and 1 GB
    pub large: u64,     // 100 MB - 1 GB
    /// Files larger than 1 GB
    pub huge: u64,      // > 1 GB
}

impl SizeDistribution {
    /// Creates a new empty size distribution.
    pub fn new() -> Self {
        Self {
            tiny: 0,
            small: 0,
            medium: 0,
            large: 0,
            huge: 0,
        }
    }

    /// Adds a file size to the appropriate bucket.
    pub fn add_size(&mut self, size: u64) {
        match size {
            0..=1023 => self.tiny += 1,
            1024..=1048575 => self.small += 1,
            1048576..=104857599 => self.medium += 1,
            104857600..=1073741823 => self.large += 1,
            _ => self.huge += 1,
        }
    }
}

impl Default for SizeDistribution {
    fn default() -> Self {
        Self::new()
    }
}

/// Output format options for displaying statistics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    /// Human-readable table format
    Table,
    /// JSON format for programmatic use
    Json,
    /// CSV format for spreadsheet import
    Csv,
    /// Compact summary format
    Summary,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Table
    }
}

/// Sorting options for file listings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortBy {
    /// Sort by file name
    Name,
    /// Sort by file size
    Size,
    /// Sort by modification time
    Modified,
    /// Sort by file type/extension
    Type,
}

impl Default for SortBy {
    fn default() -> Self {
        SortBy::Name
    }
}

/// Configuration options for the rfstat tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Output format to use
    pub format: OutputFormat,
    /// How to sort the results
    pub sort_by: SortBy,
    /// Whether to include hidden files
    pub show_hidden: bool,
    /// Whether to scan directories recursively
    pub recursive: bool,
    /// Maximum depth for recursive scanning
    pub max_depth: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            format: OutputFormat::Table,
            sort_by: SortBy::Name,
            show_hidden: false,
            recursive: true,
            max_depth: None,
        }
    }
}
