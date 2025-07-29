# How to Analyze Log Files with rfstat

This guide shows you how to use rfstat for comprehensive log file analysis, including rotation patterns, growth monitoring, and health assessment.

## Problem

You need to:
- Analyze log file growth patterns and sizes
- Monitor log rotation effectiveness
- Identify problematic or oversized log files
- Generate reports for log management decisions

## Solution Overview

We'll use rfstat to systematically analyze log directories and files through:
1. Log rotation pattern analysis
2. Growth trend monitoring
3. Size distribution analysis
4. Health and cleanup recommendations

## Prerequisites

- rfstat installed and accessible
- Access to log directories (may require sudo)
- Basic understanding of log rotation concepts
- `jq` for JSON processing (optional but recommended)

## Step 1: Basic Log Directory Analysis

### Quick Log Overview

```bash
# Get overall log directory statistics
rfstat /var/log --format summary

# Detailed analysis with file types
rfstat /var/log --extensions "log,gz,1,2,3,4,5" --format table

# Focus on current (unrotated) logs
rfstat /var/log --extensions "log" --sort size --limit 20
```

### Log Size Distribution

```bash
#!/bin/bash
# log_overview.sh - Quick log directory overview

analyze_log_directory() {
    local log_dir="${1:-/var/log}"
    
    echo "📊 Log Directory Analysis: $log_dir"
    echo "=================================="
    
    # Overall statistics
    echo "📈 Overall Statistics:"
    rfstat "$log_dir" --format json --quiet | jq -r '
        "Total Files: " + (.total_files | tostring) +
        "\nTotal Size: " + (.total_size / 1073741824 | round * 100 / 100 | tostring) + " GB" +
        "\nAverage File Size: " + (.avg_file_size / 1024 | round * 100 / 100 | tostring) + " KB"
    '
    
    echo
    echo "🔄 Log Rotation Analysis:"
    
    # Current logs
    local current_logs=$(rfstat "$log_dir" --extensions "log" --format json --quiet | jq -r '.total_files')
    local current_size=$(rfstat "$log_dir" --extensions "log" --format json --quiet | jq -r '.total_size')
    
    # Rotated logs
    local rotated_logs=$(rfstat "$log_dir" --extensions "gz,1,2,3,4,5" --format json --quiet | jq -r '.total_files')
    local rotated_size=$(rfstat "$log_dir" --extensions "gz,1,2,3,4,5" --format json --quiet | jq -r '.total_size')
    
    echo "Current logs: $current_logs files ($(numfmt --to=iec $current_size))"
    echo "Rotated logs: $rotated_logs files ($(numfmt --to=iec $rotated_size))"
    
    # Calculate rotation ratio
    if [[ $current_size -gt 0 ]]; then
        local ratio=$(echo "scale=2; $rotated_size / $current_size" | bc -l)
        echo "Rotation ratio: ${ratio}:1 (rotated:current)"
    fi
}

# Usage
analyze_log_directory "/var/log"
```

## Step 2: Log Rotation Effectiveness

### Analyze Rotation Patterns

```bash
#!/bin/bash
# rotation_analyzer.sh - Analyze log rotation effectiveness

analyze_rotation_effectiveness() {
    local log_dir="${1:-/var/log}"
    
    echo "🔄 Log Rotation Effectiveness Analysis"
    echo "====================================="
    
    # Find applications with logs
    find "$log_dir" -name "*.log" -printf "%f\n" | sed 's/\.log$//' | sort -u | while read app; do
        echo
        echo "📋 Application: $app"
        
        # Count rotated versions
        local log_files=$(find "$log_dir" -name "${app}.log*" | wc -l)
        local current_size=$(stat -c%s "$log_dir/${app}.log" 2>/dev/null || echo 0)
        
        echo "  Log files: $log_files"
        echo "  Current size: $(numfmt --to=iec $current_size)"
        
        # Check for rotated versions
        if find "$log_dir" -name "${app}.log.[0-9]*" -o -name "${app}.log.*.gz" | grep -q .; then
            echo "  ✅ Rotation active"
            
            # Analyze rotation pattern
            local rotated_count=$(find "$log_dir" -name "${app}.log.[0-9]*" -o -name "${app}.log.*.gz" | wc -l)
            echo "  Rotated files: $rotated_count"
            
            # Check compression
            local compressed=$(find "$log_dir" -name "${app}.log.*.gz" | wc -l)
            if [[ $compressed -gt 0 ]]; then
                echo "  ✅ Compression enabled ($compressed compressed files)"
            else
                echo "  ⚠️  No compression detected"
            fi
        else
            echo "  ❌ No rotation detected"
            if [[ $current_size -gt 10485760 ]]; then  # > 10MB
                echo "  ⚠️  Large unrotated log file!"
            fi
        fi
    done
}

analyze_rotation_effectiveness "/var/log"
```

## Step 3: Growth Monitoring

### Track Log Growth Over Time

```bash
#!/bin/bash
# log_growth_monitor.sh - Monitor log file growth

MONITOR_DIR="/var/log"
DATA_DIR="/tmp/log_monitoring"
ALERT_THRESHOLD_MB=100

mkdir -p "$DATA_DIR"

monitor_log_growth() {
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local snapshot_file="$DATA_DIR/snapshot_$timestamp.json"
    
    echo "📊 Capturing log snapshot: $timestamp"
    
    # Capture current state
    rfstat "$MONITOR_DIR" --extensions "log" --format json --quiet > "$snapshot_file"
    
    # Compare with previous snapshot if available
    local prev_snapshot=$(ls -t "$DATA_DIR"/snapshot_*.json 2>/dev/null | sed -n '2p')
    
    if [[ -n "$prev_snapshot" ]]; then
        echo "📈 Growth Analysis (since $(basename "$prev_snapshot" .json | cut -d_ -f2-)):"
        
        # Compare file sizes
        jq -r '.entries[] | "\(.path) \(.size)"' "$snapshot_file" | while read path current_size; do
            local prev_size=$(jq -r --arg path "$path" '.entries[] | select(.path == $path) | .size' "$prev_snapshot" 2>/dev/null || echo 0)
            
            if [[ $prev_size -gt 0 ]]; then
                local growth=$((current_size - prev_size))
                if [[ $growth -gt 0 ]]; then
                    local growth_mb=$(echo "scale=2; $growth / 1048576" | bc -l)
                    echo "  $path: +${growth_mb}MB"
                    
                    # Alert on rapid growth
                    if (( $(echo "$growth_mb > $ALERT_THRESHOLD_MB" | bc -l) )); then
                        echo "  ⚠️  ALERT: Rapid growth detected!"
                    fi
                fi
            fi
        done
    fi
    
    # Cleanup old snapshots (keep last 10)
    ls -t "$DATA_DIR"/snapshot_*.json | tail -n +11 | xargs rm -f 2>/dev/null || true
}

# Run monitoring
monitor_log_growth
```

## Step 4: Problem Detection

### Identify Problematic Logs

```bash
#!/bin/bash
# log_problem_detector.sh - Detect log file problems

detect_log_problems() {
    local log_dir="${1:-/var/log}"
    
    echo "🔍 Log Problem Detection"
    echo "======================="
    
    # Large unrotated logs
    echo "📏 Large Unrotated Logs (>50MB):"
    rfstat "$log_dir" --extensions "log" --min-size 50MB --sort size --format csv --quiet | \
    tail -n +2 | while IFS=, read -r path size_bytes size_human rest; do
        echo "  ⚠️  $size_human - $path"
    done
    
    echo
    echo "💾 Disk Space Hogs (>500MB):"
    rfstat "$log_dir" --min-size 500MB --sort size --format csv --quiet | \
    tail -n +2 | head -10 | while IFS=, read -r path size_bytes size_human rest; do
        echo "  🚨 $size_human - $path"
    done
    
    echo
    echo "📅 Old Uncompressed Rotated Logs:"
    find "$log_dir" -name "*.log.[0-9]*" -not -name "*.gz" -mtime +7 | while read file; do
        local size=$(stat -c%s "$file")
        local size_human=$(numfmt --to=iec "$size")
        echo "  📦 $size_human - $file (candidate for compression)"
    done
    
    echo
    echo "🗑️  Very Old Logs (>90 days):"
    find "$log_dir" -name "*.log*" -mtime +90 | head -10 | while read file; do
        local size=$(stat -c%s "$file")
        local size_human=$(numfmt --to=iec "$size")
        local age=$(stat -c%Y "$file")
        local age_days=$(( ($(date +%s) - age) / 86400 ))
        echo "  🕰️  $size_human - $file (${age_days} days old)"
    done
}

detect_log_problems "/var/log"
```

## Step 5: Comprehensive Log Health Report

### Generate Complete Analysis Report

```bash
#!/bin/bash
# log_health_report.sh - Generate comprehensive log health report

generate_log_health_report() {
    local log_dir="${1:-/var/log}"
    local report_file="log_health_report_$(date +%Y%m%d).md"
    
    echo "📋 Generating comprehensive log health report..."
    
    {
        echo "# Log Health Report"
        echo "**Directory:** $log_dir"
        echo "**Generated:** $(date)"
        echo
        
        # Executive Summary
        echo "## Executive Summary"
        local total_size=$(rfstat "$log_dir" --format json --quiet | jq -r '.total_size')
        local total_files=$(rfstat "$log_dir" --format json --quiet | jq -r '.total_files')
        local total_size_gb=$(echo "scale=2; $total_size / 1073741824" | bc -l)
        
        echo "- **Total log files:** $total_files"
        echo "- **Total size:** ${total_size_gb}GB"
        echo "- **Analysis date:** $(date)"
        
        # Current vs Rotated
        echo
        echo "## Log Rotation Status"
        local current_logs=$(rfstat "$log_dir" --extensions "log" --format json --quiet | jq -r '.total_files')
        local current_size=$(rfstat "$log_dir" --extensions "log" --format json --quiet | jq -r '.total_size')
        local rotated_logs=$(rfstat "$log_dir" --extensions "gz,1,2,3,4,5" --format json --quiet | jq -r '.total_files')
        local rotated_size=$(rfstat "$log_dir" --extensions "gz,1,2,3,4,5" --format json --quiet | jq -r '.total_size')
        
        echo "- **Current logs:** $current_logs files ($(numfmt --to=iec $current_size))"
        echo "- **Rotated logs:** $rotated_logs files ($(numfmt --to=iec $rotated_size))"
        
        # Top consumers
        echo
        echo "## Top Space Consumers"
        rfstat "$log_dir" --sort size --limit 10 --format csv --quiet | \
        tail -n +2 | while IFS=, read -r path size_bytes size_human rest; do
            echo "- $size_human: \`$path\`"
        done
        
        # File type breakdown
        echo
        echo "## File Type Distribution"
        rfstat "$log_dir" --format json --quiet | jq -r '
            .file_types | to_entries | sort_by(.value.total_size) | reverse | 
            .[:10] | .[] | "- **" + .key + "**: " + (.value.count | tostring) + 
            " files, " + (.value.total_size / 1048576 | round * 100 / 100 | tostring) + "MB"
        '
        
        # Recommendations
        echo
        echo "## Recommendations"
        
        # Check for large unrotated logs
        local large_logs=$(rfstat "$log_dir" --extensions "log" --min-size 50MB --format json --quiet | jq -r '.total_files')
        if [[ $large_logs -gt 0 ]]; then
            echo "1. **⚠️ Large unrotated logs detected** ($large_logs files >50MB)"
            echo "   - Review log rotation configuration"
            echo "   - Consider more frequent rotation"
        fi
        
        # Check for old files
        local old_files=$(find "$log_dir" -name "*.log*" -mtime +90 | wc -l)
        if [[ $old_files -gt 0 ]]; then
            echo "2. **🗑️ Old log files found** ($old_files files >90 days)"
            echo "   - Consider archiving or deleting old logs"
            echo "   - Review log retention policies"
        fi
        
        # Check compression
        local uncompressed_rotated=$(find "$log_dir" -name "*.log.[0-9]*" -not -name "*.gz" | wc -l)
        if [[ $uncompressed_rotated -gt 0 ]]; then
            echo "3. **📦 Uncompressed rotated logs** ($uncompressed_rotated files)"
            echo "   - Enable compression in log rotation"
            echo "   - Compress existing rotated logs"
        fi
        
        echo
        echo "## Next Steps"
        echo "1. Review and implement recommendations above"
        echo "2. Set up automated monitoring with this script"
        echo "3. Schedule regular log cleanup and archival"
        echo "4. Monitor log growth trends over time"
        
    } > "$report_file"
    
    echo "✅ Report generated: $report_file"
}

generate_log_health_report "/var/log"
```

## Best Practices

### 1. Regular Monitoring
- Run analysis weekly or monthly
- Set up automated alerts for rapid growth
- Track trends over time
- Monitor rotation effectiveness

### 2. Proactive Management
- Set appropriate rotation policies
- Enable compression for rotated logs
- Implement retention policies
- Archive old logs to cheaper storage

### 3. Performance Considerations
- Use filters to focus on relevant files
- Limit analysis scope for large log directories
- Run during off-peak hours
- Cache results for trend analysis

## Related Guides

- [Disk Monitoring](disk-monitoring.md) - Set up comprehensive disk monitoring
- [Storage Optimization](storage-optimization.md) - Optimize storage usage
- [Automation Integration](automation.md) - Integrate into existing workflows

This comprehensive approach to log analysis will help you maintain healthy log management practices and identify issues before they become problems.
