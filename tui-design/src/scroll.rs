/// Returns the scroll offset that keeps `focused` visible within `viewport` lines.
///
/// `item_heights[i]` is the rendered height (in terminal lines) of item `i`,
/// including any trailing separator the caller renders between items.
///
/// Returns 0 when the focused item fits without scrolling.
pub fn scroll_into_view(item_heights: &[u16], focused: usize, viewport: u16) -> u16 {
    let offset: u16 = item_heights[..focused].iter().sum();
    let item_h = item_heights.get(focused).copied().unwrap_or(0);
    if offset + item_h <= viewport {
        0
    } else {
        offset
    }
}
