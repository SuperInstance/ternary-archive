# Ternary Archive

**Ternary Archive** provides persistent storage and retrieval of ternary knowledge in balanced ternary {-1, 0, +1} — featuring immutable scrolls, multi-index lookup, lifecycle management, and conservation law verification.

## Why It Matters

Knowledge bases need append-only storage for audit compliance, multi-dimensional indexing for fast retrieval, and integrity verification for trust. Ternary Archive models knowledge as immutable scrolls — once written, never modified, only superseded. The ternary value system {-1 (negation), 0 (neutral), +1 (affirmation)} naturally encodes knowledge polarity, enabling downstream reasoning about whether a fact affirms or contradicts a hypothesis.

## How It Works

### Scroll Data Model

```rust
Scroll {
    id: u64,                    // Unique identifier
    category: String,           // Knowledge domain
    key: String,                // Specific fact name
    value: Ternary,             // {-1, 0, +1}
    metadata: HashMap<String, String>,
    timestamp: u64,             // Creation time
}
```

Scroll creation: **O(1)**. Immutable after creation — updates create new scrolls with incremented versions.

### Multi-Index Structure

The `Index` maintains three HashMap-based indices:

```
by_category: HashMap<String, Vec<u64>>    // Category → scroll IDs
by_key:      HashMap<String, Vec<u64>>    // Key → scroll IDs
by_value:    HashMap<Ternary, Vec<u64>>   // Value → scroll IDs
```

Insertion into index: **O(1)** amortized (append to Vec). Lookup by any dimension: **O(1)** HashMap + **O(K)** for K matching IDs.

### Lifecycle Management

```
Write → Active → Archived → Expired
         ↑                      │
         └── restore (if permitted)
```

Each scroll tracks its lifecycle state. Archived scrolls are excluded from default queries but available for explicit historical queries. Expiration: **O(N)** batch scan of timestamps.

### Conservation Law Verification

The archive enforces conservation: the sum of all ternary values in a category should be stable:

```
Σ values(category) at time T ≈ Σ values(category) at time T+1 ± tolerance
```

Violations indicate data corruption or unauthorized modifications. Verification: **O(N)** per category.

## Quick Start

```rust
use ternary_archive::{Ternary, Scroll, Index};

let scroll = Scroll::new(1, "physics", "gravity_exists", Ternary::Pos, 1000)
    .with_metadata("source", "experiment");

let mut index = Index::new();
index.add(&scroll);

let results = index.by_category("physics");
println!("Physics scrolls: {}", results.len());
```

## API

| Type | Description |
|------|-------------|
| `Scroll` | Immutable knowledge record with id, category, key, value, metadata |
| `Ternary` | `Neg (-1)`, `Zero (0)`, `Pos (+1)` |
| `Index` | Multi-dimensional lookup (by_category, by_key, by_value) |
| `Archive` | Full archive with lifecycle management and conservation checks |

## Architecture Notes

Ternary Archive provides the knowledge persistence layer for SuperInstance. In γ + η = C, archived Pos (+1) values represent γ (growth — confirmed knowledge) while Neg (-1) values represent η (avoidance — negated hypotheses). The conservation law on archives directly implements the C invariant. Integrates with `ternary-chronicle` for temporal narrative generation.

See [ARCHITECTURE.md](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md) for knowledge management architecture.

## References

1. Pat Helland (2007). "Life beyond Distributed Transactions: an Apostate's Opinion." *CIDR*.
2. Stonebraker, M. (2010). "SQL databases v. NoSQL databases." *Communications of the ACM*, 53(4).
3. Lamport, L. (1998). "The Part-Time Parliament." *ACM Transactions on Computer Systems*, 16(2).

## License

MIT
