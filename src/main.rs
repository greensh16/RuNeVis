//! Entry point for the RuNeVis application.
//! Handles CLI parsing, file loading, and dispatches operations like computing means or printing metadata.

use clap::Parser;
use netcdf::open;
use std::path::Path;

mod cli;
mod errors;
mod metadata;
mod netcdf_io;
mod parallel;
mod statistics;

use cli::Args;
use parallel::ParallelConfig;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments
    let args = Args::parse();

    // Initialize parallel processing configuration
    let parallel_config = ParallelConfig::new(args.threads);
    parallel_config
        .setup_global_pool()
        .map_err(|e| format!("Failed to setup parallel processing: {}", e))?;

    println!(
        r#"
------------------------------------------------------------------
        ______      _   _       _   _ _     
        | ___ \    | \ | |     | | | (_)    
        | |_/ /   _|  \| | ___ | | | |_ ___ 
        |    / | | | . ` |/ _ \| | | | / __|
        | |\ \ |_| | |\  |  __/\ \_/ / \__ \
        \_| \_\__,_\_| \_/\___| \___/|_|___/
                Rust-based NetCDF tool
------------------------------------------------------------------
        "#
    );

    // Open NetCDF file with error context
    let file = open(&args.file).map_err(|e| {
        format!(
            "Failed to open NetCDF file '{}': {}",
            args.file.display(),
            e
        )
    })?;

    println!(
        "✅ Successfully opened NetCDF file: {}",
        args.file.display()
    );

    // Handle different operations based on command-line options
    if args.list_vars {
        // List variables and dimensions in a clean format
        metadata::list_variables_and_dimensions(&file)
            .map_err(|e| format!("Failed listing variables and dimensions: {}", e))?;
    } else if let Some((var, dim)) = args.mean {
        // Compute mean over specified dimension
        let (result, dim_names, new_var_name) = statistics::mean_over_dimension(&file, &var, &dim)
            .map_err(|e| format!("Failed computing mean for variable '{}': {}", var, e))?;

        if let Some(output_path) = args.output_netcdf {
            let output_path = Path::new(&output_path);
            netcdf_io::write_mean_to_netcdf(
                &result,
                &dim_names,
                &new_var_name,
                &var,
                &file,
                output_path,
            )
            .map_err(|e| {
                format!(
                    "Failed writing to NetCDF '{}': {}",
                    output_path.display(),
                    e
                )
            })?;
            println!("✅ Result saved to {}", output_path.display());
        } else {
            println!("Computed mean array:\n{:#?}", result);
        }
    } else if let Some((var, dim)) = args.sum {
        // Compute sum over specified dimension
        let (result, dim_names, new_var_name) =
            statistics::sum_over_dimension(&file, &var, &dim)
                .map_err(|e| format!("Failed computing sum for variable '{}': {}", var, e))?;

        if let Some(output_path) = args.output_netcdf {
            let output_path = Path::new(&output_path);
            netcdf_io::write_sum_to_netcdf(
                &result,
                &dim_names,
                &new_var_name,
                &var,
                &file,
                output_path,
            )
            .map_err(|e| {
                format!(
                    "Failed writing to NetCDF '{}': {}",
                    output_path.display(),
                    e
                )
            })?;
            println!("✅ Result saved to {}", output_path.display());
        } else {
            println!("Computed sum array:\n{:#?}", result);
        }
    } else if let Some((var, dim)) = &args.min {
        // Compute minimum over specified dimension
        let (result, dim_names, new_var_name) = statistics::min_over_dimension(&file, var, dim)
            .map_err(|e| format!("Failed computing minimum for variable '{}': {}", var, e))?;

        if let Some(output_path) = &args.output_netcdf {
            let output_path = Path::new(output_path);
            netcdf_io::write_min_to_netcdf(
                &result,
                &dim_names,
                &new_var_name,
                var,
                &file,
                output_path,
            )
            .map_err(|e| {
                format!(
                    "Failed writing to NetCDF '{}': {}",
                    output_path.display(),
                    e
                )
            })?;
            println!("✅ Result saved to {}", output_path.display());
        } else {
            println!("Computed minimum array:\n{:#?}", result);
        }
    } else if let Some((var, dim)) = &args.max {
        // Compute maximum over specified dimension
        let (result, dim_names, new_var_name) = statistics::max_over_dimension(&file, var, dim)
            .map_err(|e| format!("Failed computing maximum for variable '{}': {}", var, e))?;

        if let Some(output_path) = &args.output_netcdf {
            let output_path = Path::new(output_path);
            netcdf_io::write_max_to_netcdf(
                &result,
                &dim_names,
                &new_var_name,
                var,
                &file,
                output_path,
            )
            .map_err(|e| {
                format!(
                    "Failed writing to NetCDF '{}': {}",
                    output_path.display(),
                    e
                )
            })?;
            println!("✅ Result saved to {}", output_path.display());
        } else {
            println!("Computed maximum array:\n{:#?}", result);
        }
    } else if let Some(var_name) = args.describe {
        // Describe a specific variable's details
        metadata::describe_variable(&file, &var_name)
            .map_err(|e| format!("Failed describing variable '{}': {}", var_name, e))?;
    } else if let Some(var_name) = args.summary {
        // Compute quick statistics for a variable
        metadata::compute_variable_summary(&file, &var_name).map_err(|e| {
            format!(
                "Failed computing summary for variable '{}': {}",
                var_name, e
            )
        })?;
    } else if let Some(slice_spec) = args.slice {
        // Extract a slice of data
        netcdf_io::extract_slice(&file, slice_spec)
            .map_err(|e| format!("Failed extracting slice: {}", e))?;
    } else {
        // Default: print full metadata
        metadata::print_metadata(&file).map_err(|e| format!("Failed printing metadata: {}", e))?;
    }

    Ok(())
}
