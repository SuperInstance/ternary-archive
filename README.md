# ternary-archive: Persistent storage and retrieval of ternary knowledge

An append-only knowledge store for balanced ternary {-1, 0, +1} systems. Scrolls are immutable records, indexed for fast lookup, organized into catalogs by category, and verified against conservation laws. A curator manages the lifecycle from active to expired.

## Why This Exists

Multi-agent ternary systems accumulate knowledge over time: discoveries, measurements, decisions. Without structured storage, this knowledge is ephemeral — lost when agents restart. This crate provides the persistence layer: write-once records with indexing, categorization, and conservation verification. The archive is the institutional memory of a ternary fleet.

## Core Concepts

- **Balanced ternary**: A number system using three values: -1, 0, +1 (Neg, Zero, Pos).
- **Scroll**: An immutable record. Like a real scroll, once written it doesn't change. Contains a category, key, ternary value, timestamp, and optional metadata.
- **Index**: Triple-indexed lookup — by category, by key, and by value. Makes retrieval O(1) for common queries.
- **Catalog**: Browse-oriented view. Groups scrolls by category for sequential access.
- **Conservation**: Tracks the sum of all stored values. In balanced ternary systems, knowledge should tend toward zero — a non-zero balance indicates bias in what's been archived.
- **ArchiveCurator**: Manages lifecycle stages (Active → Deprecated → Archived → Expired). Prevents stage-skipping.

## Quick Start

```toml
[dependencies]
ternary-archive = "0.1"
```

```rust
use ternary_archive::{Archive, Ternary, ArchiveCurator, LifecycleStage};

let mut archive = Archive::new();
let mut curator = ArchiveCurator::new();

// Store knowledge
let id = archive.store("physics", "momentum", Ternary::Pos, 1000);
curator.register(id);
archive.store("physics", "charge", Ternary::Neg, 1001);

// Retrieve it
let scroll = archive.retrieve(id).unwrap();
assert_eq!(scroll.key(), "momentum");

// Check conservation balance
assert!(archive.is_balanced()); // Pos + Neg = 0
```

## API Overview

| Type | Description |
|------|-------------|
| `Ternary` | A ternary value: Neg (-1), Zero (0), or Pos (+1) |
| `Scroll` | An immutable record with ID, category, key, value, and metadata |
| `Index` | Triple-indexed lookup by category, key, and value |
| `Catalog` | Browse scrolls grouped by category |
| `Conservation` | Verify that stored values obey conservation (sum ≈ 0) |
| `Archive` | The main knowledge store combining all of the above |
| `ArchiveCurator` | Manages scroll lifecycle: Active → Deprecated → Archived → Expired |
| `LifecycleStage` | Enum for the four stages of knowledge lifecycle |

## How It Works

The `Archive` is a write-once store. Each `store()` call creates a `Scroll` with a unique auto-incrementing ID, inserts it into a HashMap for O(1) retrieval by ID, adds it to the `Index` (three HashMaps for category/key/value lookup), and files it in the `Catalog` by category. The `Conservation` tracker records every value's contribution to the running sum.

`Scroll` is intentionally immutable — there are no mutation methods. If knowledge changes, you store a new scroll. The old one remains as historical record.

The `ArchiveCurator` enforces a strict lifecycle: Active → Deprecated → Archived → Expired. You can't skip stages. This prevents accidental data loss — deprecated knowledge gets a reason, archived knowledge gets reviewed, and only then can it expire.

## Known Limitations

- **All in-memory**: No disk persistence. Data vanishes when the process exits. For durability, serialize and write externally.
- **No deletion**: Once stored, scrolls can't be removed from the archive, only lifecycle-staged. This is by design (immutability), but means memory grows monotonically.
- **Single-threaded**: No interior mutability or locks. Wrap in `Arc<Mutex<Archive>>` for concurrent access.
- **Conservation is advisory**: Nothing prevents you from storing biased data. The `Conservation` tracker reports the balance but doesn't enforce it. Use `would_violate()` for pre-checks.
- **No compaction**: Expired scrolls still occupy memory. A production system would need compaction or offloading.

## Use Cases

- **Fleet knowledge accumulation**: Agents store discoveries, measurements, and decisions in a shared archive that persists across sessions.
- **Experiment logging**: Record ternary-valued experimental results with conservation verification to detect systematic bias.
- **Audit trail**: Every state change is an immutable scroll. Track what happened, when, and why (via lifecycle metadata).
- **Knowledge base**: Build a browsable catalog of ternary facts, indexed for fast retrieval by category, key, or value.

## Ecosystem Context

Part of the SuperInstance ternary crate family. Relates to:
- `ternary-frontier` (discoveries become archive scrolls)
- `ternary-memory` (short-term memory vs long-term archive)
- `ternary-conservation-verify` (deeper conservation analysis)
- `ternary-protocol` (archived knowledge can be shared via protocol messages)

## See Also

- **ternary-database** — Storage, indexing, and querying for ternary-valued rows
- **ternary-memory** — Short-term memory and recall for ternary agents
- **ternary-chronicle** — Chronological event logging with ternary annotations
- **ternary-replay** — Replay and playback of ternary event streams
- **ternary-compression** — Compression for ternary-valued streams

## License

MIT
