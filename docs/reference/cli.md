# CLI Reference

Complete reference for all rfstat command-line options and usage patterns.

## Synopsis

```
rfstat [OPTIONS] [PATH]
```

## Arguments

### PATH
- **Type**: String (file path)
- **Default**: `.` (current directory)
- **Description**: Path to analyze (file or directory)

**Examples:**
```bash
rfstat                    # Analyze current directory
rfstat /var/log          # Analyze /var/log directory
rfstat ~/Documents       # Analyze Documents folder
rfstat file.txt          # Analyze single file
```

## Options

### Output Format Options

#### `-f, --format <FORMAT>`
- **Type**: Enum
- **Default**: `table`
- **Values**: `table`, `json`, `csv`, `summary`
- **Description**: Output format for results

**Examples:**
```bash
rfstat --format table     # Human-readable table (default)
rfstat --format json      # JSON for automation
rfstat --format csv       # CSV for spreadsheets
rfstat --format summary   # Compact one-line output
```

**Output Examples:**

**Table Format:**
```
📊 File Statistics Summary
==================================================
Total Files:      1,234
Total Directories: 45
Total Size:       2.3 GB
[...]
```

**JSON Format:**
```json
{
  "total_files": 1234,
  "total_dirs": 45,
  "total_size": 2468013579,
  "file_types": {...}
}
```

**CSV Format:**
```csv
path,size_bytes,size_human,is_directory,file_type
./file1.txt,1024,1.02 kB,false,txt
./dir1,0,0 B,true,
```

**Summary Format:**
```
Files: 1,234 | Dirs: 45 | Size: 2.3 GB | Avg: 1.9 MB
```

### Sorting Options

#### `-s, --sort <SORT>`
- **Type**: Enum
- **Default**: `name`
- **Values**: `name`, `size`, `modified`, `type`
- **Description**: Sort results by specified field

**Examples:**
```bash
rfstat --sort name        # Sort alphabetically (default)
rfstat --sort size        # Sort by size (largest first)
rfstat --sort modified    # Sort by modification time (newest first)
rfstat --sort type        # Sort by file extension/type
```

### Filtering Options

#### `-a, --all`
- **Type**: Flag
- **Default**: `false`
- **Description**: Include hidden files and directories

```bash
rfstat --all              # Include .hidden files
rfstat -a                 # Short form
```

#### `--extensions <EXTENSIONS>`
- **Type**: String (comma-separated)
- **Default**: None (all extensions)
- **Description**: Filter by file extensions

```bash
rfstat --extensions "txt,log,conf"
rfstat --extensions "rs,toml"        # Rust project files
rfstat --extensions "jpg,png,gif"    # Image files
```

#### `--min-size <SIZE>`
- **Type**: String (size with unit)
- **Default**: None
- **Description**: Minimum file size filter

```bash
rfstat --min-size 1MB     # Files >= 1MB
rfstat --min-size 500KB   # Files >= 500KB
rfstat --min-size 1GB     # Files >= 1GB
```

#### `--max-size <SIZE>`
- **Type**: String (size with unit)
- **Default**: None
- **Description**: Maximum file size filter

```bash
rfstat --max-size 100MB   # Files <= 100MB
rfstat --max-size 1GB     # Files <= 1GB
```

**Size Units:**
- **Decimal (SI)**: `B`, `KB`, `MB`, `GB`, `TB`
- **Binary (IEC)**: `KiB`, `MiB`, `GiB`, `TiB`

### Traversal Options

#### `-R, --no-recursive`
- **Type**: Flag
- **Default**: `false` (recursive by default)
- **Description**: Disable recursive directory traversal

```bash
rfstat --no-recursive     # Only current directory level
rfstat -R                 # Short form
```

#### `-d, --depth <DEPTH>`
- **Type**: Integer
- **Default**: Unlimited
- **Description**: Maximum depth for recursive scanning

```bash
rfstat --depth 1          # Current directory only
rfstat --depth 3          # Up to 3 levels deep
rfstat --depth 10         # Up to 10 levels deep
```

### Display Options

#### `-l, --limit <COUNT>`
- **Type**: Integer
- **Default**: Unlimited
- **Description**: Limit number of files shown in detailed output

```bash
rfstat --limit 10         # Show top 10 files
rfstat --limit 100        # Show top 100 files
```

#### `--summary-only`
- **Type**: Flag
- **Default**: `false`
- **Description**: Show only summary statistics (no individual files)

```bash
rfstat --summary-only     # Skip individual file listings
```

#### `--show-permissions`
- **Type**: Flag
- **Default**: `false`
- **Description**: Show file permissions in output

```bash
rfstat --show-permissions # Include permission column
```

#### `--show-times`
- **Type**: Flag
- **Default**: `false`
- **Description**: Show modification times

```bash
rfstat --show-times       # Include modification time column
```

### Logging Options

#### `-v, --verbose`
- **Type**: Flag
- **Default**: `false`
- **Description**: Enable verbose logging

```bash
rfstat --verbose          # Show debug information
rfstat -v                 # Short form
```

#### `-q, --quiet`
- **Type**: Flag
- **Default**: `false`
- **Description**: Suppress all output except results

```bash
rfstat --quiet            # No progress messages
rfstat -q                 # Short form
```

### Help Options

#### `-h, --help`
- **Type**: Flag
- **Description**: Print help information

```bash
rfstat --help             # Show help
rfstat -h                 # Short form
```

#### `-V, --version`
- **Type**: Flag
- **Description**: Print version information

```bash
rfstat --version          # Show version
rfstat -V                 # Short form
```

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | General error (file not found, permission denied, etc.) |
| `2` | Invalid command line arguments |
| `3` | I/O error during file system operations |

## Environment Variables

### `RUST_LOG`
Controls logging level for debugging:
```bash
RUST_LOG=debug rfstat /path    # Debug logging
RUST_LOG=info rfstat /path     # Info logging
RUST_LOG=error rfstat /path    # Error logging only
```

### `NO_COLOR`
Disables colored output when set:
```bash
NO_COLOR=1 rfstat /path        # Disable colors
```

## Usage Patterns

### Basic Analysis
```bash
# Quick directory overview
rfstat /var/log --format summary

# Detailed analysis
rfstat /home/user --format table
```

### Finding Large Files
```bash
# Top 20 largest files
rfstat . --sort size --limit 20

# Files larger than 100MB
rfstat . --min-size 100MB --sort size
```

### File Type Analysis
```bash
# Analyze specific file types
rfstat . --extensions "log,txt,conf"

# Sort by file type
rfstat . --sort type --format csv
```

### Automation and Scripting
```bash
# JSON output for scripts
rfstat /data --format json --quiet > stats.json

# CSV for spreadsheet analysis
rfstat /logs --format csv --extensions "log" > log_analysis.csv

# One-liner for monitoring
rfstat /critical --format summary --quiet
```

### Performance Optimization
```bash
# Limit depth for large directories
rfstat /usr --depth 3

# Non-recursive for quick analysis
rfstat /var --no-recursive

# Limit output for performance
rfstat /big-dir --limit 100 --summary-only
```

## Common Option Combinations

### System Monitoring
```bash
rfstat /var/log --extensions "log,gz" --sort size --limit 10 --quiet
```

### Development Analysis
```bash
rfstat . --extensions "rs,toml,md" --sort modified --show-times
```

### Storage Cleanup
```bash
rfstat /tmp --min-size 100MB --sort size --format csv
```

### Security Audit
```bash
rfstat /etc --show-permissions --show-times --all --format csv
```

This reference covers all available options and common usage patterns. For practical examples, see the [How-to Guides](../how-to/) section.
