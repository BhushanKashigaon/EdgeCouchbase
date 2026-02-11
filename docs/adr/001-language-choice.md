# ADR-001: Language Choice - Rust and Go

**Status**: Accepted  
**Date**: 2026-02-11  
**Deciders**: Core Team  
**Tags**: architecture, languages, tooling

## Context

EdgeCouchbase requires a technology stack that balances:
- **Performance**: Low-latency, high-throughput data operations
- **Safety**: Memory safety to prevent crashes and security vulnerabilities  
- **Productivity**: Rapid development of control plane and operational tools
- **Ecosystem**: Strong libraries for networking, serialization, observability
- **Operational Simplicity**: Single-binary deployment, minimal runtime dependencies

We evaluated multiple language combinations for different subsystems.

## Decision

We will use a **dual-language** architecture:

### Rust for Performance-Critical Components
- **Data Service** (KV storage, vBucket management, replication)
- **Query Engine** (SQL++ parser, planner, executor)
- **Index Service** (B+tree, memory-optimized indexes)
- **Search Service** (inverted index, analyzers)
- **Analytics Service** (MPP query execution)

### Go for Operational Components
- **Cluster Management** (membership, configuration)
- **Admin API & Web UI** (REST API, metrics endpoints)
- **CLI Tools** (admin commands, cluster operations)
- **Orchestration** (node lifecycle, health checks)

### Rationale

#### Why Rust for Core Services?

1. **Memory Safety**: Ownership model prevents data races, use-after-free, and null pointer dereferences
2. **Zero-Cost Abstractions**: No garbage collection pauses; predictable latency
3. **Performance**: C/C++ level performance with modern ergonomics
4. **Async Runtime**: Tokio provides production-grade async I/O (1M+ concurrent connections)
5. **Strong Type System**: Complex distributed algorithms benefit from compile-time guarantees
6. **Ecosystem**: 
   - `tonic` (gRPC)
   - `tokio` (async runtime)
   - `serde` (serialization)
   - `tracing` (structured logging)
   - `rocksdb` bindings (storage engine)

#### Why Go for Control Plane?

1. **Fast Development**: Quick iteration on admin features and tools
2. **Excellent Standard Library**: HTTP, JSON, TLS, testing out-of-box
3. **gRPC Tooling**: Protobuf code generation is mature and well-documented
4. **Cross-Compilation**: Easy to build CLI tools for multiple platforms
5. **Operational Simplicity**: Single binary, small runtime overhead
6. **Team Familiarity**: Broader talent pool for operational tooling
7. **Ecosystem**:
   - `grpc-go` (gRPC server/client)
   - `cobra` (CLI framework)
   - `prometheus/client_golang` (metrics)
   - `zap` (structured logging)

#### Why Not Single Language?

**All-Rust**:
- Slower development cycle for UI/CLI (less mature ecosystem)
- Steeper learning curve for contributors to operational tools
- Overkill for non-performance-critical paths

**All-Go**:
- GC pauses unacceptable for sub-millisecond KV operations
- Memory safety relies on developer discipline, not compiler
- Performance ceiling lower than Rust for compute-intensive workloads

**All-C++**:
- Memory safety requires extensive tooling (sanitizers, static analysis)
- Dependency management still immature (Conan/vcpkg evolving)
- Build times significantly longer than Rust/Go
- Manual memory management error-prone in large codebase

## Consequences

### Positive

- **Best-of-Both**: Performance where it matters, productivity for tooling
- **Clear Boundaries**: Service boundaries align with language boundaries
- **Safety**: Rust's guarantees reduce crash risk in core services
- **Talent Pool**: Go lowers barrier for SRE/DevOps contributors

### Negative

- **Two Ecosystems**: Different dependency managers (Cargo + Go modules)
- **Build Complexity**: Need both toolchains in CI/CD
- **Inter-Language Calls**: gRPC boundary adds ~100µs overhead (acceptable for our use case)
- **Context Switching**: Contributors may need proficiency in both languages for full-stack changes

### Mitigation Strategies

1. **gRPC Contracts**: Strictly versioned `.proto` files decouple services
2. **Build Automation**: Single `Makefile` orchestrates Cargo + Go builds
3. **Docker Multi-Stage Builds**: Isolate build environments
4. **Documentation**: Language-specific guides for each subsystem
5. **Ownership**: Teams can specialize (Storage team = Rust, Ops team = Go)

## Alternatives Considered

### C++ for Everything
- **Pros**: Maximum performance, existing DB ecosystem (LevelDB, RocksDB)
- **Cons**: Memory safety requires heroic effort; build times; dependency hell
- **Verdict**: Performance gains not worth safety/productivity trade-off

### Java/JVM Languages
- **Pros**: Mature ecosystem, excellent tooling, GraalVM native images
- **Cons**: GC pauses problematic for KV service; higher memory footprint
- **Verdict**: GC overhead unacceptable for data path

### Zig
- **Pros**: Manual memory management with safety features, C interop
- **Cons**: Language still pre-1.0; ecosystem immature; async story unclear
- **Verdict**: Too risky for production-aimed project

### Node.js/TypeScript
- **Pros**: Huge ecosystem, async-native, JSON-friendly
- **Cons**: Single-threaded (multi-core requires clustering); V8 GC pauses
- **Verdict**: Performance inadequate for database core

## References

- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Go at Google: Language Design in the Service of Software Engineering](https://go.dev/talks/2012/splash.article)
- [Discord: Why We Switched from Go to Rust](https://discord.com/blog/why-discord-is-switching-from-go-to-rust)
- [Couchbase Architecture](https://docs.couchbase.com/server/current/learn/architecture-overview.html)

## Notes

- Re-evaluate if **C++20 modules** mature significantly (2027+)
- Monitor **Zig 1.0** readiness for future services
- Consider **Rust for CLI** if `clap` + `ratatui` ecosystem improves significantly
