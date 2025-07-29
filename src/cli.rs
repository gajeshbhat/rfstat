//! Command-line interface for the rfstat tool.
//!
//! This module handles argument parsing and provides the main CLI structure
//! for the rfstat application using the clap crate.

use crate::types::{Config, OutputFormat, SortBy};
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// A Rust-based CLI tool to display file statistics in a human-readable format
///
/// rfstat analyzes files and directories to provide comprehensive statistics
/// including file counts, size distributions, and type breakdowns. Perfect
/// for DevOps workflows and system administration tasks.
///
/// Examples:
///   rfstat                           # Analyze current directory
///   rfstat /var/log                  # Analyze specific directory
///   rfstat . --format json           # Output as JSON
///   rfstat /home --sort size --limit 10  # Top 10 largest files
#[derive(Parser, Debug)]
#[command(
    name = "rfstat",
    version,
    about = "Display file statistics in human-readable format",
    long_about = None,
    author = "Your Name <your.email@example.com>"
)]
pub struct Cli {
    /// Path to analyze (defaults to current directory)
    #[arg(value_name = "PATH", default_value = ".")]
    pub path: PathBuf,

    /// Output format
    #[arg(short, long, value_enum, default_value_t = CliOutputFormat::Table)]
    pub format: CliOutputFormat,

    /// Sort results by field
    #[arg(short, long, value_enum, default_value_t = CliSortBy::Name)]
    pub sort: CliSortBy,

    /// Include hidden files and directories
    #[arg(short = 'a', long)]
    pub all: bool,

    /// Disable recursive directory traversal
    #[arg(short = 'R', long)]
    pub no_recursive: bool,

    /// Maximum depth for recursive scanning
    #[arg(short, long, value_name = "DEPTH")]
    pub depth: Option<usize>,

    /// Limit number of files shown in detailed output
    #[arg(short, long, value_name = "COUNT")]
    pub limit: Option<usize>,

    /// Show only summary statistics (no individual files)
    #[arg(long)]
    pub summary_only: bool,

    /// Filter by file extension (e.g., "txt,log,conf")
    #[arg(long, value_name = "EXTENSIONS")]
    pub extensions: Option<String>,

    /// Minimum file size filter (e.g., "1MB", "500KB")
    #[arg(long, value_name = "SIZE")]
    pub min_size: Option<String>,

    /// Maximum file size filter (e.g., "100MB", "1GB")
    #[arg(long, value_name = "SIZE")]
    pub max_size: Option<String>,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,

    /// Suppress all output except results
    #[arg(short, long)]
    pub quiet: bool,

    /// Show file permissions in output
    #[arg(long)]
    pub show_permissions: bool,

    /// Show modification times
    #[arg(long)]
    pub show_times: bool,
}

/// CLI-compatible output format enum
#[derive(ValueEnum, Clone, Debug)]
pub enum CliOutputFormat {
    /// Human-readable table format
    Table,
    /// JSON format for programmatic use
    Json,
    /// CSV format for spreadsheet import
    Csv,
    /// Compact summary format
    Summary,
}

impl From<CliOutputFormat> for OutputFormat {
    fn from(cli_format: CliOutputFormat) -> Self {
        match cli_format {
            CliOutputFormat::Table => OutputFormat::Table,
            CliOutputFormat::Json => OutputFormat::Json,
            CliOutputFormat::Csv => OutputFormat::Csv,
            CliOutputFormat::Summary => OutputFormat::Summary,
        }
    }
}

/// CLI-compatible sort options
#[derive(ValueEnum, Clone, Debug)]
pub enum CliSortBy {
    /// Sort by file name
    Name,
    /// Sort by file size (largest first)
    Size,
    /// Sort by modification time (newest first)
    Modified,
    /// Sort by file type/extension
    Type,
}

impl From<CliSortBy> for SortBy {
    fn from(cli_sort: CliSortBy) -> Self {
        match cli_sort {
            CliSortBy::Name => SortBy::Name,
            CliSortBy::Size => SortBy::Size,
            CliSortBy::Modified => SortBy::Modified,
            CliSortBy::Type => SortBy::Type,
        }
    }
}

impl Cli {
    /// Converts CLI arguments to a Config struct
    pub fn to_config(&self) -> Config {
        Config {
            format: self.format.clone().into(),
            sort_by: self.sort.clone().into(),
            show_hidden: self.all,
            recursive: !self.no_recursive,
            max_depth: self.depth,
        }
    }

    /// Parses a human-readable size string to bytes
    ///
    /// Supports formats like "1KB", "500MB", "2GB", etc.
    pub fn parse_size(size_str: &str) -> Result<u64, String> {
        let size_str = size_str.to_uppercase();
        
        // Extract number and unit
        let (number_part, unit_part) = if let Some(pos) = size_str.find(|c: char| c.is_alphabetic()) {
            size_str.split_at(pos)
        } else {
            (size_str.as_str(), "")
        };

        let number: f64 = number_part.parse()
            .map_err(|_| format!("Invalid number: {}", number_part))?;

        let multiplier = match unit_part {
            "" | "B" => 1u64,
            "KB" => 1_000,
            "MB" => 1_000_000,
            "GB" => 1_000_000_000,
            "TB" => 1_000_000_000_000,
            "KIB" => 1_024,
            "MIB" => 1_048_576,
            "GIB" => 1_073_741_824,
            "TIB" => 1_099_511_627_776,
            _ => return Err(format!("Unknown unit: {}", unit_part)),
        };

        Ok((number * multiplier as f64) as u64)
    }

    /// Parses the extensions filter string into a vector
    pub fn parse_extensions(&self) -> Option<Vec<String>> {
        self.extensions.as_ref().map(|ext_str| {
            ext_str
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect()
        })
    }

    /// Gets the minimum size filter in bytes
    pub fn get_min_size_bytes(&self) -> Result<Option<u64>, String> {
        match &self.min_size {
            Some(size_str) => Ok(Some(Self::parse_size(size_str)?)),
            None => Ok(None),
        }
    }

    /// Gets the maximum size filter in bytes
    pub fn get_max_size_bytes(&self) -> Result<Option<u64>, String> {
        match &self.max_size {
            Some(size_str) => Ok(Some(Self::parse_size(size_str)?)),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size() {
        assert_eq!(Cli::parse_size("1024").unwrap(), 1024);
        assert_eq!(Cli::parse_size("1KB").unwrap(), 1000);
        assert_eq!(Cli::parse_size("1KiB").unwrap(), 1024);
        assert_eq!(Cli::parse_size("1MB").unwrap(), 1_000_000);
        assert_eq!(Cli::parse_size("1.5GB").unwrap(), 1_500_000_000);
        
        assert!(Cli::parse_size("invalid").is_err());
        assert!(Cli::parse_size("1XB").is_err());
    }

    #[test]
    fn test_parse_extensions() {
        let cli = Cli {
            extensions: Some("txt,log,conf".to_string()),
            ..Default::default()
        };
        
        let extensions = cli.parse_extensions().unwrap();
        assert_eq!(extensions, vec!["txt", "log", "conf"]);
    }
}

// Implement Default for Cli to support testing
impl Default for Cli {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            format: CliOutputFormat::Table,
            sort: CliSortBy::Name,
            all: false,
            no_recursive: false,
            depth: None,
            limit: None,
            summary_only: false,
            extensions: None,
            min_size: None,
            max_size: None,
            verbose: false,
            quiet: false,
            show_permissions: false,
            show_times: false,
        }
    }
}
