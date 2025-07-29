#!/usr/bin/env python3
"""
log_analyzer.py - Advanced log analysis using rfstat

This script demonstrates how to integrate rfstat with Python for
advanced log file analysis and reporting.
"""

import json
import subprocess
import sys
import argparse
from datetime import datetime, timedelta
from pathlib import Path
from typing import Dict, List, Any
import matplotlib.pyplot as plt
import pandas as pd

class LogAnalyzer:
    """Analyzes log directories using rfstat and generates reports."""
    
    def __init__(self, log_dir: str):
        self.log_dir = Path(log_dir)
        self.rfstat_data = None
        
    def collect_stats(self) -> Dict[str, Any]:
        """Collect statistics using rfstat."""
        try:
            result = subprocess.run([
                'rfstat', str(self.log_dir),
                '--format', 'json',
                '--extensions', 'log,gz,1,2,3,4,5',
                '--show-times'
            ], capture_output=True, text=True, check=True)
            
            self.rfstat_data = json.loads(result.stdout)
            return self.rfstat_data
            
        except subprocess.CalledProcessError as e:
            print(f"Error running rfstat: {e}")
            sys.exit(1)
        except json.JSONDecodeError as e:
            print(f"Error parsing rfstat output: {e}")
            sys.exit(1)
    
    def analyze_log_rotation(self) -> Dict[str, Any]:
        """Analyze log rotation patterns."""
        if not self.rfstat_data:
            self.collect_stats()
        
        rotation_stats = {
            'current_logs': 0,
            'rotated_logs': 0,
            'compressed_logs': 0,
            'total_current_size': 0,
            'total_rotated_size': 0,
            'rotation_efficiency': 0
        }
        
        for entry in self.rfstat_data['entries']:
            if entry['is_dir']:
                continue
                
            file_type = entry.get('file_type', '')
            size = entry['size']
            
            if file_type == 'log':
                rotation_stats['current_logs'] += 1
                rotation_stats['total_current_size'] += size
            elif file_type in ['gz', '1', '2', '3', '4', '5']:
                if file_type == 'gz':
                    rotation_stats['compressed_logs'] += 1
                else:
                    rotation_stats['rotated_logs'] += 1
                rotation_stats['total_rotated_size'] += size
        
        # Calculate rotation efficiency (compression ratio)
        if rotation_stats['total_current_size'] > 0:
            rotation_stats['rotation_efficiency'] = (
                rotation_stats['total_rotated_size'] / 
                rotation_stats['total_current_size']
            )
        
        return rotation_stats
    
    def find_large_logs(self, threshold_mb: int = 100) -> List[Dict[str, Any]]:
        """Find log files larger than threshold."""
        if not self.rfstat_data:
            self.collect_stats()
        
        threshold_bytes = threshold_mb * 1024 * 1024
        large_logs = []
        
        for entry in self.rfstat_data['entries']:
            if (not entry['is_dir'] and 
                entry['size'] > threshold_bytes and
                entry.get('file_type') in ['log', 'gz', '1', '2', '3', '4', '5']):
                
                large_logs.append({
                    'path': entry['path'],
                    'size_mb': entry['size'] / (1024 * 1024),
                    'file_type': entry.get('file_type', 'unknown'),
                    'modified': entry.get('modified', '')
                })
        
        return sorted(large_logs, key=lambda x: x['size_mb'], reverse=True)
    
    def analyze_growth_patterns(self) -> Dict[str, Any]:
        """Analyze log growth patterns over time."""
        # This would require historical data - simplified version
        if not self.rfstat_data:
            self.collect_stats()
        
        patterns = {
            'total_size_gb': self.rfstat_data['total_size'] / (1024**3),
            'file_count': self.rfstat_data['total_files'],
            'avg_file_size_mb': self.rfstat_data['avg_file_size'] / (1024**2),
            'size_distribution': self.rfstat_data['size_distribution']
        }
        
        return patterns
    
    def generate_report(self, output_file: str = None):
        """Generate a comprehensive analysis report."""
        if not self.rfstat_data:
            self.collect_stats()
        
        rotation_stats = self.analyze_log_rotation()
        large_logs = self.find_large_logs()
        growth_patterns = self.analyze_growth_patterns()
        
        report = f"""
# Log Analysis Report
Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}
Directory: {self.log_dir}

## Summary Statistics
- Total Files: {self.rfstat_data['total_files']:,}
- Total Size: {growth_patterns['total_size_gb']:.2f} GB
- Average File Size: {growth_patterns['avg_file_size_mb']:.2f} MB

## Log Rotation Analysis
- Current Log Files: {rotation_stats['current_logs']}
- Rotated Log Files: {rotation_stats['rotated_logs']}
- Compressed Log Files: {rotation_stats['compressed_logs']}
- Current Logs Size: {rotation_stats['total_current_size'] / (1024**2):.2f} MB
- Rotated Logs Size: {rotation_stats['total_rotated_size'] / (1024**2):.2f} MB
- Rotation Efficiency: {rotation_stats['rotation_efficiency']:.2f}

## Size Distribution
- Tiny (< 1KB): {growth_patterns['size_distribution']['tiny']}
- Small (1KB-1MB): {growth_patterns['size_distribution']['small']}
- Medium (1MB-100MB): {growth_patterns['size_distribution']['medium']}
- Large (100MB-1GB): {growth_patterns['size_distribution']['large']}
- Huge (> 1GB): {growth_patterns['size_distribution']['huge']}

## Large Log Files (> 100MB)
"""
        
        for log in large_logs[:10]:  # Top 10 largest
            report += f"- {log['path']}: {log['size_mb']:.2f} MB ({log['file_type']})\n"
        
        if output_file:
            with open(output_file, 'w') as f:
                f.write(report)
            print(f"Report saved to: {output_file}")
        else:
            print(report)
    
    def create_visualization(self, output_dir: str = "."):
        """Create visualizations of log data."""
        if not self.rfstat_data:
            self.collect_stats()
        
        output_path = Path(output_dir)
        output_path.mkdir(exist_ok=True)
        
        # Size distribution pie chart
        dist = self.rfstat_data['size_distribution']
        labels = ['Tiny (<1KB)', 'Small (1KB-1MB)', 'Medium (1MB-100MB)', 
                 'Large (100MB-1GB)', 'Huge (>1GB)']
        sizes = [dist['tiny'], dist['small'], dist['medium'], dist['large'], dist['huge']]
        
        plt.figure(figsize=(10, 8))
        plt.pie(sizes, labels=labels, autopct='%1.1f%%', startangle=90)
        plt.title('Log File Size Distribution')
        plt.savefig(output_path / 'size_distribution.png')
        plt.close()
        
        # File type breakdown
        file_types = self.rfstat_data.get('file_types', {})
        if file_types:
            types = list(file_types.keys())
            counts = [file_types[t]['count'] for t in types]
            
            plt.figure(figsize=(12, 6))
            plt.bar(types, counts)
            plt.title('Log Files by Type')
            plt.xlabel('File Type')
            plt.ylabel('Count')
            plt.xticks(rotation=45)
            plt.tight_layout()
            plt.savefig(output_path / 'file_types.png')
            plt.close()
        
        print(f"Visualizations saved to: {output_path}")

def main():
    parser = argparse.ArgumentParser(description='Analyze log directories using rfstat')
    parser.add_argument('log_dir', help='Path to log directory')
    parser.add_argument('--report', '-r', help='Output file for text report')
    parser.add_argument('--visualize', '-v', action='store_true', 
                       help='Create visualizations')
    parser.add_argument('--output-dir', '-o', default='.', 
                       help='Output directory for visualizations')
    parser.add_argument('--threshold', '-t', type=int, default=100,
                       help='Threshold in MB for large file detection')
    
    args = parser.parse_args()
    
    # Check if rfstat is available
    try:
        subprocess.run(['rfstat', '--version'], capture_output=True, check=True)
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("Error: rfstat command not found. Please install rfstat first.")
        sys.exit(1)
    
    # Create analyzer and run analysis
    analyzer = LogAnalyzer(args.log_dir)
    
    try:
        analyzer.generate_report(args.report)
        
        if args.visualize:
            try:
                analyzer.create_visualization(args.output_dir)
            except ImportError:
                print("Warning: matplotlib not available, skipping visualizations")
                print("Install with: pip install matplotlib pandas")
    
    except KeyboardInterrupt:
        print("\nAnalysis interrupted by user")
        sys.exit(1)
    except Exception as e:
        print(f"Error during analysis: {e}")
        sys.exit(1)

if __name__ == '__main__':
    main()
