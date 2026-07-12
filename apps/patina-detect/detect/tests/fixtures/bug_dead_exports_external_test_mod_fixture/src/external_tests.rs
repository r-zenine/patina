//! External test-module file: the `#[cfg(test)]` attribute gating this
//! file lives on the `mod tests;` declaration in `lib.rs`, NOT anywhere in
//! this file's own AST.

use super::make_thing;

/// Plain helper with no `#[test]` attribute — the reference to
/// `make_thing` the classifier misreads as production code.
fn setup() -> u32 {
    make_thing()
}

#[test]
fn thing_is_seven() {
    assert_eq!(setup(), 7);
}
