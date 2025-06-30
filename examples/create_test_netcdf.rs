//! Creates a sample NetCDF file for testing RuNeVis functionality.
//!
//! This utility creates a NetCDF file with dimensions, variables, and attributes
//! to demonstrate the --list-vars feature and mean computation capabilities.

use ndarray::Array1;
use netcdf::create;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_path = Path::new("test_data.nc");

    println!("🔨 Creating test NetCDF file: {}", output_path.display());

    // Remove existing file if it exists
    if output_path.exists() {
        std::fs::remove_file(output_path)?
    }

    // Create new NetCDF file
    let mut file = create(output_path)?;

    // Add global attributes
    file.add_attribute("title", "Test Climate Data")?;
    file.add_attribute("institution", "RuNeVis Test Suite")?;
    file.add_attribute("created_by", "create_test_netcdf.rs")?;

    // Add dimensions
    file.add_dimension("time", 12)?; // 12 time steps (months)
    file.add_dimension("lat", 5)?; // 5 latitude points (smaller for demo)
    file.add_dimension("lon", 8)?; // 8 longitude points (smaller for demo)
    file.add_dimension("level", 3)?; // 3 pressure levels (smaller for demo)

    // Add coordinate variables (1D only for simplicity)
    {
        let mut time_var = file.add_variable::<f64>("time", &["time"])?;
        time_var.put_attribute("units", "days since 2023-01-01")?;
        time_var.put_attribute("long_name", "time")?;
        time_var.put_attribute("calendar", "standard")?;

        let time_data: Vec<f64> = (0..12).map(|i| i as f64 * 30.0).collect();
        let time_array = Array1::from(time_data);
        time_var.put(time_array.view(), ..)?;
    }

    {
        let mut lat_var = file.add_variable::<f32>("lat", &["lat"])?;
        lat_var.put_attribute("units", "degrees_north")?;
        lat_var.put_attribute("long_name", "latitude")?;

        let lat_data: Vec<f32> = (0..5).map(|i| -40.0 + i as f32 * 20.0).collect();
        let lat_array = Array1::from(lat_data);
        lat_var.put(lat_array.view(), ..)?;
    }

    {
        let mut lon_var = file.add_variable::<f32>("lon", &["lon"])?;
        lon_var.put_attribute("units", "degrees_east")?;
        lon_var.put_attribute("long_name", "longitude")?;

        let lon_data: Vec<f32> = (0..8).map(|i| -180.0 + i as f32 * 45.0).collect();
        let lon_array = Array1::from(lon_data);
        lon_var.put(lon_array.view(), ..)?;
    }

    {
        let mut level_var = file.add_variable::<f32>("level", &["level"])?;
        level_var.put_attribute("units", "hPa")?;
        level_var.put_attribute("long_name", "pressure level")?;
        level_var.put_attribute("positive", "down")?;

        let level_data = vec![1000.0f32, 500.0, 200.0];
        let level_array = Array1::from(level_data);
        level_var.put(level_array.view(), ..)?;
    }

    // Add simpler 2D temperature variable (time, lat)
    {
        let mut temp_var = file.add_variable::<f32>("temperature", &["time", "lat"])?;
        temp_var.put_attribute("units", "K")?;
        temp_var.put_attribute("long_name", "air temperature")?;
        temp_var.put_attribute("standard_name", "air_temperature")?;
        temp_var.put_attribute("_FillValue", -999.0f32)?;

        // Create simple 2D temperature data (time=12, lat=5)
        let mut temp_data = Vec::new();
        for time_idx in 0..12 {
            for lat_idx in 0..5 {
                let base_temp = 288.0; // 15°C in Kelvin
                let lat_effect = -20.0 * ((lat_idx as f32 - 2.0) / 2.0).abs(); // Cooler at poles
                let seasonal_effect = 10.0 * (time_idx as f32 * std::f32::consts::PI / 6.0).cos(); // Seasonal variation
                temp_data.push(base_temp + lat_effect + seasonal_effect);
            }
        }

        let temp_array = Array1::from(temp_data).into_shape((12, 5)).unwrap();
        temp_var.put(temp_array.view(), ..)?;
    }

    // Add a scalar variable
    {
        let mut global_var = file.add_variable::<f32>("global_average_temp", &[])?;
        global_var.put_attribute("units", "K")?;
        global_var.put_attribute("long_name", "global average temperature")?;

        // For scalar variables, use put with a 0-dimensional array view
        use ndarray::arr0;
        let scalar_value = arr0(288.15f32);
        global_var.put(scalar_value.view(), &[] as &[usize])?;
    }

    println!("✅ Successfully created test NetCDF file with:");
    println!("   📏 Dimensions: time(12), lat(5), lon(8), level(3)");
    println!("   📈 Variables: time, lat, lon, level, temperature, global_average_temp");
    println!("   🏷️  Attributes: units, long_name, standard_name, _FillValue");
    println!("\n🧪 Test the new --list-vars feature with:");
    println!("   cargo run -- -f test_data.nc --list-vars");

    Ok(())
}
