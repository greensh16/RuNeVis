//! Zarr I/O operations
//!
//! This module provides basic Zarr array support framework.
//! Currently provides minimal functionality with placeholders for future expansion.

use crate::errors::{Result, RuNeVisError};
use crate::data_source::{DataReader, LazyDataReader, StreamingDataReader, DataWriter, DataArrayMetadata, AdvancedDataSource, FullDataSource};
use ndarray::ArrayD;
use rayon::prelude::*;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::PathBuf;
use async_trait::async_trait;
use async_stream;

/// Zarr data source
#[derive(Debug, Clone)]
pub struct ZarrSource {
    /// Local filesystem path
    pub path: PathBuf,
}

impl ZarrSource {
    /// Create a new ZarrSource from a path string
    pub fn from_path_str(s: &str) -> Result<Self> {
        // For now, only support local filesystem
        if s.starts_with("s3://") || s.starts_with("gs://") || s.starts_with("https://") {
            return Err(RuNeVisError::Generic(
                "Cloud storage not yet implemented. Please use local filesystem paths.".to_string(),
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
            return Err(RuNeVisError::ZarrError(format!(
                "Zarr store path does not exist: {:?}",
                source.path
            )));
        }

        if !source.path.is_dir() {
            return Err(RuNeVisError::ZarrError(format!(
                "Zarr store path is not a directory: {:?}",
                source.path
            )));
        }

        Ok(ZarrReader { source })
    }

    /// List all arrays in the Zarr store
    pub async fn list_arrays(&self) -> Result<Vec<String>> {
        // Read the directory to find array subdirectories
        let mut arrays = Vec::new();

        let entries = std::fs::read_dir(&self.source.path).map_err(RuNeVisError::IoError)?;

        for entry in entries {
            let entry = entry.map_err(RuNeVisError::IoError)?;
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
        // Check if array exists
        let array_path = self.source.path.join(array_name);
        if !array_path.exists() {
            return Err(RuNeVisError::ArrayNotFound {
                array: array_name.to_string(),
            });
        }

        // Read .zarray metadata
        let zarray_path = array_path.join(".zarray");
        if !zarray_path.exists() {
            return Err(RuNeVisError::ZarrError(format!(
                "Array metadata file not found: {}",
                zarray_path.display()
            )));
        }

        let metadata_content =
            std::fs::read_to_string(&zarray_path).map_err(RuNeVisError::IoError)?;

        let metadata: JsonValue = serde_json::from_str(&metadata_content)
            .map_err(|e| RuNeVisError::ZarrError(format!("Failed to parse metadata: {}", e)))?;

        // Parse basic metadata
        let shape = metadata["shape"]
            .as_array()
            .ok_or_else(|| RuNeVisError::ZarrError("Missing shape in metadata".to_string()))?
            .iter()
            .map(|v| v.as_u64().unwrap_or(0) as usize)
            .collect();

        let chunks = metadata["chunks"]
            .as_array()
            .ok_or_else(|| RuNeVisError::ZarrError("Missing chunks in metadata".to_string()))?
            .iter()
            .map(|v| v.as_u64().unwrap_or(0) as usize)
            .collect();

        let dtype = metadata["dtype"].as_str().unwrap_or("unknown").to_string();

        Ok(ArrayMetadata {
            name: array_name.to_string(),
            shape,
            dtype,
            chunks,
            attributes: HashMap::new(), // TODO: Parse attributes
        })
    }

/// Read an entire array as ndarray
    pub async fn read_array(&self, array_name: &str) -> Result<ArrayD<f32>> {
        let array_metadata = self.get_array_metadata(array_name).await?;
        let total_size: usize = array_metadata.shape.iter().product();
        let mut data = vec![0.0f32; total_size];
        let path = self.source.path.join(array_name);

        println!(
            "🚀 Loading data array '{}' with parallel processing...",
            array_name
        );

        // Parallel processing with Rayon
        data.par_iter_mut().enumerate().for_each(|(i, val)| {
            let index = i % total_size; // Simplified indexing logic for example purposes
            let element_path = path.join(format!("chunk_{}", index));
            if let Ok(bytes) = std::fs::read(&element_path) {
                let chunk_data = bytes.chunks_exact(4).map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect::<Vec<f32>>();
                *val = chunk_data.into_iter().next().unwrap_or(0.0);
            }
        });

        ArrayD::from_shape_vec(array_metadata.shape, data).map_err(|e| {
            RuNeVisError::ZarrError(format!("Failed to shape data into ndarray: {}", e))
        })
    }

/// Lazy load an array as needed (returns a lazy wrapper)
    pub async fn lazy_load_array(&self, array_name: &str) -> Result<LazyArray> {
        let metadata = self.get_array_metadata(array_name).await?;
        Ok(LazyArray {
            source: self.source.clone(),
            array_name: array_name.to_string(),
            metadata,
            loaded: None,
        })
    }

    /// Stream data chunks
    pub fn stream_chunks(&self, array_name: &str) -> std::pin::Pin<Box<dyn futures::Stream<Item = Result<ArrayD<f32>>> + Send + 'static>> {
        let array_name = array_name.to_string();
        let source_path = self.source.path.clone();
        
        Box::pin(async_stream::stream! {
            // Create a temporary reader for metadata
            let source = ZarrSource { path: source_path.clone() };
            let reader = match ZarrReader::new(source).await {
                Ok(r) => r,
                Err(e) => {
                    yield Err(e);
                    return;
                }
            };
            
            let metadata = match reader.get_array_metadata(&array_name).await {
                Ok(meta) => meta,
                Err(e) => {
                    yield Err(e);
                    return;
                }
            };
            
            let chunk_size = metadata.chunks.iter().product::<usize>();
            let total_size = metadata.shape.iter().product::<usize>();
            let num_chunks = total_size.div_ceil(chunk_size);
            
            for chunk_idx in 0..num_chunks {
                let chunk_path = source_path.join(&array_name).join(format!("chunk_{}", chunk_idx));
                
                if let Ok(bytes) = std::fs::read(&chunk_path) {
                    let chunk_data: Vec<f32> = bytes
                        .chunks_exact(4)
                        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                        .collect();
                    
                    let chunk_shape = vec![chunk_data.len()];
                    match ArrayD::from_shape_vec(chunk_shape, chunk_data) {
                        Ok(array) => yield Ok(array),
                        Err(e) => yield Err(RuNeVisError::ZarrError(format!("Failed to create chunk array: {}", e))),
                    }
                } else {
                    yield Err(RuNeVisError::ZarrError(format!("Failed to read chunk {}", chunk_idx)));
                }
            }
        })
    }
    
    /// Read a slice of an array
    pub async fn read_slice(
        &self,
        array_name: &str,
        slice_ranges: &[(usize, usize)],
    ) -> Result<ArrayD<f32>> {
        let _array_metadata = self.get_array_metadata(array_name).await?;
        let path = self.source.path.join(array_name);

        let slice_size: usize = slice_ranges.iter().map(|r| r.1 - r.0).product();
        let mut data = vec![0.0f32; slice_size];

        println!(
            "🔍 Reading slice for array '{}' with parallel processing...",
            array_name
        );

        // Parallel processing with Rayon
        data.par_iter_mut().enumerate().for_each(|(i, val)| {
            let index = i % slice_size; // Simplified indexing logic for example purposes
            let element_path = path.join(format!("chunk_{}", index));
            if let Ok(bytes) = std::fs::read(&element_path) {
                let chunk_data: Vec<f32> = bytes
                    .chunks_exact(4)
                    .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                    .collect();
                *val = chunk_data.into_iter().next().unwrap_or(0.0);
            }
        });

        // Compute the resulting shape based on slice_ranges
        let slice_shape: Vec<usize> = slice_ranges.iter().map(|r| r.1 - r.0).collect();
        ArrayD::from_shape_vec(slice_shape, data).map_err(|e| {
            RuNeVisError::ZarrError(format!("Failed to shape slice data into ndarray: {}", e))
        })
    }
}

/// Convert ArrayMetadata to DataArrayMetadata for trait compatibility
impl From<ArrayMetadata> for DataArrayMetadata {
    fn from(meta: ArrayMetadata) -> Self {
        let shape_len = meta.shape.len();
        DataArrayMetadata {
            name: meta.name,
            shape: meta.shape,
            dtype: meta.dtype,
            dimensions: (0..shape_len).map(|i| format!("dim_{}", i)).collect(),
            attributes: meta.attributes,
        }
    }
}

/// Implement DataReader trait for ZarrReader
#[async_trait]
impl DataReader for ZarrReader {
    type ArrayType = ArrayD<f32>;
    
    async fn list_arrays(&self) -> Result<Vec<String>> {
        self.list_arrays().await
    }
    
    async fn get_metadata(&self, array_name: &str) -> Result<DataArrayMetadata> {
        let meta = self.get_array_metadata(array_name).await?;
        Ok(meta.into())
    }
    
    async fn read_array(&self, array_name: &str) -> Result<ArrayD<f32>> {
        self.read_array(array_name).await
    }
    
    async fn read_slice(
        &self,
        array_name: &str,
        slice_ranges: &[(usize, usize)],
    ) -> Result<ArrayD<f32>> {
        self.read_slice(array_name, slice_ranges).await
    }
}

/// Implement LazyDataReader trait for ZarrReader
#[async_trait]
impl LazyDataReader for ZarrReader {
    type LazyArray = LazyArray;
    
    async fn lazy_load(&self, array_name: &str) -> Result<Self::LazyArray> {
        self.lazy_load_array(array_name).await
    }
}

/// Implement StreamingDataReader trait for ZarrReader
#[async_trait]
impl StreamingDataReader for ZarrReader {
    type ChunkStream = std::pin::Pin<Box<dyn futures::Stream<Item = Result<ArrayD<f32>>> + Send + 'static>>;
    
    fn stream_chunks(&self, array_name: &str) -> Self::ChunkStream {
        self.stream_chunks(array_name)
    }
}

/// Implement DataWriter trait for ZarrWriter
#[async_trait]
impl DataWriter for ZarrWriter {
    async fn write_array(
        &self,
        array_name: &str,
        data: &ArrayD<f32>,
        chunk_shape: Option<Vec<usize>>,
        attributes: Option<HashMap<String, JsonValue>>,
    ) -> Result<()> {
        self.write_array(array_name, data, chunk_shape, attributes).await
    }
    
    async fn write_statistical_result(
        &self,
        array_name: &str,
        data: &ArrayD<f32>,
        dim_names: &[String],
        operation: &str,
        original_array_name: &str,
        source_metadata: Option<&DataArrayMetadata>,
    ) -> Result<()> {
        // Convert back to ArrayMetadata if needed
        let array_meta = source_metadata.map(|meta| ArrayMetadata {
            name: meta.name.clone(),
            shape: meta.shape.clone(),
            dtype: meta.dtype.clone(),
            chunks: vec![], // Default empty chunks
            attributes: meta.attributes.clone(),
        });
        
        self.write_statistical_result(
            array_name,
            data,
            dim_names,
            operation,
            original_array_name,
            array_meta.as_ref(),
        ).await
    }
}

/// Combined Zarr data source that implements all data source traits
pub struct ZarrDataSource {
    pub reader: ZarrReader,
    pub writer: ZarrWriter,
}

impl ZarrDataSource {
    /// Create a new combined Zarr data source
    pub async fn new(source: ZarrSource) -> Result<Self> {
        let reader = ZarrReader::new(source.clone()).await?;
        let writer = ZarrWriter::new(source).await?;
        Ok(Self { reader, writer })
    }
}

/// Implement all data source traits for ZarrDataSource
#[async_trait]
impl DataReader for ZarrDataSource {
    type ArrayType = ArrayD<f32>;
    
    async fn list_arrays(&self) -> Result<Vec<String>> {
        self.reader.list_arrays().await
    }
    
    async fn get_metadata(&self, array_name: &str) -> Result<DataArrayMetadata> {
        self.reader.get_metadata(array_name).await
    }
    
    async fn read_array(&self, array_name: &str) -> Result<ArrayD<f32>> {
        self.reader.read_array(array_name).await
    }
    
    async fn read_slice(
        &self,
        array_name: &str,
        slice_ranges: &[(usize, usize)],
    ) -> Result<ArrayD<f32>> {
        self.reader.read_slice(array_name, slice_ranges).await
    }
}

#[async_trait]
impl LazyDataReader for ZarrDataSource {
    type LazyArray = LazyArray;
    
    async fn lazy_load(&self, array_name: &str) -> Result<Self::LazyArray> {
        self.reader.lazy_load(array_name).await
    }
}

#[async_trait]
impl StreamingDataReader for ZarrDataSource {
    type ChunkStream = std::pin::Pin<Box<dyn futures::Stream<Item = Result<ArrayD<f32>>> + Send + 'static>>;
    
    fn stream_chunks(&self, array_name: &str) -> Self::ChunkStream {
        self.reader.stream_chunks(array_name)
    }
}

#[async_trait]
impl DataWriter for ZarrDataSource {
    async fn write_array(
        &self,
        array_name: &str,
        data: &ArrayD<f32>,
        chunk_shape: Option<Vec<usize>>,
        attributes: Option<HashMap<String, JsonValue>>,
    ) -> Result<()> {
        self.writer.write_array(array_name, data, chunk_shape, attributes).await
    }
    
    async fn write_statistical_result(
        &self,
        array_name: &str,
        data: &ArrayD<f32>,
        dim_names: &[String],
        operation: &str,
        original_array_name: &str,
        source_metadata: Option<&DataArrayMetadata>,
    ) -> Result<()> {
        let array_meta = source_metadata.map(|meta| ArrayMetadata {
            name: meta.name.clone(),
            shape: meta.shape.clone(),
            dtype: meta.dtype.clone(),
            chunks: vec![], // Default empty chunks
            attributes: meta.attributes.clone(),
        });
        
        self.writer.write_statistical_result(
            array_name,
            data,
            dim_names,
            operation,
            original_array_name,
            array_meta.as_ref(),
        ).await
    }
}

/// Combined Zarr data source that implements all data source traits
impl FullDataSource for ZarrDataSource {}
impl AdvancedDataSource for ZarrDataSource {}

/// Lazy-loading wrapper for Zarr arrays
#[derive(Debug)]
pub struct LazyArray {
    source: ZarrSource,
    array_name: String,
    metadata: ArrayMetadata,
    loaded: Option<ArrayD<f32>>,
}

impl LazyArray {
    /// Get array metadata without loading data
    pub fn metadata(&self) -> &ArrayMetadata {
        &self.metadata
    }
    
    /// Load the array data if not already loaded
    pub async fn load(&mut self) -> Result<&ArrayD<f32>> {
        if self.loaded.is_none() {
            let reader = ZarrReader::new(self.source.clone()).await?;
            let data = reader.read_array(&self.array_name).await?;
            self.loaded = Some(data);
        }
        Ok(self.loaded.as_ref().unwrap())
    }
    
    /// Check if data is loaded
    pub fn is_loaded(&self) -> bool {
        self.loaded.is_some()
    }
    
    /// Get shape without loading data
    pub fn shape(&self) -> &[usize] {
        &self.metadata.shape
    }
    
    /// Get chunks without loading data
    pub fn chunks(&self) -> &[usize] {
        &self.metadata.chunks
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
        array_name: &str,
        data: &ArrayD<f32>,
        chunk_shape: Option<Vec<usize>>,
        _attributes: Option<HashMap<String, JsonValue>>,
    ) -> Result<()> {
        let data_shape = data.shape().to_vec();
        let chunks = chunk_shape.unwrap_or_else(|| data_shape.clone());

        println!(
            "✏️ Writing array '{}' with parallel processing...",
            array_name
        );
        println!("📊 Data shape: {:?}, Chunk shape: {:?}", data_shape, chunks);

        // Create array directory
        let array_path = self._source.path.join(array_name);
        std::fs::create_dir_all(&array_path).map_err(RuNeVisError::IoError)?;

        // Write .zarray metadata
        let metadata = serde_json::json!({
            "chunks": chunks,
            "compressor": null,
            "dtype": "<f4",
            "fill_value": 0.0,
            "filters": null,
            "order": "C",
            "shape": data_shape,
            "zarr_format": 2
        });

        let metadata_path = array_path.join(".zarray");
        std::fs::write(
            metadata_path,
            serde_json::to_string_pretty(&metadata).unwrap(),
        )
        .map_err(RuNeVisError::IoError)?;

        // Convert data to Vec for parallel processing
        let data_vec: Vec<f32> = data.iter().cloned().collect();
        let total_elements = data_vec.len();
        let chunk_size = chunks.iter().product::<usize>();

        // Calculate number of chunks needed
        let num_chunks = total_elements.div_ceil(chunk_size);

        println!(
            "⚡ Processing {} chunks in parallel across {} threads...",
            num_chunks,
            rayon::current_num_threads()
        );

        // Write chunks in parallel
        (0..num_chunks).into_par_iter().try_for_each(|chunk_idx| {
            let start_idx = chunk_idx * chunk_size;
            let end_idx = (start_idx + chunk_size).min(total_elements);
            let chunk_data = &data_vec[start_idx..end_idx];

            // Create chunk filename (simplified)
            let chunk_filename = format!("chunk_{}", chunk_idx);
            let chunk_path = array_path.join(chunk_filename);

            // Write chunk data as binary
            let bytes: Vec<u8> = chunk_data
                .iter()
                .flat_map(|&f| f.to_le_bytes().to_vec())
                .collect();

            std::fs::write(chunk_path, bytes).map_err(RuNeVisError::IoError)
        })?;

        println!(
            "✅ Successfully wrote array '{}' with {} chunks",
            array_name, num_chunks
        );
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
        println!(
            "📊 Writing statistical result '{}' ({}) with parallel processing...",
            array_name, operation
        );

        // Create enhanced attributes for statistical results
        let mut attributes = HashMap::new();
        attributes.insert(
            "operation".to_string(),
            serde_json::Value::String(operation.to_string()),
        );
        attributes.insert(
            "source_array".to_string(),
            serde_json::Value::String(original_array_name.to_string()),
        );
        attributes.insert(
            "dimensions".to_string(),
            serde_json::Value::Array(
                dim_names
                    .iter()
                    .map(|s| serde_json::Value::String(s.clone()))
                    .collect(),
            ),
        );

        if let Some(metadata) = source_metadata {
            attributes.insert(
                "source_shape".to_string(),
                serde_json::Value::Array(
                    metadata
                        .shape
                        .iter()
                        .map(|&s| serde_json::Value::Number(serde_json::Number::from(s)))
                        .collect(),
                ),
            );
            attributes.insert(
                "source_dtype".to_string(),
                serde_json::Value::String(metadata.dtype.clone()),
            );
        }

        // Use the main write_array method with enhanced attributes
        self.write_array(array_name, data, None, Some(attributes))
            .await
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
    let source = ZarrSource::from_path_str(path)?;
    let reader = ZarrReader::new(source).await?;
    reader.read_array(array_name).await
}

/// Convenience function to write a Zarr array to a path with parallel processing
pub async fn write_zarr_array(
    path: &str,
    array_name: &str,
    data: &ArrayD<f32>,
    chunk_shape: Option<Vec<usize>>,
) -> Result<()> {
    let source = ZarrSource::from_path_str(path)?;
    let writer = ZarrWriter::new(source).await?;
    writer
        .write_array(array_name, data, chunk_shape, None)
        .await
}

/// List all arrays in a Zarr store
pub async fn list_zarr_arrays(path: &str) -> Result<Vec<String>> {
    let source = ZarrSource::from_path_str(path)?;
    let reader = ZarrReader::new(source).await?;
    reader.list_arrays().await
}

/// Get metadata for a Zarr array
pub async fn get_zarr_metadata(path: &str, array_name: &str) -> Result<ArrayMetadata> {
    let source = ZarrSource::from_path_str(path)?;
    let reader = ZarrReader::new(source).await?;
    reader.get_array_metadata(array_name).await
}
