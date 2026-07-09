/// Never called anywhere in this crate — the detector's target positive.
pub fn dead_helper() -> i32 {
    42
}

/// Called from `caller` below — must not be reported.
pub fn used_helper() -> i32 {
    7
}

pub fn caller() -> i32 {
    used_helper() + 1
}

pub trait Greeter {
    fn greet(&self) -> String;
}

pub struct Thing;

/// Trait-impl method: zero direct call sites in this fixture (nothing
/// invokes `Thing::greet`), same reference-count shape as `dead_helper`,
/// but must be excluded — it's referenced through the trait, not directly.
impl Greeter for Thing {
    fn greet(&self) -> String {
        "hi".to_string()
    }
}

/// Derive-heavy struct: `name` is never read directly, but the `Debug`
/// derive uses it invisibly — must be excluded from field-write-only
/// reporting regardless of reference count.
#[derive(Debug)]
pub struct Config {
    pub name: String,
}

/// Only referenced from `#[cfg(test)]` code below — must still be reported,
/// but tagged `test_only: true` rather than dropped (spec.md's "production
/// code only tests exercise is its own finding").
pub fn test_only_helper() -> i32 {
    99
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_test_only_helper() {
        assert_eq!(test_only_helper(), 99);
    }
}
