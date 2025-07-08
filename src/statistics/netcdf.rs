//! NetCDF-specific statistical functions
//!
//! This module provides statistical computation functions specifically for NetCDF variables.

use super::operations::{StatOperation, StatisticalReduction};
use crate::errors::{Result, RuNeVisError};
use ndarray::{ArrayD, Axis};
use netcdf::{File, Variable};

/// Computes mean over a specified dimension for a NetCDF variable using parallel processing
///
/// # Arguments
///
/// * `file` - The NetCDF file containing the variable
/// * `var_name` - Name of the variable to compute statistics for
/// * `dim_name` - Name of the dimension to reduce over
///
/// # Returns
///
/// A tuple containing:
/// - The computed mean data as an ArrayD<f32>
/// - Vector of remaining dimension names
/// - Generated variable name for the result
///
/// # Errors
///
/// Returns an error if the variable or dimension is not found, or if computation fails.
pub fn mean_over_dimension(
    file: &File,
    var_name: &str,
    dim_name: &str,
) -> Result<(ArrayD<f32>, Vec<String>, String)> {
    compute_stat_over_dimension(file, var_name, dim_name, StatOperation::Mean)
}

/// Computes sum over a specified dimension for a NetCDF variable using parallel processing
///
/// # Arguments
///
/// * `file` - The NetCDF file containing the variable
/// * `var_name` - Name of the variable to compute statistics for
/// * `dim_name` - Name of the dimension to reduce over
///
/// # Returns
///
/// A tuple containing:
/// - The computed sum data as an ArrayD<f32>
/// - Vector of remaining dimension names
/// - Generated variable name for the result
///
/// # Errors
///
/// Returns an error if the variable or dimension is not found, or if computation fails.
pub fn sum_over_dimension(
    file: &File,
    var_name: &str,
    dim_name: &str,
) -> Result<(ArrayD<f32>, Vec<String>, String)> {
    compute_stat_over_dimension(file, var_name, dim_name, StatOperation::Sum)
}

/// Computes minimum over a specified dimension for a NetCDF variable using parallel processing
///
/// # Arguments
///
/// * `file` - The NetCDF file containing the variable
/// * `var_name` - Name of the variable to compute statistics for
/// * `dim_name` - Name of the dimension to reduce over
///
/// # Returns
///
/// A tuple containing:
/// - The computed minimum data as an ArrayD<f32>
/// - Vector of remaining dimension names
/// - Generated variable name for the result
///
/// # Errors
///
/// Returns an error if the variable or dimension is not found, or if computation fails.
pub fn min_over_dimension(
    file: &File,
    var_name: &str,
    dim_name: &str,
) -> Result<(ArrayD<f32>, Vec<String>, String)> {
    compute_stat_over_dimension(file, var_name, dim_name, StatOperation::Min)
}

/// Computes maximum over a specified dimension for a NetCDF variable using parallel processing
///
/// # Arguments
///
/// * `file` - The NetCDF file containing the variable
/// * `var_name` - Name of the variable to compute statistics for
/// * `dim_name` - Name of the dimension to reduce over
///
/// # Returns
///
/// A tuple containing:
/// - The computed maximum data as an ArrayD<f32>
/// - Vector of remaining dimension names
/// - Generated variable name for the result
///
/// # Errors
///
/// Returns an error if the variable or dimension is not found, or if computation fails.
pub fn max_over_dimension(
    file: &File,
    var_name: &str,
    dim_name: &str,
) -> Result<(ArrayD<f32>, Vec<String>, String)> {
    compute_stat_over_dimension(file, var_name, dim_name, StatOperation::Max)
}

/// Generic function to compute statistics over a dimension
///
/// This is the core implementation that handles loading data from NetCDF
/// and delegating to the appropriate statistical computation.
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

    let shape: Vec<usize> = var
        .dimensions()
        .iter()
        .map(netcdf::Dimension::len)
        .collect();
    let data_vec = var.get_values::<f32, _>(..)?;

    println!("🚀 Loading data array with shape: {shape:?}");
    let data = ArrayD::from_shape_vec(shape, data_vec)?;

    let operation_name = operation.as_str();

    println!("⚡ Computing {operation_name} using parallel processing over dimension '{dim_name}'");

    let result_array = data.reduce_along_axis(axis_index, operation)?;

    let kept_dim_names: Vec<String> = dim_names
        .into_iter()
        .enumerate()
        .filter_map(|(i, name)| if i == axis_index { None } else { Some(name) })
        .collect();

    let new_var_name = format!("{var_name}_{operation_name}_over_{dim_name}");

    Ok((result_array.into_dyn(), kept_dim_names, new_var_name))
}

/// Generic minimum reduction function for f64 data
///
/// Identifies axis index from `dim`, loads data into `ArrayD`<f64>,
/// and uses `fold_axis` with `f64::min`.
///
/// # Errors
///
/// Returns an error if the variable or dimension is not found, or if computation fails.
#[allow(dead_code)]
pub fn reduce_min(var: &Variable, dim: &str) -> Result<ArrayD<f64>> {
    let dim_names: Vec<String> = var
        .dimensions()
        .iter()
        .map(|d| d.name().to_string())
        .collect();

    let axis_index =
        dim_names
            .iter()
            .position(|d| d == dim)
            .ok_or_else(|| RuNeVisError::DimensionNotFound {
                var: "unknown".to_string(),
                dim: dim.to_string(),
            })?;

    let shape: Vec<usize> = var
        .dimensions()
        .iter()
        .map(netcdf::Dimension::len)
        .collect();

    // Load data and cast to f64
    let data_f32: Vec<f32> = var.get_values::<f32, _>(..)?;
    let data_f64: Vec<f64> = data_f32.into_iter().map(f64::from).collect();

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

/// Generic maximum reduction function for f64 data
///
/// Identifies axis index from `dim`, loads data into `ArrayD`<f64>,
/// and uses `fold_axis` with `f64::max`.
///
/// # Errors
///
/// Returns an error if the variable or dimension is not found, or if computation fails.
#[allow(dead_code)]
pub fn reduce_max(var: &Variable, dim: &str) -> Result<ArrayD<f64>> {
    let dim_names: Vec<String> = var
        .dimensions()
        .iter()
        .map(|d| d.name().to_string())
        .collect();

    let axis_index =
        dim_names
            .iter()
            .position(|d| d == dim)
            .ok_or_else(|| RuNeVisError::DimensionNotFound {
                var: "unknown".to_string(),
                dim: dim.to_string(),
            })?;

    let shape: Vec<usize> = var
        .dimensions()
        .iter()
        .map(netcdf::Dimension::len)
        .collect();

    // Load data and cast to f64
    let data_f32: Vec<f32> = var.get_values::<f32, _>(..)?;
    let data_f64: Vec<f64> = data_f32.into_iter().map(f64::from).collect();

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
