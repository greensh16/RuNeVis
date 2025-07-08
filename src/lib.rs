//! RuNeVis: NetCDF and Zarr variable processing and analysis
//!
//! A comprehensive Rust library for analyzing NetCDF (Network Common Data Form) and
//! Zarr files. RuNeVis provides functionality for computing statistics like means, sums,
//! minimums, and maximums over specified dimensions of NetCDF variables and Zarr arrays
//! using parallel processing.
//!
//! ## Key Features
//!
//! - **Parallel Processing**: Efficient computation using Rayon for multi-core processing
//! - **Statistical Functions**: Mean, sum, min, max calculations over any dimension
//! - **Metadata Inspection**: Detailed variable and dimension information
//! - **Data Slicing**: Extract specific slices of multi-dimensional data
//! - **NetCDF & Zarr Support**: Read and write both NetCDF and Zarr formats
//! - **Cloud Storage**: Access Zarr arrays from S3, GCS, and other cloud providers
//!
//! ## Module Organization
//!
//! The library is organized into logical modules:
//!
//! - [`metadata`]: NetCDF file inspection and variable description
//! - [`statistics`]: Statistical computations and parallel reductions for NetCDF and Zarr
//! - [`netcdf_io`]: NetCDF file I/O operations and data slicing
//! - [`zarr_io`]: Zarr array I/O operations with cloud storage support
//! - [`parallel`]: Parallel processing configuration
//! - [`errors`]: Centralized error handling
//!
//! ## Usage Examples
//!
//! ### NetCDF Usage
//! ```rust,no_run
//! use ru_ne_vis::prelude::*;
//! use netcdf::open;
//!
//! // Open a NetCDF file
//! let file = open("data.nc").unwrap();
//!
//! // Compute mean over time dimension
//! let (mean_data, dims, var_name) = ru_ne_vis::statistics::mean_over_dimension(&file, "temperature", "time").unwrap();
//!
//! // Print file metadata
//! ru_ne_vis::metadata::print_metadata(&file).unwrap();
//! ```
//!
//! ### Zarr Usage (Future Implementation)
//! ```rust,no_run
//! use ru_ne_vis::prelude::*;
//!
//! // Zarr support is currently being developed
//! // These APIs will be available in future versions:
//!
//! // Basic Zarr array creation
//! let source = ZarrSource::from_path_str("/path/to/zarr/store").unwrap();
//!
//! // Note: Full Zarr functionality is not yet implemented
//! // For now, please use NetCDF files for data processing
//! ```
//!
//! The library is designed to handle large multi-dimensional datasets efficiently
//! and provides clear error reporting for debugging and analysis.

// Core modules
pub mod data_source;
pub mod errors;
pub mod metadata;
pub mod netcdf_io;
pub mod parallel;
pub mod statistics;
pub mod zarr_io;

// Internal modules
mod cli;
mod utils;

// Direct re-exports for the public API
pub use data_source::*;
pub use errors::*;
pub use metadata::*;
pub use netcdf_io::*;
pub use parallel::*;
pub use statistics::*;
pub use zarr_io::*;

// High-level convenience API
pub mod prelude {
    //! Commonly used imports for convenience
    pub use crate::data_source::{DataReader, DataWriter, LazyDataReader, StreamingDataReader, DataArrayMetadata, AdvancedDataSource};
    pub use crate::errors::{Result, RuNeVisError};
    pub use crate::netcdf_io::NetCDFWriter;
    pub use crate::parallel::ParallelConfig;
    pub use crate::statistics::{StatOperation, StatisticalReduction};
    pub use crate::zarr_io::{ArrayMetadata, ZarrReader, ZarrSource, ZarrWriter, ZarrDataSource, LazyArray};
}

// Backwards compatibility re-exports
#[deprecated(
    since = "0.2.0",
    note = "Use the modular API instead: `metadata::compute_variable_summary`"
)]
pub use crate::metadata::compute_variable_summary;

#[deprecated(
    since = "0.2.0",
    note = "Use the modular API instead: `metadata::describe_variable`"
)]
pub use crate::metadata::describe_variable;

#[deprecated(
    since = "0.2.0",
    note = "Use the modular API instead: `io::extract_slice`"
)]
pub use crate::netcdf_io::extract_slice;

#[deprecated(
    since = "0.2.0",
    note = "Use the modular API instead: `metadata::list_variables_and_dimensions`"
)]
pub use crate::metadata::list_variables_and_dimensions;

#[deprecated(
    since = "0.2.0",
    note = "Use the modular API instead: `statistics::max_over_dimension`"
)]
pub use crate::statistics::max_over_dimension;

#[deprecated(
    since = "0.2.0",
    note = "Use the modular API instead: `statistics::mean_over_dimension`"
)]
pub use crate::statistics::mean_over_dimension;

#[deprecated(
    since = "0.2.0",
    note = "Use the modular API instead: `statistics::min_over_dimension`"
)]
pub use crate::statistics::min_over_dimension;

#[deprecated(
    since = "0.2.0",
    note = "Use the modular API instead: `metadata::print_metadata`"
)]
pub use crate::metadata::print_metadata;

#[deprecated(
    since = "0.2.0",
    note = "Use the modular API instead: `statistics::reduce_max`"
)]
pub use crate::statistics::reduce_max;

#[deprecated(
    since = "0.2.0",
    note = "Use the modular API instead: `statistics::reduce_min`"
)]
pub use crate::statistics::reduce_min;

#[deprecated(
    since = "0.2.0",
    note = "Use the modular API instead: `statistics::sum_over_dimension`"
)]
pub use crate::statistics::sum_over_dimension;

#[deprecated(
    since = "0.2.0",
    note = "Use the modular API instead: `io::write_max_to_netcdf`"
)]
pub use crate::netcdf_io::write_max_to_netcdf;

#[deprecated(
    since = "0.2.0",
    note = "Use the modular API instead: `io::write_mean_to_netcdf`"
)]
pub use crate::netcdf_io::write_mean_to_netcdf;

#[deprecated(
    since = "0.2.0",
    note = "Use the modular API instead: `io::write_min_to_netcdf`"
)]
pub use crate::netcdf_io::write_min_to_netcdf;

#[deprecated(
    since = "0.2.0",
    note = "Use the modular API instead: `io::write_sum_to_netcdf`"
)]
pub use crate::netcdf_io::write_sum_to_netcdf;
