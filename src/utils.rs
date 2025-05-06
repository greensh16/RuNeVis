use netcdf::File;

pub fn print_metadata(file: &File) -> Result<(), Box<dyn std::error::Error>> {
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
