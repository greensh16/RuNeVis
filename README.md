# RuNeVis

![workflow](https://github.com/greensh16/RuNeVis/actions/workflows/rust.yml/badge.svg)

A high-performance Rust application for working with NetCDF files, featuring parallel processing capabilities for efficient data analysis.

## Features

### Parallel Processing
- **Multi-core computation**: Leverages Rayon for parallel mean calculations across multiple CPU cores
- **Thread control**: Configure the number of threads with `--threads` option
- **Efficient memory usage**: Optimized for large datasets with smart memory management

### Data Analysis
- **Mean computation**: Calculate means over any dimension with parallel processing
- **Metadata inspection**: View global attributes, variables, and dimensions
- **Data export**: Save results to new NetCDF files with preserved metadata

### Performance Features
- **Robust error handling**: NaN and infinite value detection
- **Progress indicators**: Visual feedback for long-running operations
- **Memory efficient**: Streaming operations for large datasets

## Usage

### Basic Operations

```bash
# View file metadata
runevis -f data.nc

# Compute mean over time dimension (using all CPU cores)
runevis -f data.nc --mean temperature:time

# Compute minimum over time dimension
runevis -f data.nc --min temperature:time

# Compute maximum over time dimension
runevis -f data.nc --max temperature:time

# Save result to new NetCDF file
runevis -f data.nc --mean temperature:time --output-netcdf result.nc

# Save minimum result to NetCDF file
runevis -f data.nc --min temperature:time --output-netcdf min_result.nc

# Save maximum result to NetCDF file
runevis -f data.nc --max temperature:time --output-netcdf max_result.nc

# Control number of threads for parallel processing
runevis -f data.nc --mean temperature:time --threads 4

# Verbose output
runevis -f data.nc --mean temperature:time -v
```

### Performance Tips

1. **Thread optimization**: Use `--threads` to match your system's capabilities
2. **Large datasets**: The parallel implementation scales well with data size
3. **Memory usage**: Consider available RAM when processing very large files

## Performance Results

Based on benchmark testing:
- **1M data points**: ~5.7x speedup
- **5M data points**: ~6.5x speedup  
- **10M data points**: ~7.1x speedup

Performance scales well with:
- Dataset size (larger = better speedup)
- Number of CPU cores
- Computational complexity

## Installation

```bash
# Build with optimizations
cargo build --release

# Run
./target/release/RuNeVis --help
```

## Dependencies

- **Rayon**: Parallel processing framework
- **NetCDF**: Scientific data format support
- **ndarray**: Multi-dimensional array operations
- **clap**: Command-line interface
- **chrono**: Timestamp generation
