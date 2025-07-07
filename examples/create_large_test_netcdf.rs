//! Creates a large NetCDF file for performance profiling.
//!
//! This utility creates a NetCDF file with larger dimensions to test
//! the performance of mean calculation, I/O loading, and NetCDF writing
//! operations in RuNeVis.

use ndarray::Array1;
use netcdf::create;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_path = Path::new("large_test_data.nc");

    println!("🔨 Creating large test NetCDF file for profiling: {}", output_path.display());

    // Remove existing file if it exists
    if output_path.exists() {
        std::fs::remove_file(output_path)?
    }

    // Create new NetCDF file
    let mut file = create(output_path)?;

    // Add global attributes
    file.add_attribute("title", "Large Test Climate Data for Profiling")?;
    file.add_attribute("institution", "RuNeVis Performance Test Suite")?;
    file.add_attribute("created_by", "create_large_test_netcdf.rs")?;

    // Add larger dimensions for performance testing
    println!("📏 Adding dimensions...");
    file.add_dimension("time", 365)?;     // 365 time steps (daily data)
    file.add_dimension("lat", 180)?;      // 180 latitude points (1 degree resolution)
    file.add_dimension("lon", 360)?;      // 360 longitude points (1 degree resolution)
    file.add_dimension("level", 10)?;     // 10 pressure levels

    // Add coordinate variables
    println!("🔗 Creating coordinate variables...");
    {
        let mut time_var = file.add_variable::<f64>("time", &["time"])?;
        time_var.put_attribute("units", "days since 2023-01-01")?;
        time_var.put_attribute("long_name", "time")?;
        time_var.put_attribute("calendar", "standard")?;

        let time_data: Vec<f64> = (0..365).map(|i| i as f64).collect();
        let time_array = Array1::from(time_data);
        time_var.put(time_array.view(), ..)?;
    }

    {
        let mut lat_var = file.add_variable::<f32>("lat", &["lat"])?;
        lat_var.put_attribute("units", "degrees_north")?;
        lat_var.put_attribute("long_name", "latitude")?;

        let lat_data: Vec<f32> = (0..180).map(|i| -89.5 + i as f32).collect();
        let lat_array = Array1::from(lat_data);
        lat_var.put(lat_array.view(), ..)?;
    }

    {
        let mut lon_var = file.add_variable::<f32>("lon", &["lon"])?;
        lon_var.put_attribute("units", "degrees_east")?;
        lon_var.put_attribute("long_name", "longitude")?;

        let lon_data: Vec<f32> = (0..360).map(|i| -179.5 + i as f32).collect();
        let lon_array = Array1::from(lon_data);
        lon_var.put(lon_array.view(), ..)?;
    }

    {
        let mut level_var = file.add_variable::<f32>("level", &["level"])?;
        level_var.put_attribute("units", "hPa")?;
        level_var.put_attribute("long_name", "pressure level")?;
        level_var.put_attribute("positive", "down")?;

        let level_data: Vec<f32> = (0..10).map(|i| 1000.0 - i as f32 * 100.0).collect();
        let level_array = Array1::from(level_data);
        level_var.put(level_array.view(), ..)?;
    }

    // Add 3D temperature variable (time, lat, lon) - large dataset
    println!("🌡️  Creating temperature variable (time, lat, lon)...");
    {
        let mut temp_var = file.add_variable::<f32>("temperature", &["time", "lat", "lon"])?;
        temp_var.put_attribute("units", "K")?;
        temp_var.put_attribute("long_name", "air temperature")?;
        temp_var.put_attribute("standard_name", "air_temperature")?;
        temp_var.put_attribute("_FillValue", -999.0f32)?;

        // Create realistic temperature data - this will be about 23.4 million data points
        println!("   🏗️ Generating temperature data (365 × 180 × 360 = {} points)...", 365 * 180 * 360);
        let mut temp_data = Vec::with_capacity(365 * 180 * 360);
        
        for time_idx in 0..365 {
            if time_idx % 50 == 0 {
                println!("   📅 Processing day {}/365...", time_idx + 1);
            }
            for lat_idx in 0..180 {
                let lat = -89.5 + lat_idx as f32;
                for lon_idx in 0..360 {
                    let lon = -179.5 + lon_idx as f32;
                    
                    // Generate realistic temperature based on latitude, longitude, and season
                    let base_temp = 288.0; // 15°C in Kelvin
                    let lat_effect = -30.0 * (lat.abs() / 90.0); // Cooler at poles
                    let seasonal_effect = 15.0 * ((time_idx as f32 * 2.0 * std::f32::consts::PI / 365.0 + lat.to_radians()).cos()); // Seasonal variation
                    let noise = (lon * 0.1 + time_idx as f32 * 0.01).sin() * 2.0; // Small random variation
                    
                    temp_data.push(base_temp + lat_effect + seasonal_effect + noise);
                }
            }
        }

        println!("   💾 Writing temperature data to NetCDF...");
        let temp_array = Array1::from(temp_data).into_shape((365, 180, 360)).unwrap();
        temp_var.put(temp_array.view(), ..)?;
    }

    // Add 4D pressure variable (time, level, lat, lon) - very large dataset  
    println!("🌪️  Creating pressure variable (time, level, lat, lon)...");
    {
        let mut pres_var = file.add_variable::<f32>("pressure", &["time", "level", "lat", "lon"])?;
        pres_var.put_attribute("units", "Pa")?;
        pres_var.put_attribute("long_name", "air pressure")?;
        pres_var.put_attribute("standard_name", "air_pressure")?;
        pres_var.put_attribute("_FillValue", -999.0f32)?;

        // Create pressure data - this will be about 234 million data points!
        println!("   🏗️ Generating pressure data (365 × 10 × 180 × 360 = {} points)...", 365 * 10 * 180 * 360);
        let mut pres_data = Vec::with_capacity(365 * 10 * 180 * 360);
        
        for time_idx in 0..365 {
            if time_idx % 50 == 0 {
                println!("   📅 Processing day {}/365...", time_idx + 1);
            }
            for level_idx in 0..10 {
                let pressure_level = 1000.0 - level_idx as f32 * 100.0; // hPa
                for lat_idx in 0..180 {
                    let lat = -89.5 + lat_idx as f32;
                    for lon_idx in 0..360 {
                        let lon = -179.5 + lon_idx as f32;
                        
                        // Generate realistic pressure based on altitude and surface pressure variations
                        let base_pressure = pressure_level * 100.0; // Convert hPa to Pa
                        let surface_variation = 5000.0 * ((lat / 30.0).sin() + (lon / 60.0).cos() + (time_idx as f32 / 100.0).sin()); 
                        let altitude_factor = 1.0 - (level_idx as f32 * 0.05); // Pressure decreases with altitude
                        
                        pres_data.push((base_pressure + surface_variation) * altitude_factor);
                    }
                }
            }
        }

        println!("   💾 Writing pressure data to NetCDF...");
        let pres_array = Array1::from(pres_data).into_shape((365, 10, 180, 360)).unwrap();
        pres_var.put(pres_array.view(), ..)?;
    }

    let file_size = std::fs::metadata(output_path)?.len();
    let file_size_mb = file_size as f64 / (1024.0 * 1024.0);

    println!("✅ Successfully created large test NetCDF file:");
    println!("   📏 Dimensions: time(365), lat(180), lon(360), level(10)");
    println!("   📈 Variables: time, lat, lon, level, temperature, pressure");
    println!("   📊 Temperature data points: {} ({:.1} MB approx)", 365 * 180 * 360, (365 * 180 * 360 * 4) as f64 / (1024.0 * 1024.0));
    println!("   🌪️  Pressure data points: {} ({:.1} MB approx)", 365 * 10 * 180 * 360, (365 * 10 * 180 * 360 * 4) as f64 / (1024.0 * 1024.0));
    println!("   💾 Total file size: {:.1} MB", file_size_mb);
    println!();
    println!("🔬 Ready for performance profiling with:");
    println!("   cargo flamegraph --bin RuNeVis -- -f large_test_data.nc --mean temperature:time --threads 8");
    println!("   cargo flamegraph --bin RuNeVis -- -f large_test_data.nc --mean pressure:level --threads 8");

    Ok(())
}
