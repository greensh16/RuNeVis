//! Parallel processing configuration and management
//!
//! This module provides abstractions for configuring Rayon's global thread pool
//! and managing parallel processing configurations.

use crate::errors::{Result, RuNeVisError};
use rayon::ThreadPoolBuilder;

/// Configuration for parallel processing
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    pub num_threads: Option<usize>,
}

impl ParallelConfig {
    /// Create a new parallel configuration
    pub fn new(num_threads: Option<usize>) -> Self {
        Self { num_threads }
    }

    /// Set up the global Rayon thread pool with the specified configuration
    pub fn setup_global_pool(&self) -> Result<()> {
        if let Some(num_threads) = self.num_threads {
            ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .build_global()
                .map_err(|e| {
                    RuNeVisError::ThreadPoolError(format!(
                        "Failed to initialize thread pool with {} threads: {}",
                        num_threads, e
                    ))
                })?;
            
            println!("✅ Configured parallel processing with {} threads", num_threads);
        } else {
            println!("✅ Using default thread pool configuration");
        }
        
        Ok(())
    }

    /// Get the current number of threads being used
    pub fn current_threads(&self) -> usize {
        rayon::current_num_threads()
    }

    /// Create a configuration that uses all available CPU cores
    pub fn all_cores() -> Self {
        Self {
            num_threads: Some(num_cpus::get()),
        }
    }

    /// Create a configuration that uses a specific number of threads
    pub fn with_threads(num_threads: usize) -> Self {
        Self {
            num_threads: Some(num_threads),
        }
    }

    /// Create a configuration that uses the default thread pool
    pub fn default() -> Self {
        Self { num_threads: None }
    }
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self::default()
    }
}

/// Trait for types that can perform parallel reductions
pub trait ParallelReduction<T> {
    fn reduce_parallel(&self, config: &ParallelConfig) -> Result<T>;
}

/// Get information about the current parallel configuration
pub fn get_parallel_info() -> ParallelInfo {
    ParallelInfo {
        current_threads: rayon::current_num_threads(),
        available_cores: num_cpus::get(),
        available_parallelism: std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1),
    }
}

/// Information about the parallel processing environment
#[derive(Debug, Clone)]
pub struct ParallelInfo {
    pub current_threads: usize,
    pub available_cores: usize,
    pub available_parallelism: usize,
}

impl ParallelInfo {
    /// Print parallel processing information
    pub fn print_info(&self) {
        println!("📊 Parallel Processing Information:");
        println!("   Current threads: {}", self.current_threads);
        println!("   Available CPU cores: {}", self.available_cores);
        println!("   Available parallelism: {}", self.available_parallelism);
    }
}
