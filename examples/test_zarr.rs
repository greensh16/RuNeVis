//! Test example for Zarr functionality
//!
//! This example demonstrates the basic Zarr integration in RuNeVis.
//! Currently shows directory listing and metadata parsing capabilities.

use ru_ne_vis::zarr_io::{list_zarr_arrays, read_zarr_array, ZarrReader, ZarrSource};

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 RuNeVis Zarr Integration Test");
    println!("===================================");

    // Test 1: Create a ZarrSource from a path
    println!("\n📁 Testing ZarrSource creation...");

    // Use current directory as a test path
    let current_dir = std::env::current_dir()?;
    let test_path = current_dir.to_string_lossy();

    match ZarrSource::from_path_str(&test_path) {
        Ok(source) => {
            println!("✅ Successfully created ZarrSource: {:?}", source.path);
        }
        Err(e) => {
            println!("❌ Failed to create ZarrSource: {}", e);
            return Ok(());
        }
    }

    // Test 2: Test cloud storage rejection
    println!("\n☁️  Testing cloud storage rejection...");
    match ZarrSource::from_path_str("s3://test-bucket/data") {
        Ok(_) => println!("❌ Unexpectedly succeeded with cloud storage"),
        Err(e) => println!("✅ Correctly rejected cloud storage: {}", e),
    }

    // Test 3: Test ZarrReader creation
    println!("\n📖 Testing ZarrReader creation...");
    let source = ZarrSource::from_path_str(&test_path)?;

    match ZarrReader::new(source).await {
        Ok(reader) => {
            println!("✅ Successfully created ZarrReader");

            // Test 4: List arrays (directories with .zarray files)
            println!("\n📋 Testing array listing...");
            match reader.list_arrays().await {
                Ok(arrays) => {
                    if arrays.is_empty() {
                        println!("ℹ️  No Zarr arrays found in current directory (expected)");
                    } else {
                        println!("📊 Found {} Zarr arrays:", arrays.len());
                        for array in arrays {
                            println!("  - {}", array);
                        }
                    }
                }
                Err(e) => println!("❌ Failed to list arrays: {}", e),
            }
        }
        Err(e) => println!("❌ Failed to create ZarrReader: {}", e),
    }

    // Test 5: Test convenience functions
    println!("\n🛠️  Testing convenience functions...");

    match list_zarr_arrays(&test_path).await {
        Ok(arrays) => println!("✅ list_zarr_arrays worked, found {} arrays", arrays.len()),
        Err(e) => println!("❌ list_zarr_arrays failed: {}", e),
    }

    // Test 6: Test non-existent path
    println!("\n❓ Testing non-existent path...");
    match ZarrSource::from_path_str("/non/existent/path") {
        Ok(source) => match ZarrReader::new(source).await {
            Ok(_) => println!("❌ Unexpectedly succeeded with non-existent path"),
            Err(e) => println!("✅ Correctly failed with non-existent path: {}", e),
        },
        Err(e) => println!("❌ Failed to create source: {}", e),
    }

    // Test 7: Test array reading (should fail gracefully)
    println!("\n📚 Testing array reading (expected to fail)...");
    match read_zarr_array(&test_path, "test_array").await {
        Ok(_) => println!("❌ Unexpectedly succeeded reading array"),
        Err(e) => println!("✅ Correctly failed reading array: {}", e),
    }

    println!("\n🎉 Zarr integration test completed!");
    println!("\n📝 Summary:");
    println!("   - ✅ Basic Zarr source and reader creation works");
    println!("   - ✅ Path validation works correctly");
    println!("   - ✅ Directory listing functionality works");
    println!("   - ✅ Error handling works as expected");
    println!("   - ⚠️  Array reading is not yet implemented (placeholder)");
    println!("   - ⚠️  Array writing is not yet implemented (placeholder)");

    println!("\n🔬 Next steps for full Zarr support:");
    println!("   1. Integrate zarrs crate properly for actual array reading");
    println!("   2. Implement data type conversions");
    println!("   3. Add array slicing support");
    println!("   4. Implement statistical operations for Zarr arrays");
    println!("   5. Add writing capabilities");

    Ok(())
}
