//! # rfstat - File Statistics CLI Tool
//!
//! `rfstat` is a Rust-based command-line tool designed to display comprehensive file statistics
//! in a human-readable format, optimized for DevOps workflows and system administration tasks.
//!
//! ## Features
//!
//! - **Human-readable output**: File sizes displayed in KB, MB, GB, etc.
//! - **Multiple output formats**: Table, JSON, CSV for different use cases
//! - **Comprehensive statistics**: File counts, size distributions, type analysis
//! - **DevOps-friendly**: Scriptable output formats and filtering options
//! - **Fast and efficient**: Optimized for large directory structures
//!
//! ## Quick Start
//!
//! ```bash
//! # Basic usage - analyze current directory
//! rfstat
//!
//! # Analyze specific directory with detailed output
//! rfstat /var/log --format table --sort size
//!
//! # Export statistics as JSON for automation
//! rfstat /home/user --format json > stats.json
//! ```
//!
//! ## Architecture
//!
//! The tool is organized into several key modules:
//!
//! - [`types`]: Core data structures and type definitions
//! - [`scanner`]: File system traversal and metadata collection
//! - [`stats`]: Statistical analysis and calculations
//! - [`formatter`]: Output formatting for different formats
//! - [`cli`]: Command-line interface and argument parsing
//!
//! ## Examples
//!
//! ### Basic File Statistics
//!
//! ```rust
//! use rfstat::{calculate_stats, scan_directory, Config};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::default();
//! let entries = scan_directory(".", &config)?;
//! let stats = calculate_stats(&entries);
//! println!("Total files: {}", stats.total_files);
//! println!("Total size: {}", stats.total_size_human());
//! # Ok(())
//! # }
//! ```

pub mod types;
pub mod scanner;
pub mod stats;
pub mod formatter;
pub mod cli;
pub mod error;

pub use types::*;
pub use scanner::*;
pub use stats::*;
pub use error::*;
pub use formatter::*;
pub use cli::*;

/// Version information for the rfstat tool
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default configuration for the rfstat tool
pub const DEFAULT_CONFIG: Config = Config {
    format: OutputFormat::Table,
    sort_by: SortBy::Name,
    show_hidden: false,
    recursive: true,
    max_depth: None,
};
