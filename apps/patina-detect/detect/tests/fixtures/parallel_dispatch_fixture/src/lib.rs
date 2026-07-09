mod other;

pub use other::icon_for;

/// The type detector 7 (parallel dispatch) should flag: matched in 3 sites
/// (`describe`, `area_hint` here, `icon_for` in `other.rs`) across 2 files.
pub enum Shape {
    Circle,
    Square,
    Triangle,
}

pub fn describe(shape: Shape) -> &'static str {
    match shape {
        Shape::Circle => "circle",
        Shape::Square => "square",
        Shape::Triangle => "triangle",
    }
}

pub fn area_hint(shape: Shape) -> u32 {
    match shape {
        Shape::Circle => 1,
        Shape::Square => 2,
        Shape::Triangle => 3,
    }
}

/// Negative control: matched only twice, both in this same file — must not
/// be reported (under both the site-count and file-count thresholds).
pub enum Mode {
    Fast,
    Slow,
}

pub fn mode_label(mode: Mode) -> &'static str {
    match mode {
        Mode::Fast => "fast",
        Mode::Slow => "slow",
    }
}

pub fn mode_priority(mode: Mode) -> u32 {
    match mode {
        Mode::Fast => 1,
        Mode::Slow => 0,
    }
}

/// Negative control: a std type (`Option`) matched repeatedly — must never
/// be grouped, even though these two sites alone would need only one more
/// to clear the site/file thresholds if `Option` weren't denylisted.
pub fn handle_a(opt: Option<i32>) -> i32 {
    match opt {
        Some(x) => x,
        None => 0,
    }
}

pub fn handle_b(opt: Option<i32>) -> i32 {
    match opt {
        Some(x) => x + 1,
        None => -1,
    }
}
