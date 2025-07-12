#!/usr/bin/env python3
"""
Test the downloaded SentimentPolarity Core ML model to understand its interface.
"""

import coremltools as ct

def test_sentiment_model():
    """Test the sentiment model to understand its input/output format."""
    
    print("🔍 Testing SentimentPolarity Core ML Model")
    print("=" * 50)
    
    try:
        # Load the model
        model_path = "coreml-models/SentimentPolarity.mlmodel"
        model = ct.models.MLModel(model_path)
        
        # Get model description
        spec = model.get_spec()
        print("📋 Model Information:")
        print(f"  Description: {spec.description}")
        print(f"  Author: {spec.metadata.author}")
        print(f"  Version: {spec.metadata.versionString}")
        
        # Get input information
        print("\n📥 Input Information:")
        for input_desc in spec.description.input:
            print(f"  Name: {input_desc.name}")
            print(f"  Type: {input_desc.type}")
            if hasattr(input_desc.type, 'stringType'):
                print(f"  Input type: String")
            elif hasattr(input_desc.type, 'multiArrayType'):
                print(f"  Input type: MultiArray")
                print(f"  Shape: {input_desc.type.multiArrayType.shape}")
        
        # Get output information
        print("\n📤 Output Information:")
        for output_desc in spec.description.output:
            print(f"  Name: {output_desc.name}")
            print(f"  Type: {output_desc.type}")
            if hasattr(output_desc.type, 'dictionaryType'):
                print(f"  Output type: Dictionary")
            elif hasattr(output_desc.type, 'multiArrayType'):
                print(f"  Output type: MultiArray")
        
        # Test prediction with some text
        print("\n🧪 Testing Predictions:")
        test_sentences = [
            "I love this app!",
            "This is terrible.",
            "The weather is nice today.",
            "I has bad grammar here."
        ]
        
        for sentence in test_sentences:
            try:
                # Try different input formats
                for input_name in ["text", "input", "sentence", "reviewText"]:
                    try:
                        prediction = model.predict({input_name: sentence})
                        print(f"  '{sentence}' -> {prediction}")
                        break
                    except Exception as e:
                        continue
                else:
                    print(f"  '{sentence}' -> Could not find correct input format")
            except Exception as e:
                print(f"  '{sentence}' -> Error: {e}")
        
        print("\n✅ Model analysis complete!")
        
    except Exception as e:
        print(f"❌ Error testing model: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    test_sentiment_model()