//! Defines command-line interface options using `clap` for the RuNeVis application.

use clap::Parser;
use std::path::PathBuf;

/// A CLI tool for inspecting NetCDF files
#[derive(Parser, Debug)]
#[command(
    author = "Sam Green",
    version = "1.1.0",
    name = "RuNeVis",
    about = "App for working with NetCDF files"
)]
pub struct Args {
    /// Path to the NetCDF file
    #[arg(short, long)]
    pub file: PathBuf,

    /// Compute the mean for a variable over a specific dimension, formatted as <var>:<dim>
    #[arg(long, value_parser = parse_mean_arg)]
    pub mean: Option<(String, String)>,

    /// Compute the sum for a variable over a specific dimension, formatted as <var>:<dim>
    #[arg(long, value_parser = parse_mean_arg)]
    pub sum: Option<(String, String)>,

    /// Compute the minimum for a variable over a specific dimension, formatted as <var>:<dim>
    #[arg(long, value_parser = parse_mean_arg)]
    pub min: Option<(String, String)>,

    /// Compute the maximum for a variable over a specific dimension, formatted as <var>:<dim>
    #[arg(long, value_parser = parse_mean_arg)]
    pub max: Option<(String, String)>,

    /// Path to save result as NetCDF. If not set, prints to terminal.
    #[arg(long)]
    pub output_netcdf: Option<PathBuf>,

    /// Enable verbose output.
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// Number of threads to use for parallel processing. Defaults to number of CPU cores.
    #[arg(short = 't', long)]
    pub threads: Option<usize>,

    /// List all variables and dimensions in the NetCDF file
    #[arg(long)]
    pub list_vars: bool,

    /// Describe a specific variable (data type, shape, and attributes)
    #[arg(long)]
    pub describe: Option<String>,

    /// Compute quick statistics (min/mean/max/std) for a variable
    #[arg(long)]
    pub summary: Option<String>,

    /// Extract a slice of data from a variable, format: var:start:end,dim:start:end
    #[arg(long, value_parser = parse_slice_arg)]
    pub slice: Option<SliceSpec>,
}

#[derive(Debug, Clone)]
pub struct SliceSpec {
    pub variable: String,
    pub slices: Vec<DimSlice>,
}

#[derive(Debug, Clone)]
pub struct DimSlice {
    pub dimension: String,
    pub start: usize,
    pub end: usize,
}

fn parse_mean_arg(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.split(':').collect();
    match parts.as_slice() {
        [var, dim] => Ok((var.to_string(), dim.to_string())),
        _ => Err("Invalid format: Expected '<variable>:<dimension>'.".to_string()),
    }
}

fn parse_slice_arg(s: &str) -> Result<SliceSpec, String> {
    // Parse format: "var:start:end,dim:start:end" or "var:start:end"
    let main_parts: Vec<&str> = s.split(',').collect();

    if main_parts.is_empty() {
        return Err("Invalid slice format".to_string());
    }

    // First part should be variable:start:end
    let var_parts: Vec<&str> = main_parts[0].split(':').collect();
    if var_parts.len() != 3 {
        return Err(
            "Invalid format: Expected 'variable:start:end,dimension:start:end'".to_string(),
        );
    }

    let variable = var_parts[0].to_string();
    let var_start = var_parts[1]
        .parse::<usize>()
        .map_err(|_| "Invalid start index for variable".to_string())?;
    let var_end = var_parts[2]
        .parse::<usize>()
        .map_err(|_| "Invalid end index for variable".to_string())?;

    let mut slices = vec![DimSlice {
        dimension: "__first_dim__".to_string(), // Will be resolved later
        start: var_start,
        end: var_end,
    }];

    // Parse additional dimension slices
    for part in &main_parts[1..] {
        let dim_parts: Vec<&str> = part.split(':').collect();
        if dim_parts.len() != 3 {
            return Err(
                "Invalid dimension slice format: Expected 'dimension:start:end'".to_string(),
            );
        }

        let dimension = dim_parts[0].to_string();
        let start = dim_parts[1]
            .parse::<usize>()
            .map_err(|_| format!("Invalid start index for dimension '{}'", dimension))?;
        let end = dim_parts[2]
            .parse::<usize>()
            .map_err(|_| format!("Invalid end index for dimension '{}'", dimension))?;

        slices.push(DimSlice {
            dimension,
            start,
            end,
        });
    }

    Ok(SliceSpec { variable, slices })
}
