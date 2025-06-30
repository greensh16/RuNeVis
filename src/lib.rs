//! RuNeVis - Rust-based NetCDF Visualization and Processing Library
//!
//! This library provides utilities for working with NetCDF files, including:
//! - Metadata extraction and printing
//! - Statistical computations (mean, sum, min, max) over dimensions
//! - Parallel processing for large datasets
//! - NetCDF file creation and writing

pub mod cli;
pub mod utils;

// Re-export commonly used functions
pub use utils::{
    compute_variable_summary, describe_variable, extract_slice, list_variables_and_dimensions,
    max_over_dimension, mean_over_dimension, min_over_dimension, print_metadata, reduce_max,
    reduce_min, sum_over_dimension, write_max_to_netcdf, write_mean_to_netcdf, write_min_to_netcdf,
    write_sum_to_netcdf,
};
