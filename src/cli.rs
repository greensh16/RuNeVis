use clap::Parser;
use std::path::PathBuf;

/// A CLI tool for inspecting NetCDF files
#[derive(Parser, Debug)]
#[command(author="Sam Green", version="1.0.0", name="RuNeVis", about="App for working with NetCdf files", long_about = None)]
pub struct Args {
    /// Path to the NetCDF file
    #[arg(short, long)]
    pub file: String,

    /// Compute the mean for a variable over a specific dimension, formatted as <var>:<dim>
    #[arg(long, value_parser = parse_mean_arg)]
    pub mean: Option<(String, String)>,

    /// Path to save mean result as NetCDF. If not set, prints to terminal.
    #[arg(long)]
    pub output_netcdf: Option<PathBuf>,
}

fn parse_mean_arg(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        Err("Expected format: <variable>:<dimension>".to_string())
    } else {
        Ok((parts[0].to_string(), parts[1].to_string()))
    }
}