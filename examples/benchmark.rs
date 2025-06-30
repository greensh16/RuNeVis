//! Simple benchmark example showing the performance benefits of parallel processing.
//!
//! This example demonstrates the performance improvement when using Rayon
//! for parallel computation similar to NetCDF mean calculations.

use rayon::prelude::*;
use std::time::Instant;

fn simulate_mean_calculation(data_size: usize, use_parallel: bool) -> f64 {
    let data: Vec<f32> = (0..data_size).map(|i| (i as f32).sin()).collect();

    let start = Instant::now();

    let mean = if use_parallel {
        // Parallel version using Rayon
        let sum: f32 = data.par_iter().sum();
        sum / data.len() as f32
    } else {
        // Sequential version
        let sum: f32 = data.iter().sum();
        sum / data.len() as f32
    };

    let duration = start.elapsed();
    println!("   Mean result: {:.6}", mean);

    duration.as_secs_f64()
}

fn main() {
    println!("🔬 RuNeVis Parallel Processing Benchmark");
    println!("==========================================\n");

    let available_threads = rayon::current_num_threads();
    println!(
        "System has {} logical CPU cores available\n",
        available_threads
    );

    let data_sizes = vec![1_000_000, 5_000_000, 10_000_000];

    for data_size in data_sizes {
        println!("📊 Testing with {} data points:", data_size);
        println!("-------------------------------------------");

        println!("🐌 Sequential processing:");
        let seq_time = simulate_mean_calculation(data_size, false);
        println!("   ⏱️  Duration: {:.3} seconds\n", seq_time);

        println!("⚡ Parallel processing ({} threads):", available_threads);
        let par_time = simulate_mean_calculation(data_size, true);
        println!("   ⏱️  Duration: {:.3} seconds", par_time);

        let speedup = seq_time / par_time;
        println!("   🚀 Speedup: {:.2}x faster\n", speedup);

        if speedup > 1.0 {
            println!("✅ Parallel processing is {:.2}x faster!", speedup);
        } else {
            println!("⚠️  Sequential was faster for this dataset size");
        }
        println!("=========================================\n");
    }

    println!("💡 Key Takeaways:");
    println!("   - Larger datasets benefit more from parallel processing");
    println!("   - Use --threads option in RuNeVis to control parallelism");
    println!("   - Optimal thread count depends on your CPU and dataset size");
}
