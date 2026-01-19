use crate::diff::inline::derive_inline_diff_map;
use diffviz_core::ast_diff::BACKGROUND;
use diffviz_core::renderable_diff::{ChangeType, RenderableDiff, RenderableLine};
use ratatui::{
    layout::Rect,
    prelude::Buffer,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};
use std::borrow::Cow;
use std::collections::HashMap;

pub use crate::diff::inline::{InlineDiffMap, InlineOldLine, InlineOldSegment};

/// Position of a line within an instruction range for gutter bracket rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GutterPosition {
    /// No instruction on this line
    None,
    /// Instruction starts on this line
    InstructionStart,
    /// Instruction continues through this line
    InstructionMiddle,
    /// Instruction ends on this line
    InstructionEnd,
}

/// Maps line numbers to their gutter bracket positions for instruction visualization
pub type GutterBracketMap = HashMap<usize, GutterPosition>;

/// Rich diff viewer that renders `RenderableDiff` instances without flattening them into strings.
pub struct RenderableDiffWidget<'a> {
    diff: &'a RenderableDiff<'a>,
    pub show_all_context: bool,
    pub highlight_semantics: bool,
    pub show_inline_old: bool,
    pub title: Option<String>,
    inline_changes: Option<&'a InlineDiffMap>,
    pub scroll_offset: usize,
    selection_range: Option<(usize, usize)>,
    cursor_line: Option<usize>,
    pub border_style: Style,
    instruction_indicators: Option<&'a GutterBracketMap>,
}

impl<'a> RenderableDiffWidget<'a> {
    /// Create a new widget bound to the provided renderable diff.
    pub fn new(diff: &'a RenderableDiff<'a>) -> Self {
        Self {
            diff,
            show_all_context: true,
            highlight_semantics: true,
            show_inline_old: false,
            title: None,
            inline_changes: None,
            scroll_offset: 0,
            selection_range: None,
            cursor_line: None,
            border_style: Style::default().fg(Color::Gray),
            instruction_indicators: None,
        }
    }

    /// Set the border style for the widget.
    pub fn with_border_style(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }

    /// Toggle whether background context lines are shown or folded.
    pub fn with_context(mut self, show_all: bool) -> Self {
        self.show_all_context = show_all;
        self
    }

    /// Toggle whether semantic anchors should be rendered next to the content.
    pub fn highlight_semantics(mut self, highlight: bool) -> Self {
        self.highlight_semantics = highlight;
        self
    }

    /// Toggle whether inline virtual text with old content should appear below modified lines.
    pub fn show_inline_old(mut self, show: bool) -> Self {
        self.show_inline_old = show;
        self
    }

    /// Attach inline old-content snippets for rendering.
    pub fn with_inline_changes(mut self, inline_changes: &'a InlineDiffMap) -> Self {
        self.inline_changes = Some(inline_changes);
        self
    }

    /// Override the block title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the scroll offset (number of rows to skip from the top).
    pub fn with_scroll_offset(mut self, offset: usize) -> Self {
        self.scroll_offset = offset;
        self
    }

    /// Provide a selected line range highlighted in the view.
    pub fn with_selection(mut self, range: Option<(usize, usize)>) -> Self {
        self.selection_range = range;
        self
    }

    /// Provide the current cursor line (1-based) to emphasize.
    pub fn with_cursor_line(mut self, cursor_line: Option<usize>) -> Self {
        self.cursor_line = cursor_line;
        self
    }

    /// Attach instruction indicators for rendering gutter brackets.
    pub fn with_instruction_indicators(mut self, indicators: &'a GutterBracketMap) -> Self {
        self.instruction_indicators = Some(indicators);
        self
    }
}

impl<'a> Widget for RenderableDiffWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let RenderableDiffWidget {
            diff,
            show_all_context,
            highlight_semantics,
            show_inline_old,
            title,
            inline_changes,
            scroll_offset,
            selection_range,
            cursor_line,
            border_style,
            instruction_indicators,
        } = self;

        let inline_changes = if show_inline_old {
            Some(match inline_changes {
                Some(map) => Cow::Borrowed(map),
                None => Cow::Owned(derive_inline_diff_map(diff)),
            })
        } else {
            None
        };

        let inline_changes_ref = inline_changes.as_deref();

        let ctx = LineRenderContext {
            highlight_semantics,
            show_inline_old,
            inline_changes: inline_changes_ref,
            selection_range,
            cursor_line,
            instruction_indicators,
        };

        let mut lines: Vec<Line> = Vec::new();

        if show_all_context {
            for render_line in &diff.lines {
                append_line(&mut lines, render_line, &ctx);
            }
        } else {
            let mut idx = 0;
            while idx < diff.lines.len() {
                let line = &diff.lines[idx];

                if should_hide_line(line) {
                    let mut run_len = 1;
                    while idx + run_len < diff.lines.len()
                        && should_hide_line(&diff.lines[idx + run_len])
                    {
                        run_len += 1;
                    }

                    if run_len > 1 {
                        if let Some(indicator) = hidden_indicator(run_len) {
                            lines.push(indicator);
                        }
                        idx += run_len;
                        continue;
                    }
                }

                append_line(&mut lines, line, &ctx);
                idx += 1;
            }
        }

        render_lines(diff, lines, title, area, buf, scroll_offset, border_style);
    }
}

fn hidden_indicator(count: usize) -> Option<Line<'static>> {
    if count <= 1 {
        return None;
    }

    Some(Line::from(vec![Span::styled(
        format!("  … {count} hidden context lines …"),
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
    )]))
}

fn should_hide_line(line: &RenderableLine<'_>) -> bool {
    if line.should_fold() {
        return true;
    }

    let is_structural_context = matches!(
        line.primary_change_type(),
        Some(ChangeType::Moved | ChangeType::Reordered)
    );

    line.max_relevance() >= BACKGROUND && is_structural_context
}

/// Renders the gutter bracket based on the instruction position
fn render_gutter_bracket(position: GutterPosition) -> &'static str {
    match position {
        GutterPosition::None => "  ",
        GutterPosition::InstructionStart => "📋┐",
        GutterPosition::InstructionMiddle => " │",
        GutterPosition::InstructionEnd => " └",
    }
}

/// Context for rendering a single line
struct LineRenderContext<'a> {
    highlight_semantics: bool,
    show_inline_old: bool,
    inline_changes: Option<&'a InlineDiffMap>,
    selection_range: Option<(usize, usize)>,
    cursor_line: Option<usize>,
    instruction_indicators: Option<&'a GutterBracketMap>,
}

fn line_to_spans(
    line: &RenderableLine<'_>,
    highlight_semantics: bool,
    selection_range: Option<(usize, usize)>,
    cursor_line: Option<usize>,
    instruction_indicators: Option<&GutterBracketMap>,
) -> Line<'static> {
    let change = line.primary_change_type().cloned();
    let is_context_line = should_hide_line(line);
    let is_cursor = cursor_line == Some(line.line_number);
    let is_selected = selection_range
        .map(|(start, end)| line.line_number >= start && line.line_number <= end)
        .unwrap_or(false);

    // Get gutter position for instruction brackets
    let gutter_position = instruction_indicators
        .and_then(|map| map.get(&line.line_number).copied())
        .unwrap_or(GutterPosition::None);

    let indicator = if is_cursor {
        "▸".to_string()
    } else if is_context_line {
        " ".to_string()
    } else {
        change_indicator(change.as_ref())
    };
    let mut spans = Vec::with_capacity(8);

    // Gutter bracket (2 chars)
    let bracket_text = render_gutter_bracket(gutter_position);
    let bracket_style = Style::default().fg(Color::Cyan);
    spans.push(Span::styled(bracket_text, bracket_style));

    // Line number (4 chars, right-aligned)
    spans.push(Span::styled(
        format!("{:>4}", line.line_number),
        if is_cursor {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        },
    ));

    // Change indicator (1 char)
    let indicator_style = if is_cursor {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else if is_context_line {
        Style::default().fg(Color::DarkGray)
    } else {
        style_for_change(change.as_ref())
    };
    spans.push(Span::styled(indicator, indicator_style));

    // Space before content (1 char)
    spans.push(Span::raw(" "));

    let content_style = if is_context_line {
        Style::default().fg(Color::DarkGray)
    } else {
        style_for_change(change.as_ref())
    };
    spans.push(Span::styled(line.content.to_string(), content_style));

    if highlight_semantics {
        if let Some(anchor) = &line.semantic_anchor {
            let anchor_style = if is_context_line {
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC)
            } else {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::ITALIC)
            };
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                format!("⟶ {}", anchor.identifier),
                anchor_style,
            ));
        }
    }

    let mut rendered = Line::from(spans);
    if is_selected {
        rendered = rendered.style(Style::default().bg(Color::DarkGray).fg(Color::White));
    }
    if is_cursor {
        rendered = rendered.style(Style::default().bg(Color::Yellow).fg(Color::Black));
    }

    rendered
}

fn append_line(
    lines: &mut Vec<Line<'static>>,
    line: &RenderableLine<'_>,
    ctx: &LineRenderContext<'_>,
) {
    let rendered = line_to_spans(
        line,
        ctx.highlight_semantics,
        ctx.selection_range,
        ctx.cursor_line,
        ctx.instruction_indicators,
    );
    lines.push(rendered);

    if ctx.show_inline_old {
        if let Some(map) = ctx.inline_changes {
            if let Some(inline_line) = create_inline_old_line(line, map) {
                lines.push(inline_line);
            }
        }
    }
}

fn render_lines(
    diff: &RenderableDiff<'_>,
    lines: Vec<Line<'static>>,
    title: Option<String>,
    area: Rect,
    buf: &mut Buffer,
    scroll_offset: usize,
    border_style: Style,
) {
    let title = title
        .unwrap_or_else(|| format!("📦 {} ({:?})", diff.metadata.boundary_name, diff.language));

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style),
        )
        .wrap(Wrap { trim: false })
        .scroll((scroll_offset as u16, 0));

    paragraph.render(area, buf);
}

#[allow(clippy::vec_init_then_push)]
fn create_inline_old_line(
    line: &RenderableLine<'_>,
    inline_changes: &InlineDiffMap,
) -> Option<Line<'static>> {
    if should_hide_line(line) {
        return None;
    }

    let inline = inline_changes.get(&line.line_number)?;
    if inline.segments.is_empty() {
        return None;
    }

    let indent: String = line
        .content
        .chars()
        .take_while(|c| c.is_whitespace())
        .collect();

    let mut spans = Vec::new();
    spans.push(Span::styled("    ", Style::default().fg(Color::DarkGray)));
    spans.push(Span::raw(" "));
    spans.push(Span::styled("↺", Style::default().fg(Color::DarkGray)));
    spans.push(Span::raw(" "));
    if !indent.is_empty() {
        spans.push(Span::styled(
            indent.clone(),
            Style::default().fg(Color::DarkGray),
        ));
    }

    let mut current_col = indent.chars().count();

    for segment in &inline.segments {
        if segment.start_col > current_col {
            let pad = " ".repeat(segment.start_col - current_col);
            spans.push(Span::styled(pad, Style::default().fg(Color::DarkGray)));
            current_col = segment.start_col;
        }

        spans.push(Span::styled(
            segment.text.clone(),
            Style::default()
                .fg(Color::LightMagenta)
                .add_modifier(Modifier::ITALIC),
        ));
        current_col += segment.text.chars().count();
    }

    Some(Line::from(spans))
}

fn change_indicator(change: Option<&ChangeType>) -> String {
    match change {
        Some(ChangeType::Added) => "+".to_string(),
        Some(ChangeType::Deleted) => "-".to_string(),
        Some(ChangeType::Modified) => "~".to_string(),
        Some(ChangeType::Moved) => ">".to_string(),
        Some(ChangeType::Reordered) => "↕".to_string(),
        None => " ".to_string(),
    }
}

fn style_for_change(change: Option<&ChangeType>) -> Style {
    match change {
        Some(ChangeType::Added) => Style::default().fg(Color::Green),
        Some(ChangeType::Deleted) => Style::default().fg(Color::Red),
        Some(ChangeType::Modified) => Style::default().fg(Color::Yellow),
        Some(ChangeType::Moved) => Style::default().fg(Color::Blue),
        Some(ChangeType::Reordered) => Style::default().fg(Color::Magenta),
        None => Style::default().fg(Color::Gray),
    }
}
