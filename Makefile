# Makefile for TypoFixer

.PHONY: help build run test clean package install dev-deps

VERSION := 1.0.0
APP_NAME := TypoFixer

help: ## Show this help message
	@echo "TypoFixer v$(VERSION) - Development Commands"
	@echo "==========================================="
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

build: ## Build the application in release mode
	@echo "🔨 Building TypoFixer..."
	cargo build --release

run: ## Run the application in development mode
	@echo "🚀 Running TypoFixer in development mode..."
	cargo run

test: ## Run all tests
	@echo "🧪 Running tests..."
	cargo test

clean: ## Clean build artifacts
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	rm -rf dist/
	rm -rf target/release/$(APP_NAME).app

app: build ## Create macOS app bundle
	@echo "📱 Creating macOS app bundle..."
	./simple_build.sh

package: ## Create all distribution packages
	@echo "📦 Creating distribution packages..."
	./package.sh

install: app ## Install the app to /Applications
	@echo "📲 Installing TypoFixer to /Applications..."
	sudo cp -R target/release/$(APP_NAME).app /Applications/
	@echo "✅ Installed! Launch from Applications or Spotlight."

dev-deps: ## Install development dependencies
	@echo "🔧 Installing development dependencies..."
	@command -v ollama >/dev/null 2>&1 || { echo "Installing Ollama..."; curl -fsSL https://ollama.ai/install.sh | sh; }
	@echo "📥 Pulling recommended language model..."
	ollama pull llama3.2:1b

check: ## Check code formatting and linting
	@echo "🔍 Checking code..."
	cargo fmt --check
	cargo clippy -- -D warnings

format: ## Format code
	@echo "✨ Formatting code..."
	cargo fmt

demo: app ## Build and run demo
	@echo "🎬 Running TypoFixer demo..."
	./target/release/$(APP_NAME).app/Contents/MacOS/$(APP_NAME) &
	@echo "TypoFixer is now running in the menu bar."
	@echo "Press Cmd+Option+S in any text field to test text correction."

.DEFAULT_GOAL := help
