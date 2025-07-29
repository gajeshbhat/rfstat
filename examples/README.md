# rfstat Examples

This directory contains practical examples demonstrating how to use rfstat in various DevOps and system administration scenarios.

## Examples Overview

### 1. `disk_monitor.sh` - Automated Disk Monitoring
A comprehensive bash script that demonstrates:
- Continuous monitoring of multiple directories
- Alert thresholds and notifications
- Historical data collection and trend analysis
- JSON data processing with `jq`
- Automated cleanup of old monitoring data

**Usage:**
```bash
# Run monitoring with default settings
./disk_monitor.sh

# Run in test mode (no alerts sent)
./disk_monitor.sh --test

# View help and configuration
./disk_monitor.sh --help
```

**Features:**
- Monitors `/var/log`, `/home`, and `/tmp` by default
- Sends email alerts when directories exceed 10GB
- Generates both JSON and human-readable reports
- Maintains 30 days of historical data
- Creates trend analysis reports

### 2. `log_analyzer.py` - Advanced Log Analysis
A Python script that showcases:
- Integration with rfstat for log file analysis
- Log rotation pattern analysis
- Large file detection and reporting
- Data visualization with matplotlib
- Comprehensive reporting

**Usage:**
```bash
# Basic log analysis
python3 log_analyzer.py /var/log

# Generate report file
python3 log_analyzer.py /var/log --report log_report.txt

# Create visualizations
python3 log_analyzer.py /var/log --visualize --output-dir ./charts

# Custom threshold for large files
python3 log_analyzer.py /var/log --threshold 50
```

**Requirements:**
```bash
pip install matplotlib pandas
```

**Features:**
- Analyzes log rotation efficiency
- Identifies large log files
- Creates size distribution charts
- Generates file type breakdowns
- Provides growth pattern analysis

## Integration Patterns

### CI/CD Pipeline Integration

```yaml
# .github/workflows/disk-usage.yml
name: Monitor Disk Usage
on:
  schedule:
    - cron: '0 */6 * * *'  # Every 6 hours

jobs:
  monitor:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Install rfstat
        run: cargo install --path .
      - name: Check disk usage
        run: |
          rfstat /var/log --format json > disk-stats.json
          # Process results and send alerts if needed
```

### Docker Container Monitoring

```dockerfile
FROM rust:1.70 as builder
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y jq
COPY --from=builder /target/release/rfstat /usr/local/bin/
COPY examples/disk_monitor.sh /usr/local/bin/
CMD ["/usr/local/bin/disk_monitor.sh"]
```

### Kubernetes CronJob

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: disk-monitor
spec:
  schedule: "0 */4 * * *"
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: rfstat-monitor
            image: rfstat:latest
            command: ["/usr/local/bin/disk_monitor.sh"]
            volumeMounts:
            - name: host-var-log
              mountPath: /var/log
              readOnly: true
          volumes:
          - name: host-var-log
            hostPath:
              path: /var/log
          restartPolicy: OnFailure
```

## Common Use Cases

### 1. System Health Monitoring
```bash
# Quick system overview
rfstat /var/log --format summary

# Detailed analysis with alerts
rfstat /var/log --min-size 100MB --sort size --limit 10
```

### 2. Storage Optimization
```bash
# Find large files for cleanup
rfstat /home --sort size --limit 20 --min-size 1GB

# Analyze file type distribution
rfstat /data --format json | jq '.file_types'
```

### 3. Log Management
```bash
# Check log rotation effectiveness
rfstat /var/log --extensions "log,gz,1,2,3" --format csv

# Monitor log growth
rfstat /var/log --show-times --sort modified --limit 10
```

### 4. Development Workflow
```bash
# Analyze build artifacts
rfstat ./target --extensions "rlib,rmeta,bin" --format table

# Check project size distribution
rfstat . --format json --no-recursive | jq '.size_distribution'
```

## Best Practices

### 1. Performance Optimization
- Use `--no-recursive` for shallow analysis
- Set `--depth` limits for deep directory structures
- Use `--extensions` to filter relevant files only
- Employ `--limit` to reduce output size

### 2. Automation Integration
- Always use `--quiet` in scripts to suppress logs
- Prefer JSON format for programmatic processing
- Set appropriate timeouts for large directories
- Handle errors gracefully in automation scripts

### 3. Security Considerations
- Run with minimal required permissions
- Avoid scanning sensitive directories unnecessarily
- Use read-only mounts in containers
- Sanitize paths in automated scripts

### 4. Monitoring and Alerting
- Set reasonable thresholds based on historical data
- Implement exponential backoff for repeated alerts
- Log all monitoring activities for audit trails
- Use structured logging for better analysis

## Troubleshooting

### Common Issues

1. **Permission Denied Errors**
   ```bash
   # Run with appropriate permissions or skip inaccessible files
   rfstat /root 2>/dev/null || echo "Some files were inaccessible"
   ```

2. **Large Directory Performance**
   ```bash
   # Limit depth and use filters
   rfstat /usr --depth 3 --extensions "so,bin" --limit 100
   ```

3. **Memory Usage with Large File Sets**
   ```bash
   # Use streaming approach with smaller limits
   rfstat /big-directory --limit 1000 --format csv | head -100
   ```

### Debug Mode
```bash
# Enable verbose logging
RUST_LOG=debug rfstat /path/to/analyze --verbose
```

## Contributing Examples

To contribute new examples:

1. Create a new file in the `examples/` directory
2. Follow the naming convention: `purpose_tool.extension`
3. Include comprehensive comments and documentation
4. Add usage examples and expected output
5. Update this README with your example
6. Test thoroughly in different environments

## Support

For questions about these examples:
- Check the main README.md for general usage
- Review the inline comments in each example
- Open an issue on GitHub for bugs or feature requests
- Contribute improvements via pull requests
