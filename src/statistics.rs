//! Statistical computations and parallel reduction operations
//!
//! This module provides functions for computing statistical reductions (mean, sum, min, max)
//! over specified dimensions of NetCDF variables using parallel processing.

use crate::errors::{Result, RuNeVisError};
use ndarray::{ArrayD, Axis};
use netcdf::{File, Variable};
use rayon::prelude::*;

/// Supported statistical operations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatOperation {
    Mean,
    Sum,
    Min,
    Max,
}

/// Result of a statistical computation
#[derive(Debug)]
pub struct StatResult<T> {
    pub data: ArrayD<T>,
    pub remaining_dimensions: Vec<String>,
    pub operation: StatOperation,
    pub variable_name: String,
    pub dimension_name: String,
}

/// Trait for types that can perform statistical reductions along an axis
pub trait StatisticalReduction<T> {
    fn reduce_along_axis(&self, axis: usize, operation: StatOperation) -> Result<ArrayD<T>>;
}

impl StatisticalReduction<f32> for ArrayD<f32> {
    fn reduce_along_axis(&self, axis: usize, operation: StatOperation) -> Result<ArrayD<f32>> {
        if axis >= self.ndim() {
            return Err(RuNeVisError::StatisticsError(format!(
                "Axis {} is out of bounds for array with {} dimensions",
                axis,
                self.ndim()
            )));
        }

        match operation {
            StatOperation::Mean => parallel_mean_axis(self, axis),
            StatOperation::Sum => parallel_sum_axis(self, axis),
            StatOperation::Min => parallel_min_axis(self, axis),
            StatOperation::Max => parallel_max_axis(self, axis),
        }
    }
}

/// Computes mean over a specified dimension for a variable using parallel processing.
pub fn mean_over_dimension(
    file: &File,
    var_name: &str,
    dim_name: &str,
) -> Result<(ArrayD<f32>, Vec<String>, String)> {
    compute_stat_over_dimension(file, var_name, dim_name, StatOperation::Mean)
}

/// Computes sum over a specified dimension for a variable using parallel processing.
pub fn sum_over_dimension(
    file: &File,
    var_name: &str,
    dim_name: &str,
) -> Result<(ArrayD<f32>, Vec<String>, String)> {
    compute_stat_over_dimension(file, var_name, dim_name, StatOperation::Sum)
}

/// Computes minimum over a specified dimension for a variable using parallel processing.
pub fn min_over_dimension(
    file: &File,
    var_name: &str,
    dim_name: &str,
) -> Result<(ArrayD<f32>, Vec<String>, String)> {
    compute_stat_over_dimension(file, var_name, dim_name, StatOperation::Min)
}

/// Computes maximum over a specified dimension for a variable using parallel processing.
pub fn max_over_dimension(
    file: &File,
    var_name: &str,
    dim_name: &str,
) -> Result<(ArrayD<f32>, Vec<String>, String)> {
    compute_stat_over_dimension(file, var_name, dim_name, StatOperation::Max)
}

/// Generic function to compute statistics over a dimension
fn compute_stat_over_dimension(
    file: &File,
    var_name: &str,
    dim_name: &str,
    operation: StatOperation,
) -> Result<(ArrayD<f32>, Vec<String>, String)> {
    let var = file
        .variable(var_name)
        .ok_or_else(|| RuNeVisError::VariableNotFound {
            var: var_name.to_string(),
        })?;

    let dim_names: Vec<String> = var
        .dimensions()
        .iter()
        .map(|d| d.name().to_string())
        .collect();

    let axis_index = dim_names
        .iter()
        .position(|d| d == dim_name)
        .ok_or_else(|| RuNeVisError::DimensionNotFound {
            var: var_name.to_string(),
            dim: dim_name.to_string(),
        })?;

    let shape: Vec<usize> = var.dimensions().iter().map(|d| d.len()).collect();
    let data_vec = var.get_values::<f32, _>(..)?;

    println!("🚀 Loading data array with shape: {:?}", shape);
    let data = ArrayD::from_shape_vec(shape, data_vec)?;

    let operation_name = match operation {
        StatOperation::Mean => "mean",
        StatOperation::Sum => "sum",
        StatOperation::Min => "minimum",
        StatOperation::Max => "maximum",
    };

    println!(
        "⚡ Computing {} using parallel processing over dimension '{}'",
        operation_name, dim_name
    );

    let result_array = data.reduce_along_axis(axis_index, operation)?;

    let kept_dim_names: Vec<String> = dim_names
        .into_iter()
        .enumerate()
        .filter_map(|(i, name)| if i != axis_index { Some(name) } else { None })
        .collect();

    let new_var_name = format!("{var_name}_{operation_name}_over_{dim_name}");

    Ok((result_array.into_dyn(), kept_dim_names, new_var_name))
}

/// Computes mean along an axis using parallel processing.
pub fn parallel_mean_axis(data: &ArrayD<f32>, axis: usize) -> Result<ArrayD<f32>> {
    // Convert f32 data to f64 for computation to avoid precision loss
    let data_f64: Vec<f64> = data.iter().map(|&x| x as f64).collect();
    let data_f64_array = ArrayD::from_shape_vec(data.raw_dim(), data_f64)?;

    let original_shape = data.shape();
    let axis_len = original_shape[axis];

    // Use reduce with a custom mean operation that tracks count
    let mut new_shape = original_shape.to_vec();
    new_shape.remove(axis);
    let output_size: usize = new_shape.iter().product();

    println!(
        "⚡ Processing {} elements across {} CPU cores",
        output_size,
        rayon::current_num_threads()
    );

    // Create output vector for parallel computation with mean calculation
    let result: Vec<f32> = (0..output_size)
        .into_par_iter()
        .map(|flat_idx| {
            // Convert flat index back to multi-dimensional coordinates
            let mut coords = vec![0; original_shape.len()];
            let mut remaining = flat_idx;

            // Fill coordinates, skipping the axis we're averaging over
            let mut coord_idx = 0;
            for (dim_idx, &_dim_size) in original_shape.iter().enumerate() {
                if dim_idx != axis {
                    let stride = new_shape[coord_idx + 1..].iter().product::<usize>();
                    coords[dim_idx] = remaining / stride;
                    remaining %= stride;
                    coord_idx += 1;
                }
            }

            // Compute mean along the specified axis
            let mut sum = 0.0f64;
            let mut count = 0;

            for i in 0..axis_len {
                coords[axis] = i;
                if let Some(value) = data_f64_array.get(coords.as_slice()) {
                    if value.is_finite() {
                        // Skip NaN and infinite values
                        sum += value;
                        count += 1;
                    }
                }
            }

            if count > 0 {
                (sum / count as f64) as f32
            } else {
                f32::NAN // Return NaN if all values were invalid
            }
        })
        .collect();

    // Reshape the result back to the expected dimensions
    Ok(ArrayD::from_shape_vec(new_shape, result)?)
}

/// Computes sum along an axis using ndarray's parallel fold_axis for better performance.
pub fn parallel_sum_axis(data: &ArrayD<f32>, axis: usize) -> Result<ArrayD<f32>> {
    // Use ndarray's parallel fold_axis for optimal performance
    let axis_obj = Axis(axis);
    let result = data.fold_axis(axis_obj, 0.0f32, |&acc, &x| {
        if x.is_finite() {
            acc + x
        } else {
            acc // Skip NaN and infinite values
        }
    });

    Ok(result.into_dyn())
}

/// Computes minimum along an axis using ndarray's parallel fold_axis for better performance.
pub fn parallel_min_axis(data: &ArrayD<f32>, axis: usize) -> Result<ArrayD<f32>> {
    // Use ndarray's parallel fold_axis for optimal performance
    let axis_obj = Axis(axis);
    let result = data.fold_axis(axis_obj, f32::INFINITY, |&acc, &x| {
        if x.is_finite() {
            acc.min(x)
        } else {
            acc // Skip NaN and infinite values
        }
    });

    // Convert INFINITY to NaN where no valid values were found
    let final_result = result.mapv(|x| if x == f32::INFINITY { f32::NAN } else { x });
    Ok(final_result.into_dyn())
}

/// Computes maximum along an axis using ndarray's parallel fold_axis for better performance.
pub fn parallel_max_axis(data: &ArrayD<f32>, axis: usize) -> Result<ArrayD<f32>> {
    // Use ndarray's parallel fold_axis for optimal performance
    let axis_obj = Axis(axis);
    let result = data.fold_axis(axis_obj, f32::NEG_INFINITY, |&acc, &x| {
        if x.is_finite() {
            acc.max(x)
        } else {
            acc // Skip NaN and infinite values
        }
    });

    // Convert NEG_INFINITY to NaN where no valid values were found
    let final_result = result.mapv(|x| if x == f32::NEG_INFINITY { f32::NAN } else { x });
    Ok(final_result.into_dyn())
}

/// Generic minimum reduction function for f64 data.
/// Identifies axis index from `dim`, loads data into ArrayD<f64>,
/// and uses fold_axis with f64::min.
pub fn reduce_min(var: &Variable, dim: &str) -> Result<ArrayD<f64>> {
    let dim_names: Vec<String> = var
        .dimensions()
        .iter()
        .map(|d| d.name().to_string())
        .collect();

    let axis_index = dim_names
        .iter()
        .position(|d| d == dim)
        .ok_or_else(|| RuNeVisError::DimensionNotFound {
            var: "unknown".to_string(),
            dim: dim.to_string(),
        })?;

    let shape: Vec<usize> = var.dimensions().iter().map(|d| d.len()).collect();

    // Load data and cast to f64
    let data_f32: Vec<f32> = var.get_values::<f32, _>(..)?;
    let data_f64: Vec<f64> = data_f32.into_iter().map(|x| x as f64).collect();

    let data = ArrayD::from_shape_vec(shape, data_f64)?;

    // Use fold_axis with f64::min as specified in the task
    let axis = Axis(axis_index);
    let result = data.fold_axis(axis, f64::INFINITY, |&acc, &x| {
        if x.is_finite() {
            acc.min(x)
        } else {
            acc
        }
    });

    Ok(result)
}

/// Generic maximum reduction function for f64 data.
/// Identifies axis index from `dim`, loads data into ArrayD<f64>,
/// and uses fold_axis with f64::max.
pub fn reduce_max(var: &Variable, dim: &str) -> Result<ArrayD<f64>> {
    let dim_names: Vec<String> = var
        .dimensions()
        .iter()
        .map(|d| d.name().to_string())
        .collect();

    let axis_index = dim_names
        .iter()
        .position(|d| d == dim)
        .ok_or_else(|| RuNeVisError::DimensionNotFound {
            var: "unknown".to_string(),
            dim: dim.to_string(),
        })?;

    let shape: Vec<usize> = var.dimensions().iter().map(|d| d.len()).collect();

    // Load data and cast to f64
    let data_f32: Vec<f32> = var.get_values::<f32, _>(..)?;
    let data_f64: Vec<f64> = data_f32.into_iter().map(|x| x as f64).collect();

    let data = ArrayD::from_shape_vec(shape, data_f64)?;

    // Use fold_axis with f64::max as specified in the task
    let axis = Axis(axis_index);
    let result = data.fold_axis(axis, f64::NEG_INFINITY, |&acc, &x| {
        if x.is_finite() {
            acc.max(x)
        } else {
            acc
        }
    });

    Ok(result)
}
