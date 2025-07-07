# RuNeVis

[![Build Status](https://github.com/greensh16/RuNeVis/actions/workflows/rust.yml/badge.svg)](https://github.com/greensh16/RuNeVis/actions/workflows/rust.yml)
[![Code Quality](https://github.com/greensh16/RuNeVis/actions/workflows/code-quality.yml/badge.svg)](https://github.com/greensh16/RuNeVis/actions/workflows/code-quality.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.77.0%2B-orange.svg)](https://rustup.rs/)
[![Version](https://img.shields.io/badge/version-1.1.0-green.svg)](Cargo.toml)

> **High-Performance NetCDF Analysis Engine**

RuNeVis is a cutting-edge Rust-based platform engineered for high-performance parallel processing of NetCDF (Network Common Data Form) files. Built from the ground up with modern computational paradigms, RuNeVis delivers exceptional performance through intelligent multi-core parallelization powered by Rayon, achieving up to **7x speedup** on large datasets.

## Table of Contents

- [Features](#features)
- [Performance](#performance)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage](#usage)
- [Architecture](#architecture)
- [Examples & Use Cases](#examples--use-cases)
- [CLI Reference](#cli-reference)
- [Dependencies](#dependencies)
- [Development](#development)
- [Contributing](#contributing)
- [License](#license)

## Features

### 🚀 **Parallel Processing**
- **Multi-core computation**: Leverages Rayon for parallel mean calculations across multiple CPU cores
- **Thread control**: Configure the number of threads with `--threads` option
- **Efficient memory usage**: Optimized for large datasets with smart memory management
- **Up to 7x speedup**: Scales with dataset size and CPU cores

### 📊 **Data Analysis**
- **Statistical operations**: Calculate mean, sum, min, max over any dimension
- **Metadata inspection**: View global attributes, variables, and dimensions
- **Data slicing**: Extract specific regions or time periods
- **Data export**: Save results to new NetCDF files with preserved metadata

### 🛠️ **Performance Features**
- **Robust error handling**: NaN and infinite value detection
- **Progress indicators**: Visual feedback for long-running operations
- **Memory efficient**: Streaming operations for large datasets
- **Self-contained**: No external NetCDF library dependencies

### 🏗️ **Modern Architecture**
- **Modular design**: Clean separation of concerns with dedicated modules
- **Type safety**: Strong typing and structured error handling
- **Extensible**: Trait-based design for easy extension
- **Well-tested**: Comprehensive unit and integration tests

## Performance

### Benchmark Results

Performance results on a 10-core machine using built-in benchmark:

| Dataset Size | Speedup | Processing Time |
|-------------|---------|----------------|
| 1M points  | 2.74×   | ~0.5s → ~0.2s  |
| 5M points  | 6.91×   | ~2.5s → ~0.4s  |
| 10M points | 7.96×   | ~5.0s → ~0.6s  |

### Performance Characteristics

- **Scales with dataset size**: Larger datasets achieve better speedup
- **Multi-core efficiency**: Near-linear scaling up to available CPU cores
- **Memory efficient**: Constant memory usage regardless of dataset size
- **I/O optimized**: Efficient NetCDF file handling

## Installation

### Prerequisites

- **Rust**: Version 1.77.0 or higher
  - Install via [rustup](https://rustup.rs/): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
  - Verify installation: `rustc --version`

### Installing from Source

1. **Clone the repository**:
   ```bash
   git clone https://github.com/greensh16/RuNeVis.git
   cd RuNeVis
   ```

2. **Build the project**:
   ```bash
   cargo build --release
   ```

3. **Run the application**:
   ```bash
   ./target/release/RuNeVis --help
   ```

### NetCDF Dependencies

✅ **No system NetCDF libraries required!** This project uses the `static` feature of the netcdf-rust crate, which means:
- NetCDF C library is statically linked
- No need to install system NetCDF packages
- Completely self-contained binary
- Works across different systems without external dependencies

This eliminates the common installation issues with NetCDF system libraries and ensures consistent behavior across platforms.

## Quick Start

### Generate Test Data

```bash
# Create a small test NetCDF file
cargo run --example create_test_netcdf

# Create a large test NetCDF file for performance testing
cargo run --example create_large_test_netcdf
```

### Basic Operations

```bash
# View file metadata
runevis -f test_data.nc

# List all variables and dimensions
runevis -f test_data.nc --list-vars

# Compute mean over time dimension (using all CPU cores)
runevis -f test_data.nc --mean temperature:time

# Control thread count and save to file
runevis -f test_data.nc --mean temperature:time --threads 4 --output-netcdf result.nc
```

### Performance Testing

```bash
# Run built-in benchmark
cargo run --example benchmark --release

# Compare single vs multi-threaded performance
time runevis -f large_test_data.nc --mean temperature:time --threads 1
time runevis -f large_test_data.nc --mean temperature:time --threads 8
```

## Usage

### Command Line Interface

```bash
runevis [OPTIONS] --file <FILE>
```

### Statistical Operations

```bash
# Mean calculation
runevis -f data.nc --mean temperature:time

# Min/Max extraction
runevis -f data.nc --min temperature:time
runevis -f data.nc --max temperature:time

# Sum calculation
runevis -f data.nc --sum precipitation:time
```

### Data Inspection

```bash
# File metadata
runevis -f data.nc

# Variable details
runevis -f data.nc --describe temperature

# Quick statistics
runevis -f data.nc --summary temperature
```

### Data Export

```bash
# Save results to NetCDF
runevis -f data.nc --mean temperature:time --output-netcdf result.nc

# With verbose output
runevis -f data.nc --mean temperature:time --output-netcdf result.nc -v
```

## Architecture

### Modular Design

RuNeVis features a clean, modular architecture that promotes maintainability and extensibility:

```
RuNeVis/
├── src/
│   ├── main.rs          # Application entry point
│   ├── lib.rs           # Library root with public API
│   ├── cli.rs           # Command-line interface
│   ├── errors.rs        # Centralized error handling
│   ├── metadata.rs      # NetCDF file inspection
│   ├── netcdf_io.rs     # File I/O operations
│   ├── parallel.rs      # Parallel processing configuration
│   ├── statistics.rs    # Statistical computations
│   └── utils.rs         # Shared utilities
├── tests/               # Integration tests
├── examples/            # Usage examples
└── Cargo.toml          # Project configuration
```

### Key Modules

| Module | Purpose | Key Features |
|--------|---------|-------------|
| `statistics` | Statistical computations | Parallel mean/sum/min/max, extensible traits |
| `metadata` | File inspection | Variable analysis, dimension info |
| `netcdf_io` | File I/O operations | Data slicing, result export |
| `parallel` | Parallel processing | Thread pool management, performance tuning |
| `errors` | Error handling | Structured error types, proper error chaining |

## Examples & Use Cases

### Climate Data Analysis

#### Temperature Climatology

```bash
# Calculate seasonal temperature averages
runevis -f climate_data.nc --mean temperature:time --threads 8 --output-netcdf climatology.nc

# Find temperature extremes
runevis -f climate_data.nc --min temperature:time --output-netcdf temp_min.nc
runevis -f climate_data.nc --max temperature:time --output-netcdf temp_max.nc
```

#### Spatial Analysis

```bash
# Zonal mean (average over longitude)
runevis -f data.nc --mean temperature:longitude --output-netcdf zonal_mean.nc

# Meridional mean (average over latitude)
runevis -f data.nc --mean temperature:latitude --output-netcdf meridional_mean.nc
```

#### Data Quality Control

```bash
# Inspect dataset structure
runevis -f data.nc --list-vars -v

# Check variable properties
runevis -f data.nc --describe temperature

# Generate quality control statistics
runevis -f data.nc --summary temperature
```

### Working with Real Data

RuNeVis works with standard NetCDF files from:

- **NOAA Climate Data Online**: Weather station data, radar data
- **ERA5 Reanalysis**: Global atmospheric reanalysis data
- **CMIP6 Climate Models**: Climate projection data
- **Satellite Data**: MODIS, AVHRR, and other remote sensing products

```bash
# Example with real climate data
wget https://downloads.psl.noaa.gov/data/gridded/example.nc
runevis -f example.nc --list-vars
runevis -f example.nc --mean temperature:time --threads 8
```

## CLI Reference

### Core Options

| Option | Short | Description | Example |
|--------|-------|-------------|----------|
| `--file` | `-f` | Input NetCDF file (required) | `-f data.nc` |
| `--threads` | | Number of threads for parallel processing | `--threads 8` |
| `--verbose` | `-v` | Enable verbose output | `-v` |
| `--output-netcdf` | | Save results to NetCDF file | `--output-netcdf result.nc` |

### Statistical Operations

| Operation | Format | Description |
|-----------|--------|-------------|
| `--mean` | `variable:dimension` | Calculate mean over dimension |
| `--sum` | `variable:dimension` | Calculate sum over dimension |
| `--min` | `variable:dimension` | Find minimum over dimension |
| `--max` | `variable:dimension` | Find maximum over dimension |

### Inspection Commands

| Command | Description |
|---------|-------------|
| `--list-vars` | List all variables and dimensions |
| `--describe variable` | Show variable details |
| `--summary variable` | Generate quick statistics |
| `--slice variable:start:end` | Extract data slice |

### Usage Examples

```bash
# Basic file inspection
runevis -f data.nc

# Statistical analysis with threading
runevis -f data.nc --mean temperature:time --threads 4 -v

# Data export
runevis -f data.nc --mean temperature:time --output-netcdf result.nc

# Quality control
runevis -f data.nc --summary temperature
runevis -f data.nc --describe temperature
```

## Dependencies

### Production Dependencies

| Crate | Version | Purpose |
|-------|---------|----------|
| `netcdf` | 0.11.0 | NetCDF file format support |
| `rayon` | 1.10.0 | Parallel processing framework |
| `ndarray` | 0.15.6 | Multi-dimensional array operations |
| `clap` | 4.5.40 | Command-line interface |
| `chrono` | 0.4.41 | Date and time handling |
| `num_cpus` | 1.16.0 | CPU core detection |

### Development Dependencies

| Crate | Version | Purpose |
|-------|---------|----------|
| `tempfile` | 3.20.0 | Temporary file handling for tests |

### Feature Flags

- `netcdf[static]`: Static linking of NetCDF C library
- `ndarray[rayon]`: Parallel array operations
- `clap[derive]`: Procedural macros for CLI
- `chrono[clock,std]`: Essential time functionality only

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run with verbose output
cargo test --verbose

# Run integration tests
cargo test --test integration
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy --all-targets --all-features

# Security audit
cargo audit
```

### Building Examples

```bash
# Build all examples
cargo build --examples

# Run specific example
cargo run --example benchmark --release
```

### Performance Profiling

```bash
# Create large test data
cargo run --example create_large_test_netcdf

# Profile with perf (Linux)
perf record --call-graph dwarf target/release/RuNeVis -f large_test_data.nc --mean temperature:time
perf report
```

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Quick Start for Contributors

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes
4. Run tests: `cargo test`
5. Run quality checks: `cargo fmt && cargo clippy`
6. Submit a pull request

### Code Style

- Use `cargo fmt` for consistent formatting
- Follow Rust naming conventions
- Add tests for new functionality
- Document public APIs

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**Built with ❤️ in Rust**

*RuNeVis: Making NetCDF analysis fast, parallel, and reliable.*
