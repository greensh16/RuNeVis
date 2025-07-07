//! Centralized error handling for RuNeVis
//!
//! This module provides structured error types to replace the generic `Box<dyn Error>`
//! used throughout the codebase, enabling better error context and type safety.

use std::fmt;

/// Main error type for RuNeVis operations
#[derive(Debug)]
pub enum RuNeVisError {
    /// NetCDF file operation errors
    NetCDFError(netcdf::Error),
    
    /// Statistics computation errors
    StatisticsError(String),
    
    /// I/O operation errors
    IoError(std::io::Error),
    
    /// Variable not found in NetCDF file
    VariableNotFound { var: String },
    
    /// Dimension not found in variable
    DimensionNotFound { var: String, dim: String },
    
    /// Invalid slice specification
    InvalidSlice { message: String },
    
    /// Thread pool configuration error
    ThreadPoolError(String),
    
    /// Array shape or dimension error
    ArrayError(ndarray::ShapeError),
    
    /// Generic error for backward compatibility
    Generic(String),
}

impl fmt::Display for RuNeVisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuNeVisError::NetCDFError(e) => write!(f, "NetCDF error: {}", e),
            RuNeVisError::StatisticsError(msg) => write!(f, "Statistics computation error: {}", msg),
            RuNeVisError::IoError(e) => write!(f, "I/O error: {}", e),
            RuNeVisError::VariableNotFound { var } => write!(f, "Variable '{}' not found in file", var),
            RuNeVisError::DimensionNotFound { var, dim } => {
                write!(f, "Dimension '{}' not found in variable '{}'", dim, var)
            }
            RuNeVisError::InvalidSlice { message } => write!(f, "Invalid slice specification: {}", message),
            RuNeVisError::ThreadPoolError(msg) => write!(f, "Thread pool error: {}", msg),
            RuNeVisError::ArrayError(e) => write!(f, "Array error: {}", e),
            RuNeVisError::Generic(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for RuNeVisError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RuNeVisError::NetCDFError(e) => Some(e),
            RuNeVisError::IoError(e) => Some(e),
            RuNeVisError::ArrayError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<netcdf::Error> for RuNeVisError {
    fn from(error: netcdf::Error) -> Self {
        RuNeVisError::NetCDFError(error)
    }
}

impl From<std::io::Error> for RuNeVisError {
    fn from(error: std::io::Error) -> Self {
        RuNeVisError::IoError(error)
    }
}

impl From<ndarray::ShapeError> for RuNeVisError {
    fn from(error: ndarray::ShapeError) -> Self {
        RuNeVisError::ArrayError(error)
    }
}

impl From<String> for RuNeVisError {
    fn from(error: String) -> Self {
        RuNeVisError::Generic(error)
    }
}

impl From<&str> for RuNeVisError {
    fn from(error: &str) -> Self {
        RuNeVisError::Generic(error.to_string())
    }
}

/// Result type alias for RuNeVis operations
pub type Result<T> = std::result::Result<T, RuNeVisError>;
