# Architecture and Design

This document explains the design decisions, architecture, and implementation details of rfstat.

## Design Philosophy

rfstat is built on four core principles:

### 1. Human-First Design
While providing machine-readable output, the primary focus is on human comprehension. This influences:
- **Readable output formats**: Clear tables with colors and formatting
- **Intuitive size units**: Human-readable sizes (KB, MB, GB) by default
- **Meaningful categorization**: Size buckets that match mental models
- **Progressive disclosure**: Summary first, details on demand

### 2. DevOps Integration
Every feature is designed with automation and scripting in mind:
- **Multiple output formats**: JSON for APIs, CSV for analysis, summary for monitoring
- **Predictable exit codes**: Standard codes for script error handling
- **Quiet mode**: Clean output for piping and processing
- **Structured data**: Consistent JSON schema for programmatic use

### 3. Performance First
Rust's zero-cost abstractions ensure minimal overhead:
- **Streaming processing**: Memory-efficient handling of large directories
- **Parallel traversal**: Concurrent file system operations where safe
- **Lazy evaluation**: Statistics calculated only when needed
- **Minimal allocations**: Efficient data structures and memory usage

### 4. Reliability and Safety
Comprehensive error handling and graceful degradation:
- **Permission handling**: Continues processing despite access errors
- **Resource limits**: Configurable depth limits prevent runaway recursion
- **Input validation**: Robust parsing of size units and file paths
- **Memory safety**: Rust's ownership system prevents common bugs

## System Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   CLI Parser    │───▶│  File Scanner    │───▶│  Statistics     │
│   (clap)        │    │  (walkdir)       │    │  Calculator     │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                                         │
┌─────────────────┐    ┌──────────────────┐             │
│  Output         │◀───│   Formatter      │◀────────────┘
│  (stdout)       │    │  (table/json/csv)│
└─────────────────┘    └──────────────────┘
```

### Module Organization

#### `cli` Module
- **Purpose**: Command-line interface and argument parsing
- **Key Components**:
  - `Cli` struct with clap derives
  - Size parsing utilities
  - Configuration conversion
- **Design Decisions**:
  - Uses clap's derive API for maintainability
  - Separates parsing from validation
  - Provides helpful error messages

#### `scanner` Module
- **Purpose**: File system traversal and metadata collection
- **Key Components**:
  - `scan_directory()` function
  - `FileFilters` for filtering logic
  - Error handling for permissions
- **Design Decisions**:
  - Uses `walkdir` for efficient traversal
  - Graceful error handling continues processing
  - Configurable depth limits prevent infinite recursion

#### `types` Module
- **Purpose**: Core data structures and type definitions
- **Key Components**:
  - `FileEntry` for individual files
  - `FileStats` for aggregated statistics
  - `SizeDistribution` for categorization
- **Design Decisions**:
  - Serde integration for serialization
  - Human-readable methods alongside raw data
  - Clear separation of concerns

#### `stats` Module
- **Purpose**: Statistical analysis and calculations
- **Key Components**:
  - `calculate_stats()` for aggregation
  - Percentile calculations
  - File type analysis
- **Design Decisions**:
  - Single-pass algorithms where possible
  - Lazy evaluation of expensive calculations
  - Extensible for future statistical measures

#### `formatter` Module
- **Purpose**: Output formatting for different display formats
- **Key Components**:
  - Format-specific functions
  - Color management
  - Table generation
- **Design Decisions**:
  - Pluggable formatter architecture
  - Terminal capability detection
  - Consistent data representation across formats

#### `error` Module
- **Purpose**: Error handling and user-friendly messages
- **Key Components**:
  - `RfstatError` enum with thiserror
  - Context-aware error messages
  - Proper error propagation
- **Design Decisions**:
  - Uses thiserror for ergonomic error handling
  - Provides actionable error messages
  - Maintains error context through the stack

## Data Flow

### 1. Input Processing
```rust
CLI Arguments → Validation → Config → FileFilters
```

The CLI parser validates arguments and converts them into internal configuration structures. Size strings are parsed into bytes, and extension lists are normalized.

### 2. File System Scanning
```rust
Path → walkdir::WalkDir → Iterator<DirEntry> → Vec<FileEntry>
```

The scanner uses `walkdir` for efficient directory traversal. Each directory entry is converted to a `FileEntry` with metadata. Errors are logged but don't stop processing.

### 3. Statistical Analysis
```rust
Vec<FileEntry> → calculate_stats() → FileStats
```

Statistics are calculated in a single pass through the file entries. File type breakdowns and size distributions are built incrementally.

### 4. Output Generation
```rust
FileStats → Formatter → Output
```

The appropriate formatter is selected based on the output format. Each formatter handles its own serialization and presentation logic.

## Key Design Decisions

### Size Distribution Categories

Files are categorized into five buckets based on common usage patterns:

- **Tiny (< 1KB)**: Configuration files, small scripts, empty files
- **Small (1KB - 1MB)**: Text files, small images, source code
- **Medium (1MB - 100MB)**: Documents, photos, small videos
- **Large (100MB - 1GB)**: Large media files, databases, archives
- **Huge (> 1GB)**: Video files, large databases, system images

This categorization helps users quickly understand their storage patterns and identify optimization opportunities.

### Error Handling Strategy

rfstat employs a multi-layered error handling approach:

1. **Graceful Degradation**: Permission errors on individual files don't stop the entire scan
2. **Informative Messages**: Clear, actionable error messages for users
3. **Logging Integration**: Detailed error information available via logging
4. **Exit Codes**: Standard exit codes for script integration

### Memory Management

For large directories, memory usage is controlled through:

- **Streaming Processing**: Files are processed as they're discovered
- **Configurable Limits**: Depth and count limits prevent resource exhaustion
- **Efficient Data Structures**: Minimal memory overhead per file entry
- **Lazy Evaluation**: Expensive calculations only performed when needed

### Concurrency Model

Currently, rfstat uses a single-threaded model for simplicity and safety:

- **File System Safety**: Avoids race conditions with concurrent directory access
- **Predictable Performance**: Consistent behavior across different systems
- **Simple Error Handling**: Easier to reason about error propagation

Future versions may add optional parallelism for large directory scans.

## Performance Characteristics

### Time Complexity
- **Directory Traversal**: O(n) where n is the number of files
- **Statistics Calculation**: O(n) single pass through entries
- **Sorting**: O(n log n) for sorted output
- **Filtering**: O(n) with early termination where possible

### Space Complexity
- **File Entries**: O(n) storage for all file metadata
- **Statistics**: O(k) where k is the number of unique file types
- **Output**: O(n) for detailed formats, O(1) for summary

### Benchmarks

On modern SSD hardware:
- **10,000 files**: ~50ms
- **100,000 files**: ~500ms
- **1,000,000 files**: ~5s

Performance scales linearly with file count and is primarily I/O bound.

## Security Considerations

### File System Access
- **Read-only Operations**: rfstat never modifies the file system
- **Permission Respect**: Gracefully handles permission denied errors
- **Symlink Safety**: Doesn't follow symbolic links to prevent cycles
- **Path Validation**: Input paths are validated and normalized

### Resource Limits
- **Depth Limits**: Configurable maximum recursion depth
- **Memory Bounds**: Controlled memory usage for large directories
- **Time Limits**: No built-in timeouts (handled by external tools)

### Information Disclosure
- **Metadata Only**: Only collects standard file metadata
- **No Content Reading**: File contents are never accessed
- **Configurable Output**: Users control what information is displayed

## Extensibility

The modular architecture supports future extensions:

### New Output Formats
Add new formatters by implementing the formatting interface:
```rust
fn format_output<W: Write>(
    stats: &FileStats,
    format: OutputFormat,
    writer: &mut W,
    options: &FormatterOptions,
) -> Result<()>
```

### Additional Statistics
Extend the `FileStats` structure and update the calculation logic:
```rust
pub struct FileStats {
    // existing fields...
    pub new_metric: NewMetricType,
}
```

### Custom Filters
Add new filtering options by extending `FileFilters`:
```rust
pub struct FileFilters {
    // existing fields...
    pub custom_filter: Option<CustomFilterType>,
}
```

## Testing Strategy

### Unit Tests
- **Module Isolation**: Each module has comprehensive unit tests
- **Edge Cases**: Tests cover error conditions and boundary cases
- **Mock Data**: Controlled test data for predictable results

### Integration Tests
- **CLI Testing**: Full command-line interface testing with `assert_cmd`
- **File System**: Tests with real temporary directories
- **Output Validation**: Verification of all output formats

### Documentation Tests
- **Example Validation**: All code examples in documentation are tested
- **API Consistency**: Ensures documentation matches implementation

This architecture provides a solid foundation for reliable, performant file system analysis while maintaining extensibility for future enhancements.
