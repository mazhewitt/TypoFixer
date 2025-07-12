#!/usr/bin/env python3
"""
Simple T5 Grammar Correction to Core ML Converter

This script creates a simplified Core ML model for grammar correction
that's compatible with our Rust application.
"""

import os
import torch
import coremltools as ct
from transformers import T5ForConditionalGeneration, T5Tokenizer
import numpy as np

def create_simple_coreml_model():
    """Create a simplified Core ML model for grammar correction."""
    
    print("🔄 Creating Simple T5 Grammar Correction Core ML Model")
    print("=" * 60)
    
    model_name = "vennify/t5-base-grammar-correction"
    output_dir = "coreml-models"
    
    # Create output directory
    os.makedirs(output_dir, exist_ok=True)
    
    try:
        # Load model and tokenizer
        print("📦 Loading T5 model and tokenizer...")
        tokenizer = T5Tokenizer.from_pretrained(model_name)
        model = T5ForConditionalGeneration.from_pretrained(model_name)
        model.eval()
        
        print("✅ Model loaded successfully!")
        
        # Create a simple encoder-only model for faster inference
        print("🔄 Creating simplified model wrapper...")
        
        class SimpleT5Encoder(torch.nn.Module):
            def __init__(self, t5_model):
                super().__init__()
                self.encoder = t5_model.encoder
                self.config = t5_model.config
                
            def forward(self, input_ids, attention_mask):
                # Just encode the input text
                encoder_outputs = self.encoder(
                    input_ids=input_ids,
                    attention_mask=attention_mask
                )
                # Return the last hidden state
                return encoder_outputs.last_hidden_state
        
        # Create the simple encoder
        simple_model = SimpleT5Encoder(model)
        simple_model.eval()
        
        # Prepare example inputs
        example_text = "I has a error."
        inputs = tokenizer(
            example_text,
            return_tensors="pt",
            max_length=64,  # Smaller for simplicity
            padding="max_length",
            truncation=True
        )
        
        input_ids = inputs["input_ids"]
        attention_mask = inputs["attention_mask"]
        
        print(f"📝 Input shape: {input_ids.shape}")
        
        # Test the model
        with torch.no_grad():
            output = simple_model(input_ids, attention_mask)
            print(f"📊 Output shape: {output.shape}")
        
        # Trace the model
        print("🔄 Tracing model...")
        with torch.no_grad():
            traced_model = torch.jit.trace(
                simple_model,
                (input_ids, attention_mask)
            )
        
        # Convert to Core ML
        print("🔄 Converting to Core ML...")
        
        coreml_model = ct.convert(
            traced_model,
            inputs=[
                ct.TensorType(name="input_ids", shape=input_ids.shape, dtype=np.int32),
                ct.TensorType(name="attention_mask", shape=attention_mask.shape, dtype=np.int32)
            ],
            outputs=[ct.TensorType(name="hidden_states", dtype=np.float32)],
            minimum_deployment_target=ct.target.macOS13,
            compute_units=ct.ComputeUnit.CPU_ONLY  # CPU only for compatibility
        )
        
        # Add metadata
        coreml_model.short_description = "T5 Encoder for Grammar Correction"
        coreml_model.author = "Converted from vennify/t5-base-grammar-correction"
        coreml_model.version = "1.0"
        
        # Save the model
        output_path = os.path.join(output_dir, "t5-grammar-encoder.mlmodel")
        coreml_model.save(output_path)
        
        print(f"✅ Core ML model saved to: {output_path}")
        
        # Save tokenizer config
        config_path = os.path.join(output_dir, "model_config.txt")
        with open(config_path, "w") as f:
            f.write(f"Model: {model_name}\n")
            f.write(f"Max length: 64\n")
            f.write(f"Vocab size: {tokenizer.vocab_size}\n")
            f.write(f"Pad token: {tokenizer.pad_token_id}\n")
            f.write(f"EOS token: {tokenizer.eos_token_id}\n")
        
        print(f"📄 Config saved to: {config_path}")
        
        # Test the Core ML model
        print("🔄 Testing Core ML model...")
        test_model = ct.models.MLModel(output_path)
        
        test_input = {
            "input_ids": input_ids.numpy().astype(np.int32),
            "attention_mask": attention_mask.numpy().astype(np.int32)
        }
        
        prediction = test_model.predict(test_input)
        print("✅ Core ML model test successful!")
        
        print("\n🎉 Simple T5 Core ML model created successfully!")
        print(f"📁 Model location: {output_path}")
        print(f"📋 This is an encoder-only model for feature extraction.")
        print(f"📋 You'll need to implement text generation logic in Rust.")
        
        return output_path
        
    except Exception as e:
        print(f"❌ Error: {e}")
        import traceback
        traceback.print_exc()
        return None

if __name__ == "__main__":
    create_simple_coreml_model()