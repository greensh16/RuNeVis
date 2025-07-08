//! Statistical computations and parallel reduction operations
//!
//! This module provides functions for computing statistical reductions (mean, sum, min, max)
//! over specified dimensions of `NetCDF` variables and Zarr arrays using parallel processing.
//!
//! # Organization
//!
//! This module is organized into submodules:
//! - [`operations`]: Core statistical operations and traits
//! - [`parallel`]: Parallel computation implementations
//! - [`netcdf`]: NetCDF-specific statistical functions
//! - [`zarr`]: Zarr-specific statistical functions (future implementation)

pub mod netcdf;
pub mod operations;
pub mod parallel;
pub mod zarr;

// Re-export the main types and functions for convenience
pub use netcdf::{max_over_dimension, mean_over_dimension, min_over_dimension, sum_over_dimension};
pub use operations::{StatOperation, StatResult, StatisticalReduction};
pub use parallel::{parallel_max_axis, parallel_mean_axis, parallel_min_axis, parallel_sum_axis};

// Legacy functions for backwards compatibility
pub use netcdf::{reduce_max, reduce_min};
