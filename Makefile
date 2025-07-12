# TypoFixer Build and Cache Management

.PHONY: help build clean clean-coreml test run release check-cache

# Default target
help:
	@echo "TypoFixer Build Commands:"
	@echo ""
	@echo "  make build        - Build with Core ML model compilation"
	@echo "  make clean        - Clean all build artifacts and Core ML cache"
	@echo "  make clean-coreml - Clean only Core ML model cache"
	@echo "  make test         - Run tests"
	@echo "  make run          - Build and run the application"
	@echo "  make release      - Build optimized release version"
	@echo "  make check-cache  - Show Core ML cache status"
	@echo ""

# Build with Core ML model compilation
build:
	@echo "🔨 Building TypoFixer with Core ML model compilation..."
	cargo build

# Clean all build artifacts including Core ML cache  
clean:
	@echo "🧹 Cleaning all build artifacts and Core ML cache..."
	cargo clean
	@echo "🗑️  Core ML cache cleaned by cargo clean"

# Clean only Core ML model cache
clean-coreml:
	@echo "🧹 Cleaning Core ML model cache..."
	@if [ -d "target" ]; then \
		find target -name "coreml_models" -type d -exec rm -rf {} + 2>/dev/null || true; \
		find target -name "compile_model.swift" -type f -exec rm -f {} + 2>/dev/null || true; \
		find target -name "*.mlmodelc" -type d -exec rm -rf {} + 2>/dev/null || true; \
		echo "✅ Core ML cache cleaned"; \
	else \
		echo "✨ No target directory found - cache already clean"; \
	fi

# Run tests
test:
	@echo "🧪 Running tests..."
	cargo test

# Build and run
run: build
	@echo "🚀 Starting TypoFixer..."
	cargo run

# Build release version
release:
	@echo "🔨 Building optimized release version..."
	cargo build --release

# Check cache status
check-cache:
	@echo "📋 Core ML Cache Status:"
	@echo ""
	@if [ -d "target" ]; then \
		echo "🔍 Searching for cached models..."; \
		FOUND=0; \
		for dir in $$(find target -name "coreml_models" -type d 2>/dev/null); do \
			echo "  📁 $$dir"; \
			if [ -d "$$dir" ]; then \
				ls -la "$$dir" 2>/dev/null | sed 's/^/    /' || true; \
			fi; \
			FOUND=1; \
		done; \
		for file in $$(find target -name "*.mlmodelc" -type d 2>/dev/null); do \
			echo "  🤖 $$file"; \
			FOUND=1; \
		done; \
		if [ $$FOUND -eq 0 ]; then \
			echo "  ✨ No cached Core ML models found"; \
		fi; \
	else \
		echo "  📂 No target directory found"; \
	fi
	@echo ""

# Show build environment info
env-info:
	@echo "🔧 Build Environment:"
	@echo "  Cargo: $$(cargo --version)"
	@echo "  Rust: $$(rustc --version)"
	@if command -v swift >/dev/null 2>&1; then \
		echo "  Swift: $$(swift --version | head -n1)"; \
	else \
		echo "  Swift: ❌ Not available (Core ML compilation will fail)"; \
	fi
	@echo ""