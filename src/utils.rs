//! Utility functions for working with NetCDF files, including metadata printing,
//! computing means over dimensions, and writing processed results to new files.

use crate::cli::SliceSpec;
use chrono::Utc;
use ndarray::ArrayD;
use netcdf::{create, AttributeValue, File};
use rayon::prelude::*;
use std::{error::Error, fs, path::Path};

/// Prints global attributes and variables of a NetCDF file.
pub fn print_metadata(file: &File) -> Result<(), Box<dyn Error>> {
    println!("\n===== Global Attributes =====");
    for attr in file.attributes() {
        println!("- {}: {:?}", attr.name(), attr.value()?);
    }

    println!("\n===== Variables =====");
    for var in file.variables() {
        let dims: Vec<String> = var
            .dimensions()
            .iter()
            .map(|d| format!("{}[{}]", d.name(), d.len()))
            .collect();
        println!("- {} ({})", var.name(), dims.join(", "));
    }

    Ok(())
}

/// Computes quick statistics (min/mean/max/std) on a variable.
pub fn compute_variable_summary(file: &File, var_name: &str) -> Result<(), Box<dyn Error>> {
    let var = file.variable(var_name).ok_or("Variable not found")?;

    // Retrieve all data for the variable as f32
    let data: Vec<f32> = var.get_values::<f32, _>(..)?;

    // Compute statistics
    let min = data.iter().cloned().fold(f32::INFINITY, f32::min);
    let max = data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let mean: f32 = data.iter().sum::<f32>() / data.len() as f32;
    let std_dev =
        (data.iter().map(|&x| (x - mean).powi(2)).sum::<f32>() / data.len() as f32).sqrt();

    // Display results
    println!("\n Summary for Variable: {}", var_name);
    println!("================================");
    println!("   Min: {}", min);
    println!("   Max: {}", max);
    println!("   Mean: {:.2}", mean);
    println!("   Std Dev: {:.2}", std_dev);

    Ok(())
}

/// Lists all variables and dimensions in a clean, organized format.
pub fn list_variables_and_dimensions(file: &File) -> Result<(), Box<dyn Error>> {
    println!("\n Dimensions");
    println!("==============");

    // Get all dimensions and sort them alphabetically
    let mut dimensions: Vec<_> = file.dimensions().collect();
    dimensions.sort_by(|a, b| a.name().cmp(&b.name()));

    if dimensions.is_empty() {
        println!("   (No dimensions found)");
    } else {
        for dim in dimensions {
            let length_info = if dim.is_unlimited() {
                format!("{} (unlimited)", dim.len())
            } else {
                dim.len().to_string()
            };
            println!("    {} = {}", dim.name(), length_info);
        }
    }

    println!("\n Variables");
    println!("=============");

    // Get all variables and sort them alphabetically
    let mut variables: Vec<_> = file.variables().collect();
    variables.sort_by(|a, b| a.name().cmp(&b.name()));

    if variables.is_empty() {
        println!("   (No variables found)");
    } else {
        for var in variables {
            // Get data type as string - simplified approach
            let data_type = format!("{:?}", var.vartype()).to_lowercase();

            let dims: Vec<String> = var
                .dimensions()
                .iter()
                .map(|d| d.name().to_string())
                .collect();

            let shape: Vec<String> = var
                .dimensions()
                .iter()
                .map(|d| d.len().to_string())
                .collect();

            if dims.is_empty() {
                println!("    {} ({}): scalar", var.name(), data_type);
            } else {
                println!(
                    "    {} ({}): [{}] = ({})",
                    var.name(),
                    data_type,
                    dims.join(", "),
                    shape.join(" × ")
                );
            }

            // Show key attributes if they exist
            let mut key_attrs = Vec::new();

            if let Some(units_attr) = var.attribute("units") {
                if let Ok(units_val) = units_attr.value() {
                    if let netcdf::AttributeValue::Str(units) = units_val {
                        key_attrs.push(format!("units: {}", units));
                    }
                }
            }

            if let Some(long_name_attr) = var.attribute("long_name") {
                if let Ok(long_name_val) = long_name_attr.value() {
                    if let netcdf::AttributeValue::Str(long_name) = long_name_val {
                        key_attrs.push(format!("long_name: {}", long_name));
                    }
                }
            }

            if let Some(fill_value_attr) = var.attribute("_FillValue") {
                if let Ok(fill_val) = fill_value_attr.value() {
                    match fill_val {
                        netcdf::AttributeValue::Float(fv) => {
                            key_attrs.push(format!("_FillValue: {}", fv))
                        }
                        netcdf::AttributeValue::Double(fv) => {
                            key_attrs.push(format!("_FillValue: {}", fv))
                        }
                        netcdf::AttributeValue::Int(fv) => {
                            key_attrs.push(format!("_FillValue: {}", fv))
                        }
                        netcdf::AttributeValue::Short(fv) => {
                            key_attrs.push(format!("_FillValue: {}", fv))
                        }
                        _ => {}
                    }
                }
            }

            if !key_attrs.is_empty() {
                println!("      └─ {}", key_attrs.join(", "));
            }
        }
    }

    println!(
        "\n💡 Tip: Use --mean <variable>:<dimension> to compute means over specific dimensions"
    );
    println!("💡 Tip: Use --threads <N> to control parallel processing threads");

    Ok(())
}

/// Computes mean over a specified dimension for a variable using parallel processing.
pub fn mean_over_dimension(
    file: &File,
    var_name: &str,
    dim_name: &str,
) -> Result<(ArrayD<f32>, Vec<String>, String), Box<dyn Error>> {
    let var = file.variable(var_name).ok_or("Variable not found")?;

    let dim_names: Vec<String> = var
        .dimensions()
        .iter()
        .map(|d| d.name().to_string())
        .collect();

    let axis_index = dim_names
        .iter()
        .position(|d| d == dim_name)
        .ok_or("Dimension not found in variable")?;

    let shape: Vec<usize> = var.dimensions().iter().map(|d| d.len()).collect();
    let data_vec = var.get_values::<f32, _>(..)?;

    println!("🚀 Loading data array with shape: {:?}", shape);
    let data = ArrayD::from_shape_vec(shape, data_vec)?;

    println!(
        "⚡ Computing mean using parallel processing over dimension '{}'",
        dim_name
    );
    let mean_array = parallel_mean_axis(&data, axis_index)?.into_dyn();

    let kept_dim_names: Vec<String> = dim_names
        .into_iter()
        .enumerate()
        .filter_map(|(i, name)| if i != axis_index { Some(name) } else { None })
        .collect();

    let new_var_name = format!("{var_name}_mean_over_{dim_name}");

    Ok((mean_array, kept_dim_names, new_var_name))
}

/// Computes sum over a specified dimension for a variable using parallel processing.
pub fn sum_over_dimension(
    file: &File,
    var_name: &str,
    dim_name: &str,
) -> Result<(ArrayD<f32>, Vec<String>, String), Box<dyn Error>> {
    let var = file.variable(var_name).ok_or("Variable not found")?;

    let dim_names: Vec<String> = var
        .dimensions()
        .iter()
        .map(|d| d.name().to_string())
        .collect();

    let axis_index = dim_names
        .iter()
        .position(|d| d == dim_name)
        .ok_or("Dimension not found in variable")?;

    let shape: Vec<usize> = var.dimensions().iter().map(|d| d.len()).collect();
    let data_vec = var.get_values::<f32, _>(..)?;

    println!("🚀 Loading data array with shape: {:?}", shape);
    let data = ArrayD::from_shape_vec(shape, data_vec)?;

    println!(
        "⚡ Computing sum using parallel processing over dimension '{}'",
        dim_name
    );
    let sum_array = parallel_sum_axis(&data, axis_index)?.into_dyn();

    let kept_dim_names: Vec<String> = dim_names
        .into_iter()
        .enumerate()
        .filter_map(|(i, name)| if i != axis_index { Some(name) } else { None })
        .collect();

    let new_var_name = format!("{var_name}_sum_over_{dim_name}");

    Ok((sum_array, kept_dim_names, new_var_name))
}

/// Computes minimum over a specified dimension for a variable using parallel processing.
pub fn min_over_dimension(
    file: &File,
    var_name: &str,
    dim_name: &str,
) -> Result<(ArrayD<f32>, Vec<String>, String), Box<dyn Error>> {
    let var = file.variable(var_name).ok_or("Variable not found")?;

    let dim_names: Vec<String> = var
        .dimensions()
        .iter()
        .map(|d| d.name().to_string())
        .collect();

    let axis_index = dim_names
        .iter()
        .position(|d| d == dim_name)
        .ok_or("Dimension not found in variable")?;

    let shape: Vec<usize> = var.dimensions().iter().map(|d| d.len()).collect();
    let data_vec = var.get_values::<f32, _>(..)?;

    println!("🚀 Loading data array with shape: {:?}", shape);
    let data = ArrayD::from_shape_vec(shape, data_vec)?;

    println!(
        "⚡ Computing minimum using parallel processing over dimension '{}'",
        dim_name
    );
    let min_array = parallel_min_axis(&data, axis_index)?.into_dyn();

    let kept_dim_names: Vec<String> = dim_names
        .into_iter()
        .enumerate()
        .filter_map(|(i, name)| if i != axis_index { Some(name) } else { None })
        .collect();

    let new_var_name = format!("{var_name}_min_over_{dim_name}");

    Ok((min_array, kept_dim_names, new_var_name))
}

/// Computes maximum over a specified dimension for a variable using parallel processing.
pub fn max_over_dimension(
    file: &File,
    var_name: &str,
    dim_name: &str,
) -> Result<(ArrayD<f32>, Vec<String>, String), Box<dyn Error>> {
    let var = file.variable(var_name).ok_or("Variable not found")?;

    let dim_names: Vec<String> = var
        .dimensions()
        .iter()
        .map(|d| d.name().to_string())
        .collect();

    let axis_index = dim_names
        .iter()
        .position(|d| d == dim_name)
        .ok_or("Dimension not found in variable")?;

    let shape: Vec<usize> = var.dimensions().iter().map(|d| d.len()).collect();
    let data_vec = var.get_values::<f32, _>(..)?;

    println!(" Loading data array with shape: {:?}", shape);
    let data = ArrayD::from_shape_vec(shape, data_vec)?;

    println!(
        " Computing maximum using parallel processing over dimension '{}'",
        dim_name
    );
    let max_array = parallel_max_axis(&data, axis_index)?.into_dyn();

    let kept_dim_names: Vec<String> = dim_names
        .into_iter()
        .enumerate()
        .filter_map(|(i, name)| if i != axis_index { Some(name) } else { None })
        .collect();

    let new_var_name = format!("{var_name}_max_over_{dim_name}");

    Ok((max_array, kept_dim_names, new_var_name))
}

/// Generic reduction function that applies an operation along a specified axis using parallel processing.
/// This function serves as the foundation for mean, sum, min, max, and other reductions.
fn reduce<F>(data: ArrayD<f64>, axis: usize, init: f64, op: F) -> ArrayD<f64>
where
    F: Fn(f64, f64) -> f64 + Sync + Send,
{
    let original_shape = data.shape();
    let axis_len = original_shape[axis];

    // Calculate the shape after removing the specified axis
    let mut new_shape = original_shape.to_vec();
    new_shape.remove(axis);
    let output_size: usize = new_shape.iter().product();

    println!(
        "⚡ Processing {} elements across {} CPU cores",
        output_size,
        rayon::current_num_threads()
    );

    // Create output vector for parallel computation
    let result: Vec<f64> = (0..output_size)
        .into_par_iter()
        .map(|flat_idx| {
            // Convert flat index back to multi-dimensional coordinates
            let mut coords = vec![0; original_shape.len()];
            let mut remaining = flat_idx;

            // Fill coordinates, skipping the axis we're reducing over
            let mut coord_idx = 0;
            for (dim_idx, &_dim_size) in original_shape.iter().enumerate() {
                if dim_idx != axis {
                    let stride = new_shape[coord_idx + 1..].iter().product::<usize>();
                    coords[dim_idx] = remaining / stride;
                    remaining %= stride;
                    coord_idx += 1;
                }
            }

            // Apply the reduction operation along the specified axis
            let mut acc = init;

            for i in 0..axis_len {
                coords[axis] = i;
                if let Some(value) = data.get(coords.as_slice()) {
                    if value.is_finite() {
                        // Skip NaN and infinite values
                        acc = op(acc, *value);
                    }
                }
            }

            acc
        })
        .collect();

    // Reshape the result back to the expected dimensions
    ArrayD::from_shape_vec(new_shape, result).expect("Failed to reshape result")
}

/// Computes mean along an axis using the generic reduce function.
fn parallel_mean_axis(data: &ArrayD<f32>, axis: usize) -> Result<ArrayD<f32>, Box<dyn Error>> {
    // Convert f32 data to f64 for computation
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

/// Computes sum along an axis using the generic reduce function.
fn parallel_sum_axis(data: &ArrayD<f32>, axis: usize) -> Result<ArrayD<f32>, Box<dyn Error>> {
    // Convert f32 data to f64 for computation
    let data_f64: Vec<f64> = data.iter().map(|&x| x as f64).collect();
    let data_f64_array = ArrayD::from_shape_vec(data.raw_dim(), data_f64)?;

    // Use the generic reduce function with sum operation
    let result_f64 = reduce(data_f64_array, axis, 0.0, |a, b| a + b);

    // Convert result back to f32
    let result_f32: Vec<f32> = result_f64.iter().map(|&x| x as f32).collect();
    Ok(ArrayD::from_shape_vec(result_f64.raw_dim(), result_f32)?)
}

/// Computes minimum along an axis using the generic reduce function.
fn parallel_min_axis(data: &ArrayD<f32>, axis: usize) -> Result<ArrayD<f32>, Box<dyn Error>> {
    // Convert f32 data to f64 for computation
    let data_f64: Vec<f64> = data.iter().map(|&x| x as f64).collect();
    let data_f64_array = ArrayD::from_shape_vec(data.raw_dim(), data_f64)?;

    // Use the generic reduce function with min operation
    let result_f64 = reduce(data_f64_array, axis, f64::INFINITY, |a, b| a.min(b));

    // Convert result back to f32, handling the special case where INFINITY remains
    let result_f32: Vec<f32> = result_f64
        .iter()
        .map(|&x| {
            if x == f64::INFINITY {
                f32::NAN // No valid values were found
            } else {
                x as f32
            }
        })
        .collect();

    Ok(ArrayD::from_shape_vec(result_f64.raw_dim(), result_f32)?)
}

/// Computes maximum along an axis using the generic reduce function.
fn parallel_max_axis(data: &ArrayD<f32>, axis: usize) -> Result<ArrayD<f32>, Box<dyn Error>> {
    // Convert f32 data to f64 for computation
    let data_f64: Vec<f64> = data.iter().map(|&x| x as f64).collect();
    let data_f64_array = ArrayD::from_shape_vec(data.raw_dim(), data_f64)?;

    // Use the generic reduce function with max operation
    let result_f64 = reduce(data_f64_array, axis, f64::NEG_INFINITY, |a, b| a.max(b));

    // Convert result back to f32, handling the special case where NEG_INFINITY remains
    let result_f32: Vec<f32> = result_f64
        .iter()
        .map(|&x| {
            if x == f64::NEG_INFINITY {
                f32::NAN // No valid values were found
            } else {
                x as f32
            }
        })
        .collect();

    Ok(ArrayD::from_shape_vec(result_f64.raw_dim(), result_f32)?)
}

// Wrapper helper functions that delegate to the generic reduce function
// These pass the appropriate closure for each operation as specified in the task
// The helper functions are used internally by the parallel_*_axis functions

// Generic reduction functions as specified in the task
// These work with f64 and accept a generic Variable trait
use netcdf::Variable;

/// Generic minimum reduction function for f64 data.
/// Identifies axis index from `dim`, loads data into ArrayD<f64>,
/// and uses fold_axis with f64::min.
pub fn reduce_min(var: &Variable, dim: &str) -> Result<ArrayD<f64>, Box<dyn Error>> {
    let dim_names: Vec<String> = var
        .dimensions()
        .iter()
        .map(|d| d.name().to_string())
        .collect();

    let axis_index = dim_names
        .iter()
        .position(|d| d == dim)
        .ok_or("Dimension not found in variable")?;

    let shape: Vec<usize> = var.dimensions().iter().map(|d| d.len()).collect();

    // Load data and cast to f64
    let data_f32: Vec<f32> = var.get_values::<f32, _>(..)?;
    let data_f64: Vec<f64> = data_f32.into_iter().map(|x| x as f64).collect();

    let data = ArrayD::from_shape_vec(shape, data_f64)?;

    // Use fold_axis with f64::min as specified in the task
    let axis = ndarray::Axis(axis_index);
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
pub fn reduce_max(var: &Variable, dim: &str) -> Result<ArrayD<f64>, Box<dyn Error>> {
    let dim_names: Vec<String> = var
        .dimensions()
        .iter()
        .map(|d| d.name().to_string())
        .collect();

    let axis_index = dim_names
        .iter()
        .position(|d| d == dim)
        .ok_or("Dimension not found in variable")?;

    let shape: Vec<usize> = var.dimensions().iter().map(|d| d.len()).collect();

    // Load data and cast to f64
    let data_f32: Vec<f32> = var.get_values::<f32, _>(..)?;
    let data_f64: Vec<f64> = data_f32.into_iter().map(|x| x as f64).collect();

    let data = ArrayD::from_shape_vec(shape, data_f64)?;

    // Use fold_axis with f64::max as specified in the task
    let axis = ndarray::Axis(axis_index);
    let result = data.fold_axis(axis, f64::NEG_INFINITY, |&acc, &x| {
        if x.is_finite() {
            acc.max(x)
        } else {
            acc
        }
    });

    Ok(result)
}

/// Writes computed mean to a new NetCDF file with attributes copied.
pub fn write_mean_to_netcdf(
    data: &ArrayD<f32>,
    dim_names: &[String],
    var_name: &str,
    original_var_name: &str,
    input_file: &File,
    output_path: &Path,
) -> Result<(), Box<dyn Error>> {
    if output_path.exists() {
        fs::remove_file(output_path)?;
    }

    let mut file = create(output_path)?;

    // Define dimensions
    for (dim_name, &dim_len) in dim_names.iter().zip(data.shape()) {
        file.add_dimension(dim_name, dim_len)?;
    }

    // Extract `_FillValue`
    let orig_var = input_file
        .variable(original_var_name)
        .ok_or("Original variable not found")?;

    let fill_value = orig_var
        .attribute("_FillValue")
        .and_then(|attr| match attr.value().ok()? {
            AttributeValue::Float(v) => Some(v),
            AttributeValue::Double(v) => Some(v as f32),
            AttributeValue::Short(v) => Some(v as f32),
            _ => None,
        });

    let dim_refs: Vec<&str> = dim_names.iter().map(|s| s.as_str()).collect();
    let mut new_var = file.add_variable::<f32>(var_name, &dim_refs)?;

    if let Some(fv) = fill_value {
        new_var.put_attribute("_FillValue", fv)?;
    }

    new_var.put(data.view(), ..)?;

    // Copy remaining attributes excluding _FillValue succinctly
    for attr in orig_var.attributes().filter(|a| a.name() != "_FillValue") {
        match attr.value()? {
            AttributeValue::Str(val) => {
                new_var.put_attribute(attr.name(), val)?;
            }
            AttributeValue::Strs(vals) => {
                new_var.put_attribute(attr.name(), vals)?;
            }
            AttributeValue::Float(val) => {
                new_var.put_attribute(attr.name(), val)?;
            }
            AttributeValue::Floats(vals) => {
                new_var.put_attribute(attr.name(), vals)?;
            }
            AttributeValue::Double(val) => {
                new_var.put_attribute(attr.name(), val)?;
            }
            AttributeValue::Doubles(vals) => {
                new_var.put_attribute(attr.name(), vals)?;
            }
            AttributeValue::Int(val) => {
                new_var.put_attribute(attr.name(), val)?;
            }
            AttributeValue::Ints(vals) => {
                new_var.put_attribute(attr.name(), vals)?;
            }
            AttributeValue::Short(val) => {
                new_var.put_attribute(attr.name(), val)?;
            }
            AttributeValue::Shorts(vals) => {
                new_var.put_attribute(attr.name(), vals)?;
            }
            _ => {
                println!(" Skipped unsupported attribute type for '{}'", attr.name());
            }
        }
    }

    // Add history attribute
    file.add_attribute(
        "history",
        format!("Created by RuNeVis on {}", Utc::now().to_rfc3339()),
    )?;

    Ok(())
}

/// Writes computed sum to a new NetCDF file with attributes copied.
pub fn write_sum_to_netcdf(
    data: &ArrayD<f32>,
    dim_names: &[String],
    var_name: &str,
    original_var_name: &str,
    input_file: &File,
    output_path: &Path,
) -> Result<(), Box<dyn Error>> {
    if output_path.exists() {
        fs::remove_file(output_path)?;
    }

    let mut file = create(output_path)?;

    // Define dimensions
    for (dim_name, &dim_len) in dim_names.iter().zip(data.shape()) {
        file.add_dimension(dim_name, dim_len)?;
    }

    // Extract `_FillValue`
    let orig_var = input_file
        .variable(original_var_name)
        .ok_or("Original variable not found")?;

    let fill_value = orig_var
        .attribute("_FillValue")
        .and_then(|attr| match attr.value().ok()? {
            AttributeValue::Float(v) => Some(v),
            AttributeValue::Double(v) => Some(v as f32),
            AttributeValue::Short(v) => Some(v as f32),
            _ => None,
        });

    let dim_refs: Vec<&str> = dim_names.iter().map(|s| s.as_str()).collect();
    let mut new_var = file.add_variable::<f32>(var_name, &dim_refs)?;

    if let Some(fv) = fill_value {
        new_var.put_attribute("_FillValue", fv)?;
    }

    new_var.put(data.view(), ..)?;

    // Copy remaining attributes excluding _FillValue succinctly
    for attr in orig_var.attributes().filter(|a| a.name() != "_FillValue") {
        match attr.value()? {
            AttributeValue::Str(val) => {
                new_var.put_attribute(attr.name(), val)?;
            }
            AttributeValue::Strs(vals) => {
                new_var.put_attribute(attr.name(), vals)?;
            }
            AttributeValue::Float(val) => {
                new_var.put_attribute(attr.name(), val)?;
            }
            AttributeValue::Floats(vals) => {
                new_var.put_attribute(attr.name(), vals)?;
            }
            AttributeValue::Double(val) => {
                new_var.put_attribute(attr.name(), val)?;
            }
            AttributeValue::Doubles(vals) => {
                new_var.put_attribute(attr.name(), vals)?;
            }
            AttributeValue::Int(val) => {
                new_var.put_attribute(attr.name(), val)?;
            }
            AttributeValue::Ints(vals) => {
                new_var.put_attribute(attr.name(), vals)?;
            }
            AttributeValue::Short(val) => {
                new_var.put_attribute(attr.name(), val)?;
            }
            AttributeValue::Shorts(vals) => {
                new_var.put_attribute(attr.name(), vals)?;
            }
            _ => {
                println!(" Skipped unsupported attribute type for '{}'", attr.name());
            }
        }
    }

    // Add history attribute
    file.add_attribute(
        "history",
        format!("Created by RuNeVis on {}", Utc::now().to_rfc3339()),
    )?;

    Ok(())
}

/// Writes computed minimum to a new NetCDF file with attributes copied.
pub fn write_min_to_netcdf(
    data: &ArrayD<f32>,
    dim_names: &[String],
    var_name: &str,
    original_var_name: &str,
    input_file: &File,
    output_path: &Path,
) -> Result<(), Box<dyn Error>> {
    if output_path.exists() {
        fs::remove_file(output_path)?;
    }

    let mut file = create(output_path)?;

    // Define dimensions
    for (dim_name, &dim_len) in dim_names.iter().zip(data.shape()) {
        file.add_dimension(dim_name, dim_len)?;
    }

    // Extract `_FillValue`
    let orig_var = input_file
        .variable(original_var_name)
        .ok_or("Original variable not found")?;

    let fill_value = orig_var
        .attribute("_FillValue")
        .and_then(|attr| match attr.value().ok()? {
            AttributeValue::Float(v) => Some(v),
            AttributeValue::Double(v) => Some(v as f32),
            AttributeValue::Short(v) => Some(v as f32),
            _ => None,
        });

    let dim_refs: Vec<&str> = dim_names.iter().map(|s| s.as_str()).collect();
    let mut new_var = file.add_variable::<f32>(var_name, &dim_refs)?;

    if let Some(fv) = fill_value {
        new_var.put_attribute("_FillValue", fv)?;
    }

    new_var.put(data.view(), ..)?;

    // Copy remaining attributes excluding _FillValue succinctly
    for attr in orig_var.attributes().filter(|a| a.name() != "_FillValue") {
        match attr.value()? {
            AttributeValue::Str(val) => {
                new_var.put_attribute(attr.name(), val)?;
            }
            AttributeValue::Strs(vals) => {
                new_var.put_attribute(attr.name(), vals)?;
            }
            AttributeValue::Float(val) => {
                new_var.put_attribute(attr.name(), val)?;
            }
            AttributeValue::Floats(vals) => {
                new_var.put_attribute(attr.name(), vals)?;
            }
            AttributeValue::Double(val) => {
                new_var.put_attribute(attr.name(), val)?;
            }
            AttributeValue::Doubles(vals) => {
                new_var.put_attribute(attr.name(), vals)?;
            }
            AttributeValue::Int(val) => {
                new_var.put_attribute(attr.name(), val)?;
            }
            AttributeValue::Ints(vals) => {
                new_var.put_attribute(attr.name(), vals)?;
            }
            AttributeValue::Short(val) => {
                new_var.put_attribute(attr.name(), val)?;
            }
            AttributeValue::Shorts(vals) => {
                new_var.put_attribute(attr.name(), vals)?;
            }
            _ => {
                println!("⚠ Skipped unsupported attribute type for '{}'", attr.name());
            }
        }
    }

    // Add history attribute
    file.add_attribute(
        "history",
        format!("Created by RuNeVis on {}", Utc::now().to_rfc3339()),
    )?;

    Ok(())
}

/// Writes computed maximum to a new NetCDF file with attributes copied.
pub fn write_max_to_netcdf(
    data: &ArrayD<f32>,
    dim_names: &[String],
    var_name: &str,
    original_var_name: &str,
    input_file: &File,
    output_path: &Path,
) -> Result<(), Box<dyn Error>> {
    if output_path.exists() {
        fs::remove_file(output_path)?;
    }

    let mut file = create(output_path)?;

    // Define dimensions
    for (dim_name, &dim_len) in dim_names.iter().zip(data.shape()) {
        file.add_dimension(dim_name, dim_len)?;
    }

    // Extract `_FillValue`
    let orig_var = input_file
        .variable(original_var_name)
        .ok_or("Original variable not found")?;

    let fill_value = orig_var
        .attribute("_FillValue")
        .and_then(|attr| match attr.value().ok()? {
            AttributeValue::Float(v) => Some(v),
            AttributeValue::Double(v) => Some(v as f32),
            AttributeValue::Short(v) => Some(v as f32),
            _ => None,
        });

    let dim_refs: Vec<&str> = dim_names.iter().map(|s| s.as_str()).collect();
    let mut new_var = file.add_variable::<f32>(var_name, &dim_refs)?;

    if let Some(fv) = fill_value {
        new_var.put_attribute("_FillValue", fv)?;
    }

    new_var.put(data.view(), ..)?;

    // Copy remaining attributes excluding _FillValue succinctly
    for attr in orig_var.attributes().filter(|a| a.name() != "_FillValue") {
        match attr.value()? {
            AttributeValue::Str(val) => {
                new_var.put_attribute(attr.name(), val)?;
            }
            AttributeValue::Strs(vals) => {
                new_var.put_attribute(attr.name(), vals)?;
            }
            AttributeValue::Float(val) => {
                new_var.put_attribute(attr.name(), val)?;
            }
            AttributeValue::Floats(vals) => {
                new_var.put_attribute(attr.name(), vals)?;
            }
            AttributeValue::Double(val) => {
                new_var.put_attribute(attr.name(), val)?;
            }
            AttributeValue::Doubles(vals) => {
                new_var.put_attribute(attr.name(), vals)?;
            }
            AttributeValue::Int(val) => {
                new_var.put_attribute(attr.name(), val)?;
            }
            AttributeValue::Ints(vals) => {
                new_var.put_attribute(attr.name(), vals)?;
            }
            AttributeValue::Short(val) => {
                new_var.put_attribute(attr.name(), val)?;
            }
            AttributeValue::Shorts(vals) => {
                new_var.put_attribute(attr.name(), vals)?;
            }
            _ => {
                println!("⚠ Skipped unsupported attribute type for '{}'", attr.name());
            }
        }
    }

    // Add history attribute
    file.add_attribute(
        "history",
        format!("Created by RuNeVis on {}", Utc::now().to_rfc3339()),
    )?;

    Ok(())
}

/// Describes a specific variable showing its data type, shape, and all attributes.
pub fn describe_variable(file: &File, var_name: &str) -> Result<(), Box<dyn Error>> {
    let var = file.variable(var_name).ok_or("Variable not found")?;

    println!("\n Variable Description: {}", var_name);
    println!("={}", "=".repeat(var_name.len() + 25));

    // Display data type
    let data_type = format!("{:?}", var.vartype()).to_lowercase();
    println!(" Data type: {}", data_type);

    // Display dimensions and shape
    let dims: Vec<String> = var
        .dimensions()
        .iter()
        .map(|d| d.name().to_string())
        .collect();

    let shape: Vec<usize> = var.dimensions().iter().map(|dim| dim.len()).collect();

    if dims.is_empty() {
        println!(" Dimensions: (scalar)");
        println!(" Shape: ()");
    } else {
        println!(" Dimensions: [{}]", dims.join(", "));
        println!(
            " Shape: ({})",
            shape
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(" × ")
        );

        // Show dimension details
        println!("\n Dimension Details:");
        for dim in var.dimensions() {
            let length_info = if dim.is_unlimited() {
                format!("{} (unlimited)", dim.len())
            } else {
                dim.len().to_string()
            };
            println!("    {} = {}", dim.name(), length_info);
        }
    }

    // Display attributes
    let attributes: Vec<_> = var.attributes().collect();
    if attributes.is_empty() {
        println!("\n  Attributes: (none)");
    } else {
        println!("\n  Attributes:");
        for attr in attributes {
            match attr.value() {
                Ok(value) => match value {
                    AttributeValue::Str(s) => println!("   • {}: \"{}\"", attr.name(), s),
                    AttributeValue::Strs(ss) => println!("   • {}: {:?}", attr.name(), ss),
                    AttributeValue::Float(f) => println!("   • {}: {}", attr.name(), f),
                    AttributeValue::Floats(fs) => println!("   • {}: {:?}", attr.name(), fs),
                    AttributeValue::Double(d) => println!("   • {}: {}", attr.name(), d),
                    AttributeValue::Doubles(ds) => println!("   • {}: {:?}", attr.name(), ds),
                    AttributeValue::Int(i) => println!("   • {}: {}", attr.name(), i),
                    AttributeValue::Ints(is) => println!("   • {}: {:?}", attr.name(), is),
                    AttributeValue::Short(s) => println!("   • {}: {}", attr.name(), s),
                    AttributeValue::Shorts(ss) => println!("   • {}: {:?}", attr.name(), ss),
                    AttributeValue::Uchar(u) => println!("   • {}: {}", attr.name(), u),
                    AttributeValue::Uchars(us) => println!("   • {}: {:?}", attr.name(), us),
                    AttributeValue::Ushort(u) => println!("   • {}: {}", attr.name(), u),
                    AttributeValue::Ushorts(us) => println!("   • {}: {:?}", attr.name(), us),
                    AttributeValue::Uint(u) => println!("   • {}: {}", attr.name(), u),
                    AttributeValue::Uints(us) => println!("   • {}: {:?}", attr.name(), us),
                    _ => println!("   • {}: {:?}", attr.name(), value),
                },
                Err(e) => println!("   • {}: (error reading value: {})", attr.name(), e),
            }
        }
    }

    // Calculate and show total size
    let total_elements: usize = shape.iter().product();
    // Estimate element size based on data type string
    let data_type_str = format!("{:?}", var.vartype()).to_lowercase();
    let element_size = if data_type_str.contains("double") {
        8
    } else if data_type_str.contains("float") {
        4
    } else if data_type_str.contains("int") || data_type_str.contains("uint") {
        4
    } else if data_type_str.contains("short") || data_type_str.contains("ushort") {
        2
    } else {
        4 // Default size
    };

    let total_bytes = total_elements * element_size;

    println!("\n Storage Information:");
    println!("    Total elements: {}", total_elements);
    println!("    Element size: {} bytes", element_size);

    if total_bytes < 1024 {
        println!("    Total size: {} bytes", total_bytes);
    } else if total_bytes < 1024 * 1024 {
        println!("    Total size: {:.2} KB", total_bytes as f64 / 1024.0);
    } else if total_bytes < 1024 * 1024 * 1024 {
        println!(
            "    Total size: {:.2} MB",
            total_bytes as f64 / (1024.0 * 1024.0)
        );
    } else {
        println!(
            "    Total size: {:.2} GB",
            total_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
        );
    }

    println!(
        "\n Tip: Use --mean {}:<dimension> to compute means over specific dimensions",
        var_name
    );

    Ok(())
}

/// Extracts a slice of data from a variable based on the provided slice specification.
pub fn extract_slice(file: &File, slice_spec: SliceSpec) -> Result<(), Box<dyn Error>> {
    let var = file
        .variable(&slice_spec.variable)
        .ok_or("Variable not found")?;

    // Get variable dimensions
    let var_dims: Vec<String> = var
        .dimensions()
        .iter()
        .map(|d| d.name().to_string())
        .collect();
    let var_shape: Vec<usize> = var.dimensions().iter().map(|d| d.len()).collect();

    println!("\n Slicing Variable: {}", slice_spec.variable);
    println!("==============================");
    println!(
        " Original shape: ({})",
        var_shape
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(" × ")
    );
    println!(" Original dimensions: [{}]", var_dims.join(", "));

    // Build slice ranges for each dimension
    let mut slice_ranges = Vec::new();
    let mut slice_info = Vec::new();

    for (dim_idx, dim_name) in var_dims.iter().enumerate() {
        let dim_size = var_shape[dim_idx];

        // Find if this dimension has a specific slice
        let slice_range = if dim_idx == 0 && slice_spec.slices[0].dimension == "__first_dim__" {
            // First dimension slice from the variable:start:end format
            let start = slice_spec.slices[0].start;
            let end = slice_spec.slices[0].end;

            if start >= dim_size || end > dim_size || start >= end {
                return Err(format!(
                    "Invalid slice range for dimension '{}': {}:{} (dimension size: {})",
                    dim_name, start, end, dim_size
                )
                .into());
            }

            (start, end)
        } else {
            // Check if there's a named dimension slice
            if let Some(dim_slice) = slice_spec.slices.iter().find(|s| s.dimension == *dim_name) {
                let start = dim_slice.start;
                let end = dim_slice.end;

                if start >= dim_size || end > dim_size || start >= end {
                    return Err(format!(
                        "Invalid slice range for dimension '{}': {}:{} (dimension size: {})",
                        dim_name, start, end, dim_size
                    )
                    .into());
                }

                (start, end)
            } else {
                // No slice specified for this dimension, take all
                (0, dim_size)
            }
        };

        slice_ranges.push(slice_range);
        slice_info.push(format!(
            "{}: {}:{} (length: {})",
            dim_name,
            slice_range.0,
            slice_range.1,
            slice_range.1 - slice_range.0
        ));
    }

    println!("\n Slice specification:");
    for info in &slice_info {
        println!("    {}", info);
    }

    // Extract the slice of data
    println!("\n⚡ Extracting slice...");

    // Build the slice indices for netcdf library
    let mut slice_args = Vec::new();
    for &(start, end) in &slice_ranges {
        slice_args.push(start..end);
    }

    // Get the sliced data as f32
    let sliced_data: Vec<f32> = match slice_args.len() {
        1 => var.get_values::<f32, _>(slice_args[0].clone())?,
        2 => var.get_values::<f32, _>((slice_args[0].clone(), slice_args[1].clone()))?,
        3 => var.get_values::<f32, _>((
            slice_args[0].clone(),
            slice_args[1].clone(),
            slice_args[2].clone(),
        ))?,
        4 => var.get_values::<f32, _>((
            slice_args[0].clone(),
            slice_args[1].clone(),
            slice_args[2].clone(),
            slice_args[3].clone(),
        ))?,
        _ => return Err("Unsupported number of dimensions for slicing (max 4)".into()),
    };

    let sliced_shape: Vec<usize> = slice_ranges
        .iter()
        .map(|(start, end)| end - start)
        .collect();

    println!(" Successfully extracted slice!");
    println!(
        " Sliced shape: ({})",
        sliced_shape
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(" × ")
    );
    println!(" Total elements: {}", sliced_data.len());

    // Show basic statistics on the sliced data
    if !sliced_data.is_empty() {
        let valid_data: Vec<f32> = sliced_data
            .iter()
            .filter(|&&x| x.is_finite())
            .cloned()
            .collect();

        if !valid_data.is_empty() {
            let min = valid_data.iter().cloned().fold(f32::INFINITY, f32::min);
            let max = valid_data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let mean = valid_data.iter().sum::<f32>() / valid_data.len() as f32;

            println!("\n Slice Statistics:");
            println!("    Min: {:.2}", min);
            println!("    Max: {:.2}", max);
            println!("    Mean: {:.2}", mean);
            println!(
                "    Valid elements: {} / {}",
                valid_data.len(),
                sliced_data.len()
            );
        } else {
            println!("\n  No valid (finite) data found in slice");
        }
    }

    // Show first few values if the slice is small enough
    if sliced_data.len() <= 20 {
        println!("\n Slice data:");
        for (i, value) in sliced_data.iter().enumerate() {
            println!("   [{}]: {:.4}", i, value);
        }
    } else {
        println!("\n First 10 values of slice:");
        for (i, value) in sliced_data.iter().take(10).enumerate() {
            println!("   [{}]: {:.4}", i, value);
        }
        println!("   ... ({} more values)", sliced_data.len() - 10);
    }

    println!("\n Tip: Use --slice var:start:end,dim1:start1:end1,dim2:start2:end2 for multi-dimensional slicing");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::{Array1, Array2, Array3};
    use std::f64;

    // Mock Variable struct for testing
    #[derive(Debug)]
    struct MockDimension {
        name: String,
        len: usize,
    }

    impl MockDimension {
        fn new(name: &str, len: usize) -> Self {
            Self {
                name: name.to_string(),
                len,
            }
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn len(&self) -> usize {
            self.len
        }
    }

    struct MockVariable {
        dimensions: Vec<MockDimension>,
        data: Vec<f32>,
    }

    impl MockVariable {
        fn new(dimensions: Vec<MockDimension>, data: Vec<f32>) -> Self {
            Self { dimensions, data }
        }

        fn dimensions(&self) -> &[MockDimension] {
            &self.dimensions
        }

        fn get_values<T, R>(&self, _range: R) -> Result<Vec<T>, Box<dyn Error>>
        where
            T: Clone + From<f32>,
        {
            Ok(self.data.iter().map(|&x| T::from(x)).collect())
        }
    }

    // Helper function to create test data with synthetic ndarray
    fn create_test_array_2d() -> Array2<f64> {
        // Create a 3x4 array with known values for testing
        Array2::from_shape_vec(
            (3, 4),
            vec![
                1.0, 2.0, 3.0, 4.0, // Row 0
                5.0, 6.0, 7.0, 8.0, // Row 1
                9.0, 10.0, 11.0, 12.0, // Row 2
            ],
        )
        .unwrap()
    }

    fn create_test_array_3d() -> Array3<f64> {
        // Create a 2x3x2 array
        Array3::from_shape_vec(
            (2, 3, 2),
            vec![
                1.0, 2.0, // [0,0,:]
                3.0, 4.0, // [0,1,:]
                5.0, 6.0, // [0,2,:]
                7.0, 8.0, // [1,0,:]
                9.0, 10.0, // [1,1,:]
                11.0, 12.0, // [1,2,:]
            ],
        )
        .unwrap()
    }

    #[test]
    fn test_reduce_min_2d_axis_0() {
        // Test minimum reduction along axis 0 (rows)
        let data = create_test_array_2d();
        let result = data.fold_axis(ndarray::Axis(0), f64::INFINITY, |&acc, &x| acc.min(x));

        // Expected: min of each column
        // Column 0: min(1, 5, 9) = 1
        // Column 1: min(2, 6, 10) = 2
        // Column 2: min(3, 7, 11) = 3
        // Column 3: min(4, 8, 12) = 4
        let expected = Array1::from(vec![1.0, 2.0, 3.0, 4.0]);

        assert_eq!(result.shape(), expected.shape());
        for (actual, expected) in result.iter().zip(expected.iter()) {
            assert!((actual - expected).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_reduce_min_2d_axis_1() {
        // Test minimum reduction along axis 1 (columns)
        let data = create_test_array_2d();
        let result = data.fold_axis(ndarray::Axis(1), f64::INFINITY, |&acc, &x| acc.min(x));

        // Expected: min of each row
        // Row 0: min(1, 2, 3, 4) = 1
        // Row 1: min(5, 6, 7, 8) = 5
        // Row 2: min(9, 10, 11, 12) = 9
        let expected = Array1::from(vec![1.0, 5.0, 9.0]);

        assert_eq!(result.shape(), expected.shape());
        for (actual, expected) in result.iter().zip(expected.iter()) {
            assert!((actual - expected).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_reduce_max_2d_axis_0() {
        // Test maximum reduction along axis 0 (rows)
        let data = create_test_array_2d();
        let result = data.fold_axis(ndarray::Axis(0), f64::NEG_INFINITY, |&acc, &x| acc.max(x));

        // Expected: max of each column
        // Column 0: max(1, 5, 9) = 9
        // Column 1: max(2, 6, 10) = 10
        // Column 2: max(3, 7, 11) = 11
        // Column 3: max(4, 8, 12) = 12
        let expected = Array1::from(vec![9.0, 10.0, 11.0, 12.0]);

        assert_eq!(result.shape(), expected.shape());
        for (actual, expected) in result.iter().zip(expected.iter()) {
            assert!((actual - expected).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_reduce_max_2d_axis_1() {
        // Test maximum reduction along axis 1 (columns)
        let data = create_test_array_2d();
        let result = data.fold_axis(ndarray::Axis(1), f64::NEG_INFINITY, |&acc, &x| acc.max(x));

        // Expected: max of each row
        // Row 0: max(1, 2, 3, 4) = 4
        // Row 1: max(5, 6, 7, 8) = 8
        // Row 2: max(9, 10, 11, 12) = 12
        let expected = Array1::from(vec![4.0, 8.0, 12.0]);

        assert_eq!(result.shape(), expected.shape());
        for (actual, expected) in result.iter().zip(expected.iter()) {
            assert!((actual - expected).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_reduce_min_3d() {
        // Test minimum reduction on 3D array along different axes
        let data = create_test_array_3d();

        // Test axis 0 reduction (2x3x2 -> 3x2)
        let result_axis_0 = data.fold_axis(ndarray::Axis(0), f64::INFINITY, |&acc, &x| acc.min(x));
        assert_eq!(result_axis_0.shape(), &[3, 2]);

        // Test axis 1 reduction (2x3x2 -> 2x2)
        let result_axis_1 = data.fold_axis(ndarray::Axis(1), f64::INFINITY, |&acc, &x| acc.min(x));
        assert_eq!(result_axis_1.shape(), &[2, 2]);

        // Test axis 2 reduction (2x3x2 -> 2x3)
        let result_axis_2 = data.fold_axis(ndarray::Axis(2), f64::INFINITY, |&acc, &x| acc.min(x));
        assert_eq!(result_axis_2.shape(), &[2, 3]);
    }

    #[test]
    fn test_reduce_max_3d() {
        // Test maximum reduction on 3D array along different axes
        let data = create_test_array_3d();

        // Test axis 0 reduction (2x3x2 -> 3x2)
        let result_axis_0 =
            data.fold_axis(ndarray::Axis(0), f64::NEG_INFINITY, |&acc, &x| acc.max(x));
        assert_eq!(result_axis_0.shape(), &[3, 2]);

        // Test axis 1 reduction (2x3x2 -> 2x2)
        let result_axis_1 =
            data.fold_axis(ndarray::Axis(1), f64::NEG_INFINITY, |&acc, &x| acc.max(x));
        assert_eq!(result_axis_1.shape(), &[2, 2]);

        // Test axis 2 reduction (2x3x2 -> 2x3)
        let result_axis_2 =
            data.fold_axis(ndarray::Axis(2), f64::NEG_INFINITY, |&acc, &x| acc.max(x));
        assert_eq!(result_axis_2.shape(), &[2, 3]);
    }

    #[test]
    fn test_nan_handling() {
        // Test handling of NaN values
        let data =
            Array2::from_shape_vec((2, 3), vec![1.0, f64::NAN, 3.0, 4.0, 5.0, f64::NAN]).unwrap();

        // Test min reduction with NaN handling
        let result_min = data.fold_axis(ndarray::Axis(0), f64::INFINITY, |&acc, &x| {
            if x.is_finite() {
                acc.min(x)
            } else {
                acc
            }
        });

        // Should ignore NaN values
        assert_eq!(result_min[0], 1.0); // min(1.0, 4.0) = 1.0
        assert_eq!(result_min[1], 5.0); // min(NaN, 5.0) = 5.0 (ignoring NaN)
        assert_eq!(result_min[2], 3.0); // min(3.0, NaN) = 3.0 (ignoring NaN)

        // Test max reduction with NaN handling
        let result_max = data.fold_axis(ndarray::Axis(0), f64::NEG_INFINITY, |&acc, &x| {
            if x.is_finite() {
                acc.max(x)
            } else {
                acc
            }
        });

        // Should ignore NaN values
        assert_eq!(result_max[0], 4.0); // max(1.0, 4.0) = 4.0
        assert_eq!(result_max[1], 5.0); // max(NaN, 5.0) = 5.0 (ignoring NaN)
        assert_eq!(result_max[2], 3.0); // max(3.0, NaN) = 3.0 (ignoring NaN)
    }

    #[test]
    fn test_infinity_handling() {
        // Test handling of infinity values
        let data = Array2::from_shape_vec(
            (2, 3),
            vec![1.0, f64::INFINITY, 3.0, 4.0, 5.0, f64::NEG_INFINITY],
        )
        .unwrap();

        // Test min reduction (infinity should be ignored by our logic)
        let result_min = data.fold_axis(ndarray::Axis(0), f64::INFINITY, |&acc, &x| {
            if x.is_finite() {
                acc.min(x)
            } else {
                acc
            }
        });

        assert_eq!(result_min[0], 1.0); // min(1.0, 4.0) = 1.0
        assert_eq!(result_min[1], 5.0); // min(inf, 5.0) = 5.0 (ignoring inf)
        assert_eq!(result_min[2], 3.0); // min(3.0, -inf) = 3.0 (ignoring -inf)
    }

    #[test]
    fn test_edge_case_single_element() {
        // Test with single element array
        let data = Array2::from_shape_vec((1, 1), vec![42.0]).unwrap();

        let result_min = data.fold_axis(ndarray::Axis(0), f64::INFINITY, |&acc, &x| acc.min(x));
        let result_max = data.fold_axis(ndarray::Axis(0), f64::NEG_INFINITY, |&acc, &x| acc.max(x));

        assert_eq!(result_min.shape(), &[1]);
        assert_eq!(result_max.shape(), &[1]);
        assert_eq!(result_min[0], 42.0);
        assert_eq!(result_max[0], 42.0);
    }

    #[test]
    fn test_edge_case_all_same_values() {
        // Test with array where all values are the same
        let data = Array2::from_shape_vec((3, 3), vec![7.0; 9]).unwrap();

        let result_min = data.fold_axis(ndarray::Axis(0), f64::INFINITY, |&acc, &x| acc.min(x));
        let result_max = data.fold_axis(ndarray::Axis(0), f64::NEG_INFINITY, |&acc, &x| acc.max(x));

        assert_eq!(result_min.shape(), &[3]);
        assert_eq!(result_max.shape(), &[3]);

        for val in result_min.iter() {
            assert_eq!(*val, 7.0);
        }

        for val in result_max.iter() {
            assert_eq!(*val, 7.0);
        }
    }

    #[test]
    fn test_comprehensive_reduction_behavior() {
        // Test the actual reduction behavior with mixed positive/negative values
        let data = Array2::from_shape_vec((2, 4), vec![-5.0, -2.0, 1.0, 4.0, 3.0, -1.0, -3.0, 2.0])
            .unwrap();

        // Test min along axis 0 (columns)
        let result_min_axis_0 =
            data.fold_axis(ndarray::Axis(0), f64::INFINITY, |&acc, &x| acc.min(x));
        let expected_min = vec![-5.0, -2.0, -3.0, 2.0];

        for (actual, expected) in result_min_axis_0.iter().zip(expected_min.iter()) {
            assert!((actual - expected).abs() < f64::EPSILON);
        }

        // Test max along axis 0 (columns)
        let result_max_axis_0 =
            data.fold_axis(ndarray::Axis(0), f64::NEG_INFINITY, |&acc, &x| acc.max(x));
        let expected_max = vec![3.0, -1.0, 1.0, 4.0];

        for (actual, expected) in result_max_axis_0.iter().zip(expected_max.iter()) {
            assert!((actual - expected).abs() < f64::EPSILON);
        }

        // Test min along axis 1 (rows)
        let result_min_axis_1 =
            data.fold_axis(ndarray::Axis(1), f64::INFINITY, |&acc, &x| acc.min(x));
        let expected_min_rows = vec![-5.0, -3.0];

        for (actual, expected) in result_min_axis_1.iter().zip(expected_min_rows.iter()) {
            assert!((actual - expected).abs() < f64::EPSILON);
        }

        // Test max along axis 1 (rows)
        let result_max_axis_1 =
            data.fold_axis(ndarray::Axis(1), f64::NEG_INFINITY, |&acc, &x| acc.max(x));
        let expected_max_rows = vec![4.0, 3.0];

        for (actual, expected) in result_max_axis_1.iter().zip(expected_max_rows.iter()) {
            assert!((actual - expected).abs() < f64::EPSILON);
        }
    }
}
