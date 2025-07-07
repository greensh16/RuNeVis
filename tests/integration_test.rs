use ndarray::Array3;
use netcdf::{create, open};
use tempfile::tempdir;
use RuNeVis::statistics::{reduce_max, reduce_min};

#[test]
fn test_reduce_min_max_integration() {
    // Create a temporary NetCDF file for testing
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_data.nc");

    // Create test data - a 3D array (2x3x4)
    let test_data: Vec<f32> = vec![
        // First 2D slice [0,:,:]
        1.0, 2.0, 3.0, 4.0, // [0,0,:]
        5.0, 6.0, 7.0, 8.0, // [0,1,:]
        9.0, 10.0, 11.0, 12.0, // [0,2,:]
        // Second 2D slice [1,:,:]
        13.0, 14.0, 15.0, 16.0, // [1,0,:]
        17.0, 18.0, 19.0, 20.0, // [1,1,:]
        21.0, 22.0, 23.0, 24.0, // [1,2,:]
    ];

    // Create NetCDF file with test data
    {
        let mut file = create(&file_path).expect("Failed to create NetCDF file");

        // Define dimensions
        file.add_dimension("x", 2)
            .expect("Failed to add dimension x");
        file.add_dimension("y", 3)
            .expect("Failed to add dimension y");
        file.add_dimension("z", 4)
            .expect("Failed to add dimension z");

        // Create variable
        let mut var = file
            .add_variable::<f32>("test_var", &["x", "y", "z"])
            .expect("Failed to add variable");

        // Write data
        let data_array = Array3::from_shape_vec((2, 3, 4), test_data.clone())
            .expect("Failed to create array from test data");
        var.put(data_array.view(), ..)
            .expect("Failed to write data");
    }

    // Open file for reading and test reduce functions
    let file = open(&file_path).expect("Failed to open NetCDF file");
    let var = file.variable("test_var").expect("Variable not found");

    // Test reduce_min along different dimensions

    // Reduce along dimension "x" (axis 0) - should result in 3x4 array
    let min_result_x = reduce_min(&var, "x").expect("Failed to reduce min along x");
    assert_eq!(min_result_x.shape(), &[3, 4]);

    // Expected minimums along x dimension:
    // [0,0,:] vs [1,0,:] -> min([1,2,3,4], [13,14,15,16]) = [1,2,3,4]
    // [0,1,:] vs [1,1,:] -> min([5,6,7,8], [17,18,19,20]) = [5,6,7,8]
    // [0,2,:] vs [1,2,:] -> min([9,10,11,12], [21,22,23,24]) = [9,10,11,12]
    assert_eq!(min_result_x[[0, 0]], 1.0);
    assert_eq!(min_result_x[[0, 1]], 2.0);
    assert_eq!(min_result_x[[1, 0]], 5.0);
    assert_eq!(min_result_x[[2, 3]], 12.0);

    // Reduce along dimension "y" (axis 1) - should result in 2x4 array
    let min_result_y = reduce_min(&var, "y").expect("Failed to reduce min along y");
    assert_eq!(min_result_y.shape(), &[2, 4]);

    // Expected minimums along y dimension:
    // For x=0: min([1,2,3,4], [5,6,7,8], [9,10,11,12]) = [1,2,3,4]
    // For x=1: min([13,14,15,16], [17,18,19,20], [21,22,23,24]) = [13,14,15,16]
    assert_eq!(min_result_y[[0, 0]], 1.0);
    assert_eq!(min_result_y[[0, 3]], 4.0);
    assert_eq!(min_result_y[[1, 0]], 13.0);
    assert_eq!(min_result_y[[1, 3]], 16.0);

    // Test reduce_max along different dimensions

    // Reduce along dimension "z" (axis 2) - should result in 2x3 array
    let max_result_z = reduce_max(&var, "z").expect("Failed to reduce max along z");
    assert_eq!(max_result_z.shape(), &[2, 3]);

    // Expected maximums along z dimension:
    // For [0,0,:]: max([1,2,3,4]) = 4
    // For [0,1,:]: max([5,6,7,8]) = 8
    // For [0,2,:]: max([9,10,11,12]) = 12
    // For [1,0,:]: max([13,14,15,16]) = 16
    // For [1,1,:]: max([17,18,19,20]) = 20
    // For [1,2,:]: max([21,22,23,24]) = 24
    assert_eq!(max_result_z[[0, 0]], 4.0);
    assert_eq!(max_result_z[[0, 1]], 8.0);
    assert_eq!(max_result_z[[0, 2]], 12.0);
    assert_eq!(max_result_z[[1, 0]], 16.0);
    assert_eq!(max_result_z[[1, 1]], 20.0);
    assert_eq!(max_result_z[[1, 2]], 24.0);

    println!("✅ Integration test passed: reduce_min and reduce_max work correctly with NetCDF variables!");
}
