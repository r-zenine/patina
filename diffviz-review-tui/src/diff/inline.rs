use std::collections::{HashMap, VecDeque};

use diffviz_core::renderable_diff::{ChangeType, RenderableDiff, RenderableLine};

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
pub fn derive_inline_diff_map(diff: &RenderableDiff<'_>) -> InlineDiffMap {
    let mut map = InlineDiffMap::default();
    let mut deleted_queue: VecDeque<&RenderableLine<'_>> = VecDeque::new();

    for line in &diff.lines {
        match line.primary_change_type() {
            Some(ChangeType::Deleted) => {
                deleted_queue.push_back(line);
            }
            Some(ChangeType::Added) | Some(ChangeType::Modified) => {
                if let Some(old_line) = deleted_queue.pop_front() {
                    let segments = derive_inline_segments(old_line.content, line.content);
                    if !segments.is_empty() {
                        map.insert(line.line_number, InlineOldLine { segments });
                    }
                }
            }
            _ => {}
        }
    }

    map
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
