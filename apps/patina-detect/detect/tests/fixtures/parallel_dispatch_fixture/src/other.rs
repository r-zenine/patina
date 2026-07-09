use crate::Shape;

/// Third site, second file — pushes `Shape` over both thresholds (3 sites,
/// 2 files).
pub fn icon_for(shape: Shape) -> &'static str {
    match shape {
        Shape::Circle => "\u{25CB}",
        Shape::Square => "\u{25A1}",
        Shape::Triangle => "\u{25B3}",
    }
}
