//! Shared helper module included by both sibling test crates via
//! `mod helpers_common;` — a multi-owner file rust-analyzer attributes to
//! exactly one canonical owner crate.

/// Used ONLY by `second_tests.rs`. When rust-analyzer picks
/// `first_tests` as the file's owner crate, the call in `second_tests.rs`
/// (a different crate with its own copy of this module) is invisible to
/// `references()` — zero refs — and the detector labels this "Dead".
pub fn drive() -> u32 {
    7
}

/// Used by `first_tests.rs` — correctly resolvable from the owner crate,
/// so it gets the right test-only label while `drive` beside it is
/// mislabeled (the same-file inconsistency seen in the audit).
pub fn park() -> u32 {
    3
}
