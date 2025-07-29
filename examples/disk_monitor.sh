#!/bin/bash
# disk_monitor.sh - Monitor disk usage with rfstat
#
# This script demonstrates how to use rfstat for continuous disk monitoring
# and alerting in DevOps environments.

set -euo pipefail

# Configuration
MONITOR_PATHS=("/var/log" "/home" "/tmp")
ALERT_THRESHOLD_GB=10
LOG_DIR="/var/log/disk-monitoring"
ALERT_EMAIL="admin@example.com"

# Colors for output
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Ensure log directory exists
mkdir -p "$LOG_DIR"

# Function to convert bytes to GB
bytes_to_gb() {
    echo "scale=2; $1 / 1073741824" | bc -l
}

# Function to send alert
send_alert() {
    local path="$1"
    local size_gb="$2"
    local message="ALERT: Directory $path has grown to ${size_gb}GB, exceeding threshold of ${ALERT_THRESHOLD_GB}GB"
    
    echo -e "${RED}$message${NC}"
    
    # Send email alert (requires mail command)
    if command -v mail >/dev/null 2>&1; then
        echo "$message" | mail -s "Disk Usage Alert" "$ALERT_EMAIL"
    fi
    
    # Log alert
    echo "$(date): $message" >> "$LOG_DIR/alerts.log"
}

# Function to analyze a single path
analyze_path() {
    local path="$1"
    local timestamp=$(date +%Y-%m-%d_%H-%M-%S)
    local json_file="$LOG_DIR/$(basename "$path")-$timestamp.json"
    
    echo -e "${GREEN}Analyzing: $path${NC}"
    
    # Get statistics in JSON format
    if ! rfstat "$path" --format json --quiet > "$json_file" 2>/dev/null; then
        echo -e "${RED}Error: Failed to analyze $path${NC}"
        return 1
    fi
    
    # Extract key metrics
    local total_size=$(jq -r '.total_size' "$json_file")
    local total_files=$(jq -r '.total_files' "$json_file")
    local total_dirs=$(jq -r '.total_dirs' "$json_file")
    local size_gb=$(bytes_to_gb "$total_size")
    
    # Display summary
    echo "  Files: $total_files | Directories: $total_dirs | Size: ${size_gb}GB"
    
    # Check for alerts
    if (( $(echo "$size_gb > $ALERT_THRESHOLD_GB" | bc -l) )); then
        send_alert "$path" "$size_gb"
    fi
    
    # Generate human-readable report
    local report_file="$LOG_DIR/$(basename "$path")-$timestamp.txt"
    echo "=== Disk Usage Report for $path - $(date) ===" > "$report_file"
    rfstat "$path" --format table --quiet >> "$report_file" 2>/dev/null || true
    
    echo "  Report saved: $report_file"
    echo "  JSON data: $json_file"
}

# Function to generate trend analysis
generate_trends() {
    echo -e "${YELLOW}Generating trend analysis...${NC}"
    
    local trend_file="$LOG_DIR/trends-$(date +%Y-%m-%d).txt"
    echo "=== Disk Usage Trends - $(date) ===" > "$trend_file"
    
    for path in "${MONITOR_PATHS[@]}"; do
        echo "" >> "$trend_file"
        echo "Path: $path" >> "$trend_file"
        echo "Recent JSON files:" >> "$trend_file"
        
        # Find recent JSON files for this path
        find "$LOG_DIR" -name "$(basename "$path")-*.json" -mtime -7 | sort | tail -5 | while read -r json_file; do
            local timestamp=$(basename "$json_file" .json | sed "s/$(basename "$path")-//")
            local size=$(jq -r '.total_size' "$json_file")
            local size_gb=$(bytes_to_gb "$size")
            echo "  $timestamp: ${size_gb}GB" >> "$trend_file"
        done
    done
    
    echo "  Trend analysis saved: $trend_file"
}

# Function to cleanup old files
cleanup_old_files() {
    echo -e "${YELLOW}Cleaning up old monitoring files...${NC}"
    
    # Remove JSON files older than 30 days
    find "$LOG_DIR" -name "*.json" -mtime +30 -delete
    
    # Remove text reports older than 7 days
    find "$LOG_DIR" -name "*.txt" -mtime +7 -delete
    
    echo "  Cleanup completed"
}

# Main execution
main() {
    echo "=== Disk Monitoring Started - $(date) ==="
    
    # Check if rfstat is available
    if ! command -v rfstat >/dev/null 2>&1; then
        echo -e "${RED}Error: rfstat command not found${NC}"
        exit 1
    fi
    
    # Check if jq is available
    if ! command -v jq >/dev/null 2>&1; then
        echo -e "${RED}Error: jq command not found (required for JSON processing)${NC}"
        exit 1
    fi
    
    # Analyze each monitored path
    for path in "${MONITOR_PATHS[@]}"; do
        if [[ -d "$path" ]]; then
            analyze_path "$path"
        else
            echo -e "${YELLOW}Warning: Path $path does not exist or is not a directory${NC}"
        fi
        echo ""
    done
    
    # Generate trends and cleanup
    generate_trends
    cleanup_old_files
    
    echo "=== Disk Monitoring Completed - $(date) ==="
}

# Handle command line arguments
case "${1:-}" in
    --help|-h)
        echo "Usage: $0 [--help|--test]"
        echo ""
        echo "Options:"
        echo "  --help    Show this help message"
        echo "  --test    Run in test mode (no alerts sent)"
        echo ""
        echo "Configuration:"
        echo "  MONITOR_PATHS: ${MONITOR_PATHS[*]}"
        echo "  ALERT_THRESHOLD_GB: ${ALERT_THRESHOLD_GB}GB"
        echo "  LOG_DIR: $LOG_DIR"
        echo "  ALERT_EMAIL: $ALERT_EMAIL"
        exit 0
        ;;
    --test)
        echo "Running in test mode - no alerts will be sent"
        ALERT_EMAIL=""
        ;;
esac

# Run main function
main
