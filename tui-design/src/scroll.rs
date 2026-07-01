/// Returns the scroll offset that keeps `focused` visible within `viewport` lines.
///
/// `item_heights[i]` is the rendered height (in terminal lines) of item `i`,
/// including any trailing separator the caller renders between items.
///
/// Scrolls minimally: items above the fold stay put (offset 0); once the
/// focused item crosses the fold, its bottom edge is pinned to the viewport's
/// bottom edge so the view slides one item at a time instead of jumping.
/// An item taller than the viewport is pinned to the top instead so its
/// beginning stays visible.
pub fn scroll_into_view(item_heights: &[u16], focused: usize, viewport: u16) -> u16 {
    let top: u16 = item_heights[..focused].iter().sum();
    let item_h = item_heights.get(focused).copied().unwrap_or(0);
    let bottom = top + item_h;
    if bottom <= viewport {
        0
    } else {
        (bottom - viewport).min(top)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_scroll_when_focused_fits() {
        assert_eq!(scroll_into_view(&[3, 3, 3], 0, 10), 0);
        assert_eq!(scroll_into_view(&[3, 3, 3], 2, 10), 0);
    }

    #[test]
    fn pins_focused_bottom_to_viewport_bottom() {
        // items: 4+4+4 = 12, viewport 10 — focusing item 2 (bottom = 12)
        // needs offset 2 so its last line is the viewport's last line.
        assert_eq!(scroll_into_view(&[4, 4, 4], 2, 10), 2);
    }

    #[test]
    fn scrolls_one_step_at_a_time() {
        let heights = [3, 3, 3, 3, 3];
        let offsets: Vec<u16> = (0..5).map(|i| scroll_into_view(&heights, i, 7)).collect();
        // Monotonic, no jumps larger than an item height.
        assert_eq!(offsets, vec![0, 0, 2, 5, 8]);
    }

    #[test]
    fn oversized_item_pins_to_top() {
        // Item 1 is taller than the viewport — show its beginning.
        assert_eq!(scroll_into_view(&[2, 20, 2], 1, 10), 2);
    }

    #[test]
    fn focused_at_end_behaves_like_zero_height_item() {
        assert_eq!(scroll_into_view(&[4, 4], 2, 10), 0);
    }
}
