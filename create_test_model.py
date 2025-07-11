#!/usr/bin/env python3
"""
Create a simple test Core ML model for build-time compilation testing.
This creates a minimal text processing model that can be compiled successfully.
"""

import coremltools as ct
import numpy as np
from coremltools.models.neural_network import NeuralNetworkBuilder
from coremltools.models import datatypes

def create_simple_text_corrector():
    """Create a minimal Core ML model for text correction testing."""
    
    # Create a simple neural network builder
    builder = NeuralNetworkBuilder(
        input_features=[("input_text", datatypes.Array(128))],  # 128-dim input
        output_features=[("corrected_text", datatypes.Array(128))],  # 128-dim output
        mode=None
    )
    
    # Add a simple linear layer (identity transformation for testing)
    builder.add_inner_product(
        name="linear",
        W=np.eye(128, dtype=np.float32),  # Identity matrix
        b=np.zeros(128, dtype=np.float32),  # Zero bias
        input_channels=128,
        output_channels=128,
        has_bias=True,
        input_name="input_text",
        output_name="corrected_text"
    )
    
    # Build the model
    model = ct.models.MLModel(builder.spec)
    
    # Set metadata
    model.short_description = "Simple text corrector for testing"
    model.input_description["input_text"] = "Tokenized input text (128 dimensions)"
    model.output_description["corrected_text"] = "Corrected text tokens (128 dimensions)"
    
    return model

def main():
    print("🔨 Creating simple test Core ML model...")
    
    try:
        # Create the model
        model = create_simple_text_corrector()
        
        # Save the model
        output_path = "test_model.mlpackage"
        model.save(output_path)
        
        print(f"✅ Test model saved to: {output_path}")
        print("📋 Model details:")
        print(f"   - Input: {model.input_description}")
        print(f"   - Output: {model.output_description}")
        
        # Verify the model can be loaded
        loaded_model = ct.models.MLModel(output_path)
        print("✅ Model can be loaded successfully")
        
    except Exception as e:
        print(f"❌ Failed to create test model: {e}")
        return 1
    
    return 0

if __name__ == "__main__":
    exit(main())