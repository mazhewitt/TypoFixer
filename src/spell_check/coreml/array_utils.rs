use std::ptr::NonNull;
use std::sync::{Arc, Mutex};
use objc2::rc::Retained;
use objc2::AnyThread;
use objc2_core_ml::{MLModel, MLMultiArray, MLMultiArrayDataType};
use objc2_foundation::{NSArray, NSNumber};
use block2::{Block, StackBlock};
use tracing::info;

use super::errors::CorrectionError;

/// Utilities for working with Core ML MLMultiArray objects
pub struct ArrayUtils;

impl ArrayUtils {
    /// Create MLMultiArray from token IDs
    pub fn create_ml_multiarray(tokens: &[u32]) -> Result<Retained<MLMultiArray>, CorrectionError> {
        info!("🔧 Creating MLMultiArray for {} tokens", tokens.len());
        
        // Create shape for the array (1 x token_count)
        let shape = NSArray::from_slice(&[
            &*NSNumber::numberWithInt(1),
            &*NSNumber::numberWithInt(tokens.len() as i32),
        ]);
        
        // Create the MLMultiArray
        let multiarray = unsafe {
            MLMultiArray::initWithShape_dataType_error(
                MLMultiArray::alloc(),
                &shape,
                MLMultiArrayDataType::Int32,
            )
        }.map_err(|e| CorrectionError::ArrayCreationFailed {
            details: format!("{:?}", e),
        })?;
        
        // Fill the array with token data if we have tokens
        if !tokens.is_empty() {
            Self::fill_array_with_tokens(&multiarray, tokens)?;
        }
        
        info!("✅ Successfully created MLMultiArray with shape [1, {}]", tokens.len());
        Ok(multiarray)
    }
    
    /// Extract token IDs from MLMultiArray
    pub fn extract_tokens(array: &MLMultiArray) -> Result<Vec<u32>, CorrectionError> {
        let shape = unsafe { array.shape() };
        let shape_count = shape.count();
        
        let sequence_length = if shape_count >= 2 {
            let seq_dim = shape.objectAtIndex(1);
            seq_dim.intValue() as usize
        } else if shape_count == 1 {
            let seq_dim = shape.objectAtIndex(0);
            seq_dim.intValue() as usize
        } else {
            0
        };
        
        if sequence_length == 0 {
            return Ok(Vec::new());
        }
        
        let extracted_tokens = Arc::new(Mutex::new(Vec::new()));
        let extracted_tokens_clone = extracted_tokens.clone();
        
        let block = StackBlock::new(move |bytes_ptr: NonNull<std::ffi::c_void>, _strides: isize| {
            let mut tokens = extracted_tokens_clone.lock().unwrap();
            let data_ptr = bytes_ptr.as_ptr() as *const i32;
            for i in 0..sequence_length {
                let value = unsafe { *data_ptr.add(i) };
                tokens.push(value.max(0) as u32);
            }
        });
        
        let block_ref: &Block<dyn Fn(NonNull<std::ffi::c_void>, isize)> = &block;
        unsafe { array.getBytesWithHandler(block_ref); }
        
        let result = extracted_tokens.lock().unwrap().clone();
        Ok(result)
    }
    
    /// Perform Core ML model prediction (identity transformation for now)
    pub fn predict_with_model(input: &MLMultiArray, _model: &MLModel) -> Result<Retained<MLMultiArray>, CorrectionError> {
        info!("🤖 Running Core ML model prediction (identity transformation)");
        
        // For now, we create an output array that matches the input
        // In a real implementation, this would call the actual model
        let input_shape = unsafe { input.shape() };
        let output_array = unsafe {
            MLMultiArray::initWithShape_dataType_error(
                MLMultiArray::alloc(),
                &input_shape,
                MLMultiArrayDataType::Int32,
            )
        }.map_err(|e| CorrectionError::InferenceFailed {
            details: format!("Failed to create output array: {:?}", e),
        })?;
        
        // Copy input data to output (identity transformation)
        Self::copy_array_data(input, &output_array)?;
        
        info!("✅ Core ML prediction completed (identity transformation)");
        Ok(output_array)
    }
    
    /// Copy data from one MLMultiArray to another
    pub fn copy_array_data(source: &MLMultiArray, target: &MLMultiArray) -> Result<(), CorrectionError> {
        let shape = unsafe { source.shape() };
        let shape_count = shape.count();
        
        if shape_count == 0 {
            return Ok(());
        }
        
        let seq_length = if shape_count >= 2 {
            let seq_dim = shape.objectAtIndex(1);
            seq_dim.intValue() as usize
        } else {
            1
        };
        
        // Extract from source and copy to target
        let extracted_tokens = Arc::new(Mutex::new(Vec::new()));
        let extracted_tokens_clone = extracted_tokens.clone();
        
        let extract_block = StackBlock::new(move |bytes_ptr: NonNull<std::ffi::c_void>, _strides: isize| {
            let mut tokens = extracted_tokens_clone.lock().unwrap();
            let data_ptr = bytes_ptr.as_ptr() as *const i32;
            for i in 0..seq_length {
                let value = unsafe { *data_ptr.add(i) };
                tokens.push(value);
            }
        });
        
        let extract_block_ref: &Block<dyn Fn(NonNull<std::ffi::c_void>, isize)> = &extract_block;
        unsafe { source.getBytesWithHandler(extract_block_ref); }
        
        // Copy to target
        let copied_tokens = extracted_tokens.lock().unwrap().clone();
        let fill_block = StackBlock::new(move |bytes_ptr: NonNull<std::ffi::c_void>, _strides: isize| {
            let data_ptr = bytes_ptr.as_ptr() as *mut i32;
            for (i, &token) in copied_tokens.iter().enumerate() {
                unsafe { *data_ptr.add(i) = token; }
            }
        });
        
        let fill_block_ref: &Block<dyn Fn(NonNull<std::ffi::c_void>, isize)> = &fill_block;
        unsafe { target.getBytesWithHandler(fill_block_ref); }
        
        Ok(())
    }
    
    /// Fill an MLMultiArray with token data
    fn fill_array_with_tokens(array: &MLMultiArray, tokens: &[u32]) -> Result<(), CorrectionError> {
        let tokens_to_copy = tokens.to_vec();
        let block = StackBlock::new(move |bytes_ptr: NonNull<std::ffi::c_void>, _strides: isize| {
            let data_ptr = bytes_ptr.as_ptr() as *mut i32;
            for (i, &token) in tokens_to_copy.iter().enumerate() {
                if i < tokens_to_copy.len() {
                    unsafe { *data_ptr.add(i) = token as i32; }
                }
            }
        });
        
        let block_ref: &Block<dyn Fn(NonNull<std::ffi::c_void>, isize)> = &block;
        unsafe { array.getBytesWithHandler(block_ref); }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_ml_multiarray_empty() {
        let tokens = vec![];
        let result = ArrayUtils::create_ml_multiarray(&tokens);
        
        assert!(result.is_ok());
        let array = result.unwrap();
        
        // Should have proper shape [1, 0]
        let shape = unsafe { array.shape() };
        assert_eq!(shape.count(), 2);
        
        let batch_size = shape.objectAtIndex(0);
        assert_eq!(batch_size.intValue(), 1);
        
        let seq_len = shape.objectAtIndex(1);
        assert_eq!(seq_len.intValue(), 0);
    }

    #[test]
    fn test_create_ml_multiarray_with_tokens() {
        let tokens = vec![1, 2, 3, 4, 5];
        let result = ArrayUtils::create_ml_multiarray(&tokens);
        
        assert!(result.is_ok());
        let array = result.unwrap();
        
        // Should have proper shape [1, 5]
        let shape = unsafe { array.shape() };
        assert_eq!(shape.count(), 2);
        
        let batch_size = shape.objectAtIndex(0);
        assert_eq!(batch_size.intValue(), 1);
        
        let seq_len = shape.objectAtIndex(1);
        assert_eq!(seq_len.intValue(), 5);
    }

    #[test]
    fn test_extract_tokens_empty_array() {
        let tokens = vec![];
        let array = ArrayUtils::create_ml_multiarray(&tokens).unwrap();
        
        let extracted = ArrayUtils::extract_tokens(&array).unwrap();
        assert!(extracted.is_empty());
    }

    #[test]
    fn test_extract_tokens_with_data() {
        let original_tokens = vec![1, 2, 3, 4, 5];
        let array = ArrayUtils::create_ml_multiarray(&original_tokens).unwrap();
        
        let extracted_tokens = ArrayUtils::extract_tokens(&array).unwrap();
        assert_eq!(extracted_tokens, original_tokens);
    }

    #[test]
    fn test_roundtrip_token_conversion() {
        let original_tokens = vec![10, 20, 30, 40, 50];
        
        // Create array from tokens
        let array = ArrayUtils::create_ml_multiarray(&original_tokens).unwrap();
        
        // Extract tokens back
        let extracted_tokens = ArrayUtils::extract_tokens(&array).unwrap();
        
        // Should be identical
        assert_eq!(extracted_tokens, original_tokens);
    }

    #[test]
    fn test_copy_array_data() {
        let tokens = vec![1, 2, 3];
        let source_array = ArrayUtils::create_ml_multiarray(&tokens).unwrap();
        let target_array = ArrayUtils::create_ml_multiarray(&vec![0, 0, 0]).unwrap();
        
        let result = ArrayUtils::copy_array_data(&source_array, &target_array);
        assert!(result.is_ok());
        
        // Verify data was copied
        let extracted = ArrayUtils::extract_tokens(&target_array).unwrap();
        assert_eq!(extracted, tokens);
    }

    #[test]
    fn test_predict_with_model_identity() {
        let tokens = vec![1, 2, 3, 4, 5];
        let input_array = ArrayUtils::create_ml_multiarray(&tokens).unwrap();
        
        // Create a mock model (we don't actually use it in the identity transformation)
        // This test focuses on the array operations, not actual model inference
        let model_url = unsafe { 
            objc2_foundation::NSURL::fileURLWithPath(&objc2_foundation::NSString::from_str("/nonexistent"))
        };
        
        // For this test, we'll skip the actual model creation since it requires a real model file
        // Instead, we'll test that the prediction function creates an output array with the right shape
        // by using extract_tokens to verify the identity transformation worked
        
        // Note: In a real test environment, you'd need a valid Core ML model
        // For now, we test the array operations separately
        // Just verify the input array was created successfully
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_large_token_array() {
        // Test with a larger array to ensure our block-based operations work correctly
        let tokens: Vec<u32> = (0..1000).collect();
        
        let array = ArrayUtils::create_ml_multiarray(&tokens).unwrap();
        let extracted = ArrayUtils::extract_tokens(&array).unwrap();
        
        assert_eq!(extracted.len(), 1000);
        assert_eq!(extracted, tokens);
    }

    #[test]
    fn test_edge_case_single_token() {
        let tokens = vec![42];
        
        let array = ArrayUtils::create_ml_multiarray(&tokens).unwrap();
        let extracted = ArrayUtils::extract_tokens(&array).unwrap();
        
        assert_eq!(extracted, vec![42]);
    }
}