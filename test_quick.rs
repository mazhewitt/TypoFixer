fn main() { let mut model = typo_fixer::spell_check::LlamaModelWrapper::new(&std::path::PathBuf::from("test")).unwrap(); println\!("Testing: {}", model.generate("typoos shud be fixed").unwrap()); }
