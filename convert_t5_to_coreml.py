#!/usr/bin/env python3
"""
Convert T5 Grammar Correction Model to Core ML Format

This script converts the vennify/t5-base-grammar-correction model from Hugging Face
to Core ML format for use in our macOS typo correction application.
"""

import os
import sys
import torch
import coremltools as ct
from transformers import T5ForConditionalGeneration, T5Tokenizer
import numpy as np

def download_and_convert_model():
    """Download T5 grammar correction model and convert to Core ML format."""
    
    print("🔄 Starting T5 Grammar Correction Model Conversion to Core ML")
    print("=" * 60)
    
    # Model configuration
    model_name = "vennify/t5-base-grammar-correction"
    output_dir = "coreml-models"
    output_file = "t5-grammar-correction.mlmodel"
    
    print(f"📦 Model: {model_name}")
    print(f"📁 Output directory: {output_dir}")
    print(f"📄 Output file: {output_file}")
    
    # Create output directory
    os.makedirs(output_dir, exist_ok=True)
    
    try:
        # Step 1: Load the model and tokenizer
        print("\n🔄 Step 1: Loading T5 model and tokenizer...")
        tokenizer = T5Tokenizer.from_pretrained(model_name)
        model = T5ForConditionalGeneration.from_pretrained(model_name)
        
        # Set model to evaluation mode
        model.eval()
        print("✅ Model and tokenizer loaded successfully!")
        
        # Step 2: Create example inputs for tracing
        print("\n🔄 Step 2: Preparing model for conversion...")
        
        # Example input text for grammar correction
        example_text = "I has a error in this sentance."
        print(f"📝 Example input: '{example_text}'")
        
        # Tokenize input
        inputs = tokenizer(
            example_text,
            return_tensors="pt",
            max_length=128,
            padding="max_length",
            truncation=True
        )
        
        input_ids = inputs["input_ids"]
        attention_mask = inputs["attention_mask"]
        
        print(f"🔢 Input shape: {input_ids.shape}")
        print(f"🎯 Attention mask shape: {attention_mask.shape}")
        
        # Step 3: Create a simplified wrapper for Core ML conversion
        print("\n🔄 Step 3: Creating Core ML compatible wrapper...")
        
        class T5GrammarCorrectionWrapper(torch.nn.Module):
            def __init__(self, model, max_length=128):
                super().__init__()
                self.model = model
                self.max_length = max_length
                
            def forward(self, input_ids, attention_mask):
                # Generate corrected text
                with torch.no_grad():
                    outputs = self.model.generate(
                        input_ids=input_ids,
                        attention_mask=attention_mask,
                        max_length=self.max_length,
                        num_beams=4,
                        early_stopping=True,
                        do_sample=False
                    )
                return outputs
        
        # Create wrapper
        wrapped_model = T5GrammarCorrectionWrapper(model)
        wrapped_model.eval()
        
        # Step 4: Test the wrapper
        print("\n🔄 Step 4: Testing model wrapper...")
        with torch.no_grad():
            test_output = wrapped_model(input_ids, attention_mask)
            decoded_output = tokenizer.decode(test_output[0], skip_special_tokens=True)
            print(f"✅ Test output: '{decoded_output}'")
        
        # Step 5: Trace the model
        print("\n🔄 Step 5: Tracing model for Core ML conversion...")
        
        with torch.no_grad():
            traced_model = torch.jit.trace(
                wrapped_model,
                (input_ids, attention_mask),
                strict=False
            )
        
        print("✅ Model traced successfully!")
        
        # Step 6: Convert to Core ML
        print("\n🔄 Step 6: Converting to Core ML format...")
        
        # Define input types
        input_types = [
            ct.TensorType(name="input_ids", shape=input_ids.shape, dtype=np.int32),
            ct.TensorType(name="attention_mask", shape=attention_mask.shape, dtype=np.int32)
        ]
        
        # Convert to Core ML
        coreml_model = ct.convert(
            traced_model,
            inputs=input_types,
            outputs=[ct.TensorType(name="corrected_text_ids", dtype=np.int32)],
            minimum_deployment_target=ct.target.macOS13,
            compute_units=ct.ComputeUnit.ALL
        )
        
        # Add model metadata
        coreml_model.short_description = "T5-based grammar correction model"
        coreml_model.author = "Converted from vennify/t5-base-grammar-correction"
        coreml_model.license = "Apache 2.0"
        coreml_model.version = "1.0"
        
        # Step 7: Save the model
        output_path = os.path.join(output_dir, output_file)
        coreml_model.save(output_path)
        
        print(f"✅ Core ML model saved to: {output_path}")
        
        # Step 8: Test the Core ML model
        print("\n🔄 Step 8: Testing Core ML model...")
        
        # Load and test the saved model
        loaded_model = ct.models.MLModel(output_path)
        
        # Prepare test input
        test_input = {
            "input_ids": input_ids.numpy().astype(np.int32),
            "attention_mask": attention_mask.numpy().astype(np.int32)
        }
        
        # Make prediction
        prediction = loaded_model.predict(test_input)
        print("✅ Core ML model prediction successful!")
        
        print("\n🎉 Conversion completed successfully!")
        print("=" * 60)
        print(f"📁 Your Core ML model is ready at: {output_path}")
        print(f"📋 Model info:")
        print(f"   - Input: Tokenized text (input_ids, attention_mask)")
        print(f"   - Output: Corrected text token IDs")
        print(f"   - Max sequence length: 128 tokens")
        print(f"   - Compatible with: macOS 13+")
        
        # Create a simple tokenizer info file
        tokenizer_info_path = os.path.join(output_dir, "tokenizer_info.txt")
        with open(tokenizer_info_path, "w") as f:
            f.write(f"Tokenizer: {model_name}\n")
            f.write(f"Vocab size: {tokenizer.vocab_size}\n")
            f.write(f"Model max length: 128\n")
            f.write(f"Special tokens:\n")
            f.write(f"  - PAD: {tokenizer.pad_token_id}\n")
            f.write(f"  - EOS: {tokenizer.eos_token_id}\n")
            f.write(f"  - UNK: {tokenizer.unk_token_id}\n")
        
        print(f"📄 Tokenizer info saved to: {tokenizer_info_path}")
        
        return output_path
        
    except Exception as e:
        print(f"❌ Error during conversion: {str(e)}")
        print(f"📋 Error type: {type(e).__name__}")
        import traceback
        traceback.print_exc()
        return None

def install_requirements():
    """Install required packages for the conversion."""
    print("🔄 Installing required packages...")
    
    packages = [
        "torch",
        "transformers",
        "coremltools",
        "numpy"
    ]
    
    for package in packages:
        try:
            __import__(package.replace("-", "_"))
            print(f"✅ {package} is already installed")
        except ImportError:
            print(f"📦 Installing {package}...")
            os.system(f"{sys.executable} -m pip install {package}")

def main():
    """Main conversion process."""
    print("🚀 T5 Grammar Correction to Core ML Converter")
    print("=" * 60)
    
    # Check and install requirements
    try:
        install_requirements()
    except Exception as e:
        print(f"❌ Failed to install requirements: {e}")
        return
    
    # Convert the model
    model_path = download_and_convert_model()
    
    if model_path:
        print(f"\n🎉 SUCCESS!")
        print(f"Your Core ML model is ready at: {model_path}")
        print(f"\nNext steps:")
        print(f"1. Copy the .mlmodel file to your Rust project")
        print(f"2. Update the model path in your Rust code")
        print(f"3. Test the grammar correction functionality")
    else:
        print(f"\n❌ FAILED!")
        print(f"Conversion was not successful. Check the error messages above.")

if __name__ == "__main__":
    main()