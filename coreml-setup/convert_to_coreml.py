#!/usr/bin/env python3
"""
Convert T5-Efficient-Tiny-Grammar-Correction to Core ML format
for use with Apple's Neural Engine on macOS.
"""

from pathlib import Path
import torch
import coremltools as ct
from transformers import AutoTokenizer, AutoModelForSeq2SeqLM

def main():
    MODEL_ID = "visheratin/t5-efficient-tiny-grammar-correction"
    OUT = Path("Models")
    SEQ_LEN = 128

    print(f"🚀 Converting {MODEL_ID} to Core ML...")
    print(f"📁 Output directory: {OUT}")
    print(f"📏 Sequence length: {SEQ_LEN}")

    # Load tokenizer and model
    print("📥 Loading tokenizer...")
    tok = AutoTokenizer.from_pretrained(MODEL_ID)
    
    print("📥 Loading model...")
    model = AutoModelForSeq2SeqLM.from_pretrained(
        MODEL_ID, 
        torch_dtype=torch.float16
    )
    model.eval()

    # Create dummy input for tracing
    print("🔍 Creating dummy input for tracing...")
    dummy_text = "Hello world this is a test sentence"
    dummy = tok(dummy_text, return_tensors="pt", max_length=SEQ_LEN, padding="max_length", truncation=True).input_ids
    print(f"Dummy input shape: {dummy.shape}")

    # Trace the model
    print("📝 Tracing the model...")
    with torch.no_grad():
        traced = torch.jit.trace(model, (dummy,))

    # Convert to Core ML
    print("🔄 Converting to Core ML...")
    mlmodel = ct.convert(
        traced,
        convert_to="mlprogram",
        compute_units=ct.ComputeUnit.CPU_AND_NE,  # Use CPU and Neural Engine
        minimum_deployment_target=ct.target.iOS17,
        inputs=[ct.TensorType(
            name="input_ids", 
            shape=(1, SEQ_LEN), 
            dtype=ct.int32
        )]
    )

    # Compress the model
    print("🗜️  Compressing model (FP16 to INT4)...")
    mlmodel = ct.compress_fp16_to_int4(mlmodel)

    # Create output directory
    OUT.mkdir(exist_ok=True)
    
    # Save model
    model_path = OUT / "t5_tiny_grammar.mlmodel"
    print(f"💾 Saving Core ML model to: {model_path}")
    mlmodel.save(str(model_path))

    # Save tokenizer
    print(f"💾 Saving tokenizer to: {OUT}")
    tok.save_pretrained(str(OUT))

    print("✅ Conversion complete!")
    print(f"📊 Model size: {model_path.stat().st_size / 1024 / 1024:.1f} MB")
    print()
    print("Next steps:")
    print(f"1. Compile the model: xcrun coremlc compile {model_path} ModelsCompiled")
    print("2. Add Core ML dependencies to Cargo.toml")
    print("3. Implement Rust wrapper")

if __name__ == "__main__":
    main()