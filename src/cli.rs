use clap::Parser;

/// A CLI tool for inspecting NetCDF files
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to the NetCDF file
    #[arg(short, long)]
    pub file: String,
}