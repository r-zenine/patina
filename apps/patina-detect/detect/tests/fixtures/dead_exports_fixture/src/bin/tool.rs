/// A `pub` bin entry point: nothing in this crate calls `main` (the Rust
/// runtime does), so it has the same zero-reference shape as a genuinely
/// dead export — must be excluded via the spec's "bin entry points"
/// exclusion, not reported.
pub fn main() {
    println!("noop");
}
