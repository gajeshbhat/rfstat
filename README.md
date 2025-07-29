# rfstat

📊 **A powerful Rust-based CLI tool for comprehensive file system analysis**

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/version-0.1.0-green.svg)](https://github.com/gajeshbhat/rfstat)

**rfstat** provides comprehensive file system statistics in human-readable formats, designed for DevOps workflows, system administration, and automation tasks.

## ✨ Key Features

- 🚀 **Lightning Fast** - Optimized for large directory structures
- 📊 **Rich Statistics** - File counts, size distributions, type analysis
- 🎨 **Multiple Formats** - Table, JSON, CSV, and summary outputs
- 🔍 **Smart Filtering** - By size, extension, permissions, modification time
- 🎯 **DevOps Ready** - Perfect for automation and monitoring
- 🌈 **Beautiful Output** - Colored terminal display

## 🚀 Quick Start

### Installation
```bash
git clone https://github.com/gajeshbhat/rfstat.git
cd rfstat
cargo install --path .
```

### Basic Usage
```bash
rfstat                              # Analyze current directory
rfstat /var/log                     # Analyze specific directory
rfstat . --format json              # JSON output for automation
rfstat . --sort size --limit 10     # Find largest files
```

## 📚 Documentation

Following the [Diátaxis framework](https://diataxis.fr/), our documentation is organized into four categories:

| Type | Purpose | Location |
|------|---------|----------|
| 🎯 **[Tutorials](docs/tutorials/)** | Learning-oriented guides | Step-by-step learning |
| 🛠️ **[How-to Guides](docs/how-to/)** | Problem-solving recipes | Practical solutions |
| 📖 **[Reference](docs/reference/)** | Information lookup | Complete API/CLI docs |
| 💡 **[Explanation](docs/explanation/)** | Understanding concepts | Architecture & design |

### Quick Navigation
- **New to rfstat?** → Start with [Getting Started Tutorial](docs/tutorials/getting-started.md)
- **Need to solve a problem?** → Check [How-to Guides](docs/how-to/)
- **Looking up syntax?** → See [CLI Reference](docs/reference/cli.md)
- **Want to understand design?** → Read [Architecture](docs/explanation/architecture.md)

## 🎯 Common Use Cases

### System Administration
```bash
# Monitor disk usage growth
rfstat /var/log --format summary

# Find large files for cleanup
rfstat /home --sort size --limit 20 --min-size 100MB

# Analyze log rotation effectiveness
rfstat /var/log --extensions "log,gz" --format csv
```

### DevOps & Automation
```bash
# JSON output for monitoring systems
rfstat /critical/path --format json --quiet > metrics.json

# CSV export for analysis
rfstat /data --format csv --show-times > analysis.csv

# Quick health check
rfstat /var/log --format summary | grep -q "GB" && echo "Large logs detected"
```

### Development Workflows
```bash
# Analyze build artifacts
rfstat ./target --extensions "rlib,bin" --sort size

# Check project file distribution
rfstat . --extensions "rs,toml,md" --format table

# Find test files
rfstat . --extensions "rs" | grep -i test
```

## 🔧 Integration Examples

### Shell Scripts
```bash
#!/bin/bash
# Check if directory exceeds threshold
size=$(rfstat /data --format json --quiet | jq -r '.total_size')
if [ "$size" -gt 1073741824 ]; then  # 1GB
    echo "Directory exceeds 1GB: $(numfmt --to=iec $size)"
fi
```

### Python Integration
```python
import subprocess
import json

# Get directory statistics
result = subprocess.run(['rfstat', '/path', '--format', 'json', '--quiet'],
                       capture_output=True, text=True)
stats = json.loads(result.stdout)
print(f"Total files: {stats['total_files']}")
```

### Monitoring & Alerting
```bash
# Prometheus metrics export
rfstat /var/log --format json --quiet | \
  jq -r '"disk_usage_bytes{path=\"/var/log\"} " + (.total_size | tostring)' \
  > /var/lib/node_exporter/textfile_collector/disk_stats.prom
```

## 🧪 Development & Contributing

### Building from Source
```bash
git clone https://github.com/gajeshbhat/rfstat.git
cd rfstat
cargo build --release
```

### Running Tests
```bash
cargo test                    # Run all tests
cargo test --doc             # Run documentation tests
cargo test integration       # Run integration tests only
```

### Code Quality
```bash
cargo clippy                 # Linting
cargo fmt                    # Formatting
cargo audit                  # Security audit
```

## 📄 License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## 🤝 Support & Community

- 📖 **Documentation**: [docs/](docs/)
- 🐛 **Issues**: [GitHub Issues](https://github.com/gajeshbhat/rfstat/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/gajeshbhat/rfstat/discussions)
- 🔧 **Contributing**: See [CONTRIBUTING.md](CONTRIBUTING.md)

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) for performance and safety
- Uses [clap](https://github.com/clap-rs/clap) for CLI parsing
- Uses [walkdir](https://github.com/BurntSushi/walkdir) for efficient directory traversal
- Uses [tabled](https://github.com/zhiburt/tabled) for beautiful table formatting
- Inspired by traditional Unix tools like `du`, `find`, and `ls`

---

**Made with ❤️ and ☕ for the DevOps community**

## 🧪 Development & Contributing

### Building from Source
```bash
git clone https://github.com/gajeshbhat/rfstat.git
cd rfstat
cargo build --release
```

### Running Tests
```bash
cargo test                    # Run all tests
cargo test --doc             # Run documentation tests
cargo test integration       # Run integration tests only
```

### Code Quality
```bash
cargo clippy                 # Linting
cargo fmt                    # Formatting
cargo audit                  # Security audit
```

## 📄 License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## 🤝 Support & Community

- 📖 **Documentation**: [docs/](docs/)
- 🐛 **Issues**: [GitHub Issues](https://github.com/gajeshbhat/rfstat/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/gajeshbhat/rfstat/discussions)
- 🔧 **Contributing**: See [CONTRIBUTING.md](CONTRIBUTING.md)

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) for performance and safety
- Uses [clap](https://github.com/clap-rs/clap) for CLI parsing
- Uses [walkdir](https://github.com/BurntSushi/walkdir) for efficient directory traversal
- Uses [tabled](https://github.com/zhiburt/tabled) for beautiful table formatting
- Inspired by traditional Unix tools like `du`, `find`, and `ls`

---

**Made with ❤️ and ☕ for the DevOps community**



## 🧪 Development

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/rfstat.git
cd rfstat

# Build in development mode
cargo build

# Build optimized release
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- /path/to/analyze
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test scanner::tests

# Run integration tests
cargo test --test integration
```

### Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass (`cargo test`)
6. Run clippy for linting (`cargo clippy`)
7. Format code (`cargo fmt`)
8. Commit your changes (`git commit -m 'Add amazing feature'`)
9. Push to the branch (`git push origin feature/amazing-feature`)
10. Open a Pull Request

### Code Style

This project follows standard Rust conventions:
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Write comprehensive documentation and tests

---

## 📄 License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## 🤝 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) for performance and safety
- Uses [clap](https://github.com/clap-rs/clap) for CLI parsing
- Uses [walkdir](https://github.com/BurntSushi/walkdir) for efficient directory traversal
- Uses [tabled](https://github.com/zhiburt/tabled) for beautiful table formatting
- Inspired by traditional Unix tools like `du`, `find`, and `ls`

## 📞 Support

- 📖 [Documentation](https://github.com/yourusername/rfstat/wiki)
- 🐛 [Issue Tracker](https://github.com/yourusername/rfstat/issues)
- 💬 [Discussions](https://github.com/yourusername/rfstat/discussions)

---

**Made with ❤️ and ☕ by the rfstat team**
