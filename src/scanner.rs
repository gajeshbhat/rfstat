//! File system scanning and metadata collection.
//!
//! This module provides functionality to traverse directories, collect file metadata,
//! and handle various file system edge cases like permissions and symbolic links.

use crate::error::{Result, RfstatError};
use crate::types::{Config, FileEntry};
use log::{debug, warn};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

/// Scans a directory and returns a vector of file entries.
///
/// This is the main entry point for file system scanning. It handles
/// recursive traversal, permission errors, and applies filtering based
/// on the provided configuration.
///
/// # Arguments
///
/// * `path` - The directory path to scan
/// * `config` - Configuration options for scanning behavior
///
/// # Examples
///
/// ```rust
/// use rfstat::{scan_directory, Config};
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config = Config::default();
/// let entries = scan_directory(Path::new("."), &config)?;
/// println!("Found {} entries", entries.len());
/// # Ok(())
/// # }
/// ```
pub fn scan_directory<P: AsRef<Path>>(path: P, config: &Config) -> Result<Vec<FileEntry>> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(RfstatError::path_not_found(path));
    }

    if !path.is_dir() {
        // If it's a single file, return it as a single-entry vector
        return Ok(vec![create_file_entry(path)?]);
    }

    debug!("Scanning directory: {}", path.display());

    let mut entries = Vec::new();
    let walker = create_walker(path, config);

    for entry in walker {
        match entry {
            Ok(dir_entry) => {
                match process_dir_entry(&dir_entry, config) {
                    Ok(Some(file_entry)) => entries.push(file_entry),
                    Ok(None) => {
                        // Entry was filtered out, continue
                        debug!("Filtered out: {}", dir_entry.path().display());
                    }
                    Err(e) => {
                        warn!("Error processing {}: {}", dir_entry.path().display(), e);
                        // Continue processing other files instead of failing completely
                    }
                }
            }
            Err(e) => {
                warn!("Error walking directory: {e}");
                // Continue processing instead of failing
            }
        }
    }

    debug!("Scanned {} entries", entries.len());
    Ok(entries)
}

/// Creates a WalkDir iterator with appropriate configuration.
fn create_walker(path: &Path, config: &Config) -> walkdir::IntoIter {
    let mut walker = WalkDir::new(path);

    if !config.recursive {
        walker = walker.max_depth(1);
    } else if let Some(max_depth) = config.max_depth {
        walker = walker.max_depth(max_depth);
    }

    walker
        .follow_links(false) // Don't follow symbolic links to avoid cycles
        .into_iter()
}

/// Processes a single directory entry and converts it to a FileEntry if it passes filters.
fn process_dir_entry(dir_entry: &DirEntry, config: &Config) -> Result<Option<FileEntry>> {
    let path = dir_entry.path();

    // Skip hidden files unless explicitly requested
    if !config.show_hidden && is_hidden(path) {
        return Ok(None);
    }

    // Skip the root directory itself when doing recursive scans
    if dir_entry.depth() == 0 && path.is_dir() {
        return Ok(None);
    }

    create_file_entry(path).map(Some)
}

/// Creates a FileEntry from a file path.
fn create_file_entry<P: AsRef<Path>>(path: P) -> Result<FileEntry> {
    let path = path.as_ref();
    let metadata = fs::metadata(path)?;

    let size = if metadata.is_file() {
        metadata.len()
    } else {
        0 // Directories have size 0 for our purposes
    };

    let modified = metadata.modified()?.into();

    let permissions = metadata.permissions().mode();

    let file_type = if metadata.is_file() {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase())
    } else {
        None
    };

    Ok(FileEntry {
        path: path.to_path_buf(),
        size,
        is_dir: metadata.is_dir(),
        modified,
        permissions,
        file_type,
    })
}

/// Checks if a file or directory is hidden (starts with a dot).
fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with('.'))
        .unwrap_or(false)
}

/// Filters file entries based on various criteria.
pub fn filter_entries(entries: &[FileEntry], filters: &FileFilters) -> Vec<FileEntry> {
    entries
        .iter()
        .filter(|entry| apply_filters(entry, filters))
        .cloned()
        .collect()
}

/// File filtering options.
#[derive(Debug, Default)]
pub struct FileFilters {
    /// Only include files with these extensions
    pub extensions: Option<Vec<String>>,
    /// Minimum file size in bytes
    pub min_size: Option<u64>,
    /// Maximum file size in bytes
    pub max_size: Option<u64>,
    /// Only include files (exclude directories)
    pub files_only: bool,
    /// Only include directories (exclude files)
    pub dirs_only: bool,
}

/// Applies all filters to a single file entry.
fn apply_filters(entry: &FileEntry, filters: &FileFilters) -> bool {
    // File type filter
    if filters.files_only && entry.is_dir {
        return false;
    }
    if filters.dirs_only && !entry.is_dir {
        return false;
    }

    // Extension filter (only applies to files)
    if let Some(ref allowed_extensions) = filters.extensions {
        if !entry.is_dir {
            match &entry.file_type {
                Some(ext) => {
                    if !allowed_extensions.contains(ext) {
                        return false;
                    }
                }
                None => return false, // No extension, but we're filtering by extension
            }
        }
    }

    // Size filters (only apply to files)
    if !entry.is_dir {
        if let Some(min_size) = filters.min_size {
            if entry.size < min_size {
                return false;
            }
        }

        if let Some(max_size) = filters.max_size {
            if entry.size > max_size {
                return false;
            }
        }
    }

    true
}

/// Sorts file entries according to the specified criteria.
pub fn sort_entries(entries: &mut [FileEntry], sort_by: crate::types::SortBy) {
    use crate::types::SortBy;

    match sort_by {
        SortBy::Name => {
            entries.sort_by(|a, b| a.path.cmp(&b.path));
        }
        SortBy::Size => {
            entries.sort_by(|a, b| b.size.cmp(&a.size)); // Largest first
        }
        SortBy::Modified => {
            entries.sort_by(|a, b| b.modified.cmp(&a.modified)); // Newest first
        }
        SortBy::Type => {
            entries.sort_by(|a, b| match (&a.file_type, &b.file_type) {
                (Some(a_type), Some(b_type)) => a_type.cmp(b_type),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.path.cmp(&b.path),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::fs::File;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_scan_empty_directory() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::default();

        let entries = scan_directory(temp_dir.path(), &config)?;
        assert_eq!(entries.len(), 0);

        Ok(())
    }

    #[test]
    fn test_scan_directory_with_files() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::default();

        // Create test files
        File::create(temp_dir.path().join("test1.txt")).unwrap();
        File::create(temp_dir.path().join("test2.log")).unwrap();

        let entries = scan_directory(temp_dir.path(), &config)?;
        assert_eq!(entries.len(), 2);

        Ok(())
    }

    #[test]
    fn test_filter_by_extension() {
        let entries = vec![
            FileEntry {
                path: PathBuf::from("test.txt"),
                size: 100,
                is_dir: false,
                modified: Utc::now(),
                permissions: 0o644,
                file_type: Some("txt".to_string()),
            },
            FileEntry {
                path: PathBuf::from("test.log"),
                size: 200,
                is_dir: false,
                modified: Utc::now(),
                permissions: 0o644,
                file_type: Some("log".to_string()),
            },
        ];

        let filters = FileFilters {
            extensions: Some(vec!["txt".to_string()]),
            ..Default::default()
        };

        let filtered = filter_entries(&entries, &filters);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].file_type, Some("txt".to_string()));
    }

    #[test]
    fn test_is_hidden() {
        assert!(is_hidden(Path::new(".hidden")));
        assert!(is_hidden(Path::new("/path/to/.hidden")));
        assert!(!is_hidden(Path::new("visible")));
        assert!(!is_hidden(Path::new("/path/to/visible")));
    }
}
