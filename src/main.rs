use clap::Parser;
use netcdf::open;

mod cli;
mod utils;

use cli::Args;
use utils::print_metadata;

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

    print_metadata(&file)?;

    Ok(())
}
