# How to Monitor Disk Usage with rfstat

This guide shows you how to set up automated disk monitoring using rfstat for DevOps and system administration tasks.

## Problem

You need to:
- Monitor disk usage growth over time
- Set up automated alerts for disk space issues
- Track storage trends across multiple directories
- Generate reports for capacity planning

## Solution Overview

We'll create a monitoring system that:
1. Regularly scans specified directories
2. Stores historical data in JSON format
3. Sends alerts when thresholds are exceeded
4. Generates trend reports for analysis

## Prerequisites

- rfstat installed and accessible in PATH
- `jq` for JSON processing
- `mail` command for email alerts (optional)
- Cron access for scheduling

## Step 1: Basic Monitoring Script

Create a monitoring script:

```bash
#!/bin/bash
# disk_monitor.sh - Basic disk monitoring with rfstat

MONITOR_PATHS=("/var/log" "/home" "/tmp")
ALERT_THRESHOLD_GB=10
LOG_DIR="/var/log/disk-monitoring"

# Create log directory
mkdir -p "$LOG_DIR"

for path in "${MONITOR_PATHS[@]}"; do
    if [[ -d "$path" ]]; then
        echo "Monitoring: $path"
        
        # Generate timestamp
        timestamp=$(date +%Y-%m-%d_%H-%M-%S)
        
        # Collect statistics
        rfstat "$path" --format json --quiet > "$LOG_DIR/$(basename "$path")-$timestamp.json"
        
        # Extract total size in GB
        size_bytes=$(jq -r '.total_size' "$LOG_DIR/$(basename "$path")-$timestamp.json")
        size_gb=$(echo "scale=2; $size_bytes / 1073741824" | bc -l)
        
        echo "  Size: ${size_gb}GB"
        
        # Check threshold
        if (( $(echo "$size_gb > $ALERT_THRESHOLD_GB" | bc -l) )); then
            echo "  ⚠️  ALERT: Exceeds ${ALERT_THRESHOLD_GB}GB threshold!"
        fi
    fi
done
```

## Step 2: Enhanced Monitoring with Alerts

Enhance the script with proper alerting:

```bash
#!/bin/bash
# enhanced_monitor.sh

set -euo pipefail

# Configuration
MONITOR_PATHS=("/var/log" "/home" "/tmp")
ALERT_THRESHOLD_GB=10
LOG_DIR="/var/log/disk-monitoring"
ALERT_EMAIL="admin@example.com"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Ensure log directory exists
mkdir -p "$LOG_DIR"

send_alert() {
    local path="$1"
    local size_gb="$2"
    local message="ALERT: Directory $path has grown to ${size_gb}GB"
    
    echo -e "${RED}$message${NC}"
    
    # Send email if mail command is available
    if command -v mail >/dev/null 2>&1; then
        echo "$message" | mail -s "Disk Usage Alert" "$ALERT_EMAIL"
    fi
    
    # Log alert
    echo "$(date): $message" >> "$LOG_DIR/alerts.log"
}

analyze_path() {
    local path="$1"
    local timestamp=$(date +%Y-%m-%d_%H-%M-%S)
    local json_file="$LOG_DIR/$(basename "$path")-$timestamp.json"
    
    echo -e "${GREEN}Analyzing: $path${NC}"
    
    # Get statistics
    if rfstat "$path" --format json --quiet > "$json_file" 2>/dev/null; then
        # Extract metrics
        local total_size=$(jq -r '.total_size' "$json_file")
        local total_files=$(jq -r '.total_files' "$json_file")
        local size_gb=$(echo "scale=2; $total_size / 1073741824" | bc -l)
        
        echo "  Files: $total_files | Size: ${size_gb}GB"
        
        # Check for alerts
        if (( $(echo "$size_gb > $ALERT_THRESHOLD_GB" | bc -l) )); then
            send_alert "$path" "$size_gb"
        fi
        
        # Generate human-readable report
        local report_file="$LOG_DIR/$(basename "$path")-$timestamp.txt"
        echo "=== Report for $path - $(date) ===" > "$report_file"
        rfstat "$path" --format table --quiet >> "$report_file" 2>/dev/null
        
    else
        echo -e "${YELLOW}  Warning: Could not analyze $path${NC}"
    fi
}

# Main execution
echo "=== Disk Monitoring Started - $(date) ==="

for path in "${MONITOR_PATHS[@]}"; do
    if [[ -d "$path" ]]; then
        analyze_path "$path"
    else
        echo -e "${YELLOW}Warning: $path does not exist${NC}"
    fi
    echo
done

echo "=== Monitoring Complete ==="
```

## Step 3: Trend Analysis

Add trend analysis capabilities:

```bash
generate_trends() {
    echo "Generating trend analysis..."
    
    local trend_file="$LOG_DIR/trends-$(date +%Y-%m-%d).txt"
    echo "=== Disk Usage Trends - $(date) ===" > "$trend_file"
    
    for path in "${MONITOR_PATHS[@]}"; do
        echo "" >> "$trend_file"
        echo "Path: $path" >> "$trend_file"
        echo "Recent measurements:" >> "$trend_file"
        
        # Find recent JSON files for this path
        find "$LOG_DIR" -name "$(basename "$path")-*.json" -mtime -7 | sort | tail -5 | while read -r json_file; do
            local timestamp=$(basename "$json_file" .json | sed "s/$(basename "$path")-//")
            local size=$(jq -r '.total_size' "$json_file")
            local size_gb=$(echo "scale=2; $size / 1073741824" | bc -l)
            echo "  $timestamp: ${size_gb}GB" >> "$trend_file"
        done
    done
    
    echo "Trend analysis saved: $trend_file"
}

# Add this to your main script
generate_trends
```

## Step 4: Automated Scheduling

Set up automated monitoring with cron:

```bash
# Edit crontab
crontab -e

# Add entries for different monitoring frequencies:

# Every hour - basic monitoring
0 * * * * /path/to/disk_monitor.sh >> /var/log/disk-monitoring/cron.log 2>&1

# Every 6 hours - detailed analysis with trends
0 */6 * * * /path/to/enhanced_monitor.sh >> /var/log/disk-monitoring/detailed.log 2>&1

# Daily - cleanup old files
0 2 * * * find /var/log/disk-monitoring -name "*.json" -mtime +30 -delete
```

## Step 5: Integration with Monitoring Systems

### Prometheus Integration

Export metrics for Prometheus:

```bash
# prometheus_exporter.sh
#!/bin/bash

METRICS_FILE="/var/lib/node_exporter/textfile_collector/disk_stats.prom"

for path in "${MONITOR_PATHS[@]}"; do
    if [[ -d "$path" ]]; then
        # Get current stats
        stats=$(rfstat "$path" --format json --quiet)
        
        # Extract metrics
        total_size=$(echo "$stats" | jq -r '.total_size')
        total_files=$(echo "$stats" | jq -r '.total_files')
        
        # Generate Prometheus metrics
        path_label=$(echo "$path" | sed 's/[^a-zA-Z0-9]/_/g')
        
        cat >> "$METRICS_FILE" << EOF
disk_usage_bytes{path="$path"} $total_size
disk_file_count{path="$path"} $total_files
EOF
    fi
done
```

### Grafana Dashboard Query Examples

```promql
# Disk usage over time
disk_usage_bytes{path="/var/log"}

# File count growth rate
rate(disk_file_count{path="/home"}[1h])

# Alert when usage exceeds threshold
disk_usage_bytes > 10737418240  # 10GB in bytes
```

## Step 6: Advanced Filtering and Analysis

### Monitor Specific File Types

```bash
# Monitor only log files
rfstat /var/log --extensions "log,gz,1,2,3" --format json

# Monitor large files only
rfstat /data --min-size 100MB --format json
```

### Custom Analysis Scripts

```bash
#!/bin/bash
# analyze_growth.sh - Detect rapid growth

analyze_growth() {
    local path="$1"
    local current_size=$(rfstat "$path" --format json --quiet | jq -r '.total_size')
    
    # Compare with size from 24 hours ago
    local yesterday_file=$(find "$LOG_DIR" -name "$(basename "$path")-*" -mtime 1 | head -1)
    
    if [[ -f "$yesterday_file" ]]; then
        local yesterday_size=$(jq -r '.total_size' "$yesterday_file")
        local growth=$((current_size - yesterday_size))
        local growth_percent=$(echo "scale=2; $growth * 100 / $yesterday_size" | bc -l)
        
        echo "Growth in $path: ${growth_percent}% ($(numfmt --to=iec $growth))"
        
        # Alert on rapid growth
        if (( $(echo "$growth_percent > 50" | bc -l) )); then
            echo "⚠️  Rapid growth detected in $path!"
        fi
    fi
}
```

## Troubleshooting

### Common Issues

**Permission Errors**
```bash
# Run with appropriate permissions or use sudo
sudo rfstat /root --format json
```

**Large Directory Performance**
```bash
# Limit depth and use filters for better performance
rfstat /usr --depth 3 --extensions "so,bin" --limit 1000
```

**JSON Processing Errors**
```bash
# Validate JSON output
rfstat /path --format json --quiet | jq empty
```

### Debugging

Enable verbose logging:
```bash
RUST_LOG=debug rfstat /path --verbose
```

## Best Practices

1. **Set Reasonable Thresholds**: Base alerts on historical data
2. **Use Appropriate Intervals**: Don't monitor too frequently
3. **Clean Up Old Data**: Implement log rotation
4. **Test Alerts**: Verify notification systems work
5. **Monitor the Monitor**: Ensure monitoring scripts are running
6. **Document Thresholds**: Keep track of why thresholds were set

## Related Guides

- [Log File Analysis](log-analysis.md)
- [Storage Optimization](storage-optimization.md)
- [Automation Integration](automation.md)

This monitoring setup provides a solid foundation for disk usage tracking and can be extended based on your specific needs.
