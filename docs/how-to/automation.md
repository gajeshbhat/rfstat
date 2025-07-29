# How to Integrate rfstat with Automation Systems

This guide shows you how to integrate rfstat into various automation workflows, CI/CD pipelines, and monitoring systems.

## Problem

You need to:
- Integrate rfstat into CI/CD pipelines
- Set up automated monitoring and alerting
- Use rfstat in scripts and automation workflows
- Export data to monitoring systems

## Solution Overview

We'll cover integration patterns for:
1. CI/CD pipeline integration
2. Monitoring system integration
3. Custom automation scripts
4. Error handling and reliability

## Prerequisites

- rfstat installed and accessible
- Basic understanding of your automation platform
- `jq` for JSON processing
- Access to your monitoring/CI systems

## Step 1: CI/CD Pipeline Integration

### GitHub Actions Integration

```yaml
# .github/workflows/storage-check.yml
name: Storage Analysis

on:
  schedule:
    - cron: '0 2 * * 1'  # Weekly on Monday at 2 AM
  workflow_dispatch:

jobs:
  analyze-storage:
    runs-on: ubuntu-latest
    steps:
    - name: Checkout code
      uses: actions/checkout@v4

    - name: Install rfstat
      run: |
        curl -L https://github.com/gajeshbhat/rfstat/releases/latest/download/rfstat-linux-x86_64 -o rfstat
        chmod +x rfstat
        sudo mv rfstat /usr/local/bin/

    - name: Analyze repository storage
      run: |
        echo "## Repository Storage Analysis" >> $GITHUB_STEP_SUMMARY
        rfstat . --format json > storage_stats.json
        
        # Extract key metrics
        TOTAL_SIZE=$(jq -r '.total_size' storage_stats.json)
        TOTAL_FILES=$(jq -r '.total_files' storage_stats.json)
        
        echo "- **Total Files:** $TOTAL_FILES" >> $GITHUB_STEP_SUMMARY
        echo "- **Total Size:** $(numfmt --to=iec $TOTAL_SIZE)" >> $GITHUB_STEP_SUMMARY
        
        # Check for large files
        echo "### Large Files (>10MB)" >> $GITHUB_STEP_SUMMARY
        rfstat . --min-size 10MB --format csv | tail -n +2 | while IFS=, read -r path size_bytes size_human rest; do
          echo "- $size_human: \`$path\`" >> $GITHUB_STEP_SUMMARY
        done

    - name: Upload storage report
      uses: actions/upload-artifact@v3
      with:
        name: storage-analysis
        path: storage_stats.json
```

### GitLab CI Integration

```yaml
# .gitlab-ci.yml
storage_analysis:
  stage: test
  image: ubuntu:latest
  before_script:
    - apt-get update && apt-get install -y curl jq bc
    - curl -L https://github.com/gajeshbhat/rfstat/releases/latest/download/rfstat-linux-x86_64 -o rfstat
    - chmod +x rfstat && mv rfstat /usr/local/bin/
  script:
    - echo "Analyzing repository storage..."
    - rfstat . --format json > storage_stats.json
    - |
      # Generate report
      echo "## Storage Analysis Report" > storage_report.md
      echo "**Date:** $(date)" >> storage_report.md
      
      TOTAL_SIZE=$(jq -r '.total_size' storage_stats.json)
      TOTAL_FILES=$(jq -r '.total_files' storage_stats.json)
      
      echo "- Total Files: $TOTAL_FILES" >> storage_report.md
      echo "- Total Size: $(numfmt --to=iec $TOTAL_SIZE)" >> storage_report.md
      
      # Check thresholds
      SIZE_MB=$(echo "scale=0; $TOTAL_SIZE / 1048576" | bc)
      if [ $SIZE_MB -gt 1000 ]; then
        echo "⚠️ Repository size exceeds 1GB threshold" >> storage_report.md
        exit 1
      fi
  artifacts:
    reports:
      junit: storage_report.md
    paths:
      - storage_stats.json
  only:
    - schedules
    - web
```

## Step 2: Monitoring System Integration

### Prometheus Integration

```bash
#!/bin/bash
# prometheus_rfstat_exporter.sh - Export rfstat metrics to Prometheus

METRICS_FILE="/var/lib/node_exporter/textfile_collector/rfstat.prom"
TEMP_FILE=$(mktemp)
MONITORED_PATHS=("/var/log" "/tmp" "/home" "/opt")

generate_prometheus_metrics() {
    {
        echo "# HELP rfstat_directory_size_bytes Directory size in bytes"
        echo "# TYPE rfstat_directory_size_bytes gauge"
        echo "# HELP rfstat_directory_files_total Total files in directory"
        echo "# TYPE rfstat_directory_files_total gauge"
        echo "# HELP rfstat_directory_large_files_total Files larger than 100MB"
        echo "# TYPE rfstat_directory_large_files_total gauge"
        
        for path in "${MONITORED_PATHS[@]}"; do
            if [[ -d "$path" ]]; then
                echo "Processing $path..." >&2
                
                # Get basic stats
                stats=$(rfstat "$path" --format json --quiet 2>/dev/null)
                if [[ $? -eq 0 ]]; then
                    size=$(echo "$stats" | jq -r '.total_size // 0')
                    files=$(echo "$stats" | jq -r '.total_files // 0')
                    
                    # Get large files count
                    large_files=$(rfstat "$path" --min-size 100MB --format json --quiet 2>/dev/null | jq -r '.total_files // 0')
                    
                    # Sanitize path for Prometheus label
                    label=$(echo "$path" | sed 's/[^a-zA-Z0-9]/_/g' | sed 's/^_//' | sed 's/_$//')
                    
                    echo "rfstat_directory_size_bytes{path=\"$path\",label=\"$label\"} $size"
                    echo "rfstat_directory_files_total{path=\"$path\",label=\"$label\"} $files"
                    echo "rfstat_directory_large_files_total{path=\"$path\",label=\"$label\"} $large_files"
                fi
            fi
        done
        
        # Add timestamp
        echo "# HELP rfstat_last_update_timestamp Last update timestamp"
        echo "# TYPE rfstat_last_update_timestamp gauge"
        echo "rfstat_last_update_timestamp $(date +%s)"
        
    } > "$TEMP_FILE"
    
    # Atomic update
    mv "$TEMP_FILE" "$METRICS_FILE"
    echo "Metrics updated: $METRICS_FILE"
}

# Run with error handling
if ! generate_prometheus_metrics; then
    echo "Error generating metrics" >&2
    exit 1
fi
```

### InfluxDB Integration

```bash
#!/bin/bash
# influxdb_rfstat_integration.sh - Send rfstat data to InfluxDB

INFLUX_URL="http://localhost:8086"
INFLUX_DB="system_metrics"
INFLUX_USER="rfstat"
INFLUX_PASS="your_password"

send_to_influxdb() {
    local path="$1"
    local measurement="disk_usage"
    
    if [[ ! -d "$path" ]]; then
        echo "Path $path does not exist" >&2
        return 1
    fi
    
    echo "Analyzing $path..."
    
    # Get statistics
    local stats=$(rfstat "$path" --format json --quiet 2>/dev/null)
    if [[ $? -ne 0 ]]; then
        echo "Failed to analyze $path" >&2
        return 1
    fi
    
    # Extract metrics
    local total_size=$(echo "$stats" | jq -r '.total_size')
    local total_files=$(echo "$stats" | jq -r '.total_files')
    local total_dirs=$(echo "$stats" | jq -r '.total_dirs')
    local avg_size=$(echo "$stats" | jq -r '.avg_file_size')
    
    # Create InfluxDB line protocol
    local timestamp=$(date +%s)000000000  # nanoseconds
    local path_tag=$(echo "$path" | sed 's/[^a-zA-Z0-9]/_/g')
    
    local line_protocol="${measurement},path=${path_tag},host=$(hostname) total_size=${total_size}i,total_files=${total_files}i,total_dirs=${total_dirs}i,avg_file_size=${avg_size}i ${timestamp}"
    
    # Send to InfluxDB
    curl -i -XPOST "${INFLUX_URL}/write?db=${INFLUX_DB}" \
        -u "${INFLUX_USER}:${INFLUX_PASS}" \
        --data-binary "$line_protocol"
    
    echo "Sent metrics for $path to InfluxDB"
}

# Monitor multiple paths
PATHS=("/var/log" "/tmp" "/home" "/opt")
for path in "${PATHS[@]}"; do
    send_to_influxdb "$path"
done
```

## Step 3: Custom Automation Scripts

### Automated Cleanup Script

```bash
#!/bin/bash
# automated_cleanup.sh - Automated cleanup based on rfstat analysis

set -euo pipefail

# Configuration
CLEANUP_PATHS=("/tmp" "/var/log" "/var/cache")
DRY_RUN=${DRY_RUN:-true}
LOG_FILE="/var/log/automated_cleanup.log"

log_message() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1" | tee -a "$LOG_FILE"
}

cleanup_directory() {
    local path="$1"
    local max_size_gb="$2"
    local max_age_days="$3"
    
    log_message "Analyzing $path for cleanup..."
    
    # Check current size
    local current_size=$(rfstat "$path" --format json --quiet | jq -r '.total_size')
    local current_size_gb=$(echo "scale=2; $current_size / 1073741824" | bc -l)
    
    log_message "Current size: ${current_size_gb}GB (threshold: ${max_size_gb}GB)"
    
    # Check if cleanup is needed
    if (( $(echo "$current_size_gb > $max_size_gb" | bc -l) )); then
        log_message "⚠️ Size threshold exceeded, starting cleanup..."
        
        # Find old large files
        rfstat "$path" --min-size 10MB --format csv --quiet | \
        tail -n +2 | while IFS=, read -r filepath size_bytes size_human rest; do
            # Check file age
            local file_age_days=$(( ($(date +%s) - $(stat -c%Y "$filepath")) / 86400 ))
            
            if [[ $file_age_days -gt $max_age_days ]]; then
                log_message "🗑️ Would delete: $filepath ($size_human, ${file_age_days} days old)"
                
                if [[ "$DRY_RUN" != "true" ]]; then
                    rm -f "$filepath"
                    log_message "✅ Deleted: $filepath"
                fi
            fi
        done
        
        # Recheck size after cleanup
        local new_size=$(rfstat "$path" --format json --quiet | jq -r '.total_size')
        local new_size_gb=$(echo "scale=2; $new_size / 1073741824" | bc -l)
        local saved_gb=$(echo "scale=2; $current_size_gb - $new_size_gb" | bc -l)
        
        log_message "✅ Cleanup complete. Saved: ${saved_gb}GB"
    else
        log_message "✅ No cleanup needed"
    fi
}

# Main execution
log_message "Starting automated cleanup (DRY_RUN=$DRY_RUN)"

cleanup_directory "/tmp" 5 7        # 5GB max, 7 days old
cleanup_directory "/var/log" 10 30  # 10GB max, 30 days old
cleanup_directory "/var/cache" 2 14 # 2GB max, 14 days old

log_message "Automated cleanup completed"
```

### Health Check Script

```bash
#!/bin/bash
# health_check.sh - System health check using rfstat

ALERT_EMAIL="admin@example.com"
ALERT_WEBHOOK="https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK"

check_system_health() {
    local issues=()
    
    # Check critical directories
    local critical_dirs=("/" "/var" "/home" "/tmp")
    
    for dir in "${critical_dirs[@]}"; do
        if [[ -d "$dir" ]]; then
            local stats=$(rfstat "$dir" --format json --quiet 2>/dev/null)
            if [[ $? -eq 0 ]]; then
                local size_gb=$(echo "$stats" | jq -r '.total_size / 1073741824')
                local files=$(echo "$stats" | jq -r '.total_files')
                
                # Define thresholds per directory
                case "$dir" in
                    "/")
                        if (( $(echo "$size_gb > 80" | bc -l) )); then
                            issues+=("Root filesystem: ${size_gb}GB (>80GB threshold)")
                        fi
                        ;;
                    "/var")
                        if (( $(echo "$size_gb > 20" | bc -l) )); then
                            issues+=("/var directory: ${size_gb}GB (>20GB threshold)")
                        fi
                        ;;
                    "/tmp")
                        if (( $(echo "$files > 10000" | bc -l) )); then
                            issues+=("/tmp directory: $files files (>10k threshold)")
                        fi
                        ;;
                esac
            fi
        fi
    done
    
    # Report issues
    if [[ ${#issues[@]} -gt 0 ]]; then
        local message="🚨 System Health Alert\n\nIssues detected:\n"
        for issue in "${issues[@]}"; do
            message="$message- $issue\n"
        done
        
        # Send alert
        echo -e "$message" | mail -s "System Health Alert" "$ALERT_EMAIL"
        
        # Send to Slack
        curl -X POST -H 'Content-type: application/json' \
            --data "{\"text\":\"$message\"}" \
            "$ALERT_WEBHOOK"
        
        return 1
    else
        echo "✅ All systems healthy"
        return 0
    fi
}

# Run health check
check_system_health
```

## Step 4: Error Handling and Reliability

### Robust Wrapper Script

```bash
#!/bin/bash
# rfstat_wrapper.sh - Robust wrapper for rfstat with error handling

set -euo pipefail

# Configuration
MAX_RETRIES=3
RETRY_DELAY=5
TIMEOUT=300
LOG_LEVEL=${LOG_LEVEL:-INFO}

log() {
    local level="$1"
    shift
    echo "$(date '+%Y-%m-%d %H:%M:%S') [$level] $*" >&2
}

run_rfstat_with_retry() {
    local path="$1"
    shift
    local args=("$@")
    
    local attempt=1
    while [[ $attempt -le $MAX_RETRIES ]]; do
        log "INFO" "Attempt $attempt/$MAX_RETRIES: rfstat $path ${args[*]}"
        
        if timeout "$TIMEOUT" rfstat "$path" "${args[@]}"; then
            log "INFO" "Success on attempt $attempt"
            return 0
        else
            local exit_code=$?
            log "WARN" "Attempt $attempt failed with exit code $exit_code"
            
            if [[ $attempt -lt $MAX_RETRIES ]]; then
                log "INFO" "Retrying in $RETRY_DELAY seconds..."
                sleep "$RETRY_DELAY"
            fi
        fi
        
        ((attempt++))
    done
    
    log "ERROR" "All attempts failed for: rfstat $path ${args[*]}"
    return 1
}

validate_path() {
    local path="$1"
    
    if [[ ! -e "$path" ]]; then
        log "ERROR" "Path does not exist: $path"
        return 1
    fi
    
    if [[ ! -r "$path" ]]; then
        log "ERROR" "Path is not readable: $path"
        return 1
    fi
    
    return 0
}

# Main function
main() {
    local path="${1:-.}"
    shift || true
    
    # Validate input
    if ! validate_path "$path"; then
        exit 1
    fi
    
    # Run with retry logic
    if ! run_rfstat_with_retry "$path" "$@"; then
        log "ERROR" "Failed to analyze $path after $MAX_RETRIES attempts"
        exit 1
    fi
}

# Execute if run directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
```

## Best Practices

### 1. Error Handling
- Always check exit codes
- Implement retry logic for transient failures
- Use timeouts for long-running operations
- Log all operations for debugging

### 2. Performance
- Use appropriate filters to reduce processing time
- Cache results when possible
- Run during off-peak hours
- Monitor resource usage

### 3. Security
- Validate all inputs
- Use least privilege principles
- Secure credentials and API keys
- Audit access to sensitive directories

### 4. Monitoring
- Monitor the monitoring scripts themselves
- Set up alerts for script failures
- Track execution times and resource usage
- Maintain audit logs

## Related Guides

- [Disk Monitoring](disk-monitoring.md) - Set up disk usage monitoring
- [Storage Optimization](storage-optimization.md) - Optimize storage usage
- [Log Analysis](log-analysis.md) - Analyze log file patterns

This comprehensive automation integration guide will help you incorporate rfstat into your existing workflows and monitoring systems effectively.
