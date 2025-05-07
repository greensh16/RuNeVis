//! Entry point for the RuNeVis application.
//! Handles CLI parsing, file loading, and dispatches operations like computing means or printing metadata.

use clap::Parser;
use netcdf::open;
use std::path::Path;
mod cli;
mod utils;

use cli::Args;
use utils::{print_metadata,mean_over_dimension,write_mean_to_netcdf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments
    let args = Args::parse();

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

    // Open NetCDF file
    let file = open(&args.file)?;
    println!("Successfully opened NetCDF file: {}", &args.file);

    // If mean is specified, compute it; otherwise just print metadata
    if let Some((var, dim)) = args.mean {
        let (result, dim_names, new_var_name) = mean_over_dimension(&file, &var, &dim)?;

        if let Some(output_path) = args.output_netcdf {
            let output_path = Path::new(&output_path);
            write_mean_to_netcdf(
                &result,
                &dim_names,
                &new_var_name,
                &var,
                &file,
                output_path,
            )?;
            println!("✅ Saved result to {}", output_path.display());
        } else {
            println!("Mean result: {:?}", result);
        }
    } else {
        print_metadata(&file)?;
    }

    Ok(())
}
