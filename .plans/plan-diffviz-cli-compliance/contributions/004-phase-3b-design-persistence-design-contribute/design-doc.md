# Design: State Persistence Module (Phase 3b)

## Objective

Move `load_review_state` / `save_review_state` + 3 DTO structs from `diffviz-cli/src/main.rs`
into `diffviz-review` to restore the clean architecture boundary.

---

## Target Module

**New file**: `diffviz-review/src/persistence.rs`

**lib.rs additions**:
```rust
pub mod persistence;
pub use persistence::{load_review_state, save_review_state, PersistenceError};
```

---

## Public API

```rust
pub fn load_review_state(
    folder: &Path,
    engine: &mut ReviewEngine,
) -> Result<(), PersistenceError>

pub fn save_review_state(
    folder: &Path,
    engine: &ReviewEngine,
) -> Result<(), PersistenceError>
```

Both functions mirror current `main.rs` signatures exactly — only the crate location changes.

---

## Error Type

```rust
#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Import failed: {0}")]
    Import(String),
}
```

`thiserror` and `serde_json` are already in `diffviz-review/Cargo.toml` — **no new dependencies**.

The `Import` variant replaces the `anyhow::anyhow!("Failed to import instructions: {e}")` call
in the current `load_review_state`. Call sites in `main.rs` use `anyhow::Result`, and
`anyhow::Error` auto-converts from any `std::error::Error`, so `?` works unchanged at the
call site.

---

## Private DTOs

These three structs stay private to `persistence.rs` — they are serialization-only implementation
details and no external code needs to construct or inspect them:

- `PersistedApproval`
- `PersistedDecisionApproval`
- `ReviewStateFile`

---

## Call Site Change (main.rs)

Remove the 3 struct definitions and 2 function definitions (lines 100–203).
Replace with:

```rust
use diffviz_review::persistence::{load_review_state, save_review_state};

// existing call sites are unchanged:
load_review_state(&folder, &mut engine)?;
save_review_state(&folder, &engine)?;
```

---

## Out of Scope

- `review-state.json` file format: **unchanged** (backward compat preserved)
- No new Cargo.toml entries required
- No changes to `ReviewEngine` public API
