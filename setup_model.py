#!/usr/bin/env python3
"""
Script to download and compile the T5-Efficient-Tiny-Grammar-Correction model
for use with CoreML in the TypoFixer application.

This script automates the process described in Download instructions.md
"""

import os
import sys
import subprocess
import shutil
from pathlib import Path

def run_command(cmd, description="Running command"):
    """Run a shell command and handle errors"""
    print(f"\n🔄 {description}...")
    print(f"Command: {' '.join(cmd) if isinstance(cmd, list) else cmd}")
    
    result = subprocess.run(cmd, shell=isinstance(cmd, str), capture_output=True, text=True)
    
    if result.returncode != 0:
        print(f"❌ Error: {description} failed")
        print(f"stdout: {result.stdout}")
        print(f"stderr: {result.stderr}")
        return False
    
    print(f"✅ {description} completed successfully")
    if result.stdout.strip():
        print(f"Output: {result.stdout.strip()}")
    
    return True

def check_requirements():
    """Check if required tools are available"""
    print("🔍 Checking requirements...")
    
    # Check Python version
    if sys.version_info < (3, 10):
        print("❌ Python 3.10 or higher is required")
        return False
    print("✅ Python version is sufficient")
    
    # Check if we're on macOS
    if sys.platform != "darwin":
        print("❌ This script is designed for macOS only")
        return False
    print("✅ Running on macOS")
    
    # Check if xcrun is available
    if not shutil.which("xcrun"):
        print("❌ Xcode command line tools not found. Please install with: xcode-select --install")
        return False
    print("✅ Xcode command line tools found")
    
    return True

def setup_virtual_environment():
    """Create and activate virtual environment"""
    venv_path = Path(".venv")
    
    if venv_path.exists():
        print("✅ Virtual environment already exists")
    else:
        if not run_command([sys.executable, "-m", "venv", ".venv"], "Creating virtual environment"):
            return False
    
    # Return the path to the Python executable in the venv
    if sys.platform == "win32":
        python_exe = venv_path / "Scripts" / "python"
    else:
        python_exe = venv_path / "bin" / "python"
    
    return str(python_exe)

def install_dependencies(python_exe):
    """Install required Python packages"""
    packages = [
        "coremltools==8.3",
        "torch", 
        "transformers",
        "tokenizers",
        "accelerate"
    ]
    
    for package in packages:
        if not run_command([python_exe, "-m", "pip", "install", package], f"Installing {package}"):
            return False
    
    return True

def create_conversion_script():
    """Create the CoreML conversion script"""
    script_content = '''from pathlib import Path
import torch
import coremltools as ct
from transformers import AutoTokenizer, AutoModelForSeq2SeqLM

MODEL_ID = "visheratin/t5-efficient-tiny-grammar-correction"
OUT = Path("Models")
SEQ_LEN = 128

print("🔄 Loading tokenizer and model...")
tok = AutoTokenizer.from_pretrained(MODEL_ID)
model = AutoModelForSeq2SeqLM.from_pretrained(MODEL_ID, torch_dtype=torch.float16)
model.eval()

print("🔄 Creating dummy input for tracing...")
dummy = tok("Hello world", return_tensors="pt").input_ids
with torch.no_grad():
    traced = torch.jit.trace(model, (dummy,))

print("🔄 Converting to CoreML...")
mlmodel = ct.convert(
    traced,
    convert_to="mlprogram",
    compute_units=ct.ComputeUnit.CPU_AND_NE,
    minimum_deployment_target=ct.target.iOS17,
    inputs=[ct.TensorType(name="input_ids", shape=(1, SEQ_LEN), dtype=ct.int32)]
)

print("🔄 Compressing model...")
mlmodel = ct.compress_fp16_to_int4(mlmodel)

print("🔄 Saving model and tokenizer...")
OUT.mkdir(exist_ok=True)
mlmodel.save(OUT / "t5_tiny_grammar.mlmodel")
tok.save_pretrained(OUT)

print("✅ Model conversion complete!")
'''
    
    with open("convert_to_coreml.py", "w") as f:
        f.write(script_content)
    
    print("✅ Created conversion script")
    return True

def convert_model(python_exe):
    """Run the model conversion"""
    if not run_command([python_exe, "convert_to_coreml.py"], "Converting model to CoreML"):
        return False
    
    return True

def compile_model():
    """Compile the CoreML model"""
    models_compiled = Path("ModelsCompiled")
    models_compiled.mkdir(exist_ok=True)
    
    if not run_command([
        "xcrun", "coremlc", "compile", 
        "Models/t5_tiny_grammar.mlmodel", 
        "ModelsCompiled"
    ], "Compiling CoreML model"):
        return False
    
    return True

def verify_installation():
    """Verify that all files were created correctly"""
    print("\n🔍 Verifying installation...")
    
    required_files = [
        "Models/tokenizer.json",
        "Models/config.json",
        "ModelsCompiled/t5_tiny_grammar.mlmodelc"
    ]
    
    all_good = True
    for file_path in required_files:
        if Path(file_path).exists():
            print(f"✅ {file_path}")
        else:
            print(f"❌ {file_path} - Missing!")
            all_good = False
    
    return all_good

def cleanup():
    """Clean up temporary files"""
    print("\n🧹 Cleaning up...")
    
    files_to_remove = ["convert_to_coreml.py"]
    for file_path in files_to_remove:
        if Path(file_path).exists():
            os.remove(file_path)
            print(f"🗑️  Removed {file_path}")

def main():
    """Main setup function"""
    print("🚀 TypoFixer CoreML Model Setup")
    print("=" * 50)
    
    # Check requirements
    if not check_requirements():
        sys.exit(1)
    
    # Setup virtual environment
    python_exe = setup_virtual_environment()
    if not python_exe:
        sys.exit(1)
    
    # Install dependencies
    if not install_dependencies(python_exe):
        sys.exit(1)
    
    # Create conversion script
    if not create_conversion_script():
        sys.exit(1)
    
    # Convert model
    if not convert_model(python_exe):
        sys.exit(1)
    
    # Compile model
    if not compile_model():
        sys.exit(1)
    
    # Verify installation
    if not verify_installation():
        print("\n❌ Setup completed with errors. Please check the output above.")
        sys.exit(1)
    
    # Cleanup
    cleanup()
    
    print("\n" + "=" * 50)
    print("✅ Setup completed successfully!")
    print("\nYour CoreML model is ready. You can now build and run TypoFixer:")
    print("  cargo build --release")
    print("  ./target/release/typo-fixer")
    print("\nModel files created:")
    print("  📁 Models/")
    print("     📄 tokenizer.json")
    print("     📄 config.json")
    print("  📁 ModelsCompiled/")
    print("     📄 t5_tiny_grammar.mlmodelc/")

if __name__ == "__main__":
    main()
