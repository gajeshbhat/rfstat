# Getting Started with rfstat

Welcome to rfstat! This tutorial will guide you through your first steps with the tool, from installation to performing your first file system analysis.

## What You'll Learn

By the end of this tutorial, you'll be able to:
- Install and set up rfstat
- Perform basic directory analysis
- Understand the output formats
- Use basic filtering and sorting options

## Prerequisites

- A Linux or macOS system
- Rust 1.70 or later (for building from source)
- Basic familiarity with the command line

## Step 1: Installation

### Option A: Build from Source (Recommended)

```bash
# Clone the repository
git clone https://github.com/gajeshbhat/rfstat.git
cd rfstat

# Build and install
cargo install --path .

# Verify installation
rfstat --version
```

### Option B: Development Build

```bash
# For development and testing
cargo build --release

# Run directly from target directory
./target/release/rfstat --version
```

## Step 2: Your First Analysis

Let's start with analyzing the current directory:

```bash
# Basic analysis of current directory
rfstat
```

You should see output similar to this:

```
📊 File Statistics Summary
==================================================
Total Files:      1,234
Total Directories: 45
Total Size:       2.3 GB
Average File Size: 1.9 MB
Largest File:     45.2 MB
Smallest File:    0 B

Size Distribution:
  Tiny (< 1KB):     234
  Small (1KB-1MB):  567
  Medium (1MB-100MB): 89
  Large (100MB-1GB): 12
  Huge (> 1GB):     0

📁 File Details
------------------------------
[Individual file listings...]

📋 File Types
--------------------
[File type breakdown...]
```

## Step 3: Understanding the Output

The output is organized into several sections:

### Summary Statistics
- **Total Files/Directories**: Count of files and folders
- **Total Size**: Combined size of all files
- **Average File Size**: Mean file size
- **Largest/Smallest File**: Size extremes

### Size Distribution
Files are categorized into five buckets:
- **Tiny** (< 1KB): Configuration files, small scripts
- **Small** (1KB-1MB): Text files, source code
- **Medium** (1MB-100MB): Documents, images
- **Large** (100MB-1GB): Large media files, databases
- **Huge** (> 1GB): Video files, system images

### File Details
Individual file listings with:
- Name and size
- File type/extension
- Optional permissions and timestamps

### File Types
Breakdown by file extension showing:
- Count of files per type
- Total and average size per type

## Step 4: Analyzing Different Directories

Try analyzing different directories:

```bash
# Analyze your home directory
rfstat ~

# Analyze system logs (may require sudo)
rfstat /var/log

# Analyze a specific project directory
rfstat /path/to/your/project
```

## Step 5: Using Different Output Formats

rfstat supports multiple output formats for different use cases:

### Table Format (Default)
```bash
rfstat . --format table
```
Human-readable with colors and formatting.

### JSON Format
```bash
rfstat . --format json
```
Perfect for scripts and automation:
```json
{
  "total_files": 1234,
  "total_dirs": 45,
  "total_size": 2468013579,
  "file_types": {...},
  "entries": [...]
}
```

### CSV Format
```bash
rfstat . --format csv
```
Spreadsheet-compatible output.

### Summary Format
```bash
rfstat . --format summary
```
Compact one-line output:
```
Files: 1,234 | Dirs: 45 | Size: 2.3 GB | Avg: 1.9 MB
```

## Step 6: Basic Filtering and Sorting

### Sorting Options
```bash
# Sort by file size (largest first)
rfstat . --sort size

# Sort by modification time (newest first)
rfstat . --sort modified

# Sort by file type
rfstat . --sort type
```

### Basic Filtering
```bash
# Show only the top 10 files
rfstat . --limit 10

# Include hidden files
rfstat . --all

# Analyze only current directory (no subdirectories)
rfstat . --no-recursive
```

## Step 7: Practical Examples

### Find Large Files
```bash
# Find the 20 largest files
rfstat . --sort size --limit 20
```

### Analyze Specific File Types
```bash
# Analyze only log files
rfstat /var/log --extensions "log,gz"
```

### Quick Directory Overview
```bash
# Get a quick summary
rfstat /important/directory --format summary
```

## What's Next?

Now that you've mastered the basics, you can:

1. **Explore Advanced Features**: Check out the [Advanced Usage Tutorial](advanced-usage.md)
2. **Solve Specific Problems**: Browse our [How-to Guides](../how-to/)
3. **Learn All Options**: Read the [CLI Reference](../reference/cli.md)
4. **Understand the Design**: Explore [Architecture Documentation](../explanation/architecture.md)

## Troubleshooting

### Common Issues

**Permission Denied Errors**
```bash
# Some files may be inaccessible - this is normal
rfstat /root  # May show permission warnings
```

**Large Directory Performance**
```bash
# Limit depth for better performance
rfstat /usr --depth 3
```

**Command Not Found**
```bash
# Make sure cargo's bin directory is in your PATH
export PATH="$HOME/.cargo/bin:$PATH"
```

## Getting Help

- Use `rfstat --help` for quick reference
- Check our [How-to Guides](../how-to/) for specific problems
- Visit the [CLI Reference](../reference/cli.md) for complete documentation
- Open an issue on [GitHub](https://github.com/gajeshbhat/rfstat) for bugs

Congratulations! You've completed the getting started tutorial. You now have the foundation to use rfstat effectively for file system analysis.
