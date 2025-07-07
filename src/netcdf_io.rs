//! NetCDF I/O operations including data slicing and file writing
//!
//! This module provides functions for extracting data slices from NetCDF files
//! and writing computed statistical results to new NetCDF files with proper
//! metadata preservation.

use crate::cli::SliceSpec;
use crate::errors::{Result, RuNeVisError};
use chrono::Utc;
use ndarray::ArrayD;
use netcdf::{create, AttributeValue, File};
use std::{fs, path::Path};

/// Unified NetCDF writer for statistical results
pub struct NetCDFWriter<'a> {
    input_file: &'a File,
    output_path: &'a Path,
}

impl<'a> NetCDFWriter<'a> {
    /// Create a new NetCDF writer
    pub fn new(input_file: &'a File, output_path: &'a Path) -> Self {
        Self {
            input_file,
            output_path,
        }
    }

    /// Write statistical result to NetCDF file
    pub fn write_result(
        &self,
        data: &ArrayD<f32>,
        dim_names: &[String],
        var_name: &str,
        original_var_name: &str,
    ) -> Result<()> {
        if self.output_path.exists() {
            fs::remove_file(self.output_path)?;
        }

        let mut file = create(self.output_path)?;

        // Define dimensions
        for (dim_name, &dim_len) in dim_names.iter().zip(data.shape()) {
            file.add_dimension(dim_name, dim_len)?;
        }

        // Extract `_FillValue` from original variable
        let orig_var = self
            .input_file
            .variable(original_var_name)
            .ok_or_else(|| RuNeVisError::VariableNotFound {
                var: original_var_name.to_string(),
            })?;

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

        // Copy remaining attributes excluding _FillValue
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
}

/// Writes computed mean to a new NetCDF file with attributes copied.
pub fn write_mean_to_netcdf(
    data: &ArrayD<f32>,
    dim_names: &[String],
    var_name: &str,
    original_var_name: &str,
    input_file: &File,
    output_path: &Path,
) -> Result<()> {
    let writer = NetCDFWriter::new(input_file, output_path);
    writer.write_result(data, dim_names, var_name, original_var_name)
}

/// Writes computed sum to a new NetCDF file with attributes copied.
pub fn write_sum_to_netcdf(
    data: &ArrayD<f32>,
    dim_names: &[String],
    var_name: &str,
    original_var_name: &str,
    input_file: &File,
    output_path: &Path,
) -> Result<()> {
    let writer = NetCDFWriter::new(input_file, output_path);
    writer.write_result(data, dim_names, var_name, original_var_name)
}

/// Writes computed minimum to a new NetCDF file with attributes copied.
pub fn write_min_to_netcdf(
    data: &ArrayD<f32>,
    dim_names: &[String],
    var_name: &str,
    original_var_name: &str,
    input_file: &File,
    output_path: &Path,
) -> Result<()> {
    let writer = NetCDFWriter::new(input_file, output_path);
    writer.write_result(data, dim_names, var_name, original_var_name)
}

/// Writes computed maximum to a new NetCDF file with attributes copied.
pub fn write_max_to_netcdf(
    data: &ArrayD<f32>,
    dim_names: &[String],
    var_name: &str,
    original_var_name: &str,
    input_file: &File,
    output_path: &Path,
) -> Result<()> {
    let writer = NetCDFWriter::new(input_file, output_path);
    writer.write_result(data, dim_names, var_name, original_var_name)
}

/// Extracts a slice of data from a variable based on the provided slice specification.
pub fn extract_slice(file: &File, slice_spec: SliceSpec) -> Result<()> {
    let var = file
        .variable(&slice_spec.variable)
        .ok_or_else(|| RuNeVisError::VariableNotFound {
            var: slice_spec.variable.clone(),
        })?;

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
                return Err(RuNeVisError::InvalidSlice {
                    message: format!(
                        "Invalid slice range for dimension '{}': {}:{} (dimension size: {})",
                        dim_name, start, end, dim_size
                    ),
                });
            }

            (start, end)
        } else {
            // Check if there's a named dimension slice
            if let Some(dim_slice) = slice_spec.slices.iter().find(|s| s.dimension == *dim_name) {
                let start = dim_slice.start;
                let end = dim_slice.end;

                if start >= dim_size || end > dim_size || start >= end {
                    return Err(RuNeVisError::InvalidSlice {
                        message: format!(
                            "Invalid slice range for dimension '{}': {}:{} (dimension size: {})",
                            dim_name, start, end, dim_size
                        ),
                    });
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
        _ => {
            return Err(RuNeVisError::InvalidSlice {
                message: "Unsupported number of dimensions for slicing (max 4)".to_string(),
            })
        }
    };

    let sliced_shape: Vec<usize> = slice_ranges
        .iter()
        .map(|(start, end)| end - start)
        .collect();

    println!("✅ Successfully extracted slice!");
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
            println!("\n⚠ No valid (finite) data found in slice");
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

    println!("\n💡 Tip: Use --slice var:start:end,dim1:start1:end1,dim2:start2:end2 for multi-dimensional slicing");

    Ok(())
}
