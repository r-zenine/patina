//! Fixture for transitive deadness (mirrors tui-design's `surface2`, whose
//! only caller is the separately-dead `stylesheet::selection`).

/// Transitively dead: its only caller is `dead_entry`, which is itself
/// dead — deleting `dead_entry` makes this dead too, so both are
/// effectively unreachable today.
pub fn used_only_by_dead() -> u32 {
    1
}

/// Directly dead (zero references) — reported today, and doubles as the
/// indexing-settled anchor.
pub fn dead_entry() -> u32 {
    used_only_by_dead()
}
