# Advanced Usage Tutorial

This tutorial covers advanced features and techniques for power users who want to get the most out of rfstat.

## What You'll Learn

- Advanced filtering and sorting techniques
- Automation and scripting patterns
- Performance optimization for large directories
- Integration with other tools and systems
- Custom analysis workflows

## Prerequisites

- Completed the [Getting Started Tutorial](getting-started.md)
- Familiarity with command-line tools and scripting
- Basic understanding of JSON and CSV formats

## Advanced Filtering Techniques

### Combining Multiple Filters

You can combine various filters to create precise queries:

```bash
# Find large log files from the last week
rfstat /var/log \
  --extensions "log,gz" \
  --min-size 10MB \
  --sort modified \
  --show-times \
  --limit 20

# Analyze only configuration files
rfstat /etc \
  --extensions "conf,cfg,ini,yaml,yml,json" \
  --sort size \
  --show-permissions
```

### Size Range Analysis

Use size filters to analyze specific file size ranges:

```bash
# Medium-sized files (good candidates for compression)
rfstat /home \
  --min-size 1MB \
  --max-size 100MB \
  --sort size \
  --format csv > medium_files.csv

# Tiny files (potential for consolidation)
rfstat /var/log \
  --max-size 1KB \
  --sort type \
  --format table
```

### Directory Depth Control

Control scanning depth for performance and focus:

```bash
# Quick overview of top-level directories
rfstat /usr --depth 1 --no-recursive

# Analyze up to 3 levels deep
rfstat /home --depth 3 --sort size --limit 50

# Compare shallow vs deep analysis
rfstat /var --depth 2 --format summary
rfstat /var --format summary
```

## Automation and Scripting Patterns

### JSON Processing with jq

Extract specific information from JSON output:

```bash
# Get total size in GB
rfstat /data --format json --quiet | \
  jq -r '(.total_size / 1073741824 | round * 100 / 100) | tostring + " GB"'

# List file types by count
rfstat . --format json --quiet | \
  jq -r '.file_types | to_entries | sort_by(.value.count) | reverse | 
         .[] | "\(.value.count)\t\(.key)"'

# Find directories with most files
rfstat /home --format json --quiet | \
  jq -r '.entries[] | select(.is_dir) | .path' | \
  while read dir; do
    count=$(rfstat "$dir" --format json --quiet 2>/dev/null | jq -r '.total_files // 0')
    echo "$count $dir"
  done | sort -nr | head -10
```

### CSV Analysis with Standard Tools

Process CSV output with Unix tools:

```bash
# Sort files by size (largest first)
rfstat /var/log --format csv --quiet | \
  sort -t, -k2 -nr | head -20

# Count files by extension
rfstat . --format csv --quiet | \
  cut -d, -f5 | sort | uniq -c | sort -nr

# Find files modified in the last day
rfstat /tmp --format csv --show-times --quiet | \
  awk -F, 'NR>1 && $6 > "'$(date -d '1 day ago' '+%Y-%m-%d')'"'
```

### Monitoring Scripts

Create sophisticated monitoring solutions:

```bash
#!/bin/bash
# advanced_monitor.sh - Advanced monitoring with thresholds

declare -A THRESHOLDS=(
    ["/var/log"]="5GB"
    ["/tmp"]="2GB"
    ["/home"]="50GB"
)

declare -A ALERTS=()

check_directory() {
    local path="$1"
    local threshold="$2"
    
    # Convert threshold to bytes
    local threshold_bytes=$(numfmt --from=iec "$threshold")
    
    # Get current size
    local current_size=$(rfstat "$path" --format json --quiet | jq -r '.total_size')
    
    if [[ $current_size -gt $threshold_bytes ]]; then
        local human_size=$(numfmt --to=iec "$current_size")
        ALERTS["$path"]="$human_size exceeds $threshold"
        
        # Get top contributors
        echo "Top contributors in $path:"
        rfstat "$path" --sort size --limit 5 --format csv --quiet | \
          tail -n +2 | while IFS=, read -r path size_bytes size_human rest; do
            echo "  $size_human - $path"
          done
    fi
}

# Check all monitored directories
for path in "${!THRESHOLDS[@]}"; do
    if [[ -d "$path" ]]; then
        check_directory "$path" "${THRESHOLDS[$path]}"
    fi
done

# Report alerts
if [[ ${#ALERTS[@]} -gt 0 ]]; then
    echo "🚨 ALERTS:"
    for path in "${!ALERTS[@]}"; do
        echo "  $path: ${ALERTS[$path]}"
    done
    exit 1
else
    echo "✅ All directories within thresholds"
fi
```

## Performance Optimization

### Large Directory Strategies

When dealing with very large directories:

```bash
# Use summary-only for quick overview
rfstat /massive-directory --summary-only --quiet

# Limit depth to avoid deep recursion
rfstat /usr --depth 3 --format summary

# Use filters to reduce processing
rfstat /var --extensions "log" --min-size 1MB --limit 100

# Sample analysis for very large directories
rfstat /huge-dir --limit 1000 --sort size --format csv | \
  head -100 > sample_analysis.csv
```

### Memory-Efficient Processing

For systems with limited memory:

```bash
# Process in chunks
find /large-directory -maxdepth 1 -type d | while read dir; do
    echo "Processing: $dir"
    rfstat "$dir" --no-recursive --format summary
done

# Use streaming approach
rfstat /big-dir --format csv --quiet | \
  while IFS=, read -r path size_bytes size_human is_dir file_type; do
    # Process each file individually
    if [[ "$size_bytes" -gt 1073741824 ]]; then  # > 1GB
        echo "Large file: $path ($size_human)"
    fi
  done
```

## Integration Patterns

### Database Integration

Store results in SQLite for analysis:

```bash
#!/bin/bash
# db_integration.sh

DB="file_stats.db"

# Create table
sqlite3 "$DB" "
CREATE TABLE IF NOT EXISTS file_stats (
    timestamp TEXT,
    path TEXT,
    size_bytes INTEGER,
    size_human TEXT,
    is_directory BOOLEAN,
    file_type TEXT
);"

# Import data
rfstat /data --format csv --quiet | tail -n +2 | \
while IFS=, read -r path size_bytes size_human is_dir file_type; do
    sqlite3 "$DB" "
    INSERT INTO file_stats VALUES (
        '$(date -Iseconds)',
        '$path',
        $size_bytes,
        '$size_human',
        $is_dir,
        '$file_type'
    );"
done

# Query examples
echo "Largest files:"
sqlite3 "$DB" "
SELECT path, size_human 
FROM file_stats 
WHERE is_directory = 'false' 
ORDER BY size_bytes DESC 
LIMIT 10;"
```

### Prometheus Metrics

Export metrics for monitoring systems:

```bash
#!/bin/bash
# prometheus_exporter.sh

METRICS_FILE="/var/lib/node_exporter/textfile_collector/rfstat.prom"
TEMP_FILE=$(mktemp)

# Generate metrics
{
    echo "# HELP rfstat_directory_size_bytes Directory size in bytes"
    echo "# TYPE rfstat_directory_size_bytes gauge"
    
    echo "# HELP rfstat_directory_files_total Total files in directory"
    echo "# TYPE rfstat_directory_files_total gauge"
    
    for path in "/var/log" "/tmp" "/home"; do
        if [[ -d "$path" ]]; then
            stats=$(rfstat "$path" --format json --quiet 2>/dev/null)
            if [[ $? -eq 0 ]]; then
                size=$(echo "$stats" | jq -r '.total_size')
                files=$(echo "$stats" | jq -r '.total_files')
                
                # Sanitize path for Prometheus label
                label=$(echo "$path" | sed 's/[^a-zA-Z0-9]/_/g')
                
                echo "rfstat_directory_size_bytes{path=\"$path\"} $size"
                echo "rfstat_directory_files_total{path=\"$path\"} $files"
            fi
        fi
    done
} > "$TEMP_FILE"

# Atomic update
mv "$TEMP_FILE" "$METRICS_FILE"
```

### Log Analysis Pipeline

Create a comprehensive log analysis workflow:

```bash
#!/bin/bash
# log_analysis_pipeline.sh

LOG_DIR="/var/log"
OUTPUT_DIR="/tmp/log_analysis"
DATE=$(date +%Y-%m-%d)

mkdir -p "$OUTPUT_DIR"

echo "🔍 Starting log analysis pipeline..."

# 1. Overall log directory analysis
echo "📊 Generating overall statistics..."
rfstat "$LOG_DIR" --format json --quiet > "$OUTPUT_DIR/overall_stats.json"

# 2. Log rotation analysis
echo "🔄 Analyzing log rotation..."
rfstat "$LOG_DIR" --extensions "log,gz,1,2,3,4,5" --format csv --quiet > "$OUTPUT_DIR/rotation_analysis.csv"

# 3. Large log detection
echo "📈 Detecting large logs..."
rfstat "$LOG_DIR" --min-size 100MB --sort size --format csv --quiet > "$OUTPUT_DIR/large_logs.csv"

# 4. Recent activity analysis
echo "⏰ Analyzing recent activity..."
rfstat "$LOG_DIR" --sort modified --show-times --limit 50 --format csv --quiet > "$OUTPUT_DIR/recent_activity.csv"

# 5. Generate summary report
echo "📋 Generating summary report..."
{
    echo "# Log Analysis Report - $DATE"
    echo
    echo "## Overall Statistics"
    jq -r '"Total Files: " + (.total_files | tostring) + 
           "\nTotal Size: " + (.total_size / 1073741824 | round * 100 / 100 | tostring) + " GB" +
           "\nAverage File Size: " + (.avg_file_size / 1024 | round * 100 / 100 | tostring) + " KB"' \
           "$OUTPUT_DIR/overall_stats.json"
    
    echo
    echo "## Top File Types"
    jq -r '.file_types | to_entries | sort_by(.value.total_size) | reverse | 
           .[:5] | .[] | "- " + .key + ": " + (.value.count | tostring) + " files, " + 
           (.value.total_size / 1048576 | round * 100 / 100 | tostring) + " MB"' \
           "$OUTPUT_DIR/overall_stats.json"
    
    echo
    echo "## Large Logs (>100MB)"
    if [[ -s "$OUTPUT_DIR/large_logs.csv" ]]; then
        tail -n +2 "$OUTPUT_DIR/large_logs.csv" | head -10 | \
        while IFS=, read -r path size_bytes size_human rest; do
            echo "- $path: $size_human"
        done
    else
        echo "No large logs found."
    fi
    
} > "$OUTPUT_DIR/summary_report.md"

echo "✅ Analysis complete. Results in $OUTPUT_DIR/"
echo "📄 Summary report: $OUTPUT_DIR/summary_report.md"
```

## Troubleshooting Advanced Usage

### Performance Issues

```bash
# Profile large directory scans
time rfstat /large-directory --summary-only

# Monitor memory usage
/usr/bin/time -v rfstat /large-directory --format json > /dev/null

# Use strace to debug I/O patterns
strace -c rfstat /directory 2>&1 | grep -E "(openat|stat|getdents)"
```

### Debugging Complex Filters

```bash
# Test filters incrementally
rfstat /path --extensions "log" --format summary
rfstat /path --extensions "log" --min-size 1MB --format summary
rfstat /path --extensions "log" --min-size 1MB --sort size --format summary

# Validate JSON output
rfstat /path --format json --quiet | jq empty && echo "Valid JSON"

# Check CSV format
rfstat /path --format csv --quiet | head -1  # Check headers
```

## Next Steps

Now that you've mastered advanced usage:

1. **Explore Specific Use Cases**: Check out our [How-to Guides](../how-to/) for targeted solutions
2. **Understand the Architecture**: Read the [Architecture Documentation](../explanation/architecture.md)
3. **Contribute**: Help improve rfstat by contributing to the project
4. **Share**: Create your own automation scripts and share them with the community

You're now equipped to use rfstat as a powerful tool in your DevOps and system administration toolkit!
