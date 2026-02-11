# ADR-002: Storage Engine Design

**Status**: Accepted  
**Date**: 2026-02-11  
**Deciders**: Core Team  
**Tags**: storage, performance, durability

## Context

The data service requires a storage engine that provides:
- **Durability**: Configurable persistence guarantees (none, majority, persist-to-majority)
- **Performance**: Sub-5ms p99 reads, >100K writes/sec per node
- **Caching**: Memory-resident hot data with overflow to disk
- **Atomic Operations**: Single-document CAS (compare-and-swap)
- **Change Streams**: Ability to expose ordered mutation stream (DCP-like)
- **vBucket Awareness**: Efficient per-vBucket snapshots and iteration

We evaluated custom storage vs. embedding existing engines.

## Decision

We will implement a **two-tier storage architecture**:

### MemDB (In-Memory Tier)
- **Purpose**: Hot data, write buffer, crash recovery
- **Implementation**: Custom Rust data structure
- **Structure**: HashMap per vBucket with sequence number ordering
- **Persistence**: Write-ahead log (WAL) on disk for durability
- **Eviction**: LRU eviction to disk when memory threshold exceeded

### DiskStore (Persistent Tier)
- **Purpose**: Cold data, long-term persistence
- **Implementation**: Embedded **RocksDB** (via `rust-rocksdb` crate)
- **Schema**: `<vBucket_id>:<doc_key>:<sequence> -> <doc_value>`
- **Indexes**: Separate column families for metadata, sequence index
- **Compaction**: Periodic compaction to reclaim deleted document space

### Hybrid Architecture

```
┌─────────────────────────────────────────┐
│          vBucket Manager                │
├─────────────────────────────────────────┤
│  ┌──────────┐         ┌──────────┐     │
│  │  MemDB   │ ──LRU──▶│ DiskStore│     │
│  │ (Active) │ ◀─Load─ │ (RocksDB)│     │
│  └──────────┘         └──────────┘     │
│       │                     │           │
│       ▼                     ▼           │
│    [WAL]              [LSM Tree]        │
└─────────────────────────────────────────┘
```

## Rationale

### Why RocksDB?

1. **Production Proven**: Powers Facebook, LinkedIn, Netflix storage systems
2. **LSM Tree**: Optimized for write-heavy workloads (log-structured merge tree)
3. **Column Families**: Logical separation of documents, metadata, indexes
4. **Tunable Compaction**: Control I/O amplification vs. space amplification
5. **Bloom Filters**: Fast negative lookups (key doesn't exist)
6. **Snapshots**: Consistent point-in-time reads for replication
7. **Mature Rust Bindings**: `rust-rocksdb` is actively maintained

### Why Custom MemDB?

1. **Control**: Fine-grained control over memory layout and eviction
2. **Lock-Free Reads**: Per-vBucket concurrent HashMap (dashmap)
3. **Sequence Ordering**: Native support for sequence number iteration (critical for DCP)
4. **Zero-Copy**: Direct memory access for hot documents (no serialization)
5. **Tail Latency**: Avoid RocksDB's worst-case compaction pauses in memory tier

### Data Flow

**Write Path**:
1. Acquire vBucket lock
2. Write to WAL (fsync if durability = `persistToMajority`)
3. Insert into MemDB HashMap with sequence number
4. Release lock (acknowledge to client if durability = `none` or `majority`)
5. Async flush to RocksDB when MemDB reaches size threshold

**Read Path**:
1. Check MemDB (fast path)
2. On miss, load from RocksDB
3. Optionally promote to MemDB (configurable)

**Eviction**:
- LRU eviction when MemDB memory > 70% threshold
- Flush to RocksDB in batches
- Keep tombstones in MemDB to avoid disk reads for deleted docs

## Consequences

### Positive

- **Predictable Latency**: MemDB avoids RocksDB compaction pauses for hot data
- **High Throughput**: LSM tree excels at write-heavy workloads
- **Durability Options**: WAL fsync configurable per request
- **Operational Simplicity**: RocksDB handles background compaction, no manual tuning needed initially
- **Change Stream**: Snapshot RocksDB per vBucket for DCP streaming

### Negative

- **Complexity**: Two storage layers to maintain and debug
- **Memory Overhead**: MemDB + RocksDB block cache both consume RAM
- **Eviction Tuning**: LRU policy may not be optimal for all workloads
- **RocksDB Dependency**: Relies on C++ codebase (security updates, bugs)

### Mitigation

1. **Memory Budgeting**: Reserve 60% RAM for MemDB, 30% for RocksDB cache, 10% overhead
2. **Metrics**: Expose hit rate, eviction rate, compaction metrics via Prometheus
3. **Testing**: Benchmark worst-case scenarios (cold start, full eviction)
4. **Future**: Consider pluggable storage backends (custom LSM, memory-only mode)

## Implementation Details

### vBucket Storage Layout

Each vBucket maintains:
```
MemDB:
  HashMap<DocKey, Document>
  SequenceIndex: BTreeMap<SeqNo, DocKey>
  
RocksDB:
  CF_DEFAULT: <vb_id>:<doc_key> -> <doc_value>
  CF_SEQUENCE: <vb_id>:<seq_no> -> <doc_key>
  CF_METADATA: <vb_id>:meta -> <failover_log, high_seq_no>
```

### Document Format

```rust
struct Document {
    key: String,
    value: Vec<u8>,  // JSON bytes
    cas: u64,        // Compare-and-swap
    expiry: u64,     // Unix timestamp (0 = no expiry)
    flags: u32,      // User-defined
    datatype: u8,    // JSON, binary, compressed
    seq_no: u64,     // Monotonic sequence number
}
```

### Durability Levels

| Level | Behavior |
|-------|----------|
| `none` | Ack immediately after MemDB write (WAL async flush) |
| `majority` | Ack after majority replicas confirm MemDB write |
| `majorityAndPersistActive` | Ack after majority MemDB + active node WAL fsync |
| `persistToMajority` | Ack after majority nodes WAL fsync |

## Alternatives Considered

### FoundationDB as Storage Backend
- **Pros**: Distributed transactions, proven consistency
- **Cons**: Adds external dependency; limits deployment flexibility; overkill for single-node storage
- **Verdict**: Too heavy; EdgeCouchbase should be self-contained

### LMDB (Lightning Memory-Mapped Database)
- **Pros**: B+tree, memory-mapped, simple API
- **Cons**: Poor write concurrency (single writer lock); not write-optimized
- **Verdict**: Write throughput insufficient for high-ingest workloads

### Custom LSM Tree
- **Pros**: Full control, optimized for our access patterns
- **Cons**: 2+ years of engineering effort; reinventing well-tested code; compaction bugs are subtle
- **Verdict**: Focus on unique distributed features, not storage internals

### Sled (Rust-native embedded DB)
- **Pros**: Pure Rust, modern API, async-aware
- **Cons**: Still beta; production readiness unclear; smaller community than RocksDB
- **Verdict**: Monitor for future; too risky for v1

### All-in-Memory (MemDB only)
- **Pros**: Maximum performance; simple eviction = OOM crash
- **Cons**: No durability; dataset limited to RAM; expensive infrastructure
- **Verdict**: Useful for caching use case but not general-purpose DB

## Performance Targets

| Operation | Target Latency (p99) | Notes |
|-----------|----------------------|-------|
| Read (MemDB hit) | < 100µs | HashMap lookup |
| Read (Disk) | < 5ms | RocksDB + SSD |
| Write (none durability) | < 500µs | WAL + MemDB |
| Write (persistToMajority) | < 10ms | fsync + replication |
| Scan (vBucket) | > 10K docs/sec | Sequence iterator |

## References

- [RocksDB Architecture](https://github.com/facebook/rocksdb/wiki/RocksDB-Basics)
- [Couchbase Storage Architecture](https://docs.couchbase.com/server/current/learn/buckets-memory-and-storage/storage.html)
- [LSM Tree Performance](https://www.vldb.org/pvldb/vol12/p2272-sarkar.pdf)
- [rust-rocksdb Documentation](https://docs.rs/rocksdb/)

## Future Work

- **Compression**: Snappy/LZ4 for cold data
- **Encryption at Rest**: RocksDB encryption layer
- **Tiered Storage**: S3 integration for archival data
- **Memory-Optimized Index**: Skip-list alternative to RocksDB for indexes
