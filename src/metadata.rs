//! NetCDF metadata inspection and variable description functionality
//!
//! This module provides functions for examining NetCDF file structure,
//! listing variables and dimensions, and describing variable properties.

use crate::errors::{Result, RuNeVisError};
use netcdf::{AttributeValue, File};
use std::collections::HashMap;

/// Structured metadata for a NetCDF variable
#[derive(Debug, Clone)]
pub struct VariableMetadata {
    pub name: String,
    pub data_type: String,
    pub dimensions: Vec<DimensionInfo>,
    pub attributes: HashMap<String, AttributeValue>,
    pub total_elements: usize,
    pub estimated_size_bytes: usize,
}

/// Information about a dimension
#[derive(Debug, Clone)]
pub struct DimensionInfo {
    pub name: String,
    pub length: usize,
    pub is_unlimited: bool,
}

/// Prints global attributes and variables of a NetCDF file.
pub fn print_metadata(file: &File) -> Result<()> {
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
pub fn compute_variable_summary(file: &File, var_name: &str) -> Result<()> {
    let var = file
        .variable(var_name)
        .ok_or_else(|| RuNeVisError::VariableNotFound {
            var: var_name.to_string(),
        })?;

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
pub fn list_variables_and_dimensions(file: &File) -> Result<()> {
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

/// Describes a specific variable showing its data type, shape, and all attributes.
pub fn describe_variable(file: &File, var_name: &str) -> Result<()> {
    let var = file
        .variable(var_name)
        .ok_or_else(|| RuNeVisError::VariableNotFound {
            var: var_name.to_string(),
        })?;

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

/// Get structured metadata for a variable
pub fn get_variable_metadata(file: &File, var_name: &str) -> Result<VariableMetadata> {
    let var = file
        .variable(var_name)
        .ok_or_else(|| RuNeVisError::VariableNotFound {
            var: var_name.to_string(),
        })?;

    let data_type = format!("{:?}", var.vartype()).to_lowercase();

    let dimensions: Vec<DimensionInfo> = var
        .dimensions()
        .iter()
        .map(|d| DimensionInfo {
            name: d.name().to_string(),
            length: d.len(),
            is_unlimited: d.is_unlimited(),
        })
        .collect();

    let mut attributes = HashMap::new();
    for attr in var.attributes() {
        if let Ok(value) = attr.value() {
            attributes.insert(attr.name().to_string(), value);
        }
    }

    let total_elements: usize = dimensions.iter().map(|d| d.length).product();

    // Estimate element size based on data type
    let element_size = if data_type.contains("double") {
        8
    } else if data_type.contains("float") {
        4
    } else if data_type.contains("int") || data_type.contains("uint") {
        4
    } else if data_type.contains("short") || data_type.contains("ushort") {
        2
    } else {
        4 // Default size
    };

    let estimated_size_bytes = total_elements * element_size;

    Ok(VariableMetadata {
        name: var_name.to_string(),
        data_type,
        dimensions,
        attributes,
        total_elements,
        estimated_size_bytes,
    })
}
