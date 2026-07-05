//! Single source of truth for line<->byte conversions.
//!
//! Four call sites across the crate each hand-rolled their own version of this
//! conversion and disagreed with each other (see plan-core-hardening decision
//! D005) — two verified bugs (end-line exclusion, column-0 off-by-one) lived in
//! those disagreements. `LineIndex` is a newline-start table built in one pass;
//! `line_start` is O(1) indexing, `byte_to_line` is O(log n) via
//! `partition_point`. `byte_range_of_lines` is the ONLY place an inclusive
//! 1-based line range converts to a half-open byte range.

use std::ops::Range;

/// Byte offsets where each line starts, 1-based line `n` at `line_starts[n-1]`.
/// Line numbering and counting matches `str::lines()`: a source ending in `\n`
/// has no synthetic trailing empty line; an empty source has zero lines.
#[derive(Debug, Clone)]
pub struct LineIndex {
    line_starts: Vec<usize>,
    source_len: usize,
}

impl LineIndex {
    pub fn new(source: &str) -> Self {
        let bytes = source.as_bytes();
        let mut line_starts = vec![0usize];
        for (i, &b) in bytes.iter().enumerate() {
            if b == b'\n' {
                line_starts.push(i + 1);
            }
        }
        // A source ending in '\n' leaves a phantom "start of the next line" at
        // EOF with no actual line content there — drop it. An empty source
        // has zero lines, not one phantom line at offset 0 — drop it too.
        if source.is_empty() || bytes.last() == Some(&b'\n') {
            line_starts.pop();
        }

        Self {
            line_starts,
            source_len: bytes.len(),
        }
    }

    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// Byte offset of the start of 1-based `line`. `None` if `line` is 0 or
    /// beyond `line_count()`.
    pub fn line_start(&self, line: usize) -> Option<usize> {
        if line == 0 {
            return None;
        }
        self.line_starts.get(line - 1).copied()
    }

    /// 1-based line number containing `offset`. Returns 0 for an empty source
    /// (there is no line to report).
    pub fn byte_to_line(&self, offset: usize) -> usize {
        self.line_starts.partition_point(|&start| start <= offset)
    }

    /// Half-open byte range covering inclusive 1-based lines
    /// `start_line..=end_line`, terminator included. `start_line` must be a
    /// valid line (panics otherwise — a caller precondition, not external
    /// input validation, which callers must do before reaching here).
    /// `end_line + 1` beyond the last line is expected (the common case of
    /// asking for the range through the last line) and resolves to EOF.
    pub fn byte_range_of_lines(&self, start_line: usize, end_line: usize) -> Range<usize> {
        let start = self
            .line_start(start_line)
            .expect("start_line out of bounds");
        let end = self.line_start(end_line + 1).unwrap_or(self.source_len);
        start..end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_source_has_no_lines() {
        let idx = LineIndex::new("");
        assert_eq!(idx.line_count(), 0);
        assert_eq!(idx.line_start(1), None);
        assert_eq!(idx.byte_to_line(0), 0);
    }

    #[test]
    fn no_trailing_newline() {
        let idx = LineIndex::new("a\nb");
        assert_eq!(idx.line_count(), 2);
        assert_eq!(idx.line_start(1), Some(0));
        assert_eq!(idx.line_start(2), Some(2));
        assert_eq!(idx.line_start(3), None);
        assert_eq!(idx.byte_to_line(0), 1);
        assert_eq!(idx.byte_to_line(2), 2);
        assert_eq!(idx.byte_range_of_lines(1, 2), 0..3);
    }

    #[test]
    fn trailing_newline_has_no_phantom_line() {
        let idx = LineIndex::new("a\nb\n");
        assert_eq!(idx.line_count(), 2);
        assert_eq!(idx.line_start(1), Some(0));
        assert_eq!(idx.line_start(2), Some(2));
        assert_eq!(idx.line_start(3), None);
        // End of line 2 falls back to EOF since there is no line 3 start.
        assert_eq!(idx.byte_range_of_lines(1, 2), 0..4);
        assert_eq!(idx.byte_range_of_lines(2, 2), 2..4);
    }

    #[test]
    fn crlf_terminated_lines() {
        let idx = LineIndex::new("a\r\nb\r\n");
        assert_eq!(idx.line_count(), 2);
        assert_eq!(idx.line_start(1), Some(0));
        assert_eq!(idx.line_start(2), Some(3));
        assert_eq!(idx.byte_to_line(4), 2); // the '\r' of line 2's terminator
        assert_eq!(idx.byte_range_of_lines(1, 2), 0..6);
    }

    #[test]
    fn single_line_no_newline() {
        let idx = LineIndex::new("abc");
        assert_eq!(idx.line_count(), 1);
        assert_eq!(idx.byte_range_of_lines(1, 1), 0..3);
    }

    #[test]
    fn column_zero_start_of_line_reports_correct_line() {
        // Regression fixture for the column-0 off-by-one bug: a node starting
        // exactly at a line's first byte must report that line, not the
        // previous one.
        let idx = LineIndex::new("line one\nline two\nline three\n");
        assert_eq!(idx.byte_to_line(9), 2); // 'l' of "line two"
        assert_eq!(idx.byte_to_line(18), 3); // 'l' of "line three"
    }

    #[test]
    #[should_panic(expected = "start_line out of bounds")]
    fn byte_range_of_lines_rejects_invalid_start() {
        let idx = LineIndex::new("a\nb\n");
        idx.byte_range_of_lines(0, 1);
    }
}
