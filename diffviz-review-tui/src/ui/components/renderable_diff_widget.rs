use diffviz_core::renderable_diff::{ChangeType, RenderableDiff, RenderableLine};
use ratatui::{
    layout::Rect,
    prelude::Buffer,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget, Wrap},
};
use std::collections::HashMap;
use tui_design::{Theme, stylesheet};

/// An inline reasoning annotation to inject before a specific diff line
pub struct ReasoningAnnotation {
    /// 1-based line number within the rendered diff (relative, not absolute file position).
    /// Use `absolute_line.saturating_sub(diff_start_line).saturating_add(1)` to convert.
    pub trigger_line: usize,
    /// Decision label displayed in the gutter, e.g. "D1"
    pub label: String,
    /// Reasoning text from `CodeImpact.reasoning`
    pub reasoning: String,
}

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
    pub title: Option<String>,
    pub scroll_offset: usize,
    selection_range: Option<(usize, usize)>,
    cursor_line: Option<usize>,
    pub focused: bool,
    instruction_indicators: Option<&'a GutterBracketMap>,
    reasoning_annotations: &'a [ReasoningAnnotation],
}

impl<'a> RenderableDiffWidget<'a> {
    /// Create a new widget bound to the provided renderable diff.
    pub fn new(diff: &'a RenderableDiff<'a>) -> Self {
        Self {
            diff,
            show_all_context: true,
            highlight_semantics: true,
            title: None,
            scroll_offset: 0,
            selection_range: None,
            cursor_line: None,
            focused: false,
            instruction_indicators: None,
            reasoning_annotations: &[],
        }
    }

    pub fn with_focus(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn with_context(mut self, show_all: bool) -> Self {
        self.show_all_context = show_all;
        self
    }

    pub fn highlight_semantics(mut self, highlight: bool) -> Self {
        self.highlight_semantics = highlight;
        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_scroll_offset(mut self, offset: usize) -> Self {
        self.scroll_offset = offset;
        self
    }

    pub fn with_selection(mut self, range: Option<(usize, usize)>) -> Self {
        self.selection_range = range;
        self
    }

    pub fn with_cursor_line(mut self, cursor_line: Option<usize>) -> Self {
        self.cursor_line = cursor_line;
        self
    }

    pub fn with_instruction_indicators(mut self, indicators: &'a GutterBracketMap) -> Self {
        self.instruction_indicators = Some(indicators);
        self
    }

    pub fn with_reasoning_annotations(mut self, annotations: &'a [ReasoningAnnotation]) -> Self {
        self.reasoning_annotations = annotations;
        self
    }
}

impl<'a> Widget for RenderableDiffWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let RenderableDiffWidget {
            diff,
            show_all_context,
            highlight_semantics,
            title,
            scroll_offset,
            selection_range,
            cursor_line,
            focused,
            instruction_indicators,
            reasoning_annotations,
        } = self;

        let theme = Theme::mocha();

        let ctx = LineRenderContext {
            highlight_semantics,
            selection_range,
            cursor_line,
            instruction_indicators,
        };

        let annotation_map: HashMap<usize, &ReasoningAnnotation> = reasoning_annotations
            .iter()
            .map(|a| (a.trigger_line, a))
            .collect();

        let mut lines: Vec<Line> = Vec::new();

        if show_all_context {
            for render_line in &diff.lines {
                if let Some(ann) = annotation_map.get(&render_line.line_number) {
                    lines.push(annotation_line(ann, &theme));
                }
                append_line(&mut lines, render_line, &ctx, &theme);
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
                        if let Some(indicator) = hidden_indicator(run_len, &theme) {
                            lines.push(indicator);
                        }
                        idx += run_len;
                        continue;
                    }
                }

                if let Some(ann) = annotation_map.get(&line.line_number) {
                    lines.push(annotation_line(ann, &theme));
                }
                append_line(&mut lines, line, &ctx, &theme);
                idx += 1;
            }
        }

        let title = title
            .unwrap_or_else(|| format!("📦 {} ({:?})", diff.metadata.boundary_name, diff.language));
        let title_style = if focused {
            stylesheet::title_active(&theme)
        } else {
            stylesheet::title_inactive(&theme)
        };
        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(ratatui::text::Span::styled(title, title_style))
                    .style(stylesheet::layer_raised(&theme)),
            )
            .wrap(Wrap { trim: false })
            .scroll((scroll_offset as u16, 0));
        paragraph.render(area, buf);
    }
}

fn hidden_indicator(count: usize, theme: &Theme) -> Option<Line<'static>> {
    if count <= 1 {
        return None;
    }

    Some(Line::from(vec![Span::styled(
        format!("  … {count} hidden context lines …"),
        stylesheet::muted(theme),
    )]))
}

fn annotation_line(ann: &ReasoningAnnotation, theme: &Theme) -> Line<'static> {
    let border = Style::default().fg(theme.accents.mauve);
    Line::from(vec![
        Span::styled("│ ", border),
        Span::styled("\u{25c6} ", border),
        Span::styled(
            ann.label.clone(),
            Style::default()
                .fg(theme.accents.mauve)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default()),
        Span::styled(ann.reasoning.clone(), stylesheet::body(theme)),
    ])
}

fn should_hide_line(line: &RenderableLine<'_>) -> bool {
    line.should_fold()
}

fn render_gutter_bracket(_position: GutterPosition) -> &'static str {
    "  "
}

struct LineRenderContext<'a> {
    highlight_semantics: bool,
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
    theme: &Theme,
) -> Line<'static> {
    let change = line.primary_change_type().cloned();
    let is_context_line = should_hide_line(line);
    let is_cursor = cursor_line == Some(line.line_number);
    let is_selected = selection_range
        .map(|(start, end)| line.line_number >= start && line.line_number <= end)
        .unwrap_or(false);

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

    let bracket_text = render_gutter_bracket(gutter_position);
    spans.push(Span::styled(
        bracket_text,
        Style::default().fg(theme.accents.sky),
    ));

    let context_style = Style::default().fg(theme.surface.surface0());
    let indicator_style = if is_cursor {
        stylesheet::body(theme).add_modifier(Modifier::BOLD)
    } else if is_context_line {
        context_style
    } else {
        style_for_change(change.as_ref(), theme).add_modifier(Modifier::BOLD)
    };
    spans.push(Span::styled(indicator, indicator_style));

    spans.push(Span::raw(" "));

    let content_style = if is_context_line {
        context_style
    } else {
        style_for_change(change.as_ref(), theme)
    };
    spans.push(Span::styled(line.content.to_string(), content_style));

    // Apply selection/cursor styling per-span on content spans BEFORE pushing anchor spans.
    // This avoids line-level style inheritance: if we set Line::style() later, the anchor
    // span would inherit the selection bg. Patching content spans individually keeps the
    // anchor bg-free regardless of selection state.
    if is_cursor {
        let cursor_style = stylesheet::success(theme).add_modifier(Modifier::BOLD);
        for span in &mut spans {
            span.style = span.style.patch(cursor_style);
        }
    } else if is_selected {
        let sel_style = stylesheet::selection(theme);
        for span in &mut spans {
            span.style = span.style.patch(sel_style);
        }
    }

    if highlight_semantics && let Some(anchor) = &line.semantic_anchor {
        let anchor_style = if is_context_line {
            stylesheet::muted(theme)
        } else {
            stylesheet::info(theme)
        };
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            format!("⟶ {}", anchor.identifier),
            anchor_style,
        ));
    }

    Line::from(spans)
}

fn append_line(
    lines: &mut Vec<Line<'static>>,
    line: &RenderableLine<'_>,
    ctx: &LineRenderContext<'_>,
    theme: &Theme,
) {
    let rendered = line_to_spans(
        line,
        ctx.highlight_semantics,
        ctx.selection_range,
        ctx.cursor_line,
        ctx.instruction_indicators,
        theme,
    );
    lines.push(rendered);
}

fn change_indicator(change: Option<&ChangeType>) -> String {
    match change {
        Some(ChangeType::Added) => "+".to_string(),
        Some(ChangeType::Deleted) => "-".to_string(),
        Some(ChangeType::Modified) => "~".to_string(),
        None => " ".to_string(),
    }
}

fn style_for_change(change: Option<&ChangeType>, theme: &Theme) -> Style {
    match change {
        Some(ChangeType::Added) => stylesheet::diff_added(theme),
        Some(ChangeType::Deleted) => stylesheet::diff_removed(theme),
        Some(ChangeType::Modified) => stylesheet::diff_modified(theme),
        None => stylesheet::diff_context(theme),
    }
}
