# How-to Guides

*Problem-oriented recipes for getting things done*

How-to guides are practical instructions for solving specific real-world problems with rfstat. They assume you have some basic knowledge and focus on achieving specific goals.

## 🛠️ Available Guides

### System Administration

#### [Disk Monitoring](disk-monitoring.md)
**Problem:** Need to monitor disk usage growth and set up alerts  
**Solution:** Automated monitoring scripts with thresholds and notifications

- Set up continuous monitoring
- Configure alert thresholds
- Generate trend reports
- Integration with monitoring systems

#### [Storage Optimization](storage-optimization.md)
**Problem:** Need to identify and optimize storage usage  
**Solution:** Systematic analysis to find optimization opportunities

- Identify large files and storage consumers
- Find compression and archival candidates
- Detect potential duplicates
- Generate optimization reports

### DevOps & Automation

#### [Log Analysis](log-analysis.md)
**Problem:** Need to analyze log file patterns and growth  
**Solution:** Comprehensive log analysis workflows

- Analyze log rotation effectiveness
- Detect log growth patterns
- Identify problematic logs
- Generate log health reports

#### [Automation Integration](automation.md)
**Problem:** Need to integrate rfstat into existing workflows  
**Solution:** Patterns for CI/CD, monitoring, and scripting integration

- CI/CD pipeline integration
- Monitoring system integration
- Custom automation scripts
- Error handling and reliability

### Development Workflows



## 🎯 How to Use These Guides

### Choose the Right Guide
1. **Identify your specific problem** - What exactly are you trying to achieve?
2. **Find the matching guide** - Look for guides that address your specific use case
3. **Check prerequisites** - Ensure you have the required knowledge and tools
4. **Follow the solution** - Implement the provided solution step-by-step

### Guide Structure
Each guide follows this structure:

```
Problem Statement → Solution Overview → Prerequisites → Step-by-step Solution → Best Practices → Related Guides
```

### Adaptation Guidelines
- **Modify for your environment** - Adapt paths, thresholds, and configurations
- **Test in safe environments** - Always test scripts before production use
- **Add error handling** - Enhance scripts with appropriate error handling
- **Document customizations** - Keep track of your modifications

## 🔧 Common Patterns

### Monitoring Pattern
```bash
# 1. Collect data
rfstat /path --format json --quiet > data.json

# 2. Analyze data
threshold_check=$(jq -r '.total_size > 1073741824' data.json)

# 3. Take action
if [[ "$threshold_check" == "true" ]]; then
    send_alert "Directory exceeds 1GB"
fi
```

### Analysis Pattern
```bash
# 1. Gather statistics
rfstat /path --format json --quiet > stats.json

# 2. Extract insights
jq -r '.file_types | to_entries | sort_by(.value.total_size) | reverse' stats.json

# 3. Generate report
generate_report stats.json > analysis_report.md
```

### Automation Pattern
```bash
# 1. Validate inputs
[[ -d "$TARGET_PATH" ]] || exit 1

# 2. Perform analysis
rfstat "$TARGET_PATH" --format json --quiet > /tmp/analysis.json

# 3. Process results
process_results /tmp/analysis.json

# 4. Cleanup
rm -f /tmp/analysis.json
```

## 📋 Best Practices

### Script Development
- **Start simple** - Begin with basic functionality, then add features
- **Test thoroughly** - Test with various directory structures and edge cases
- **Handle errors gracefully** - Provide meaningful error messages
- **Log operations** - Keep audit trails of automated operations

### Production Deployment
- **Use configuration files** - Avoid hardcoded values in scripts
- **Implement monitoring** - Monitor the monitoring scripts themselves
- **Set up alerting** - Alert when automation fails
- **Plan for maintenance** - Regular updates and health checks

### Security Considerations
- **Principle of least privilege** - Run with minimal required permissions
- **Validate inputs** - Sanitize and validate all input parameters
- **Secure credentials** - Use secure methods for storing credentials
- **Audit access** - Log and monitor access to sensitive directories

## 🚀 Quick Problem Solver

**Need to monitor disk usage?** → [Disk Monitoring](disk-monitoring.md)

**Want to optimize storage?** → [Storage Optimization](storage-optimization.md)

**Analyzing log files?** → [Log Analysis](log-analysis.md)

**Integrating with automation?** → [Automation Integration](automation.md)



## 🤝 Contributing New Guides

Have a problem that's not covered? Help us expand this collection:

### Guide Requirements
- **Addresses a specific problem** - Clear problem statement
- **Provides working solution** - Tested, practical solution
- **Includes examples** - Real-world examples and use cases
- **Follows our format** - Consistent structure and style

### Contribution Process
1. **Identify the problem** - Ensure it's not already covered
2. **Develop the solution** - Create and test your approach
3. **Write the guide** - Follow our how-to guide template
4. **Submit for review** - Open a pull request with your guide

## 📞 Getting Help

### Guide-Specific Issues
- **Solution doesn't work?** Check prerequisites and environment differences
- **Need modifications?** Adapt the solution to your specific needs
- **Found errors?** Report issues with specific guides

### General Support
- **New to rfstat?** Start with [Tutorials](../tutorials/)
- **Need reference info?** Check [Reference](../reference/) documentation
- **Want to understand concepts?** Read [Explanation](../explanation/) articles

---

*Remember: How-to guides are goal-oriented. If you're learning rfstat basics, start with [Tutorials](../tutorials/). If you need to look up specific information, use the [Reference](../reference/) documentation.*
