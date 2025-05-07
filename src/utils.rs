use netcdf::{File, create, AttributeValue};
use ndarray::{ArrayD, Axis, ArrayViewD};
use std::{
    path::Path,
    error::Error,
    fs,
    };
use chrono::Utc;

pub fn print_metadata(file: &File) -> Result<(), Box<dyn Error>> {
    // Print global attributes
    println!("\n===== Global Attributes =====");
    for attr in file.attributes() {
        println!("- {}: {:?}", attr.name(), attr.value()?);
    }

    // List variables
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

pub fn mean_over_dimension(
    file: &File,
    var_name: &str,
    dim_name: &str,
) -> Result<(ArrayD<f32>, Vec<String>, String), Box<dyn std::error::Error>> {
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
    let data_vec: Vec<f32> = var.get_values(..)?;
    let data = ArrayD::from_shape_vec(shape.clone(), data_vec)?;

    let mean_arr = data
        .mean_axis(Axis(axis_index))
        .ok_or("Failed to compute mean")?;
    let mean_array = mean_arr.into_dyn();

    let kept_dim_names: Vec<String> = dim_names
        .into_iter()
        .enumerate()
        .filter_map(|(i, name)| if i != axis_index { Some(name) } else { None })
        .collect();

    let new_var_name = format!("{var_name}_mean_over_{dim_name}");

    Ok((mean_array, kept_dim_names, new_var_name))
}

pub fn write_mean_to_netcdf(
    data: &ArrayD<f32>,
    dim_names: &[String],
    var_name: &str,
    original_var_name: &str,
    input_file: &File,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if output_path.exists() {
        fs::remove_file(output_path)?;
    }

    let mut file = create(output_path)?;

    let shape = data.shape();
    for (i, &len) in shape.iter().enumerate() {
        file.add_dimension(&dim_names[i], len)?;
    }

    let dim_refs: Vec<&str> = dim_names.iter().map(String::as_str).collect();

    // Step 0: Extract `_FillValue` from original variable
    let mut fill_value: Option<f32> = None;
    if let Some(orig_var) = input_file.variable(original_var_name) {
        for attr in orig_var.attributes() {
            if attr.name() == "_FillValue" {
                match attr.value()? {
                    AttributeValue::Float(v) => fill_value = Some(v),
                    AttributeValue::Double(v) => fill_value = Some(v as f32),
                    AttributeValue::Short(v) => fill_value = Some(v as f32),
                    _ => println!("⚠️ Unsupported _FillValue type"),
                }
            }
        }
    }

    // Step 1: Create the variable with or without fill_value
    let mut var = file.add_variable::<f32>(var_name, &dim_refs)?;

    // Apply _FillValue manually BEFORE any data is written
    if let Some(fv) = fill_value {
        println!("Setting _FillValue before writing: {}", fv);
        var.put_attribute("_FillValue", fv)?;
    }

    // Step 2: Write data after _FillValue is set
    let view: ArrayViewD<f32> = data.view();
    var.put(view, ..)?;

    // Step 3: Copy other attributes (excluding _FillValue)
    if let Some(orig_var) = input_file.variable(original_var_name) {
        for attr in orig_var.attributes() {
            let name = attr.name();
            if name == "_FillValue" {
                continue;
            }
            let value = attr.value()?;
            match value {
                AttributeValue::Str(val)      => { var.put_attribute(name, val)?; }
                AttributeValue::Strs(val)     => { var.put_attribute(name, val)?; }
                AttributeValue::Float(val)    => { var.put_attribute(name, val)?; }
                AttributeValue::Floats(val)   => { var.put_attribute(name, val)?; }
                AttributeValue::Double(val)   => { var.put_attribute(name, val)?; }
                AttributeValue::Doubles(val)  => { var.put_attribute(name, val)?; }
                AttributeValue::Int(val)      => { var.put_attribute(name, val)?; }
                AttributeValue::Ints(val)     => { var.put_attribute(name, val)?; }
                AttributeValue::Uint(val)     => { var.put_attribute(name, val)?; }
                AttributeValue::Uints(val)    => { var.put_attribute(name, val)?; }
                AttributeValue::Short(val)    => { var.put_attribute(name, val)?; }
                AttributeValue::Shorts(val)   => { var.put_attribute(name, val)?; }
                AttributeValue::Ushort(val)   => { var.put_attribute(name, val)?; }
                AttributeValue::Ushorts(val)  => { var.put_attribute(name, val)?; }
                AttributeValue::Schar(val)    => { var.put_attribute(name, val)?; }
                AttributeValue::Schars(val)   => { var.put_attribute(name, val)?; }
                AttributeValue::Uchar(val)    => { var.put_attribute(name, val)?; }
                AttributeValue::Uchars(val)   => { var.put_attribute(name, val)?; }
                AttributeValue::Longlong(val) => { var.put_attribute(name, val)?; }
                AttributeValue::Longlongs(val)=> { var.put_attribute(name, val)?; }
                AttributeValue::Ulonglong(val)=> { var.put_attribute(name, val)?; }
                AttributeValue::Ulonglongs(val)=>{ var.put_attribute(name, val)?; }
            }
        }
    }

    file.add_attribute(
        "history",
        format!("Created by RuNeVis on {}", Utc::now().to_rfc3339()),
    )?;

    Ok(())
}