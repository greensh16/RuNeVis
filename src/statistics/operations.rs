//! Core statistical operations and traits
//!
//! This module defines the fundamental types and traits for statistical operations.

use crate::errors::{Result, RuNeVisError};
use ndarray::ArrayD;

/// Supported statistical operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatOperation {
    /// Arithmetic mean
    Mean,
    /// Sum of values
    Sum,
    /// Minimum value
    Min,
    /// Maximum value
    Max,
}

impl StatOperation {
    /// Get the string representation of the operation
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Mean => "mean",
            Self::Sum => "sum",
            Self::Min => "minimum",
            Self::Max => "maximum",
        }
    }
}

/// Result of a statistical computation
#[derive(Debug)]
#[allow(dead_code)]
pub struct StatResult<T> {
    /// The computed data array
    pub data: ArrayD<T>,
    /// Names of remaining dimensions after reduction
    pub remaining_dimensions: Vec<String>,
    /// The operation that was performed
    pub operation: StatOperation,
    /// Original variable name
    pub variable_name: String,
    /// Dimension that was reduced over
    pub dimension_name: String,
}

impl<T> StatResult<T> {
    /// Create a new statistical result
    #[must_use]
    pub const fn new(
        data: ArrayD<T>,
        remaining_dimensions: Vec<String>,
        operation: StatOperation,
        variable_name: String,
        dimension_name: String,
    ) -> Self {
        Self {
            data,
            remaining_dimensions,
            operation,
            variable_name,
            dimension_name,
        }
    }

    /// Get the shape of the result data
    #[must_use]
    pub fn shape(&self) -> &[usize] {
        self.data.shape()
    }

    /// Get the number of dimensions in the result
    #[must_use]
    pub fn ndim(&self) -> usize {
        self.data.ndim()
    }
}

/// Trait for types that can perform statistical reductions along an axis
pub trait StatisticalReduction<T> {
    /// Perform a statistical reduction along the specified axis
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The axis is out of bounds for the array
    /// - The operation cannot be performed on the data type
    /// - Memory allocation fails
    fn reduce_along_axis(&self, axis: usize, operation: StatOperation) -> Result<ArrayD<T>>;
}

impl StatisticalReduction<f32> for ArrayD<f32> {
    fn reduce_along_axis(&self, axis: usize, operation: StatOperation) -> Result<ArrayD<f32>> {
        if axis >= self.ndim() {
            return Err(RuNeVisError::StatisticsError(format!(
                "Axis {axis} is out of bounds for array with {} dimensions",
                self.ndim()
            )));
        }

        match operation {
            StatOperation::Mean => super::parallel::parallel_mean_axis(self, axis),
            StatOperation::Sum => super::parallel::parallel_sum_axis(self, axis),
            StatOperation::Min => super::parallel::parallel_min_axis(self, axis),
            StatOperation::Max => super::parallel::parallel_max_axis(self, axis),
        }
    }
}
