//! Main entry point for the rfstat CLI tool.
//!
//! This module contains the main function and application logic that ties together
//! all the components of rfstat: CLI parsing, file scanning, statistics calculation,
//! and output formatting.

use clap::Parser;
use log::{debug, error, info, warn};
use rfstat::{
    calculate_stats, filter_entries, format_output, scan_directory, sort_entries,
    scanner::FileFilters, Cli, Config, FormatterOptions, Result, RfstatError,
};
use std::io::{self, IsTerminal};
use std::process;

fn main() {
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Initialize logging
    if let Err(e) = init_logging(&cli) {
        eprintln!("Failed to initialize logging: {}", e);
        process::exit(1);
    }
    
    // Run the main application logic
    if let Err(e) = run(cli) {
        if !matches!(e, RfstatError::Generic { .. }) {
            error!("Application error: {}", e);
        }
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

/// Main application logic.
fn run(cli: Cli) -> Result<()> {
    debug!("Starting rfstat with path: {}", cli.path.display());
    
    // Validate input path
    if !cli.path.exists() {
        return Err(RfstatError::path_not_found(&cli.path));
    }
    
    // Convert CLI args to config
    let config = cli.to_config();
    debug!("Configuration: {:?}", config);
    
    // Scan the directory
    info!("Scanning directory: {}", cli.path.display());
    let mut entries = scan_directory(&cli.path, &config)?;
    info!("Found {} entries", entries.len());
    
    // Apply additional filters from CLI
    let filters = create_file_filters(&cli)?;
    if has_active_filters(&filters) {
        let original_count = entries.len();
        entries = filter_entries(&entries, &filters);
        debug!("Filtered from {} to {} entries", original_count, entries.len());
    }
    
    // Sort entries
    sort_entries(&mut entries, config.sort_by);
    debug!("Sorted entries by {:?}", config.sort_by);
    
    // Calculate statistics
    let stats = calculate_stats(&entries);
    debug!("Calculated statistics for {} files, {} directories", 
           stats.total_files, stats.total_dirs);
    
    // Create formatter options
    let formatter_options = FormatterOptions {
        use_colors: should_use_colors(&cli),
        limit: cli.limit,
        summary_only: cli.summary_only,
        show_permissions: cli.show_permissions,
        show_times: cli.show_times,
        show_file_types: !cli.summary_only,
    };
    
    // Format and output results
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    format_output(&stats, config.format, &mut handle, &formatter_options)?;
    
    info!("Successfully processed {} files and {} directories", 
          stats.total_files, stats.total_dirs);
    
    Ok(())
}

/// Initializes logging based on CLI options.
fn init_logging(cli: &Cli) -> Result<()> {
    let log_level = if cli.verbose {
        "debug"
    } else if cli.quiet {
        "error"
    } else {
        "info"
    };
    
    env_logger::Builder::from_env(
        env_logger::Env::default()
            .default_filter_or(log_level)
    )
    .format_timestamp(None)
    .format_module_path(false)
    .format_target(false)
    .init();
    
    Ok(())
}

/// Creates file filters from CLI arguments.
fn create_file_filters(cli: &Cli) -> Result<FileFilters> {
    let extensions = cli.parse_extensions();
    let min_size = cli.get_min_size_bytes()
        .map_err(|e| RfstatError::config(format!("Invalid min-size: {}", e)))?;
    let max_size = cli.get_max_size_bytes()
        .map_err(|e| RfstatError::config(format!("Invalid max-size: {}", e)))?;
    
    Ok(FileFilters {
        extensions,
        min_size,
        max_size,
        files_only: false,
        dirs_only: false,
    })
}

/// Checks if any filters are active.
fn has_active_filters(filters: &FileFilters) -> bool {
    filters.extensions.is_some() 
        || filters.min_size.is_some() 
        || filters.max_size.is_some()
        || filters.files_only 
        || filters.dirs_only
}

/// Determines whether to use colors in output.
fn should_use_colors(cli: &Cli) -> bool {
    // Don't use colors if:
    // 1. Output is not a terminal (being piped/redirected)
    // 2. Quiet mode is enabled
    // 3. Output format is not table or summary
    if cli.quiet {
        return false;
    }
    
    match cli.format {
        rfstat::cli::CliOutputFormat::Json | rfstat::cli::CliOutputFormat::Csv => false,
        rfstat::cli::CliOutputFormat::Table | rfstat::cli::CliOutputFormat::Summary => {
            io::stdout().is_terminal()
        }
    }
}

/// Handles graceful shutdown on interrupt signals.
fn setup_signal_handlers() {
    // This is a placeholder for signal handling
    // In a real implementation, you might want to handle SIGINT/SIGTERM
    // to allow for graceful cleanup of resources
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_create_file_filters_empty() {
        let cli = Cli {
            extensions: None,
            min_size: None,
            max_size: None,
            ..Default::default()
        };
        
        let filters = create_file_filters(&cli).unwrap();
        assert!(filters.extensions.is_none());
        assert!(filters.min_size.is_none());
        assert!(filters.max_size.is_none());
    }

    #[test]
    fn test_create_file_filters_with_values() {
        let cli = Cli {
            extensions: Some("txt,log".to_string()),
            min_size: Some("1KB".to_string()),
            max_size: Some("1MB".to_string()),
            ..Default::default()
        };
        
        let filters = create_file_filters(&cli).unwrap();
        assert_eq!(filters.extensions, Some(vec!["txt".to_string(), "log".to_string()]));
        assert_eq!(filters.min_size, Some(1000));
        assert_eq!(filters.max_size, Some(1_000_000));
    }

    #[test]
    fn test_has_active_filters() {
        let empty_filters = FileFilters::default();
        assert!(!has_active_filters(&empty_filters));
        
        let filters_with_extension = FileFilters {
            extensions: Some(vec!["txt".to_string()]),
            ..Default::default()
        };
        assert!(has_active_filters(&filters_with_extension));
    }

    #[test]
    fn test_should_use_colors() {
        let cli = Cli {
            quiet: true,
            ..Default::default()
        };
        assert!(!should_use_colors(&cli));
        
        let cli = Cli {
            format: rfstat::cli::CliOutputFormat::Json,
            ..Default::default()
        };
        assert!(!should_use_colors(&cli));
    }

    #[test]
    fn test_run_with_temp_directory() {
        let temp_dir = TempDir::new().unwrap();
        let cli = Cli {
            path: temp_dir.path().to_path_buf(),
            quiet: true,
            ..Default::default()
        };
        
        // This should not panic or return an error
        let result = run(cli);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_with_nonexistent_path() {
        let cli = Cli {
            path: PathBuf::from("/nonexistent/path"),
            ..Default::default()
        };
        
        let result = run(cli);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RfstatError::PathNotFound { .. }));
    }
}
