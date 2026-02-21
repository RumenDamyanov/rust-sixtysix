## Minimal helper Makefile (optional ergonomics)

PORT ?= 8080
VERSION ?= dev

.PHONY: build test cover run docker-build docker-run clean fmt clippy help

build: ## Compile all packages
	cargo build

test: ## Run unit tests
	cargo test

cover: ## Coverage (requires cargo-tarpaulin)
	cargo tarpaulin --out Html

run: ## Run example server locally
	PORT=$(PORT) cargo run --example server

fmt: ## Format code
	cargo fmt

clippy: ## Run clippy lints
	cargo clippy -- -D warnings

docker-build: ## Build container image
	docker build -t rust-sixtysix:$(VERSION) .

docker-run: ## Run container exposing PORT
	docker run --rm -e PORT=$(PORT) -p $(PORT):$(PORT) rust-sixtysix:$(VERSION)

clean: ## Remove build artifacts
	cargo clean

help: ## Show targets
	@grep -E '^[a-zA-Z_-]+:.*?##' $(MAKEFILE_LIST) | sed 's/:.*##/: /'
