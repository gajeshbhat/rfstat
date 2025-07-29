# How to Optimize Storage with rfstat

This guide shows you how to identify storage optimization opportunities using rfstat's analysis capabilities.

## Problem

You need to:
- Identify files consuming excessive disk space
- Find candidates for compression or archival
- Detect duplicate or unnecessary files
- Optimize storage allocation across directories

## Solution Overview

We'll use rfstat to systematically analyze storage patterns and identify optimization opportunities through:
1. Large file detection and analysis
2. File type distribution analysis
3. Age-based cleanup identification
4. Storage pattern optimization

## Prerequisites

- rfstat installed and accessible
- Sufficient permissions to analyze target directories
- Basic understanding of file systems and storage concepts

## Step 1: Identify Large Files

### Find the Biggest Storage Consumers

```bash
# Find the 50 largest files system-wide
rfstat /home --sort size --limit 50 --min-size 100MB --format csv > large_files.csv

# Analyze by directory
for dir in /home/*/; do
    echo "=== $(basename "$dir") ==="
    rfstat "$dir" --sort size --limit 5 --format summary
done
```

### Analyze Large File Patterns

```bash
#!/bin/bash
# large_file_analyzer.sh

analyze_large_files() {
    local path="$1"
    local threshold="${2:-100MB}"
    
    echo "🔍 Analyzing large files in $path (threshold: $threshold)"
    
    # Get large files data
    rfstat "$path" --min-size "$threshold" --sort size --format json --quiet > /tmp/large_files.json
    
    # Extract insights
    local total_large=$(jq -r '.total_files' /tmp/large_files.json)
    local total_size=$(jq -r '.total_size' /tmp/large_files.json)
    local total_size_gb=$(echo "scale=2; $total_size / 1073741824" | bc -l)
    
    echo "📊 Found $total_large large files consuming ${total_size_gb}GB"
    
    # Top file types by size
    echo "📋 Top file types:"
    jq -r '.file_types | to_entries | sort_by(.value.total_size) | reverse | 
           .[:5] | .[] | "  " + .key + ": " + (.value.total_size / 1073741824 | 
           round * 100 / 100 | tostring) + "GB (" + (.value.count | tostring) + " files)"' \
           /tmp/large_files.json
    
    # Largest individual files
    echo "📁 Largest files:"
    jq -r '.entries | sort_by(.size) | reverse | .[:10] | 
           .[] | "  " + (.size / 1073741824 | round * 100 / 100 | tostring) + 
           "GB - " + .path' /tmp/large_files.json
}

# Usage examples
analyze_large_files "/home" "500MB"
analyze_large_files "/var/log" "100MB"
```

## Step 2: Compression Candidates

### Identify Files Suitable for Compression

```bash
# Find medium-sized text files (good compression candidates)
rfstat /var/log --extensions "log,txt,csv,json,xml" \
  --min-size 1MB --max-size 100MB \
  --sort size --format csv > compression_candidates.csv

# Analyze compression potential
#!/bin/bash
# compression_analyzer.sh

analyze_compression_potential() {
    local path="$1"
    
    echo "🗜️  Analyzing compression potential in $path"
    
    # Text-based files that compress well
    local text_extensions="log,txt,csv,json,xml,html,css,js,sql,conf,cfg,ini"
    
    rfstat "$path" --extensions "$text_extensions" \
      --min-size 1MB --format json --quiet > /tmp/compress_analysis.json
    
    local total_size=$(jq -r '.total_size' /tmp/compress_analysis.json)
    local total_files=$(jq -r '.total_files' /tmp/compress_analysis.json)
    local size_gb=$(echo "scale=2; $total_size / 1073741824" | bc -l)
    
    # Estimate compression savings (typical 70-80% for text files)
    local estimated_savings=$(echo "scale=2; $size_gb * 0.75" | bc -l)
    
    echo "📊 Compression Analysis:"
    echo "  Files: $total_files"
    echo "  Current size: ${size_gb}GB"
    echo "  Estimated savings: ${estimated_savings}GB (75% compression)"
    
    # Show top candidates
    echo "🎯 Top compression candidates:"
    jq -r '.entries | sort_by(.size) | reverse | .[:10] | 
           .[] | "  " + (.size / 1048576 | round * 100 / 100 | tostring) + 
           "MB - " + .path' /tmp/compress_analysis.json
}

analyze_compression_potential "/var/log"
analyze_compression_potential "/home/user/documents"
```

### Automated Compression Script

```bash
#!/bin/bash
# auto_compress.sh - Compress old log files

COMPRESS_THRESHOLD_DAYS=7
MIN_SIZE="1MB"
LOG_EXTENSIONS="log,txt"

compress_old_files() {
    local path="$1"
    
    echo "🗜️  Compressing old files in $path"
    
    # Find files older than threshold
    find "$path" -type f -mtime +$COMPRESS_THRESHOLD_DAYS \
      \( -name "*.log" -o -name "*.txt" \) \
      -size +1M -not -name "*.gz" | while read -r file; do
        
        local size_before=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file")
        
        echo "  Compressing: $file"
        gzip "$file"
        
        local size_after=$(stat -f%z "${file}.gz" 2>/dev/null || stat -c%s "${file}.gz")
        local savings=$((size_before - size_after))
        local savings_mb=$(echo "scale=2; $savings / 1048576" | bc -l)
        
        echo "    Saved: ${savings_mb}MB"
    done
}

# Usage
compress_old_files "/var/log"
```

## Step 3: Archive Candidates

### Identify Old Files for Archival

```bash
# Find old large files (candidates for archival)
#!/bin/bash
# archive_candidates.sh

find_archive_candidates() {
    local path="$1"
    local age_days="${2:-365}"  # Default 1 year
    local min_size="${3:-10MB}"
    
    echo "📦 Finding archive candidates in $path"
    echo "   Criteria: older than $age_days days, larger than $min_size"
    
    # Create temporary file list
    find "$path" -type f -mtime +$age_days -size +$(numfmt --from=iec $min_size)c \
      -printf "%s %p\n" | sort -nr > /tmp/archive_candidates.txt
    
    local total_files=$(wc -l < /tmp/archive_candidates.txt)
    local total_size=$(awk '{sum+=$1} END {print sum}' /tmp/archive_candidates.txt)
    local total_size_gb=$(echo "scale=2; $total_size / 1073741824" | bc -l)
    
    echo "📊 Archive potential:"
    echo "  Files: $total_files"
    echo "  Total size: ${total_size_gb}GB"
    
    echo "🎯 Largest archive candidates:"
    head -20 /tmp/archive_candidates.txt | while read -r size path; do
        local size_mb=$(echo "scale=1; $size / 1048576" | bc -l)
        echo "  ${size_mb}MB - $path"
    done
    
    # Analyze by file type
    echo "📋 Archive candidates by type:"
    cut -d' ' -f2- /tmp/archive_candidates.txt | \
      sed 's/.*\.//' | sort | uniq -c | sort -nr | head -10 | \
      while read -r count ext; do
        echo "  $ext: $count files"
      done
}

# Usage examples
find_archive_candidates "/home" 365 "50MB"
find_archive_candidates "/var/log" 90 "10MB"
```

### Automated Archival System

```bash
#!/bin/bash
# auto_archive.sh - Archive old files to compressed storage

ARCHIVE_DIR="/archive"
ARCHIVE_AGE_DAYS=365
MIN_ARCHIVE_SIZE="100MB"

archive_old_files() {
    local source_path="$1"
    local archive_name="$(basename "$source_path")-$(date +%Y%m%d)"
    
    echo "📦 Archiving old files from $source_path"
    
    # Create archive directory structure
    mkdir -p "$ARCHIVE_DIR/$archive_name"
    
    # Find and move old large files
    find "$source_path" -type f -mtime +$ARCHIVE_AGE_DAYS \
      -size +$(numfmt --from=iec $MIN_ARCHIVE_SIZE)c | while read -r file; do
        
        # Preserve directory structure
        local rel_path=$(realpath --relative-to="$source_path" "$file")
        local archive_file="$ARCHIVE_DIR/$archive_name/$rel_path"
        
        mkdir -p "$(dirname "$archive_file")"
        
        echo "  Archiving: $file"
        mv "$file" "$archive_file"
        
        # Compress in archive
        gzip "$archive_file"
    done
    
    # Create archive manifest
    find "$ARCHIVE_DIR/$archive_name" -type f > "$ARCHIVE_DIR/$archive_name/MANIFEST.txt"
    
    echo "✅ Archive created: $ARCHIVE_DIR/$archive_name"
}

# Usage
archive_old_files "/var/log"
```

## Step 4: Duplicate Detection

### Find Potential Duplicates by Size

```bash
#!/bin/bash
# duplicate_detector.sh - Find potential duplicates

find_size_duplicates() {
    local path="$1"
    local min_size="${2:-1MB}"
    
    echo "🔍 Finding potential duplicates in $path (min size: $min_size)"
    
    # Get file sizes and group by size
    rfstat "$path" --min-size "$min_size" --format csv --quiet | \
      tail -n +2 | cut -d, -f1,2 | sort -t, -k2 -n | \
      awk -F, '{
        if ($2 == prev_size && prev_size != "") {
          if (!printed_header) {
            print "Size: " prev_size " bytes"
            print "  " prev_path
            printed_header = 1
          }
          print "  " $1
        } else {
          if (printed_header) print ""
          printed_header = 0
        }
        prev_size = $2
        prev_path = $1
      }'
}

# Enhanced duplicate detection with checksums
find_exact_duplicates() {
    local path="$1"
    local min_size="${2:-1MB}"
    
    echo "🎯 Finding exact duplicates in $path"
    
    # Find files of same size, then checksum them
    rfstat "$path" --min-size "$min_size" --format csv --quiet | \
      tail -n +2 | while IFS=, read -r filepath size_bytes rest; do
        echo "$size_bytes $filepath"
      done | sort -n | \
      awk '{
        if ($1 == prev_size) {
          if (length(same_size) == 0) {
            same_size = prev_path "\n" $2
          } else {
            same_size = same_size "\n" $2
          }
        } else {
          if (length(same_size) > 0) {
            print same_size
            print "---"
          }
          same_size = ""
        }
        prev_size = $1
        prev_path = $2
      }' | while read -r line; do
        if [[ "$line" == "---" ]]; then
          # Process group of same-size files
          echo "$group" | while read -r file; do
            [[ -n "$file" ]] && md5sum "$file" 2>/dev/null
          done | sort | uniq -w32 -D
          group=""
        else
          group="$group$line\n"
        fi
      done
}

# Usage
find_size_duplicates "/home/user/Downloads"
find_exact_duplicates "/home/user/Documents" "5MB"
```

## Step 5: Storage Optimization Report

### Generate Comprehensive Optimization Report

```bash
#!/bin/bash
# storage_optimization_report.sh

generate_optimization_report() {
    local path="$1"
    local report_file="storage_optimization_$(date +%Y%m%d).md"
    
    echo "📊 Generating storage optimization report for $path"
    
    {
        echo "# Storage Optimization Report"
        echo "**Path:** $path"
        echo "**Generated:** $(date)"
        echo
        
        # Overall statistics
        echo "## Overall Statistics"
        rfstat "$path" --format json --quiet | jq -r '
          "- Total Files: " + (.total_files | tostring) +
          "\n- Total Directories: " + (.total_dirs | tostring) +
          "\n- Total Size: " + (.total_size / 1073741824 | round * 100 / 100 | tostring) + " GB" +
          "\n- Average File Size: " + (.avg_file_size / 1024 | round * 100 / 100 | tostring) + " KB"
        '
        
        echo
        echo "## Size Distribution"
        rfstat "$path" --format json --quiet | jq -r '
          "- Tiny (< 1KB): " + (.size_distribution.tiny | tostring) + " files" +
          "\n- Small (1KB-1MB): " + (.size_distribution.small | tostring) + " files" +
          "\n- Medium (1MB-100MB): " + (.size_distribution.medium | tostring) + " files" +
          "\n- Large (100MB-1GB): " + (.size_distribution.large | tostring) + " files" +
          "\n- Huge (> 1GB): " + (.size_distribution.huge | tostring) + " files"
        '
        
        echo
        echo "## Optimization Opportunities"
        
        # Large files
        echo "### Large Files (>100MB)"
        local large_count=$(rfstat "$path" --min-size 100MB --format json --quiet | jq -r '.total_files')
        if [[ $large_count -gt 0 ]]; then
            echo "Found $large_count large files:"
            rfstat "$path" --min-size 100MB --sort size --limit 10 --format csv --quiet | \
              tail -n +2 | while IFS=, read -r filepath size_bytes size_human rest; do
                echo "- $size_human: $filepath"
              done
        else
            echo "No large files found."
        fi
        
        echo
        echo "### Compression Candidates"
        local text_size=$(rfstat "$path" --extensions "log,txt,csv,json,xml" --format json --quiet | jq -r '.total_size')
        if [[ $text_size -gt 0 ]]; then
            local text_size_gb=$(echo "scale=2; $text_size / 1073741824" | bc -l)
            local estimated_savings=$(echo "scale=2; $text_size_gb * 0.75" | bc -l)
            echo "- Text files: ${text_size_gb}GB"
            echo "- Estimated compression savings: ${estimated_savings}GB"
        fi
        
        echo
        echo "### File Type Analysis"
        rfstat "$path" --format json --quiet | jq -r '
          .file_types | to_entries | sort_by(.value.total_size) | reverse | 
          .[:10] | .[] | "- " + .key + ": " + (.value.count | tostring) + 
          " files, " + (.value.total_size / 1048576 | round * 100 / 100 | tostring) + " MB"
        '
        
        echo
        echo "## Recommendations"
        echo "1. **Archive old files**: Move files older than 1 year to compressed archive"
        echo "2. **Compress text files**: Compress log and text files older than 30 days"
        echo "3. **Review large files**: Manually review files >100MB for necessity"
        echo "4. **Implement cleanup policies**: Set up automated cleanup for temporary files"
        
    } > "$report_file"
    
    echo "✅ Report generated: $report_file"
}

# Usage
generate_optimization_report "/home/user"
generate_optimization_report "/var/log"
```

## Best Practices

### 1. Safety First
- Always test scripts on non-critical data first
- Create backups before making changes
- Use dry-run modes when available
- Verify free space before operations

### 2. Automation Guidelines
- Set up monitoring for storage optimization scripts
- Log all operations for audit trails
- Implement rollback procedures
- Use file locks to prevent concurrent operations

### 3. Performance Considerations
- Run optimization during off-peak hours
- Limit I/O intensive operations
- Monitor system resources during operations
- Use incremental approaches for large datasets

## Related Guides

- [Disk Monitoring](disk-monitoring.md) - Set up continuous monitoring
- [Log Analysis](log-analysis.md) - Analyze log file patterns
- [Automation Integration](automation.md) - Integrate with existing systems

This comprehensive approach to storage optimization will help you maintain efficient storage usage and identify opportunities for space savings.
