//! Zarr I/O operations including data reading and writing
//!
//! This module provides functions for reading data from Zarr arrays and writing
//! computed statistical results to new Zarr arrays with proper metadata.
//! Currently supports local filesystem storage.

use crate::errors::{Result, RuNeVisError};
use ndarray::{Array, ArrayD, IxDyn};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use zarrs::{
    array::{Array as ZarrArray, ArrayBuilder, ChunkShape, DataType, DimensionName, FillValue},
    filesystem::FilesystemStore,
    storage::{ReadableWritableListableStorage, StoreKey},
};
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
    store: FilesystemStore,
}

impl ZarrReader {
    /// Create a new ZarrReader from a source
    pub async fn new(source: ZarrSource) -> Result<Self> {
        let store = FilesystemStore::new(&source.path)
            .map_err(|e| RuNeVisError::Generic(format!("Failed to create Zarr store: {}", e)))?;
        
        Ok(ZarrReader { store })
    }

    /// List all arrays in the Zarr store
    pub async fn list_arrays(&self) -> Result<Vec<String>> {
        let keys = self.store.list()
            .map_err(|e| RuNeVisError::Generic(format!("Failed to list arrays: {}", e)))?;
        
        let mut arrays = Vec::new();
        for key in keys {
            let key_str = key.as_str();
            if key_str.ends_with("/.zarray") || key_str.ends_with("/.zattrs") {
                let array_name = key_str.trim_end_matches("/.zarray").trim_end_matches("/.zattrs");
                if !array_name.is_empty() && !arrays.contains(&array_name.to_string()) {
                    arrays.push(array_name.to_string());
                }
            }
        }
        
        Ok(arrays)
    }

    /// Get array metadata
    pub async fn get_array_metadata(&self, array_name: &str) -> Result<ArrayMetadata> {
        let array = ZarrArray::new(&self.store, array_name)
            .map_err(|e| RuNeVisError::ArrayNotFound {
                array: array_name.to_string(),
            })?;
        
        let shape = array.shape().to_vec();
        let dtype = array.data_type();
        let chunks = array.chunk_shape().to_vec();
        
        // Get attributes
        let attributes = array.attributes().clone();
        
        Ok(ArrayMetadata {
            name: array_name.to_string(),
            shape,
            dtype: format!("{:?}", dtype),
            chunks,
            attributes,
        })
    }

    /// Read an entire array as ndarray
    pub async fn read_array(&self, array_name: &str) -> Result<ArrayD<f32>> {
        let array = ZarrArray::new(&self.store, array_name)
            .map_err(|e| RuNeVisError::ArrayNotFound {
                array: array_name.to_string(),
            })?;
        
        // Read the entire array
        let data: Vec<f32> = array.retrieve_array_subset_ndarray::<f32, _>(..)
            .map_err(|e| RuNeVisError::Generic(format!("Failed to read array: {}", e)))?
            .iter()
            .cloned()
            .collect();
        
        // Convert to ndarray
        let shape = array.shape();
        let nd_array = Array::from_shape_vec(IxDyn(shape), data)?;
        
        Ok(nd_array)
    }

    /// Read a slice of an array
    pub async fn read_slice(
        &self,
        array_name: &str,
        slice_ranges: &[(usize, usize)],
    ) -> Result<ArrayD<f32>> {
        let array = ZarrArray::new(&self.store, array_name)
            .map_err(|e| RuNeVisError::ArrayNotFound {
                array: array_name.to_string(),
            })?;
        
        // Convert slice ranges to zarrs selection format
        let selection: Vec<_> = slice_ranges
            .iter()
            .map(|(start, end)| *start..*end)
            .collect();
        
        let data: Vec<f32> = array.retrieve_array_subset_ndarray::<f32, _>(&selection)
            .map_err(|e| RuNeVisError::Generic(format!("Failed to read array slice: {}", e)))?
            .iter()
            .cloned()
            .collect();
        
        // Calculate the shape of the slice
        let slice_shape: Vec<usize> = slice_ranges
            .iter()
            .map(|(start, end)| end - start)
            .collect();
        
        let nd_array = Array::from_shape_vec(IxDyn(&slice_shape), data)?;
        
        Ok(nd_array)
    }
}

/// Zarr writer for creating new Zarr arrays
pub struct ZarrWriter {
    store: FilesystemStore,
}

impl ZarrWriter {
    /// Create a new ZarrWriter from a source
    pub async fn new(source: ZarrSource) -> Result<Self> {
        // Ensure the directory exists
        std::fs::create_dir_all(&source.path)?;
        
        let store = FilesystemStore::new(&source.path)
            .map_err(|e| RuNeVisError::Generic(format!("Failed to create Zarr store: {}", e)))?;
        
        Ok(ZarrWriter { store })
    }

    /// Write an ndarray to a Zarr array
    pub async fn write_array(
        &self,
        array_name: &str,
        data: &ArrayD<f32>,
        chunk_shape: Option<Vec<usize>>,
        attributes: Option<HashMap<String, JsonValue>>,
    ) -> Result<()> {
        let shape = data.shape().to_vec();
        let chunks = chunk_shape.unwrap_or_else(|| {
            // Default chunk shape: try to make reasonably sized chunks
            shape
                .iter()
                .map(|&s| std::cmp::min(s, 1024))
                .collect()
        });

        // Create the array
        let array = ArrayBuilder::new(
            shape,
            DataType::Float32,
            ChunkShape::try_from(chunks)
                .map_err(|e| RuNeVisError::Generic(format!("Invalid chunk shape: {}", e)))?,
            FillValue::from(0.0f32),
        )
        .build(&self.store, array_name)
        .map_err(|e| RuNeVisError::Generic(format!("Failed to create array: {}", e)))?;

        // Set attributes if provided
        if let Some(attrs) = attributes {
            for (key, value) in attrs {
                array.store_attribute(&key, &value)
                    .map_err(|e| RuNeVisError::Generic(format!("Failed to set attribute {}: {}", key, e)))?;
            }
        }

        // Write the data
        let flat_data: Vec<f32> = data.iter().cloned().collect();
        array.store_array_subset_ndarray(.., &flat_data)
            .map_err(|e| RuNeVisError::Generic(format!("Failed to write data: {}", e)))?;

        Ok(())
    }

    /// Write statistical result to Zarr array with metadata
    pub async fn write_statistical_result(
        &self,
        array_name: &str,
        data: &ArrayD<f32>,
        dim_names: &[String],
        operation: &str,
        original_array_name: &str,
        source_metadata: Option<&ArrayMetadata>,
    ) -> Result<()> {
        let shape = data.shape().to_vec();
        let chunks: Vec<usize> = shape
            .iter()
            .map(|&s| std::cmp::min(s, 1024))
            .collect();

        // Create the array
        let array = ArrayBuilder::new(
            shape,
            DataType::Float32,
            ChunkShape::try_from(chunks)
                .map_err(|e| RuNeVisError::Generic(format!("Invalid chunk shape: {}", e)))?,
            FillValue::from(0.0f32),
        )
        .build(&self.store, array_name)
        .map_err(|e| RuNeVisError::Generic(format!("Failed to create array: {}", e)))?;

        // Set standard attributes
        let mut attributes = HashMap::new();
        attributes.insert(
            "operation".to_string(),
            JsonValue::String(operation.to_string()),
        );
        attributes.insert(
            "source_array".to_string(),
            JsonValue::String(original_array_name.to_string()),
        );
        attributes.insert(
            "dimensions".to_string(),
            JsonValue::Array(
                dim_names
                    .iter()
                    .map(|s| JsonValue::String(s.clone()))
                    .collect(),
            ),
        );
        attributes.insert(
            "created_by".to_string(),
            JsonValue::String("RuNeVis".to_string()),
        );
        attributes.insert(
            "created_at".to_string(),
            JsonValue::String(chrono::Utc::now().to_rfc3339()),
        );

        // Copy relevant attributes from source if available
        if let Some(source) = source_metadata {
            // Copy selected attributes that make sense for the result
            for (key, value) in &source.attributes {
                if !["shape", "chunks", "dtype", "created_at", "created_by"].contains(&key.as_str()) {
                    attributes.insert(key.clone(), value.clone());
                }
            }
        }

        // Set all attributes
        for (key, value) in attributes {
            array.store_attribute(&key, &value)
                .map_err(|e| RuNeVisError::Generic(format!("Failed to set attribute {}: {}", key, e)))?;
        }

        // Write the data
        let flat_data: Vec<f32> = data.iter().cloned().collect();
        array.store_array_subset_ndarray(.., &flat_data)
            .map_err(|e| RuNeVisError::Generic(format!("Failed to write data: {}", e)))?;

        Ok(())
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
    path: &str,
    array_name: &str,
    data: &ArrayD<f32>,
    chunk_shape: Option<Vec<usize>>,
) -> Result<()> {
    let source = ZarrSource::from_str(path)?;
    let writer = ZarrWriter::new(source).await?;
    writer.write_array(array_name, data, chunk_shape, None).await
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
