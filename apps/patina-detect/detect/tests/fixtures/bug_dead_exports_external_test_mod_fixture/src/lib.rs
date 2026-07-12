//! Fixture for the external-`#[cfg(test)]`-module recall gap (mirrors
//! gitkit's `TestRepo` referenced from `decision_tests.rs`, a file pulled
//! in via `#[cfg(test)] #[path = ...] mod tests;` in another file).

/// Referenced ONLY from test code — but the reference lives in
/// `external_tests.rs` (included via the `#[cfg(test)]` mod declaration
/// below) inside a plain helper fn carrying no test attribute of its own,
/// so a per-file AST walk of `external_tests.rs` sees no test context.
pub fn make_thing() -> u32 {
    7
}

/// Indexing-settled anchor — genuinely dead, zero references anywhere.
pub fn dead_anchor() -> u32 {
    0
}

#[cfg(test)]
#[path = "external_tests.rs"]
mod tests;
