.PHONY: all build test clean proto docker help

# Colors for output
BLUE := \033[0;34m
GREEN := \033[0;32m
NC := \033[0m # No Color

help: ## Show this help message
	@echo '$(BLUE)EdgeCouchbase Build System$(NC)'
	@echo ''
	@echo 'Available targets:'
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(GREEN)%-15s$(NC) %s\n", $$1, $$2}'

all: proto build ## Build everything

proto: ## Generate code from protobuf definitions
	@echo "$(BLUE)Generating protobuf code...$(NC)"
	@cd proto && buf generate || (cd admin/v1 && protoc --go_out=. --go_opt=paths=source_relative --go-grpc_out=. --go-grpc_opt=paths=source_relative *.proto)

build: build-rust build-go ## Build all services

build-rust: ## Build Rust services
	@echo "$(BLUE)Building Rust services...$(NC)"
	cargo build --release

build-go: ## Build Go services
	@echo "$(BLUE)Building Go services...$(NC)"
	cd ops && go build -o ../bin/edgecb-admin ./cmd/admin
	cd ops && go build -o ../bin/edgecb-cli ./cmd/cli

test: test-rust test-go ## Run all tests

test-rust: ## Run Rust tests
	@echo "$(BLUE)Running Rust tests...$(NC)"
	cargo test --all

test-go: ## Run Go tests
	@echo "$(BLUE)Running Go tests...$(NC)"
	cd ops && go test ./...

bench: ## Run Rust benchmarks
	@echo "$(BLUE)Running benchmarks...$(NC)"
	cd bench && cargo bench

lint: lint-rust lint-go ## Run linters

lint-rust: ## Run Rust linter
	cargo clippy --all-targets --all-features -- -D warnings

lint-go: ## Run Go linter
	cd ops && go vet ./...
	cd ops && golangci-lint run || echo "Install golangci-lint for full linting"

fmt: ## Format code
	cargo fmt --all
	cd ops && go fmt ./...

clean: ## Clean build artifacts
	cargo clean
	rm -rf bin/
	cd ops && go clean

docker: ## Build Docker images
	@echo "$(BLUE)Building Docker images...$(NC)"
	docker-compose build

docker-up: ## Start Docker cluster
	docker-compose up -d

docker-down: ## Stop Docker cluster
	docker-compose down

docker-logs: ## Show Docker logs
	docker-compose logs -f

dev: ## Start development cluster
	@echo "$(BLUE)Starting development cluster...$(NC)"
	docker-compose up

install-tools: ## Install development tools
	cargo install cargo-watch
	cargo install cargo-tarpaulin
	go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest
	go install github.com/bufbuild/buf/cmd/buf@latest

.DEFAULT_GOAL := help
