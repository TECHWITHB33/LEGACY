.PHONY: help build test lint format clean install docker-build docker-run

help:
	@echo "ID Card System - Development Commands"
	@echo ""
	@echo "Setup"
	@echo "  make install              Install all dependencies"
	@echo ""
	@echo "Building"
	@echo "  make build                Build all components"
	@echo "  make build-backend        Build Rust backend"
	@echo ""
	@echo "Testing"
	@echo "  make test                 Run all tests"
	@echo "  make test-backend         Run backend tests"
	@echo "  make test-coverage        Generate test coverage"
	@echo ""
	@echo "Code Quality"
	@echo "  make lint                 Run all linters"
	@echo "  make format               Format all code"
	@echo ""

build:
	@echo "Building..."
	cd backend && cargo build

test:
	@echo "Running tests..."
	cd backend && cargo test

lint:
	@echo "Linting..."
	cd backend && cargo clippy

format:
	@echo "Formatting..."
	cd backend && cargo fmt

.DEFAULT_GOAL := help
