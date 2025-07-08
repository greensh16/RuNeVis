//! Parallel computation implementations for statistical operations
//!
//! This module contains the actual parallel computation logic for statistical reductions.

use crate::errors::Result;
use ndarray::{ArrayD, Axis};
use rayon::prelude::*;

/// Computes mean along an axis using parallel processing
///
/// This function converts data to f64 for computation to avoid precision loss,
/// then uses parallel processing to compute the mean along the specified axis.
///
/// # Errors
///
/// Returns an error if array reshaping fails or if the axis is invalid.
pub fn parallel_mean_axis(data: &ArrayD<f32>, axis: usize) -> Result<ArrayD<f32>> {
    // Convert f32 data to f64 for computation to avoid precision loss
    let data_f64: Vec<f64> = data.iter().map(|&x| f64::from(x)).collect();
    let data_f64_array = ArrayD::from_shape_vec(data.raw_dim(), data_f64)?;

    let original_shape = data.shape();
    let axis_len = original_shape[axis];

    // Use reduce with a custom mean operation that tracks count
    let mut new_shape = original_shape.to_vec();
    new_shape.remove(axis);
    let output_size: usize = new_shape.iter().product();

    println!(
        "⚡ Processing {output_size} elements across {} CPU cores",
        rayon::current_num_threads()
    );

    // Create output vector for parallel computation with mean calculation
    let result: Vec<f32> = (0..output_size)
        .into_par_iter()
        .map(|flat_idx| {
            // Convert flat index back to multi-dimensional coordinates
            let mut coords = vec![0; original_shape.len()];
            let mut remaining = flat_idx;

            // Fill coordinates, skipping the axis we're averaging over
            let mut coord_idx = 0;
            for (dim_idx, &_dim_size) in original_shape.iter().enumerate() {
                if dim_idx != axis {
                    let stride = new_shape[coord_idx + 1..].iter().product::<usize>();
                    coords[dim_idx] = remaining / stride;
                    remaining %= stride;
                    coord_idx += 1;
                }
            }

            // Compute mean along the specified axis
            let mut sum = 0.0_f64;
            let mut count = 0_i32;

            for i in 0..axis_len {
                coords[axis] = i;
                if let Some(value) = data_f64_array.get(coords.as_slice()) {
                    if value.is_finite() {
                        // Skip NaN and infinite values
                        sum += value;
                        count += 1;
                    }
                }
            }

            if count > 0 {
                #[allow(clippy::cast_possible_truncation)]
                {
                    (sum / f64::from(count)) as f32
                }
            } else {
                f32::NAN // Return NaN if all values were invalid
            }
        })
        .collect();

    // Reshape the result back to the expected dimensions
    Ok(ArrayD::from_shape_vec(new_shape, result)?)
}

/// Computes sum along an axis using ndarray's parallel `fold_axis` for better performance
///
/// # Errors
///
/// Returns an error if the axis is invalid.
pub fn parallel_sum_axis(data: &ArrayD<f32>, axis: usize) -> Result<ArrayD<f32>> {
    // Use ndarray's parallel fold_axis for optimal performance
    let axis_obj = Axis(axis);
    let result = data.fold_axis(axis_obj, 0.0_f32, |&acc, &x| {
        if x.is_finite() {
            acc + x
        } else {
            acc // Skip NaN and infinite values
        }
    });

    Ok(result.into_dyn())
}

/// Computes minimum along an axis using ndarray's parallel `fold_axis` for better performance
///
/// # Errors
///
/// Returns an error if the axis is invalid.
pub fn parallel_min_axis(data: &ArrayD<f32>, axis: usize) -> Result<ArrayD<f32>> {
    // Use ndarray's parallel fold_axis for optimal performance
    let axis_obj = Axis(axis);
    let result = data.fold_axis(axis_obj, f32::INFINITY, |&acc, &x| {
        if x.is_finite() {
            acc.min(x)
        } else {
            acc // Skip NaN and infinite values
        }
    });

    // Convert INFINITY to NaN where no valid values were found
    let final_result = result.mapv(|x| if x == f32::INFINITY { f32::NAN } else { x });
    Ok(final_result.into_dyn())
}

/// Computes maximum along an axis using ndarray's parallel `fold_axis` for better performance
///
/// # Errors
///
/// Returns an error if the axis is invalid.
pub fn parallel_max_axis(data: &ArrayD<f32>, axis: usize) -> Result<ArrayD<f32>> {
    // Use ndarray's parallel fold_axis for optimal performance
    let axis_obj = Axis(axis);
    let result = data.fold_axis(axis_obj, f32::NEG_INFINITY, |&acc, &x| {
        if x.is_finite() {
            acc.max(x)
        } else {
            acc // Skip NaN and infinite values
        }
    });

    // Convert NEG_INFINITY to NaN where no valid values were found
    let final_result = result.mapv(|x| if x == f32::NEG_INFINITY { f32::NAN } else { x });
    Ok(final_result.into_dyn())
}
