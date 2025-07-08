# Instructions: Using T5-Efficient-Tiny-Grammar-Correction in Rust via Core ML

This document provides step-by-step guidance to run `visheratin/t5-efficient-tiny-grammar-correction` locally using Apple’s Neural Engine, with a Rust-based interface.

---

## Step 1: Python-based Model Conversion

### Requirements

- macOS 14+ on Apple Silicon
- Python ≥ 3.10
- Xcode command-line tools
- Virtual environment (recommended)

### Setup

```bash
python -m venv .venv && source .venv/bin/activate
pip install --upgrade \
    coremltools==8.3 \
    torch==2.3.0 \
    transformers==4.41.1 \
    tokenizers==0.19.0 \
    accelerate==0.29.3
```

### Convert the Model

Create a file `convert_to_coreml.py` with:

```python
from pathlib import Path
import torch, coremltools as ct
from transformers import AutoTokenizer, AutoModelForSeq2SeqLM

MODEL_ID = "visheratin/t5-efficient-tiny-grammar-correction"
OUT = Path("Models")
SEQ_LEN = 128

tok = AutoTokenizer.from_pretrained(MODEL_ID)
model = AutoModelForSeq2SeqLM.from_pretrained(MODEL_ID, torch_dtype=torch.float16)
model.eval()

dummy = tok("Hello world", return_tensors="pt").input_ids
with torch.no_grad():
    traced = torch.jit.trace(model, (dummy,))

mlmodel = ct.convert(
    traced,
    convert_to="mlprogram",
    compute_units=ct.ComputeUnit.CPU_AND_NE,
    minimum_deployment_target=ct.target.iOS17,
    inputs=[ct.TensorType(name="input_ids", shape=(1, SEQ_LEN), dtype=ct.int32)]
)

mlmodel = ct.compress_fp16_to_int4(mlmodel)

OUT.mkdir(exist_ok=True)
mlmodel.save(OUT / "t5_tiny_grammar.mlmodel")
tok.save_pretrained(OUT)
```

Then run:

```bash
python convert_to_coreml.py
xcrun coremlc compile Models/t5_tiny_grammar.mlmodel ModelsCompiled
```

---

## Step 2: Rust Wrapper

Add to `Cargo.toml`:

```toml
coreml-rs = "0.5"
tokenizers = "0.17"
anyhow = "1"
```

### Wrapper Code

```rust
use coreml_rs::{ComputePlatform, CoreMLModelOptions, CoreMLModelWithState};
use tokenizers::{Tokenizer, Encoding};
use anyhow::{Result, Context};

pub struct CoreMlCorrector {
    model: CoreMLModelWithState,
    tokenizer: Tokenizer,
}

impl CoreMlCorrector {
    pub fn new(modelc_dir: &str, tokenizer_path: &str) -> Result<Self> {
        let opts = CoreMLModelOptions {
            compute_platform: ComputePlatform::CpuAndANE,
            ..Default::default()
        };
        let bytes = std::fs::read(modelc_dir).context("model read")?;
        let model = CoreMLModelWithState::from_buf(bytes, opts);

        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .context("tokenizer read")?;

        Ok(Self { model, tokenizer })
    }

    pub fn correct(&mut self, sentence: &str) -> Result<String> {
        let enc: Encoding = self.tokenizer.encode(sentence, true)?;
        let mut ids = enc.get_ids().to_vec();
        ids.resize(128, 0);
        self.model.add_input("input_ids", ids)?;

        let out = self.model.predict()?;
        let out_ids: Vec<i32> = out.bytes_from("output_ids")?;

        let mut text = self.tokenizer.decode(out_ids, true)?;
        text = text.trim_matches(&[' ', '"', '''][..]).to_string();
        Ok(text)
    }
}
```

---

## Step 3: Integration

```rust
pub enum Engine {
    CoreMl(CoreMlCorrector),
}

pub fn generate_correction(text: &str, engine: &mut Engine) -> Result<String> {
    match engine {
        Engine::CoreMl(m) => m.correct(text),
    }
}
```

---

## Step 4: Test

```rust
#[test]
fn smoke_coreml() {
    let mut m = CoreMlCorrector::new(
        "ModelsCompiled/t5_tiny_grammar.mlmodelc",
        "Models/tokenizer.json"
    ).unwrap();
    let got = m.correct("I has a appl").unwrap();
    assert_eq!(got, "I have an apple");
}
```

---

## Notes

- `SEQ_LEN` in Python and Rust must match.
- Input strings shorter than `SEQ_LEN` are padded.
- Expect ~200 tokens/sec on M1-class devices.
- License: Apache-2.0 (open for commercial use).