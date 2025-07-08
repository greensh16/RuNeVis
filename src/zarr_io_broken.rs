//! Zarr I/O operations
//!
//! This module provides basic Zarr array support for reading Zarr files.
//! Currently supports local filesystem stores with basic read operations.

use crate::errors::{Result, RuNeVisError};
use ndarray::{ArrayD, IxDyn};
use std::collections::HashMap;
use std::path::PathBuf;
use serde_json::Value as JsonValue;

/// Zarr data source
#[derive(Debug, Clone)]
pub struct ZarrSource {
    /// Local filesystem path
    pub path: PathBuf,
}

impl ZarrSource {
    /// Create a new ZarrSource from a path string
    pub fn from_str(s: &str) -> Result<Self> {
        // For now, only support local filesystem
        if s.starts_with("s3://") || s.starts_with("gs://") || s.starts_with("https://") {
            return Err(RuNeVisError::Generic(
                "Cloud storage not yet implemented. Please use local filesystem paths.".to_string()
            ));
        }
        Ok(ZarrSource {
            path: PathBuf::from(s),
        })
    }
}

/// Zarr reader for accessing Zarr arrays
pub struct ZarrReader {
    source: ZarrSource,
}

impl ZarrReader {
    /// Create a new ZarrReader from a source
pub async fn new(source: ZarrSource) -> Result<Self> {
    // Verify the path exists and is a directory
    if !source.path.exists() {
        return Err(RuNeVisError::ZarrError(
            format!("Zarr store path does not exist: {:?}", source.path)
        ));
    }
    
    if !source.path.is_dir() {
        return Err(RuNeVisError::ZarrError(
            format!("Zarr store path is not a directory: {:?}", source.path)
        ));
    }
    
    // Placeholder for creating a ZarrReader
    Ok(ZarrReader { source })
}

    /// List all arrays in the Zarr store
    pub async fn list_arrays(&self) -> Result<Vec<String>> {
        // Read the directory to find array subdirectories
        let mut arrays = Vec::new();
        
        let entries = std::fs::read_dir(&self.source.path)
            .map_err(|e| RuNeVisError::IoError(e))?;
            
        for entry in entries {
            let entry = entry.map_err(|e| RuNeVisError::IoError(e))?;
            let path = entry.path();
            
            if path.is_dir() {
                // Check if this directory contains a .zarray file
                let zarray_path = path.join(".zarray");
                if zarray_path.exists() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        arrays.push(name.to_string());
                    }
                }
            }
        }
        
        Ok(arrays)
    }

    /// Get array metadata
    pub async fn get_array_metadata(&self, array_name: &str) -> Result<ArrayMetadata> {
        let array = Array::new(self.store.clone(), array_name)
            .map_err(|e| RuNeVisError::ZarrError(format!("Failed to open array '{}': {}", array_name, e)))?;

        let shape: Vec<usize> = array.shape().iter().map(|&s| s as usize).collect();
        let chunks: Vec<usize> = array.chunk_shape().iter().map(|&s| s as usize).collect();
        
        let dtype = match array.data_type() {
            DataType::Float32 => "float32".to_string(),
            DataType::Float64 => "float64".to_string(),
            DataType::Int8 => "int8".to_string(),
            DataType::Int16 => "int16".to_string(),
            DataType::Int32 => "int32".to_string(),
            DataType::Int64 => "int64".to_string(),
            DataType::UInt8 => "uint8".to_string(),
            DataType::UInt16 => "uint16".to_string(),
            DataType::UInt32 => "uint32".to_string(),
            DataType::UInt64 => "uint64".to_string(),
            _ => "unknown".to_string(),
        };

        // Get attributes
        let attributes = array.attributes().clone().into_iter().collect();

        Ok(ArrayMetadata {
            name: array_name.to_string(),
            shape,
            dtype,
            chunks,
            attributes,
        })
    }

    /// Read an entire array as ndarray
    pub async fn read_array(&self, array_name: &str) -> Result<ArrayD<f32>> {
        let array = Array::new(self.store.clone(), array_name)
            .map_err(|e| RuNeVisError::ZarrError(format!("Failed to open array '{}': {}", array_name, e)))?;

        // Check if the array is compatible with f32
        match array.data_type() {
            DataType::Float32 | DataType::Float64 | DataType::Int8 | DataType::Int16 | 
            DataType::Int32 | DataType::UInt8 | DataType::UInt16 => {},
            _ => {
                return Err(RuNeVisError::ZarrError(
                    format!("Unsupported data type for array '{}': {:?}", array_name, array.data_type())
                ));
            }
        }

        // Read the entire array
        let array_subset = ArraySubset::new_with_shape(array.shape().to_vec());
        let data = array.retrieve_array_subset(&array_subset)
            .map_err(|e| RuNeVisError::ZarrError(format!("Failed to read array '{}': {}", array_name, e)))?;

        // Convert to ndarray based on data type
        let shape: Vec<usize> = array.shape().iter().map(|&s| s as usize).collect();
        
        match array.data_type() {
            DataType::Float32 => {
                let vec_data: Vec<f32> = data.into_fixed().map_err(|e| {
                    RuNeVisError::ZarrError(format!("Failed to convert data to f32: {}", e))
                })?;
                ArrayD::from_shape_vec(IxDyn(&shape), vec_data)
                    .map_err(|e| RuNeVisError::ArrayError(e))
            },
            DataType::Float64 => {
                let vec_data: Vec<f64> = data.into_fixed().map_err(|e| {
                    RuNeVisError::ZarrError(format!("Failed to convert data to f64: {}", e))
                })?;
                let f32_data: Vec<f32> = vec_data.iter().map(|&x| x as f32).collect();
                ArrayD::from_shape_vec(IxDyn(&shape), f32_data)
                    .map_err(|e| RuNeVisError::ArrayError(e))
            },
            DataType::Int32 => {
                let vec_data: Vec<i32> = data.into_fixed().map_err(|e| {
                    RuNeVisError::ZarrError(format!("Failed to convert data to i32: {}", e))
                })?;
                let f32_data: Vec<f32> = vec_data.iter().map(|&x| x as f32).collect();
                ArrayD::from_shape_vec(IxDyn(&shape), f32_data)
                    .map_err(|e| RuNeVisError::ArrayError(e))
            },
            _ => {
                Err(RuNeVisError::ZarrError(
                    format!("Data type conversion not implemented for: {:?}", array.data_type())
                ))
            }
        }
    }

    /// Read a slice of an array
    pub async fn read_slice(
        &self,
        array_name: &str,
        slice_ranges: &[(usize, usize)],
    ) -> Result<ArrayD<f32>> {
        let array = Array::new(self.store.clone(), array_name)
            .map_err(|e| RuNeVisError::ZarrError(format!("Failed to open array '{}': {}", array_name, e)))?;

        // Validate slice ranges
        if slice_ranges.len() != array.shape().len() {
            return Err(RuNeVisError::InvalidSlice {
                message: format!(
                    "Slice dimensions ({}) don't match array dimensions ({})",
                    slice_ranges.len(),
                    array.shape().len()
                )
            });
        }

        // Convert slice ranges to ArraySubset
        let mut start = Vec::new();
        let mut shape = Vec::new();
        
        for (i, &(start_idx, end_idx)) in slice_ranges.iter().enumerate() {
            let dim_size = array.shape()[i] as usize;
            
            if start_idx >= dim_size || end_idx > dim_size || start_idx >= end_idx {
                return Err(RuNeVisError::InvalidSlice {
                    message: format!(
                        "Invalid slice range [{}, {}) for dimension {} with size {}",
                        start_idx, end_idx, i, dim_size
                    )
                });
            }
            
            start.push(start_idx as u64);
            shape.push((end_idx - start_idx) as u64);
        }

        let array_subset = ArraySubset::new_with_start_shape(start, shape)
            .map_err(|e| RuNeVisError::ZarrError(format!("Failed to create array subset: {}", e)))?;
        
        let data = array.retrieve_array_subset(&array_subset)
            .map_err(|e| RuNeVisError::ZarrError(format!("Failed to read array slice '{}': {}", array_name, e)))?;

        // Convert to ndarray
        let result_shape: Vec<usize> = shape.iter().map(|&s| s as usize).collect();
        
        match array.data_type() {
            DataType::Float32 => {
                let vec_data: Vec<f32> = data.into_fixed().map_err(|e| {
                    RuNeVisError::ZarrError(format!("Failed to convert data to f32: {}", e))
                })?;
                ArrayD::from_shape_vec(IxDyn(&result_shape), vec_data)
                    .map_err(|e| RuNeVisError::ArrayError(e))
            },
            DataType::Float64 => {
                let vec_data: Vec<f64> = data.into_fixed().map_err(|e| {
                    RuNeVisError::ZarrError(format!("Failed to convert data to f64: {}", e))
                })?;
                let f32_data: Vec<f32> = vec_data.iter().map(|&x| x as f32).collect();
                ArrayD::from_shape_vec(IxDyn(&result_shape), f32_data)
                    .map_err(|e| RuNeVisError::ArrayError(e))
            },
            _ => {
                Err(RuNeVisError::ZarrError(
                    format!("Data type conversion not implemented for: {:?}", array.data_type())
                ))
            }
        }
    }
}

/// Zarr writer for creating new Zarr arrays
pub struct ZarrWriter {
    _source: ZarrSource,
}

impl ZarrWriter {
    /// Create a new ZarrWriter from a source
    pub async fn new(source: ZarrSource) -> Result<Self> {
        Ok(ZarrWriter { _source: source })
    }

    /// Write an ndarray to a Zarr array
    pub async fn write_array(
        &self,
        _array_name: &str,
        _data: &ArrayD<f32>,
        _chunk_shape: Option<Vec<usize>>,
        _attributes: Option<HashMap<String, JsonValue>>,
    ) -> Result<()> {
        Err(RuNeVisError::Generic(
            "Zarr support is not fully implemented yet. Please use NetCDF files.".to_string()
        ))
    }

    /// Write statistical result to Zarr array with metadata
    pub async fn write_statistical_result(
        &self,
        _array_name: &str,
        _data: &ArrayD<f32>,
        _dim_names: &[String],
        _operation: &str,
        _original_array_name: &str,
        _source_metadata: Option<&ArrayMetadata>,
    ) -> Result<()> {
        Err(RuNeVisError::Generic(
            "Zarr support is not fully implemented yet. Please use NetCDF files.".to_string()
        ))
    }
}

/// Metadata for a Zarr array
#[derive(Debug, Clone)]
pub struct ArrayMetadata {
    pub name: String,
    pub shape: Vec<usize>,
    pub dtype: String,
    pub chunks: Vec<usize>,
    pub attributes: HashMap<String, JsonValue>,
}

impl ArrayMetadata {
    /// Print array metadata in a formatted way
    pub fn print(&self) {
        println!("Array: {}", self.name);
        println!("  Shape: {:?}", self.shape);
        println!("  Data type: {}", self.dtype);
        println!("  Chunks: {:?}", self.chunks);
        println!("  Attributes:");
        for (key, value) in &self.attributes {
            println!("    {}: {}", key, value);
        }
    }
}

/// Convenience function to read a Zarr array from a path
pub async fn read_zarr_array(path: &str, array_name: &str) -> Result<ArrayD<f32>> {
    let source = ZarrSource::from_str(path)?;
    let reader = ZarrReader::new(source).await?;
    reader.read_array(array_name).await
}

/// Convenience function to write a Zarr array to a path
pub async fn write_zarr_array(
    _path: &str,
    _array_name: &str,
    _data: &ArrayD<f32>,
    _chunk_shape: Option<Vec<usize>>,
) -> Result<()> {
    // TODO: Implement Zarr writing functionality
    Err(RuNeVisError::Generic(
        "Zarr writing is not yet implemented. Please use read operations for now.".to_string()
    ))
}

/// List all arrays in a Zarr store
pub async fn list_zarr_arrays(path: &str) -> Result<Vec<String>> {
    let source = ZarrSource::from_str(path)?;
    let reader = ZarrReader::new(source).await?;
    reader.list_arrays().await
}

/// Get metadata for a Zarr array
pub async fn get_zarr_metadata(path: &str, array_name: &str) -> Result<ArrayMetadata> {
    let source = ZarrSource::from_str(path)?;
    let reader = ZarrReader::new(source).await?;
    reader.get_array_metadata(array_name).await
}
