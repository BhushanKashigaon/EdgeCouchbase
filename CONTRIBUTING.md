# Contributing to EdgeCouchbase

Thank you for your interest in contributing to EdgeCouchbase! This document provides guidelines and instructions for contributing.

## Code of Conduct

This project adheres to the [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## Development Setup

### Prerequisites

- **Rust**: 1.75+ (MSRV for async traits support)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  rustup default stable
  ```

- **Go**: 1.22+ (for control plane and CLI tools)
  ```bash
  # Install from https://go.dev/dl/
  ```

- **Protobuf Compiler**: 3.20+
  ```bash
  # Ubuntu/Debian
  sudo apt install protobuf-compiler
  
  # macOS
  brew install protobuf
  
  # Windows
  choco install protoc
  ```

- **Docker**: 24.0+ and Docker Compose 2.20+ (for integration tests)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/EdgeCouchbase.git
cd EdgeCouchbase

# Build all Rust services
cargo build --workspace

# Build Go services
cd ops
go build -o ../bin/edgecb-admin ./cmd/admin
go build -o ../bin/edgecb-cli ./cmd/cli

# Run tests
cargo test --workspace
cd ops && go test ./...
```

### Running Locally

```bash
# Start a 3-node cluster with docker-compose
docker-compose up -d

# Check cluster health
curl http://localhost:8091/health

# View logs
docker-compose logs -f node1
```

## Project Structure

```
/docs              - Architecture docs, ADRs, runbooks
/proto             - gRPC/Protobuf service definitions
/server            - Data service (KV storage, vBuckets, DCP)
/cluster           - Cluster membership, Raft, rebalance (Go)
/index             - Secondary index service
/query             - SQL++ query engine
/search            - Full-text search service
/analytics         - MPP analytics engine
/xdcr              - Cross-cluster replication
/sdk               - Client SDKs
/ops               - Admin API, RBAC, metrics (Go)
/bench             - Benchmarking suite
/deploy            - Docker, K8s manifests
/ci                - CI/CD configuration
```

## Contribution Workflow

### 1. Find or Create an Issue

- Browse [existing issues](https://github.com/yourusername/EdgeCouchbase/issues)
- For bugs: provide minimal reproduction steps, expected vs. actual behavior
- For features: describe use case, proposed API, and alternatives considered
- Wait for maintainer feedback before starting work on large changes

### 2. Fork and Branch

```bash
# Fork the repo on GitHub, then:
git clone https://github.com/YOUR_USERNAME/EdgeCouchbase.git
cd EdgeCouchbase
git remote add upstream https://github.com/yourusername/EdgeCouchbase.git

# Create a feature branch
git checkout -b feature/my-awesome-feature
# or
git checkout -b fix/issue-123
```

### 3. Make Changes

#### Code Style

**Rust:**
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Add documentation comments (`///`) for public APIs
- Use `tracing` for logging, not `println!`

**Go:**
- Follow [Effective Go](https://go.dev/doc/effective_go)
- Run `go fmt` before committing
- Run `golangci-lint run` and fix warnings
- Add godoc comments for exported functions

#### Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): short description

Longer explanation if needed.

Fixes #123
```

**Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`

**Examples:**
```
feat(query): add support for window functions
fix(kv): prevent data race in vBucket assignment
docs(adr): add ADR for consistency model
perf(index): use SIMD for index scan
```

#### Tests

- **Unit tests**: Place in `#[cfg(test)]` modules or `tests/` dir
- **Integration tests**: Place in `/tests` at workspace root
- **Benchmarks**: Use `criterion` crate, place in `benches/`

```bash
# Run unit tests
cargo test --workspace

# Run integration tests
cargo test --test '*' --workspace

# Run benchmarks
cargo bench --workspace
```

#### Documentation

- Update relevant docs in `/docs` for architectural changes
- Add ADRs (Architecture Decision Records) for significant decisions
- Update API docs for public interfaces
- Add examples to `/examples` for new features

### 4. Test Your Changes

```bash
# Format code
cargo fmt --all
cd ops && go fmt ./...

# Lint
cargo clippy --workspace -- -D warnings
cd ops && golangci-lint run

# Test
cargo test --workspace
cargo test --workspace --release  # Test optimized builds too

# Integration tests (requires Docker)
docker-compose -f docker-compose.test.yml up --abort-on-container-exit

# Benchmarks (for performance-critical changes)
cargo bench --workspace
```

### 5. Submit a Pull Request

```bash
# Commit your changes
git add .
git commit -m "feat(query): add JOIN support"

# Rebase on latest main
git fetch upstream
git rebase upstream/main

# Push to your fork
git push origin feature/my-awesome-feature
```

Create a pull request on GitHub:
- Use a clear title (will become the merge commit message)
- Reference related issues (`Fixes #123`, `Relates to #456`)
- Describe what changed and why
- Add screenshots/demos for UI changes
- Check all CI checks pass

### 6. Code Review

- Respond to feedback promptly
- Push additional commits to the same branch (don't force-push during review)
- Request re-review after addressing feedback
- Maintainers may edit your PR (title, commits) before merging

## Specialized Guidelines

### Adding a New Service

1. Create a new Cargo workspace member or Go module
2. Define the service in `/proto/<service>/v1/<service>_service.proto`
3. Implement skeleton service with health endpoint
4. Add to `docker-compose.yml` with appropriate ports
5. Update cluster map schema to support service placement
6. Write ADR explaining service boundaries and API design
7. Add integration tests

### Modifying Protocol Buffers

- **Never break wire compatibility** without a major version bump
- Add new fields with high field numbers (leave room for future additions)
- Use `reserved` for deprecated fields
- Update both Rust and Go generated code
- Test backward compatibility with old clients

### Performance-Critical Code

- Profile before and after with `cargo flamegraph` or `perf`
- Include benchmark results in PR description
- Use `#[inline]` judiciously (measure, don't guess)
- Consider SIMD for hot loops (use `std::simd` or crates like `packed_simd`)
- Avoid allocations in hot paths (use object pools if needed)

### Distributed Systems Invariants

- Document invariants in code comments (e.g., "vBucket map version must be monotonic")
- Add assertions for invariants in debug builds
- Write property-based tests with `proptest`
- Consider failure modes: partition, crash, corruption, clock skew
- Use deterministic testing where possible (seed RNGs, mock time)

## Release Process

Maintainers follow this process for releases:

1. Create a release branch: `release/v0.2.0`
2. Update `CHANGELOG.md` with all changes since last release
3. Bump version in all `Cargo.toml` and `go.mod` files
4. Run full test suite including integration and chaos tests
5. Tag the release: `git tag -a v0.2.0 -m "Release v0.2.0"`
6. Push tag: `git push upstream v0.2.0`
7. CI builds and publishes Docker images, binaries, and crates
8. Create GitHub release with changelog excerpt

## Getting Help

- **Discord**: [Join our server](https://discord.gg/edgecouchbase) for real-time discussion
- **GitHub Discussions**: For questions and ideas
- **Office Hours**: Fridays 2-3 PM UTC on Discord voice channel

## Recognition

Contributors are recognized in:
- `CONTRIBUTORS.md` (all contributors)
- Release notes (per-release contributors)
- Annual blog post highlighting major contributions

Thank you for contributing to EdgeCouchbase! 🚀
