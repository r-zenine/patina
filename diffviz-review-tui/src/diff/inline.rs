use std::collections::HashMap;

use diffviz_core::renderable_diff::RenderableDiff;

#[derive(Debug, Clone)]
pub struct InlineOldSegment {
    pub start_col: usize,
    pub text: String,
}

#[derive(Debug, Clone, Default)]
pub struct InlineOldLine {
    pub segments: Vec<InlineOldSegment>,
}

pub type InlineDiffMap = HashMap<usize, InlineOldLine>;

/// Derive inline "old" snippets by pairing deleted lines with their corresponding additions.
///
/// With semantic pairing enabled, all Deleted+Added pairs are now adjacent (from Modify operations),
/// making inline old text visualization redundant. The Deleted line already shows the old version
/// directly above the Added line.
pub fn derive_inline_diff_map(_diff: &RenderableDiff<'_>) -> InlineDiffMap {
    // Disabled: semantic pairing renders all modifications as adjacent Deleted+Added lines,
    // making inline old text redundant. The deleted line is always directly above the added line.
    InlineDiffMap::default()
}

/// Compute the differing fragment between two strings, returning start column and text.
pub fn derive_inline_segments(old: &str, new: &str) -> Vec<InlineOldSegment> {
    let old_trim = old.trim_end_matches('\n');
    let new_trim = new.trim_end_matches('\n');

    if old_trim == new_trim {
        return Vec::new();
    }

    let old_chars: Vec<char> = old_trim.chars().collect();
    let new_chars: Vec<char> = new_trim.chars().collect();

    let mut prefix = 0;
    while prefix < old_chars.len()
        && prefix < new_chars.len()
        && old_chars[prefix] == new_chars[prefix]
    {
        prefix += 1;
    }

    let mut suffix = 0;
    while suffix < old_chars.len().saturating_sub(prefix)
        && suffix < new_chars.len().saturating_sub(prefix)
        && old_chars[old_chars.len() - 1 - suffix] == new_chars[new_chars.len() - 1 - suffix]
    {
        suffix += 1;
    }

    let start = prefix;
    let end = old_chars.len().saturating_sub(suffix);

    if start >= end {
        return Vec::new();
    }

    let text: String = old_chars[start..end].iter().collect();

    if text.trim().is_empty() {
        return Vec::new();
    }

    vec![InlineOldSegment {
        start_col: start,
        text,
    }]
}
