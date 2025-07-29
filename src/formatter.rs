//! Output formatting for different display formats.
//!
//! This module provides functions to format file statistics and entries
//! in various output formats including tables, JSON, CSV, and summary views.
//! Each format is optimized for different use cases and workflows.

use crate::error::Result;
use crate::types::{FileStats, OutputFormat};
use colored::*;
use serde_json;
use std::io::Write;
use tabled::{Table, Tabled};

/// Formats and displays file statistics according to the specified output format.
///
/// This is the main entry point for output formatting. It takes file statistics
/// and renders them in the requested format to the provided writer.
///
/// # Arguments
///
/// * `stats` - The file statistics to format
/// * `format` - The desired output format
/// * `writer` - Where to write the formatted output
/// * `options` - Additional formatting options
///
/// # Examples
///
/// ```rust
/// use rfstat::{format_output, FileStats, OutputFormat, FormatterOptions};
/// use std::io;
///
/// let stats = FileStats::new();
/// let options = FormatterOptions::default();
/// format_output(&stats, OutputFormat::Table, &mut io::stdout(), &options).unwrap();
/// ```
pub fn format_output<W: Write>(
    stats: &FileStats,
    format: OutputFormat,
    writer: &mut W,
    options: &FormatterOptions,
) -> Result<()> {
    match format {
        OutputFormat::Table => format_table(stats, writer, options),
        OutputFormat::Json => format_json(stats, writer, options),
        OutputFormat::Csv => format_csv(stats, writer, options),
        OutputFormat::Summary => format_summary(stats, writer, options),
    }
}

/// Options for controlling output formatting.
#[derive(Debug, Clone)]
pub struct FormatterOptions {
    /// Whether to use colors in output
    pub use_colors: bool,
    /// Maximum number of entries to display
    pub limit: Option<usize>,
    /// Whether to show only summary statistics
    pub summary_only: bool,
    /// Whether to show file permissions
    pub show_permissions: bool,
    /// Whether to show modification times
    pub show_times: bool,
    /// Whether to show detailed file type breakdown
    pub show_file_types: bool,
}

impl Default for FormatterOptions {
    fn default() -> Self {
        Self {
            use_colors: true,
            limit: None,
            summary_only: false,
            show_permissions: false,
            show_times: false,
            show_file_types: true,
        }
    }
}

/// Formats output as a human-readable table.
fn format_table<W: Write>(
    stats: &FileStats,
    writer: &mut W,
    options: &FormatterOptions,
) -> Result<()> {
    // Print summary statistics first
    write_summary_header(writer, stats, options)?;
    
    if !options.summary_only && !stats.entries.is_empty() {
        writeln!(writer)?;
        write_file_table(writer, stats, options)?;
    }
    
    if options.show_file_types && !stats.file_types.is_empty() {
        writeln!(writer)?;
        write_file_types_table(writer, stats, options)?;
    }
    
    Ok(())
}

/// Writes the summary header with key statistics.
fn write_summary_header<W: Write>(
    writer: &mut W,
    stats: &FileStats,
    options: &FormatterOptions,
) -> Result<()> {
    let title = if options.use_colors {
        "📊 File Statistics Summary".bold().blue()
    } else {
        "File Statistics Summary".normal()
    };
    
    writeln!(writer, "{}", title)?;
    writeln!(writer, "{}", "=".repeat(50))?;
    
    writeln!(writer, "Total Files:      {}", format_number(stats.total_files))?;
    writeln!(writer, "Total Directories: {}", format_number(stats.total_dirs))?;
    writeln!(writer, "Total Size:       {}", stats.total_size_human())?;
    
    if stats.total_files > 0 {
        writeln!(writer, "Average File Size: {}", stats.avg_file_size_human())?;
        writeln!(writer, "Largest File:     {}", humansize::format_size(stats.max_file_size, humansize::DECIMAL))?;
        writeln!(writer, "Smallest File:    {}", humansize::format_size(
            if stats.min_file_size == u64::MAX { 0 } else { stats.min_file_size },
            humansize::DECIMAL
        ))?;
    }
    
    // Size distribution
    writeln!(writer)?;
    writeln!(writer, "Size Distribution:")?;
    writeln!(writer, "  Tiny (< 1KB):     {}", format_number(stats.size_distribution.tiny))?;
    writeln!(writer, "  Small (1KB-1MB):  {}", format_number(stats.size_distribution.small))?;
    writeln!(writer, "  Medium (1MB-100MB): {}", format_number(stats.size_distribution.medium))?;
    writeln!(writer, "  Large (100MB-1GB): {}", format_number(stats.size_distribution.large))?;
    writeln!(writer, "  Huge (> 1GB):     {}", format_number(stats.size_distribution.huge))?;
    
    Ok(())
}

/// Writes a table of individual files.
fn write_file_table<W: Write>(
    writer: &mut W,
    stats: &FileStats,
    options: &FormatterOptions,
) -> Result<()> {
    let title = if options.use_colors {
        "📁 File Details".bold().green()
    } else {
        "File Details".normal()
    };
    
    writeln!(writer, "{}", title)?;
    writeln!(writer, "{}", "-".repeat(30))?;
    
    let entries = if let Some(limit) = options.limit {
        &stats.entries[..stats.entries.len().min(limit)]
    } else {
        &stats.entries
    };
    
    // Create table data
    let mut table_data = Vec::new();
    for entry in entries {
        let mut row = FileTableRow {
            name: entry.name(),
            size: entry.size_human(),
            type_field: if entry.is_dir {
                "DIR".to_string()
            } else {
                entry.file_type.as_deref().unwrap_or("").to_uppercase()
            },
            permissions: if options.show_permissions {
                format!("{:o}", entry.permissions)
            } else {
                "".to_string()
            },
            modified: if options.show_times {
                entry.modified.format("%Y-%m-%d %H:%M").to_string()
            } else {
                "".to_string()
            },
        };
        
        // Apply colors if enabled
        if options.use_colors {
            if entry.is_dir {
                row.name = row.name.blue().to_string();
                row.type_field = row.type_field.blue().to_string();
            } else if entry.size > 100_000_000 { // > 100MB
                row.size = row.size.red().to_string();
            } else if entry.size > 1_000_000 { // > 1MB
                row.size = row.size.yellow().to_string();
            }
        }
        
        table_data.push(row);
    }
    
    if !table_data.is_empty() {
        let table = Table::new(table_data).to_string();
        writeln!(writer, "{}", table)?;
    }
    
    Ok(())
}

/// Writes a table of file type statistics.
fn write_file_types_table<W: Write>(
    writer: &mut W,
    stats: &FileStats,
    options: &FormatterOptions,
) -> Result<()> {
    let title = if options.use_colors {
        "📋 File Types".bold().cyan()
    } else {
        "File Types".normal()
    };
    
    writeln!(writer, "{}", title)?;
    writeln!(writer, "{}", "-".repeat(20))?;
    
    let mut type_data: Vec<_> = stats.file_types.iter().collect();
    type_data.sort_by(|a, b| b.1.count.cmp(&a.1.count));
    
    let type_rows: Vec<FileTypeRow> = type_data
        .into_iter()
        .map(|(ext, type_stats)| FileTypeRow {
            extension: ext.clone(),
            count: format_number(type_stats.count),
            total_size: type_stats.total_size_human(),
            avg_size: humansize::format_size(type_stats.avg_size, humansize::DECIMAL),
        })
        .collect();
    
    if !type_rows.is_empty() {
        let table = Table::new(type_rows).to_string();
        writeln!(writer, "{}", table)?;
    }
    
    Ok(())
}

/// Formats output as JSON.
fn format_json<W: Write>(
    stats: &FileStats,
    writer: &mut W,
    _options: &FormatterOptions,
) -> Result<()> {
    let json = serde_json::to_string_pretty(stats)?;
    writeln!(writer, "{}", json)?;
    Ok(())
}

/// Formats output as CSV.
fn format_csv<W: Write>(
    stats: &FileStats,
    writer: &mut W,
    options: &FormatterOptions,
) -> Result<()> {
    let mut csv_writer = csv::Writer::from_writer(writer);
    
    // Write header
    let mut headers = vec!["path", "size_bytes", "size_human", "is_directory", "file_type"];
    if options.show_permissions {
        headers.push("permissions");
    }
    if options.show_times {
        headers.push("modified");
    }
    csv_writer.write_record(&headers)?;
    
    // Write data rows
    let entries = if let Some(limit) = options.limit {
        &stats.entries[..stats.entries.len().min(limit)]
    } else {
        &stats.entries
    };
    
    for entry in entries {
        let mut record = vec![
            entry.path.to_string_lossy().to_string(),
            entry.size.to_string(),
            entry.size_human(),
            entry.is_dir.to_string(),
            entry.file_type.as_deref().unwrap_or("").to_string(),
        ];
        
        if options.show_permissions {
            record.push(format!("{:o}", entry.permissions));
        }
        if options.show_times {
            record.push(entry.modified.format("%Y-%m-%d %H:%M:%S").to_string());
        }
        
        csv_writer.write_record(&record)?;
    }
    
    csv_writer.flush()?;
    Ok(())
}

/// Formats output as a compact summary.
fn format_summary<W: Write>(
    stats: &FileStats,
    writer: &mut W,
    _options: &FormatterOptions,
) -> Result<()> {
    writeln!(writer, "Files: {} | Dirs: {} | Size: {} | Avg: {}",
        format_number(stats.total_files),
        format_number(stats.total_dirs),
        stats.total_size_human(),
        stats.avg_file_size_human()
    )?;
    Ok(())
}

/// Helper function to format numbers with thousand separators.
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    
    for (i, &ch) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    
    result
}

/// Table row structure for file details.
#[derive(Tabled)]
struct FileTableRow {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Size")]
    size: String,
    #[tabled(rename = "Type")]
    type_field: String,
    #[tabled(rename = "Permissions")]
    permissions: String,
    #[tabled(rename = "Modified")]
    modified: String,
}

/// Table row structure for file type statistics.
#[derive(Tabled)]
struct FileTypeRow {
    #[tabled(rename = "Extension")]
    extension: String,
    #[tabled(rename = "Count")]
    count: String,
    #[tabled(rename = "Total Size")]
    total_size: String,
    #[tabled(rename = "Avg Size")]
    avg_size: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FileStats;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(1234), "1,234");
        assert_eq!(format_number(1234567), "1,234,567");
        assert_eq!(format_number(123), "123");
    }

    #[test]
    fn test_format_json() {
        let stats = FileStats::new();
        let mut output = Vec::new();
        let options = FormatterOptions::default();
        
        format_json(&stats, &mut output, &options).unwrap();
        
        let json_str = String::from_utf8(output).unwrap();
        assert!(json_str.contains("total_files"));
        assert!(json_str.contains("total_size"));
    }

    #[test]
    fn test_format_summary() {
        let stats = FileStats::new();
        let mut output = Vec::new();
        let options = FormatterOptions::default();
        
        format_summary(&stats, &mut output, &options).unwrap();
        
        let summary_str = String::from_utf8(output).unwrap();
        assert!(summary_str.contains("Files:"));
        assert!(summary_str.contains("Dirs:"));
        assert!(summary_str.contains("Size:"));
    }
}
