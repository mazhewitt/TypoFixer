#!/bin/bash

# T5 Grammar Correction Model to Core ML Converter Setup
# This script sets up the environment and converts the model

echo "🚀 T5 Grammar Correction to Core ML Converter Setup"
echo "=================================================="

# Check if Python 3 is available
if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 is required but not installed."
    echo "   Please install Python 3 and try again."
    exit 1
fi

echo "✅ Python 3 found: $(python3 --version)"

# Create virtual environment
echo "🔄 Creating virtual environment..."
python3 -m venv venv_coreml
source venv_coreml/bin/activate

# Upgrade pip
echo "🔄 Upgrading pip..."
python -m pip install --upgrade pip

# Install requirements
echo "🔄 Installing requirements..."
pip install -r requirements.txt

# Run the conversion
echo "🔄 Starting T5 model conversion..."
python convert_t5_to_coreml.py

echo ""
echo "🎉 Setup and conversion complete!"
echo "Check the 'coreml-models' directory for your converted model."