# ADR-003: Consistency and Replication Model

**Status**: Accepted  
**Date**: 2026-02-11  
**Deciders**: Core Team  
**Tags**: consistency, replication, distributed-systems

## Context

EdgeCouchbase must provide a consistency model that balances:
- **Availability**: Survive node failures without downtime
- **Performance**: Low-latency reads/writes under normal operation
- **Correctness**: Avoid data loss and corruption
- **Flexibility**: Allow applications to choose trade-offs per request

CAP theorem forces us to choose between consistency and availability during partitions. Our target workloads include both latency-sensitive OLTP and analytics, requiring different guarantees.

## Decision

We implement a **tunable consistency model** with multiple durability levels per operation:

### Primary Architecture: Active-Passive Replication (v1)

#### vBucket Distribution
- **1024 vBuckets** per bucket (same as Couchbase)
- Each vBucket has **1 active** + **N replicas** (typically N=1 or N=2)
- Active vBucket handles all writes; replicas receive changes via **DCP**
- Cluster map maintained by Raft-based cluster manager

#### Write Path
1. Client sends write to **any node**
2. Node consults cluster map, proxies to **active vBucket owner**
3. Active node writes to MemDB + WAL
4. Based on **durability level**, ack to client:
   - `none`: Ack after active MemDB write
   - `majority`: Ack after ⌈(N+1)/2⌉ nodes confirm MemDB write
   - `majorityAndPersistActive`: Ack after majority + active WAL fsync
   - `persistToMajority`: Ack after ⌈(N+1)/2⌉ nodes WAL fsync

#### Read Path
- **Default**: Read from **active vBucket** (causal consistency)
- **`staleness=ok`**: Read from **any replica** (eventual consistency, lower latency)
- **`staleness=bounded`**: Read from replica only if lag < 100ms

### Change Stream (DCP)

Each vBucket exposes a **Database Change Protocol** stream:
- **Sequence Numbers**: Monotonically increasing per vBucket (64-bit)
- **Snapshots**: Marker messages denoting consistent snapshot boundaries
- **Failover Log**: History of active vBucket ownership changes
- **Mutations**: CREATE, UPDATE, DELETE, EXPIRATION events

Replicas subscribe to DCP stream from active vBucket and apply mutations in order.

### Failover Handling

#### Automatic Failover
1. Cluster manager detects active node down (heartbeat timeout)
2. Raft consensus elects **new active** from replicas
3. New active promotes itself; updates cluster map
4. Clients redirect writes to new active
5. Old active (when recovered) rejoins as replica, catches up via DCP

#### Split-Brain Prevention
- **Raft Quorum**: Cluster map changes require majority vote
- **vBucket Ownership**: Only one active vBucket per cluster (enforced by Raft)
- **Fencing**: Old active cannot accept writes after cluster map change (epoch number check)

### Multi-Document Transactions (Phase 2)

- **Isolation**: Snapshot Isolation (SI) with optimistic concurrency control
- **Durability**: Transaction commit respects same durability levels as single writes
- **Conflict Resolution**: First-committer-wins; losing transactions abort and retry
- **Scope**: Transactions span vBuckets (but not cross-cluster in v1)

**Transaction Log**:
- Maintained in separate RocksDB column family
- 2PC protocol: Prepare → Commit/Abort
- Transaction coordinator: node that receives BEGIN

## Consistency Guarantees

| Durability Level | Guarantee | Latency | Use Case |
|------------------|-----------|---------|----------|
| `none` | At-least-once after crash recovery | ~500µs | Session cache, temporary data |
| `majority` | Survives (N-1)/2 node failures | ~2ms | Default for most apps |
| `majorityAndPersistActive` | Survives active crash + replica failure | ~5ms | Financial transactions |
| `persistToMajority` | Survives power loss on majority nodes | ~10ms | Critical audit logs |

### Cross-vBucket Consistency

- **No Guarantee**: Writes to different vBuckets are independent
- **Client-Side**: Applications can use transaction API for multi-vBucket atomicity
- **Analytics Service**: Creates shadow datasets via DCP for eventual consistency

## Consequences

### Positive

- **Tunable**: Apps choose latency vs. durability per request
- **Familiar**: Consistency model similar to Couchbase (migration path)
- **Simple Failure Handling**: Active-passive is easier to reason about than multi-leader
- **DCP Flexibility**: Same change stream powers replication, indexing, analytics, eventing

### Negative

- **No Linearizability**: Majority reads not implemented in v1 (requires Raft per vBucket)
- **Cross-vBucket**: No atomic writes across vBuckets without transactions
- **Stale Reads**: Replica reads may lag; applications must handle
- **Failover Latency**: 10-30 seconds for automatic failover (configurable)

### Mitigation

1. **Bounded Staleness**: Expose replica lag metrics; reject reads if lag > threshold
2. **Transactions**: Multi-document ACID for critical cross-vBucket operations
3. **Fast Failover**: Tune heartbeat intervals (trade-off: false positives)
4. **Client Library**: Smart client retries writes on node failure (idempotency keys)

## Alternatives Considered

### Strong Consistency (Raft per vBucket)
- **Pros**: Linearizable reads, no stale data
- **Cons**: 3x+ disk writes (Raft log + storage), higher latency, complex
- **Verdict**: Over-engineered for document DB use case; phase 2 consideration

### Multi-Master Replication
- **Pros**: Write availability in all nodes
- **Cons**: Conflict resolution complexity, eventual consistency mandatory
- **Verdict**: Requires CRDT or LWW (last-write-wins); conflicts are UX burden

### Quorum Reads/Writes (Dynamo-style)
- **Pros**: Tunable R + W > N for consistency
- **Cons**: Always-on quorum reads hurt performance; no causal ordering
- **Verdict**: Doesn't align with vBucket ownership model

### Chain Replication
- **Pros**: Linearizable reads from tail, high throughput
- **Cons**: Reconfiguration complexity; tail node is bottleneck for reads
- **Verdict**: Interesting for future; active-passive simpler for v1

### Spanner-style TrueTime
- **Pros**: External consistency, global ordering
- **Cons**: Requires atomic clocks/GPS; Google-specific infrastructure
- **Verdict**: Infeasible for self-hosted deployments

## Implementation Details

### Cluster Map

```rust
struct ClusterMap {
    version: u64,  // Raft log index
    vbuckets: Vec<VBucketMap>,
}

struct VBucketMap {
    vbucket_id: u16,
    active: NodeId,
    replicas: Vec<NodeId>,
}
```

### Durability Enforcement

```rust
enum DurabilityLevel {
    None,
    Majority,
    MajorityAndPersistActive,
    PersistToMajority,
}

async fn write_with_durability(
    doc: Document,
    level: DurabilityLevel,
) -> Result<CAS> {
    let ack_count = match level {
        None => 1,  // Just active
        Majority => (replica_count + 1) / 2 + 1,
        _ => /* similar logic */
    };
    
    // Write to active
    let seq_no = local_store.write(doc).await?;
    
    // Replicate via DCP
    let mut confirmations = 1;
    for replica in replicas {
        dcp_stream.send(replica, mutation(seq_no)).await?;
        if level.persist() {
            wait_wal_fsync(replica).await?;
        }
        confirmations += 1;
        if confirmations >= ack_count {
            break;
        }
    }
    
    Ok(doc.cas)
}
```

### Sequence Number Allocation

- **Per-vBucket Counter**: Atomic 64-bit counter in MemDB
- **Monotonic**: Sequence numbers never decrease (even across failover)
- **Failover Log**: Track (vbucket_uuid, start_seq_no) tuples

## Performance Targets

| Operation | Latency (p99) | Throughput |
|-----------|---------------|------------|
| Write (`none`) | < 1ms | 200K ops/sec |
| Write (`majority`, 1 replica) | < 3ms | 100K ops/sec |
| Write (`persistToMajority`) | < 15ms | 50K ops/sec |
| Read (active) | < 5ms | 500K ops/sec |
| Read (replica, `staleness=ok`) | < 2ms | 1M ops/sec |

## References

- [Couchbase Durability](https://docs.couchbase.com/server/current/learn/data/durability.html)
- [DCP Protocol](https://docs.couchbase.com/server/current/learn/clusters-and-availability/database-change-protocol.html)
- [Raft Consensus](https://raft.github.io/)
- [Consistency Models](https://jepsen.io/consistency)

## Future Work

- **Read-Your-Writes**: Session consistency via client-tracked sequence numbers
- **Raft per vBucket**: Linearizable reads option for critical data
- **Cross-Region Replication**: XDCR with conflict resolution strategies
- **Causality Tracking**: Vector clocks for multi-datacenter setups
