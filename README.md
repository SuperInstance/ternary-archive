# ternary-archive

**Persistent storage and retrieval of ternary knowledge with conservation-law verification.**

`ternary-archive` provides an append-only knowledge store where each record (a "scroll") is an immutable ternary-valued entry drawn from balanced ternary $\{-1, 0, +1\}$. It features multi-index lookup, catalog browsing, lifecycle management, and mathematical conservation verification ensuring that the archive's ternary balance tends toward zero.

## Why It Matters

In a balanced ternary system, knowledge has *value* — and value must be conserved. Every record written to the archive carries a ternary charge: $-1$, $0$, or $+1$. Over time, the sum of all charges should remain near zero; a significant imbalance signals information drift, data corruption, or adversarial injection.

This crate enforces that invariant through a `Conservation` tracker with $O(1)$ per-record overhead. Combined with multi-index retrieval (by category, key, and value) and a structured lifecycle (Active → Deprecated → Archived → Expired), the archive provides the persistence layer for ternary-native knowledge systems.

## How It Works

### Ternary Values and Scrolls

Each scroll is an immutable record with a monotonically increasing ID:

$$\text{Scroll} = (\text{id},\; \text{category},\; \text{key},\; v,\; t)$$

where $v \in \{-1, 0, +1\}$ is the ternary value and $t$ is a logical timestamp. Scrolls are write-once: once stored, their content never changes. This immutability enables safe concurrent reads without locking.

### Multi-Index Structure

The archive maintains three inverted indexes as hash maps:

| Index | Key | Value | Lookup |
|-------|-----|-------|--------|
| `by_category` | `String` | `Vec<u64>` | $O(1)$ average |
| `by_key` | `String` | `Vec<u64>` | $O(1)$ average |
| `by_value` | `Ternary` | `Vec<u64>` | $O(1)$ average |

Insertion appends to each index in $O(1)$ amortized time. Retrieval returns a slice of scroll IDs in $O(1)$, followed by $O(k)$ to fetch $k$ scrolls from the main `HashMap<u64, Scroll>`.

### Conservation Law Verification

The `Conservation` tracker maintains a running sum and count:

$$S = \sum_{i=1}^{n} v_i, \qquad \delta = \frac{S}{n}$$

where $S$ is the total sum, $n$ is the record count, and $\delta \in [-1, +1]$ is the deviation ratio. The archive is **balanced** when $S = 0$.

The `would_violate` check tests whether adding a new value would push $|S|$ beyond a threshold $\theta$:

$$\text{would\_violate}(v, \theta) = |S + v| > \theta$$

This is an $O(1)$ operation — no scan of existing records is needed because $S$ is maintained incrementally.

### Knowledge Lifecycle

The `ArchiveCurator` manages a four-stage lifecycle with enforced ordering:

```
Active ──deprecate()──▶ Deprecated ──archive()──▶ Archived ──expire()──▶ Expired
```

Transitions skip-proof: each stage can only transition to the next. Attempts to skip stages (e.g., Active → Archived) return `false` without modifying state.

**Complexity:** All lifecycle operations are $O(1)$ hash map lookups and updates. The `count_by_stage` operation is $O(n)$ over all tracked scrolls.

### Catalog Browsing

The `Catalog` provides a hierarchical view grouped by category, storing full `Scroll` objects (not just IDs). This enables efficient category-level browsing in $O(1)$ for category lookup, with $O(k)$ to return $k$ scrolls in a category.

## Quick Start

```toml
[dependencies]
ternary-archive = "0.1"
```

```rust
use ternary_archive::{Archive, Ternary, ArchiveCurator, LifecycleStage};

let mut archive = Archive::new();

// Store knowledge
let id1 = archive.store("physics", "momentum", Ternary::Pos, 100);
let id2 = archive.store("physics", "drag", Ternary::Neg, 101);
let id3 = archive.store("meta", "version", Ternary::Zero, 102);

// Conservation check: +1 + (-1) + 0 = 0 → balanced
assert!(archive.is_balanced());
assert_eq!(archive.conservation_balance(), 0);

// Multi-index retrieval
assert_eq!(archive.find_by_category("physics").len(), 2);
assert_eq!(archive.find_by_value(Ternary::Pos).len(), 1);

// Lifecycle management
let mut curator = ArchiveCurator::new();
curator.register(id1);
curator.deprecate(id1, "superseded by relativity");
assert_eq!(curator.stage(id1), Some(LifecycleStage::Deprecated));
```

## API

| Type | Purpose | Key Methods |
|------|---------|-------------|
| `Ternary` | The {-1, 0, +1} value type | `from_i8()`, `to_i8()` |
| `Scroll` | Immutable archive record | `id()`, `category()`, `key()`, `value()`, `with_metadata()` |
| `Index` | Three-way inverted index | `lookup_category()`, `lookup_key()`, `lookup_value()` |
| `Catalog` | Category-grouped browsing | `add()`, `browse()`, `categories()` |
| `Conservation` | O(1) conservation tracker | `record()`, `balance()`, `is_balanced()`, `deviation()`, `would_violate()` |
| `Archive` | Main knowledge store | `store()`, `retrieve()`, `find_by_*()`, `is_balanced()` |
| `ArchiveCurator` | Lifecycle management | `register()`, `deprecate()`, `archive()`, `expire()` |
| `LifecycleStage` | Active / Deprecated / Archived / Expired | — |

## Architecture Notes

The archive enforces the SuperInstance conservation law **γ + η = C**. Each ternary value contributes to the system's total charge: $+1$ values increase $\gamma$ (growth), $-1$ values increase $\eta$ (entropy), and $0$ values are neutral. The conservation invariant requires:

$$\sum_{i} v_i = \sum_{i} \gamma_i - \sum_{i} \eta_i \approx 0$$

The `Conservation` tracker directly computes this sum in $O(1)$ per insertion. A balanced archive ($S = 0$) means the system is at equilibrium — growth and entropy are in perfect tension, satisfying the constraint $\gamma + \eta = C$.

The lifecycle stages map to energy states: Active scrolls have high $\gamma$ (useful energy), Deprecated scrolls are transitioning to $\eta$, and Expired scrolls represent pure entropy — stored for audit but contributing no growth.

## References

- Cover, T.M. & Thomas, J.A. *Elements of Information Theory.* 2nd ed., Wiley, 2006. — Shannon entropy and information conservation.
- Knuth, D.E. *The Art of Computer Programming, Vol. 3: Sorting and Searching.* §6.5, on multi-key retrieval structures.
- Lamport, L. *The Part-Time Parliament.* ACM TOCS 1998. — Consensus and immutability in distributed logs.
- Bernstein, P.A. & Newcomer, E. *Principles of Transaction Processing.* Ch. 7, on append-only storage and lifecycle management.

## License

MIT
