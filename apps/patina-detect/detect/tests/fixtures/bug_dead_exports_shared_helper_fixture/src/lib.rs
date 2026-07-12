//! Fixture for the shared-integration-test-helper mislabel: the interesting
//! symbol is `drive` in `tests/helpers_common/mod.rs`, included by BOTH
//! sibling integration-test crates via `mod helpers_common;` (mirrors
//! diffviz-review-tui's `tests/drillnav_common/mod.rs`).

/// Indexing-settled anchor — genuinely dead, zero references anywhere.
pub fn dead_anchor() -> u32 {
    0
}

/// Production symbol so the crate isn't empty; referenced by `caller`.
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

pub fn caller() -> u32 {
    add(3, 4)
}
