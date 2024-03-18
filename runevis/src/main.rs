use netcdf::open;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <path-to-netcdf-file>", args[0]);
        std::process::exit(1);
    }
    let path = &args[1];

    let file = open(path)?;

    println!("Dimensions:");
    for dimension in file.dimensions() {
        println!("  Name: {}, Size: {}", dimension.name(), dimension.len());
    }

    println!("\nVariables:");
    for variable in file.variables() {
        println!("  Name: {}", variable.name());
        println!("  Dimensions: {:?}", variable.dimensions());
        println!("  Type: {:?}", variable.vartype());
        println!("  Attributes:");
        for attr in variable.attributes() {
            println!("    Name: {}, Value: {:?}", attr.name(), attr.value());
        }
        println!();
    }

    println!("Global Attributes:");
    for attr in file.root().attributes() {
        println!("  Name: {}, Value: {:?}", attr.name(), attr.value());
    }

    Ok(())
}