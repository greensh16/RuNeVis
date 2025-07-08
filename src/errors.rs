//! Centralized error handling for `RuNeVis`
//!
//! This module provides structured error types to replace the generic `Box<dyn Error>`
//! used throughout the codebase, enabling better error context and type safety.

use std::fmt;

/// Main error type for `RuNeVis` operations
#[derive(Debug)]
pub enum RuNeVisError {
    /// `NetCDF` file operation errors
    NetCDFError(netcdf::Error),

    /// Zarr file operation errors
    #[allow(dead_code)]
    ZarrError(String),

    /// Statistics computation errors
    StatisticsError(String),

    /// I/O operation errors
    IoError(std::io::Error),

    /// Variable not found in `NetCDF` file
    VariableNotFound { var: String },

    /// Array not found in Zarr store
    #[allow(dead_code)]
    ArrayNotFound { array: String },

    /// Dimension not found in variable
    DimensionNotFound { var: String, dim: String },

    /// Invalid slice specification
    InvalidSlice { message: String },

    /// Thread pool configuration error
    ThreadPoolError(String),

    /// Array shape or dimension error
    ArrayError(ndarray::ShapeError),

    /// Async runtime error
    #[allow(dead_code)]
    AsyncError(String),

    /// Generic error for backward compatibility
    Generic(String),
}

impl fmt::Display for RuNeVisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NetCDFError(e) => write!(f, "NetCDF error: {e}"),
            Self::ZarrError(e) => write!(f, "Zarr error: {e}"),
            Self::StatisticsError(msg) => {
                write!(f, "Statistics computation error: {msg}")
            }
            Self::IoError(e) => write!(f, "I/O error: {e}"),
            Self::VariableNotFound { var } => {
                write!(f, "Variable '{var}' not found in file")
            }
            Self::ArrayNotFound { array } => {
                write!(f, "Array '{array}' not found in Zarr store")
            }
            Self::DimensionNotFound { var, dim } => {
                write!(f, "Dimension '{dim}' not found in variable '{var}'")
            }
            Self::InvalidSlice { message } => {
                write!(f, "Invalid slice specification: {message}")
            }
            Self::ThreadPoolError(msg) => write!(f, "Thread pool error: {msg}"),
            Self::ArrayError(e) => write!(f, "Array error: {e}"),
            Self::AsyncError(msg) => write!(f, "Async runtime error: {msg}"),
            Self::Generic(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for RuNeVisError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NetCDFError(e) => Some(e),
            Self::IoError(e) => Some(e),
            Self::ArrayError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<netcdf::Error> for RuNeVisError {
    fn from(error: netcdf::Error) -> Self {
        Self::NetCDFError(error)
    }
}

impl From<std::io::Error> for RuNeVisError {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(error)
    }
}

impl From<ndarray::ShapeError> for RuNeVisError {
    fn from(error: ndarray::ShapeError) -> Self {
        Self::ArrayError(error)
    }
}

impl From<String> for RuNeVisError {
    fn from(error: String) -> Self {
        Self::Generic(error)
    }
}

impl From<&str> for RuNeVisError {
    fn from(error: &str) -> Self {
        Self::Generic(error.to_string())
    }
}

/// Result type alias for `RuNeVis` operations
///
/// This is a convenience type alias that uses [`RuNeVisError`] as the error type.
/// Most functions in this library return this type.
pub type Result<T> = std::result::Result<T, RuNeVisError>;
