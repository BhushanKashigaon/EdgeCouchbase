# EdgeCouchbase: Distributed Document Database

> A production-aimed, open-source distributed document database inspired by Couchbase's service-per-role architecture

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](ci/)

## Vision

EdgeCouchbase is a scalable, distributed document database system that implements a **multi-dimensional scaling (MDS)** architecture, enabling independent deployment and scaling of specialized services. Built with Rust for performance-critical components and Go for operational tooling, EdgeCouchbase delivers enterprise-grade features including multi-document ACID transactions, full-text search, analytics, and cross-datacenter replication.

## Architecture Overview

EdgeCouchbase follows a **service-per-role** architecture with these independent services:

- **Data Service (KV)**: JSON document storage with vBucket-based sharding,MemDB/Couchstore backend
- **Query Service**: SQL++ (N1QL) query engine with cost-based optimization
- **Index Service**: Secondary indexes (B+tree, memory-optimized) for query acceleration
- **Search Service**: Full-text search with analyzers, scoring, and vector search support
- **Analytics Service**: Shadow datasets via change stream; MPP query engine isolated from OLTP
- **Eventing Service**: Functions-as-a-service with triggers on mutations
- **Backup Service**: Incremental backup/restore with point-in-time recovery
- **XDCR**: Cross-datacenter replication with filtering and conflict resolution

### Key Design Principles

1. **Multi-Dimensional Scaling**: Deploy and scale each service independently across nodes
2. **Zero-Copy I/O**: Minimize memory copies for maximum throughput
3. **Bounded Resources**: Memory limits, backpressure, cooperative cancellation
4. **Observability First**: Structured logging, OpenTelemetry metrics/traces
5. **Security by Default**: mTLS everywhere, RBAC, comprehensive audit logging

## Data Model

```
Cluster
└── Bucket (logical container, resource limits)
    └── Scope (namespace for collections)
        └── Collection (actual document storage)
            └── Documents (JSON with auto-generated or user-defined keys)
```

### Key Concepts

- **vBuckets**: 1024 virtual buckets per bucket for distribution and rebalancing
- **DCP (Database Change Protocol)**: Internal change stream with sequence numbers, snapshots, failover logs
- **Tunable Durability**: `none`, `majority`, `majorityAndPersistActive`, `persistToMajority`
- **Multi-Document Transactions**: ACID with optimistic concurrency control

## Technology Stack

| Component | Language | Rationale |
|-----------|----------|-----------|
| Data/Storage/Replication | Rust | Memory safety, zero-cost abstractions, performance |
| Query Engine | Rust | Complex parsing/planning benefits from strong typing |
| Index Service | Rust | Performance-critical path |
| Control Plane | Go | Fast development, excellent stdlib, gRPC tooling |
| CLI/Admin Tools | Go | Cross-platform binaries, ergonomic CLI libraries |
| Web UI | Go + HTMX | Simple, maintainable, server-side rendering |

## Project Structure

```
/docs              - Architecture docs, ADRs, threat models, SLOs
/design            - UML/Mermaid diagrams
/proto             - gRPC/Protobuf service definitions
/server            - Data service (KV storage, vBuckets, replication)
/cluster           - Cluster membership, Raft consensus, rebalance
/index             - Secondary indexing service
/query             - SQL++ parser, planner, executor
/search            - Full-text search with analyzers, inverted index
/analytics         - MPP analytics engine with shadow datasets
/eventing          - Event-driven functions service
/xdcr              - Cross-datacenter replication
/sdk               - Client SDKs (C++, Java, Go)
/ops               - Admin API, RBAC, metrics, TLS management
/bench             - YCSB-like benchmarks and microbenchmarks
/deploy            - Docker, docker-compose, Kubernetes/Helm charts
/ci                - CI/CD pipelines, fuzzing, linting, SAST
```

## Roadmap

### Phase 1: Foundation (Current)
- [x] Repository structure
- [x] Core RPC definitions (gRPC/Protobuf)
- [ ] Data service with basic KV operations
- [ ] Cluster membership with Raft
- [ ] Basic replication (active-passive)
- [ ] Docker-compose 3-node cluster

### Phase 2: Core Services (Q2 2026)
- [ ] Query service with SQL++ subset (SELECT, INSERT, UPDATE, DELETE)
- [ ] Secondary indexes (GSI)
- [ ] vBucket-based sharding and rebalancing
- [ ] DCP change stream implementation
- [ ] Multi-document transactions

### Phase 3: Advanced Services (Q3 2026)
- [ ] Full-text search with analyzers
- [ ] Analytics service with shadow datasets
- [ ] Eventing service
- [ ] XDCR with filtering

### Phase 4: Production Hardening (Q4 2026)
- [ ] Vector search capabilities
- [ ] Backup/restore service
- [ ] Enhanced observability (distributed tracing)
- [ ] Performance optimization (zero-copy, SIMD)
- [ ] Kubernetes operator

### Phase 5: Ecosystem (2027)
- [ ] Python, Node.js, .NET SDKs
- [ ] Query profiling and optimization tools
- [ ] Migration tools from Couchbase/MongoDB
- [ ] Benchmarking suite vs. competitors

## Quick Start

### Prerequisites

- **Rust**: 1.75+ (MSRV for async traits)
- **Go**: 1.22+
- **Protobuf**: 3.20+
- **Docker**: 24.0+
- **Docker Compose**: 2.20+

### Build from Source

```bash
# Clone repository
git clone https://github.com/yourusername/EdgeCouchbase.git
cd EdgeCouchbase

# Build all Rust services
cargo build --release

# Build Go services
cd ops && go build -o ../bin/edgecb-admin ./cmd/admin
cd ../cluster && go build -o ../bin/edgecb-cli ./cmd/cli

# Run tests
cargo test --all
go test ./...
```

### Run Local 3-Node Cluster

```bash
docker-compose up -d
```

Access the cluster:
- **Admin UI**: http://localhost:8091
- **Health Check**: http://localhost:8091/health
- **Node 1**: localhost:11210
- **Node 2**: localhost:11211
- **Node 3**: localhost:11212

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for build instructions, coding standards, and PR workflow.

## Security

EdgeCouchbase takes security seriously:

- **mTLS**: All inter-node and client-node communication encrypted
- **RBAC**: Fine-grained role-based access control
- **Audit Logging**: Comprehensive audit trail for compliance
- **Secret Management**: Integration with HashiCorp Vault, AWS Secrets Manager

Report security vulnerabilities to security@edgecouchbase.io (PGP key in [SECURITY.md](SECURITY.md)).

## Known Limitations & Risks

### Current Scope Limitations

1. **Query Language**: Initial SQL++ subset; advanced features (CTEs, window functions) in later phases
2. **Index Types**: B+tree only; bitmap indexes planned for Q3 2026
3. **Consistency**: Tunable consistency; linearizability requires majority reads
4. **Vector Search**: Basic ANN search; advanced HNSW coming in Q4 2026

### Technical Risks

- **Rebalance Complexity**: vBucket migration under load requires extensive testing
- **Query Optimization**: Cost-based optimizer is complex; initial version uses heuristics
- **Memory Management**: Bounded memory enforcement across services needs careful tuning
- **Split-Brain**: Raft quorum prevents split-brain but requires majority for availability

### Operational Risks

- **Migration**: No automated migration from Couchbase; manual ETL required
- **Maturity**: Early-stage project; expect breaking changes until 1.0
- **Tooling**: Limited ecosystem compared to mature databases

## Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Point Read Latency (p99) | < 5ms | SSD, local read |
| Point Write Latency (p99) | < 10ms | Majority durability |
| Bulk Insert Throughput | > 100K ops/sec | Single node, batch size 1000 |
| Query Latency (simple) | < 50ms | Indexed lookup, 1M docs |
| Full-Text Search (p95) | < 100ms | 10M documents indexed |

## License

Apache License 2.0 - see [LICENSE](LICENSE) for details.

## Acknowledgments

Inspired by:
- **Couchbase Server**: Service architecture, vBuckets, DCP, SQL++
- **CockroachDB**: Distributed transactions, rebalancing algorithms
- **FoundationDB**: Deterministic simulation testing (future work)
- **TiDB**: Layer separation, Raft-based storage

## Community

- **Discussions**: [GitHub Discussions](https://github.com/yourusername/EdgeCouchbase/discussions)
- **Discord**: [Join our server](https://discord.gg/edgecouchbase)
- **Blog**: [edgecouchbase.io/blog](https://edgecouchbase.io/blog)
- **Twitter**: [@EdgeCouchbase](https://twitter.com/EdgeCouchbase)

## Contributing

We welcome contributions! Please read [CONTRIBUTING.md](CONTRIBUTING.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

---

**Status**: 🚧 Active Development | **Version**: 0.1.0-alpha | **Last Updated**: February 2026
