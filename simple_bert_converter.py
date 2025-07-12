#!/usr/bin/env python3
"""
Simple BERT Grammar Correction to Core ML Converter

This script creates a Core ML model using a simpler BERT-based approach
that doesn't require SentencePiece.
"""

import os
import torch
import coremltools as ct
from transformers import AutoTokenizer, AutoModelForSequenceClassification
import numpy as np

def create_bert_coreml_model():
    """Create a BERT-based Core ML model for text correction."""
    
    print("🔄 Creating BERT-based Grammar Correction Core ML Model")
    print("=" * 60)
    
    # Use a grammar correction model that doesn't require SentencePiece
    model_name = "grammarly/coedit-large"  # Alternative model
    backup_model = "textattack/bert-base-uncased-CoLA"  # Grammar acceptability
    
    output_dir = "coreml-models"
    os.makedirs(output_dir, exist_ok=True)
    
    try:
        print(f"📦 Trying to load model: {backup_model}")
        
        # Load tokenizer and model
        tokenizer = AutoTokenizer.from_pretrained(backup_model)
        model = AutoModelForSequenceClassification.from_pretrained(backup_model)
        model.eval()
        
        print("✅ Model and tokenizer loaded successfully!")
        
        # Create example inputs
        example_text = "I has a error in this sentence."
        inputs = tokenizer(
            example_text,
            return_tensors="pt",
            max_length=128,
            padding="max_length",
            truncation=True
        )
        
        input_ids = inputs["input_ids"]
        attention_mask = inputs["attention_mask"]
        
        print(f"📝 Input shape: {input_ids.shape}")
        
        # Create a simple wrapper for binary classification (grammatical/not grammatical)
        class BERTGrammarChecker(torch.nn.Module):
            def __init__(self, model):
                super().__init__()
                self.model = model
                
            def forward(self, input_ids, attention_mask):
                outputs = self.model(input_ids=input_ids, attention_mask=attention_mask)
                # Return logits for grammatical acceptability
                return outputs.logits
        
        # Create wrapper
        wrapped_model = BERTGrammarChecker(model)
        wrapped_model.eval()
        
        # Test the model
        with torch.no_grad():
            test_output = wrapped_model(input_ids, attention_mask)
            print(f"📊 Output shape: {test_output.shape}")
            print(f"📊 Output logits: {test_output}")
        
        # Trace the model
        print("🔄 Tracing model...")
        with torch.no_grad():
            traced_model = torch.jit.trace(
                wrapped_model,
                (input_ids, attention_mask)
            )
        
        print("✅ Model traced successfully!")
        
        # Convert to Core ML
        print("🔄 Converting to Core ML...")
        
        coreml_model = ct.convert(
            traced_model,
            inputs=[
                ct.TensorType(name="input_ids", shape=input_ids.shape, dtype=np.int32),
                ct.TensorType(name="attention_mask", shape=attention_mask.shape, dtype=np.int32)
            ],
            outputs=[ct.TensorType(name="grammar_scores", dtype=np.float32)],
            minimum_deployment_target=ct.target.macOS13,
            compute_units=ct.ComputeUnit.CPU_ONLY
        )
        
        # Add metadata
        coreml_model.short_description = "BERT Grammar Checker"
        coreml_model.author = f"Converted from {backup_model}"
        coreml_model.version = "1.0"
        
        # Save the model
        output_path = os.path.join(output_dir, "bert-grammar-checker.mlmodel")
        coreml_model.save(output_path)
        
        print(f"✅ Core ML model saved to: {output_path}")
        
        # Save tokenizer info
        config_path = os.path.join(output_dir, "bert_model_config.txt")
        with open(config_path, "w") as f:
            f.write(f"Model: {backup_model}\n")
            f.write(f"Task: Grammar acceptability classification\n")
            f.write(f"Max length: 128\n")
            f.write(f"Vocab size: {tokenizer.vocab_size}\n")
            f.write(f"Output: Binary classification (grammatical/ungrammatical)\n")
            f.write(f"Threshold: Use sigmoid and threshold at 0.5\n")
        
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
        print(f"📊 Prediction output: {prediction}")
        
        print("\n🎉 BERT Grammar Checker Core ML model created successfully!")
        print(f"📁 Model location: {output_path}")
        print(f"📋 This model classifies text as grammatical or ungrammatical.")
        print(f"📋 You can use this to detect errors and suggest corrections.")
        
        return output_path
        
    except Exception as e:
        print(f"❌ Error: {e}")
        import traceback
        traceback.print_exc()
        return None

if __name__ == "__main__":
    create_bert_coreml_model()