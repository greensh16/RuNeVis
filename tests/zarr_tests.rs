use ru_ne_vis::zarr_io::{ZarrReader, ZarrWriter, ZarrSource};
use ndarray::ArrayD;
use tempfile::tempdir;
use futures::StreamExt;

#[tokio::test]
async fn test_read_write_zarr() {
    let test_dir = tempdir().unwrap();
    let test_path = test_dir.path().to_str().unwrap();
    let source = ZarrSource::from_path_str(test_path).unwrap();

    // Test data to write
    let data: Vec<f32> = (0..100).map(|x| x as f32).collect();
    let array = ArrayD::from_shape_vec(vec![10, 10], data).unwrap();

    // Test writing
    let writer = ZarrWriter::new(source.clone()).await.unwrap();
    writer.write_array("test_array", &array, Some(vec![10, 10]), None).await.unwrap();
    
    // Test reading
    let reader = ZarrReader::new(source.clone()).await.unwrap();
    let loaded_array = reader.read_array("test_array").await.unwrap();
    assert_eq!(loaded_array, array);
}

#[tokio::test]
async fn test_lazy_loading() {
    let test_dir = tempdir().unwrap();
    let test_path = test_dir.path().to_str().unwrap();
    let source = ZarrSource::from_path_str(test_path).unwrap();

    // Test data to write
    let data: Vec<f32> = (0..100).map(|x| x as f32).collect();
    let array = ArrayD::from_shape_vec(vec![10, 10], data.clone()).unwrap();

    // Test writing
    let writer = ZarrWriter::new(source.clone()).await.unwrap();
    writer.write_array("lazy_array", &array, Some(vec![10, 10]), None).await.unwrap();

    // Test lazy loading
    let reader = ZarrReader::new(source.clone()).await.unwrap();
    let mut lazy_array = reader.lazy_load_array("lazy_array").await.unwrap();
    assert!(!lazy_array.is_loaded());

    let loaded_array = lazy_array.load().await.unwrap();
    assert_eq!(loaded_array, array);
    assert!(lazy_array.is_loaded());
}

#[tokio::test]
async fn test_stream_loading() {
    let test_dir = tempdir().unwrap();
    let test_path = test_dir.path().to_str().unwrap();
    let source = ZarrSource::from_path_str(test_path).unwrap();

    // Test data to write
    let data: Vec<f32> = (0..100).map(|x| x as f32).collect();
    let array = ArrayD::from_shape_vec(vec![10, 10], data.clone()).unwrap();

    // Test writing
    let writer = ZarrWriter::new(source.clone()).await.unwrap();
    writer.write_array("stream_array", &array, Some(vec![10, 10]), None).await.unwrap();

    // Test streaming
    let reader = ZarrReader::new(source.clone()).await.unwrap();
    let mut stream = reader.stream_chunks("stream_array");

    let mut collected_data = Vec::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.unwrap();
        collected_data.extend_from_slice(chunk.as_slice().unwrap());
    }

    assert_eq!(collected_data, data);
}

