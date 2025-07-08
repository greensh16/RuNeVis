//! Data source abstraction for unified NetCDF and Zarr interfaces
//!
//! This module provides trait-based abstractions that allow interoperability
//! between NetCDF and Zarr data sources following the Interface Segregation Principle.

use crate::errors::Result;
use ndarray::ArrayD;
use std::collections::HashMap;
use serde_json::Value as JsonValue;
use async_trait::async_trait;

/// Metadata for array-like data
#[derive(Debug, Clone)]
pub struct DataArrayMetadata {
    pub name: String,
    pub shape: Vec<usize>,
    pub dtype: String,
    pub dimensions: Vec<String>,
    pub attributes: HashMap<String, JsonValue>,
}

/// Basic data source interface for reading arrays
#[async_trait]
pub trait DataReader {
    type ArrayType;
    
    /// List all available arrays in the data source
    async fn list_arrays(&self) -> Result<Vec<String>>;
    
    /// Get metadata for a specific array
    async fn get_metadata(&self, array_name: &str) -> Result<DataArrayMetadata>;
    
    /// Read an entire array
    async fn read_array(&self, array_name: &str) -> Result<ArrayD<f32>>;
    
    /// Read a slice of an array
    async fn read_slice(
        &self,
        array_name: &str,
        slice_ranges: &[(usize, usize)],
    ) -> Result<ArrayD<f32>>;
}

/// Lazy loading interface for deferred array loading
#[async_trait]
pub trait LazyDataReader: DataReader {
    type LazyArray;
    
    /// Create a lazy reference to an array without loading data
    async fn lazy_load(&self, array_name: &str) -> Result<Self::LazyArray>;
}

/// Streaming interface for chunk-based data processing
#[async_trait]
pub trait StreamingDataReader: DataReader {
    type ChunkStream;
    
    /// Create a stream of data chunks for processing large arrays
    fn stream_chunks(&self, array_name: &str) -> Self::ChunkStream;
}

/// Data writing interface
#[async_trait]
pub trait DataWriter {
    /// Write an array to the data source
    async fn write_array(
        &self,
        array_name: &str,
        data: &ArrayD<f32>,
        chunk_shape: Option<Vec<usize>>,
        attributes: Option<HashMap<String, JsonValue>>,
    ) -> Result<()>;
    
    /// Write statistical results with enhanced metadata
    async fn write_statistical_result(
        &self,
        array_name: &str,
        data: &ArrayD<f32>,
        dim_names: &[String],
        operation: &str,
        original_array_name: &str,
        source_metadata: Option<&DataArrayMetadata>,
    ) -> Result<()>;
}

/// Interface segregation: combine only needed capabilities
pub trait FullDataSource: DataReader + DataWriter {}

/// Interface segregation: read-only data sources
pub trait ReadOnlyDataSource: DataReader {}

/// Interface segregation: lazy and streaming capabilities
pub trait AdvancedDataSource: LazyDataReader + StreamingDataReader + DataWriter {}

/// Conversion trait for compatibility between data source types
pub trait DataSourceConverter<T> {
    /// Convert from one data source format to another
    fn convert_from(&self, other: &T) -> Result<()>;
}
