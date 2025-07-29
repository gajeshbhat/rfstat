//! Statistical analysis and calculations for file system data.
//!
//! This module provides functions to analyze collections of file entries
//! and generate comprehensive statistics including size distributions,
//! file type breakdowns, and summary metrics.

use crate::types::{FileEntry, FileStats, SizeDistribution, TypeStats};
use std::collections::HashMap;

/// Calculates comprehensive statistics from a collection of file entries.
///
/// This is the main function for generating file system statistics. It analyzes
/// all provided entries and returns a complete FileStats structure with
/// aggregated data, distributions, and breakdowns.
///
/// # Arguments
///
/// * `entries` - A slice of FileEntry objects to analyze
///
/// # Examples
///
/// ```rust
/// use rfstat::{calculate_stats, FileEntry};
/// use std::path::PathBuf;
/// use chrono::Utc;
///
/// let entries = vec![
///     FileEntry {
///         path: PathBuf::from("test.txt"),
///         size: 1024,
///         is_dir: false,
///         modified: Utc::now(),
///         permissions: 0o644,
///         file_type: Some("txt".to_string()),
///     }
/// ];
///
/// let stats = calculate_stats(&entries);
/// assert_eq!(stats.total_files, 1);
/// assert_eq!(stats.total_size, 1024);
/// ```
pub fn calculate_stats(entries: &[FileEntry]) -> FileStats {
    let mut stats = FileStats::new();
    let mut file_sizes = Vec::new();

    // First pass: collect basic statistics
    for entry in entries {
        if entry.is_dir {
            stats.total_dirs += 1;
        } else {
            stats.total_files += 1;
            stats.total_size += entry.size;
            file_sizes.push(entry.size);

            // Update size distribution
            stats.size_distribution.add_size(entry.size);

            // Update file type statistics
            let file_type = entry.file_type.as_deref().unwrap_or("no_extension");
            let type_stats = stats
                .file_types
                .entry(file_type.to_string())
                .or_default();
            type_stats.count += 1;
            type_stats.total_size += entry.size;
        }
    }

    // Calculate derived statistics
    if stats.total_files > 0 {
        stats.avg_file_size = stats.total_size / stats.total_files;

        if !file_sizes.is_empty() {
            stats.max_file_size = *file_sizes.iter().max().unwrap();
            stats.min_file_size = *file_sizes.iter().min().unwrap();
        }
    }

    // Calculate average sizes for each file type
    for type_stats in stats.file_types.values_mut() {
        if type_stats.count > 0 {
            type_stats.avg_size = type_stats.total_size / type_stats.count;
        }
    }

    // Store the entries for detailed output
    stats.entries = entries.to_vec();

    stats
}

/// Calculates the top N largest files from the entries.
///
/// # Arguments
///
/// * `entries` - File entries to analyze
/// * `n` - Number of top files to return
///
/// # Returns
///
/// A vector of the N largest files, sorted by size (largest first)
pub fn get_largest_files(entries: &[FileEntry], n: usize) -> Vec<&FileEntry> {
    let mut files: Vec<&FileEntry> = entries.iter().filter(|e| !e.is_dir).collect();
    files.sort_by(|a, b| b.size.cmp(&a.size));
    files.into_iter().take(n).collect()
}

/// Calculates the top N most common file types.
///
/// # Arguments
///
/// * `stats` - FileStats containing file type information
/// * `n` - Number of top file types to return
///
/// # Returns
///
/// A vector of tuples containing (file_type, TypeStats) sorted by count
pub fn get_top_file_types(stats: &FileStats, n: usize) -> Vec<(&String, &TypeStats)> {
    let mut types: Vec<(&String, &TypeStats)> = stats.file_types.iter().collect();
    types.sort_by(|a, b| b.1.count.cmp(&a.1.count));
    types.into_iter().take(n).collect()
}

/// Calculates directory-specific statistics.
///
/// This function analyzes entries to provide statistics about directory
/// structure, including depth distribution and directory sizes.
pub fn calculate_directory_stats(entries: &[FileEntry]) -> DirectoryStats {
    let mut dir_stats = DirectoryStats::new();
    let mut directory_sizes: HashMap<String, u64> = HashMap::new();

    for entry in entries {
        if entry.is_dir {
            dir_stats.total_directories += 1;

            // Calculate directory depth
            let depth = entry.path.components().count();
            dir_stats.max_depth = dir_stats.max_depth.max(depth);

            // Initialize directory size tracking
            directory_sizes.insert(entry.path.to_string_lossy().to_string(), 0);
        } else {
            // Add file size to its parent directory
            if let Some(parent) = entry.path.parent() {
                let parent_str = parent.to_string_lossy().to_string();
                *directory_sizes.entry(parent_str).or_insert(0) += entry.size;
            }
        }
    }

    // Find largest directory by content size
    if let Some((largest_dir, largest_size)) = directory_sizes.iter().max_by_key(|(_, &size)| size)
    {
        dir_stats.largest_directory = Some(largest_dir.clone());
        dir_stats.largest_directory_size = *largest_size;
    }

    dir_stats
}

/// Statistics specific to directory structure and organization.
#[derive(Debug, Clone)]
pub struct DirectoryStats {
    /// Total number of directories
    pub total_directories: u64,
    /// Maximum directory depth found
    pub max_depth: usize,
    /// Path of the directory containing the most data
    pub largest_directory: Option<String>,
    /// Size of the largest directory's contents
    pub largest_directory_size: u64,
}

impl DirectoryStats {
    /// Creates a new empty DirectoryStats instance.
    pub fn new() -> Self {
        Self {
            total_directories: 0,
            max_depth: 0,
            largest_directory: None,
            largest_directory_size: 0,
        }
    }

    /// Returns the largest directory size in human-readable format.
    pub fn largest_directory_size_human(&self) -> String {
        humansize::format_size(self.largest_directory_size, humansize::DECIMAL)
    }
}

impl Default for DirectoryStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculates percentile values for file sizes.
///
/// # Arguments
///
/// * `entries` - File entries to analyze
/// * `percentiles` - Slice of percentile values to calculate (0.0 to 1.0)
///
/// # Returns
///
/// A vector of file sizes corresponding to the requested percentiles
pub fn calculate_size_percentiles(entries: &[FileEntry], percentiles: &[f64]) -> Vec<u64> {
    let mut file_sizes: Vec<u64> = entries
        .iter()
        .filter(|e| !e.is_dir)
        .map(|e| e.size)
        .collect();

    if file_sizes.is_empty() {
        return vec![0; percentiles.len()];
    }

    file_sizes.sort_unstable();

    percentiles
        .iter()
        .map(|&p| {
            let index = ((file_sizes.len() as f64 - 1.0) * p) as usize;
            file_sizes[index.min(file_sizes.len() - 1)]
        })
        .collect()
}

/// Generates a summary report of the most important statistics.
pub fn generate_summary_report(stats: &FileStats) -> SummaryReport {
    let dir_stats = calculate_directory_stats(&stats.entries);
    let largest_files = get_largest_files(&stats.entries, 5);
    let top_types = get_top_file_types(stats, 5);
    let percentiles = calculate_size_percentiles(&stats.entries, &[0.5, 0.75, 0.9, 0.95, 0.99]);

    SummaryReport {
        total_files: stats.total_files,
        total_directories: dir_stats.total_directories,
        total_size: stats.total_size,
        avg_file_size: stats.avg_file_size,
        median_file_size: percentiles.first().copied().unwrap_or(0),
        largest_file_size: stats.max_file_size,
        smallest_file_size: if stats.min_file_size == u64::MAX {
            0
        } else {
            stats.min_file_size
        },
        most_common_type: top_types.first().map(|(name, _)| (*name).clone()),
        size_distribution: stats.size_distribution.clone(),
        largest_files: largest_files.into_iter().cloned().collect(),
        top_file_types: top_types
            .into_iter()
            .map(|(name, stats)| (name.clone(), stats.clone()))
            .collect(),
    }
}

/// A comprehensive summary report of file system statistics.
#[derive(Debug, Clone)]
pub struct SummaryReport {
    pub total_files: u64,
    pub total_directories: u64,
    pub total_size: u64,
    pub avg_file_size: u64,
    pub median_file_size: u64,
    pub largest_file_size: u64,
    pub smallest_file_size: u64,
    pub most_common_type: Option<String>,
    pub size_distribution: SizeDistribution,
    pub largest_files: Vec<FileEntry>,
    pub top_file_types: Vec<(String, TypeStats)>,
}

impl SummaryReport {
    /// Returns the total size in human-readable format.
    pub fn total_size_human(&self) -> String {
        humansize::format_size(self.total_size, humansize::DECIMAL)
    }

    /// Returns the average file size in human-readable format.
    pub fn avg_file_size_human(&self) -> String {
        humansize::format_size(self.avg_file_size, humansize::DECIMAL)
    }

    /// Returns the median file size in human-readable format.
    pub fn median_file_size_human(&self) -> String {
        humansize::format_size(self.median_file_size, humansize::DECIMAL)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::path::PathBuf;

    fn create_test_entry(
        name: &str,
        size: u64,
        is_dir: bool,
        file_type: Option<&str>,
    ) -> FileEntry {
        FileEntry {
            path: PathBuf::from(name),
            size,
            is_dir,
            modified: Utc::now(),
            permissions: 0o644,
            file_type: file_type.map(|s| s.to_string()),
        }
    }

    #[test]
    fn test_calculate_stats_empty() {
        let entries = vec![];
        let stats = calculate_stats(&entries);

        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.total_dirs, 0);
        assert_eq!(stats.total_size, 0);
    }

    #[test]
    fn test_calculate_stats_with_files() {
        let entries = vec![
            create_test_entry("file1.txt", 1000, false, Some("txt")),
            create_test_entry("file2.txt", 2000, false, Some("txt")),
            create_test_entry("dir1", 0, true, None),
        ];

        let stats = calculate_stats(&entries);

        assert_eq!(stats.total_files, 2);
        assert_eq!(stats.total_dirs, 1);
        assert_eq!(stats.total_size, 3000);
        assert_eq!(stats.avg_file_size, 1500);
        assert_eq!(stats.max_file_size, 2000);
        assert_eq!(stats.min_file_size, 1000);

        // Check file type statistics
        let txt_stats = stats.file_types.get("txt").unwrap();
        assert_eq!(txt_stats.count, 2);
        assert_eq!(txt_stats.total_size, 3000);
        assert_eq!(txt_stats.avg_size, 1500);
    }

    #[test]
    fn test_get_largest_files() {
        let entries = vec![
            create_test_entry("small.txt", 100, false, Some("txt")),
            create_test_entry("large.txt", 1000, false, Some("txt")),
            create_test_entry("medium.txt", 500, false, Some("txt")),
            create_test_entry("dir", 0, true, None),
        ];

        let largest = get_largest_files(&entries, 2);

        assert_eq!(largest.len(), 2);
        assert_eq!(largest[0].size, 1000);
        assert_eq!(largest[1].size, 500);
    }

    #[test]
    fn test_size_distribution() {
        let entries = vec![
            create_test_entry("tiny.txt", 500, false, Some("txt")), // tiny
            create_test_entry("small.txt", 50000, false, Some("txt")), // small
            create_test_entry("medium.txt", 5000000, false, Some("txt")), // medium
            create_test_entry("large.txt", 500000000, false, Some("txt")), // large
            create_test_entry("huge.txt", 5000000000, false, Some("txt")), // huge
        ];

        let stats = calculate_stats(&entries);

        assert_eq!(stats.size_distribution.tiny, 1);
        assert_eq!(stats.size_distribution.small, 1);
        assert_eq!(stats.size_distribution.medium, 1);
        assert_eq!(stats.size_distribution.large, 1);
        assert_eq!(stats.size_distribution.huge, 1);
    }
}
