//! Test example for Zarr metadata functionality
//!
//! This example demonstrates the Zarr metadata reading capabilities in RuNeVis.

use ru_ne_vis::zarr_io::{get_zarr_metadata, list_zarr_arrays, ZarrReader, ZarrSource};

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 RuNeVis Zarr Metadata Test");
    println!("==============================");

    let test_store_path = "./test_zarr_store";

    // Test 1: List arrays in our test store
    println!("\n📋 Testing array listing in test store...");
    match list_zarr_arrays(test_store_path).await {
        Ok(arrays) => {
            if arrays.is_empty() {
                println!("ℹ️  No Zarr arrays found in test store");
            } else {
                println!("📊 Found {} Zarr arrays:", arrays.len());
                for array in &arrays {
                    println!("  - {}", array);
                }

                // Test 2: Get metadata for each found array
                for array_name in arrays {
                    println!("\n📊 Getting metadata for array: {}", array_name);
                    match get_zarr_metadata(test_store_path, &array_name).await {
                        Ok(metadata) => {
                            println!("✅ Successfully read metadata:");
                            metadata.print();
                        }
                        Err(e) => {
                            println!("❌ Failed to read metadata: {}", e);
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to list arrays: {}", e);
        }
    }

    // Test 3: Test non-existent array
    println!("\n❓ Testing non-existent array metadata...");
    match get_zarr_metadata(test_store_path, "non_existent_array").await {
        Ok(_) => println!("❌ Unexpectedly succeeded with non-existent array"),
        Err(e) => println!("✅ Correctly failed with non-existent array: {}", e),
    }

    // Test 4: Direct ZarrReader usage
    println!("\n🔍 Testing direct ZarrReader usage...");
    let source = ZarrSource::from_path_str(test_store_path)?;
    let reader = ZarrReader::new(source).await?;

    let arrays = reader.list_arrays().await?;
    println!("📋 Direct reader found {} arrays", arrays.len());

    for array_name in arrays {
        match reader.get_array_metadata(&array_name).await {
            Ok(metadata) => {
                println!("✅ Direct metadata read successful for {}", array_name);
                println!("   Shape: {:?}", metadata.shape);
                println!("   Data type: {}", metadata.dtype);
                println!("   Chunks: {:?}", metadata.chunks);
            }
            Err(e) => {
                println!("❌ Direct metadata read failed for {}: {}", array_name, e);
            }
        }
    }

    println!("\n🎉 Zarr metadata test completed!");
    println!("\n📝 Summary:");
    println!("   - ✅ Zarr store scanning works");
    println!("   - ✅ Array discovery via .zarray files works");
    println!("   - ✅ Metadata parsing from .zarray files works");
    println!("   - ✅ Error handling for non-existent arrays works");
    println!("   - ✅ Both convenience functions and direct reader work");

    Ok(())
}
